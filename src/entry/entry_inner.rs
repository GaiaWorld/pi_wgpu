// 设计：pi-wgpu-rebuild
//! entry 芯：公开面对应实现的唯一可写处。错误纪律：校验违约=短码断言；运行期不便上递处 log::error!(短码)+降级。
#![allow(clippy::needless_pass_by_value)]
use crate::binds::{self, SlotTable};
use crate::caps;
use crate::context;
use crate::entry::*;
use crate::formats;
use crate::objects::{self, DirectHandle, DirectKind, GlHandle, HandleCache, ReuseKey};
use crate::shader::{self, CompiledEs300};
use crate::state::StateMirror;
use crate::types::*;
use glow::HasContext;
use pi_share::{Share, ShareMutex};
use std::num::NonZeroU32;
use std::path::Path;

/// GL 纹理目标常量（GL_* 名面未导出故内联规范值）。
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_TEXTURE_3D: u32 = 0x806F;
const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;
const GL_TEXTURE_CUBE_MAP: u32 = 0x8513;
const GL_TEXTURE_CUBE_MAP_ARRAY: u32 = 0x9009;
/// get_parameter_string 名：供应商/渲染器/版本。
const GL_VENDOR: u32 = 0x1F00;
const GL_RENDERER: u32 = 0x1F01;
const GL_VERSION: u32 = 0x1F02;
/// 着色器类型与编译/链接状态、参数名（GL 规范值）。
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TEXTURE_COMPARE_MODE: u32 = 0x884C;
const GL_TEXTURE_COMPARE_FUNC: u32 = 0x884D;
const GL_COMPARE_REF_TO_TEXTURE: i32 = 0x884E;
/// GL_COPY_WRITE_BUFFER（通用子写/分配目标；与 upload 芯同源）。
const GL_COPY_WRITE_BUFFER: u32 = 0x8F3F;
/// GL_DYNAMIC_DRAW（存储用途提示）。
const GL_DYNAMIC_DRAW: u32 = 0x88E8;
/// 缓冲用途合一位掩码（VERTEX|INDEX|UNIFORM|COPY_DST）。
const BUFFER_USAGE_MASK: u32 = 0x0F;
/// 纹理用途合一位掩码（COPY_DST|RENDER_ATTACHMENT|TEXTURE_BINDING）。
const TEXTURE_USAGE_MASK: u32 = 0x07;

/// 句柄名→非零包装（零名=非法断言）。
fn nz(name: GlHandle) -> NonZeroU32 {
    NonZeroU32::new(name.0).unwrap_or_else(|| panic!("entry 断言: 非法零句柄名"))
}

/// 能力快照→上限（兼容子集下限拍板；宁低报不虚高，不逐项真探）。
fn limits_of(_c: &caps::Caps) -> Limits {
    Limits {
        max_texture_dimension_2d: 4096,
        max_buffer_size: 1 << 28,
        max_bind_groups: 4,
        max_uniform_buffer_bindings: 16,
        // desc-align-wgpu25：GL 路无 push constants，下限拍板恒 0（如实不谎报）。
        max_push_constant_size: 0,
    }
}

pub(crate) fn new_impl(d: &InstanceDescriptor) -> Instance {
    let bad = d.backends.0 & !Backends::GL.0;
    assert_eq!(
        bad, 0,
        "entry 断言: descriptor.backends 含不支持位（field=backends, bits={bad:#x}）"
    );
    let slot = context::attach();
    assert!(
        slot.is_some(),
        "entry 断言: 上下文附着失败（无可用 GL 附着）"
    );
    let _ = caps::probe();
    Instance { _attach: slot }
}

pub(crate) fn create_surface_impl<'s>(
    t: crate::surface::SurfaceTarget<'s>,
) -> std::result::Result<crate::surface::Surface<'s>, CreateSurfaceError> {
    #[cfg(not(target_arch = "wasm32"))]
    let ok = matches!(t, crate::surface::SurfaceTarget::Window(_));
    #[cfg(target_arch = "wasm32")]
    let ok = matches!(
        t,
        crate::surface::SurfaceTarget::Canvas(_)
            | crate::surface::SurfaceTarget::OffscreenCanvas(_)
    );
    if !ok {
        return Err(CreateSurfaceError);
    }
    let Some(slot) = context::attach() else {
        return Err(CreateSurfaceError);
    };
    Ok(crate::surface::Surface {
        _target: t,
        _cfg: core::cell::RefCell::new(None),
        _attach: Some(slot),
    })
}

pub(crate) fn request_adapter_impl(_o: &RequestAdapterOptions<'_>) -> Option<Adapter> {
    context::attach()?;
    let c = caps::probe();
    Some(Adapter { _caps: c })
}

pub(crate) fn request_device_impl(
    a: &Adapter,
    d: &DeviceDescriptor<'_>,
    _t: Option<&Path>,
) -> std::result::Result<(Device, Queue), RequestDeviceError> {
    let avail = caps::fill_features(&a._caps);
    if d.required_features.0 & !avail.0 != 0 {
        return Err(RequestDeviceError);
    }
    let cap = limits_of(&a._caps);
    let r = d.required_limits;
    if r.max_texture_dimension_2d > cap.max_texture_dimension_2d
        || r.max_buffer_size > cap.max_buffer_size
        || r.max_bind_groups > cap.max_bind_groups
        || r.max_uniform_buffer_bindings > cap.max_uniform_buffer_bindings
        || r.max_push_constant_size > cap.max_push_constant_size
    {
        return Err(RequestDeviceError);
    }
    let mut state = StateMirror::default();
    state.generation = crate::context_inner::generation_now();
    let dev = Device {
        _inner: Share::new(DeviceInner {
            state: ShareMutex::new(state),
            cache: ShareMutex::new(HandleCache::default()),
            slots: ShareMutex::new(SlotTable::default()),
        }),
    };
    let q = Queue {
        _dev: Some(Device {
            _inner: dev._inner.clone(),
        }),
    };
    Ok((dev, q))
}

pub(crate) fn adapter_features(a: &Adapter) -> Features {
    caps::fill_features(&a._caps)
}
pub(crate) fn adapter_limits(a: &Adapter) -> Limits {
    limits_of(&a._caps)
}
pub(crate) fn adapter_info(_a: &Adapter) -> AdapterInfo {
    let (vendor, name, driver_info) = crate::context_inner::with_gl(|gl| unsafe {
        (
            gl.get_parameter_string(GL_VENDOR),
            gl.get_parameter_string(GL_RENDERER),
            gl.get_parameter_string(GL_VERSION),
        )
    });
    AdapterInfo {
        backend: Backend::Gl,
        device_type: DeviceType::Other,
        vendor,
        name,
        driver_info,
    }
}
pub(crate) fn adapter_downlevel(a: &Adapter) -> DownlevelCapabilities {
    DownlevelCapabilities {
        flags: DownlevelFlags(0),
        limits: limits_of(&a._caps),
    }
}
pub(crate) fn adapter_format_features(_a: &Adapter, f: TextureFormat) -> TextureFormatFeatures {
    let supported = matches!(
        f,
        TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb
            | TextureFormat::Rgba16Float
            | TextureFormat::Depth24PlusStencil8
            | TextureFormat::Depth32Float
    );
    let allowed = if supported {
        TextureUsages(TextureUsages::COPY_DST.0 | TextureUsages::RENDER_ATTACHMENT.0)
    } else {
        TextureUsages(0)
    };
    TextureFormatFeatures {
        allowed_usages: allowed,
        flags: TextureFormatFeatureFlags(0),
    }
}
pub(crate) fn adapter_surface_supported(_a: &Adapter, s: &crate::surface::Surface<'_>) -> bool {
    match &s._attach {
        Some(att) => att.generation == crate::context_inner::generation_now(),
        None => false,
    }
}
pub(crate) fn adapter_let_go(_a: &Adapter) {
    context::let_go();
}

pub(crate) fn create_buffer_impl(_d: &Device, b: &BufferDescriptor<'_>) -> Buffer {
    assert!(
        b.usage.0 != 0,
        "entry 断言: buffer usage 为空（field=usage, label={:?}）",
        b.label
    );
    assert_eq!(
        b.usage.0 & !BUFFER_USAGE_MASK,
        0,
        "entry 断言: buffer usage 含非支持位（field=usage={:#x}, label={:?}）",
        b.usage.0,
        b.label
    );
    assert!(
        b.size > 0 && b.size % COPY_BUFFER_ALIGNMENT == 0,
        "entry 断言: buffer size 须>0 且 4 字节倍数（field=size={}, label={:?}）",
        b.size,
        b.label
    );
    let h = crate::objects_inner::create_direct(DirectKind::Buffer);
    let name = h.name().expect("entry 断言: 新缓冲无名");
    // 分配驱动侧存储（glBufferData 等价）——write_buffer 只发子写入，无存储则子写失败、缓冲为空；
    // 届时顶点读取越界取数（实测在本机 NVIDIA 驱动侧直接崩）。目标用通用 COPY_WRITE_BUFFER（与 upload 芯同源）。
    crate::context_inner::with_gl(|gl| unsafe {
        gl.bind_buffer(GL_COPY_WRITE_BUFFER, Some(glow::NativeBuffer(nz(name))));
        gl.buffer_data_size(GL_COPY_WRITE_BUFFER, b.size as i32, GL_DYNAMIC_DRAW);
    });
    Buffer {
        _h: h,
        size: b.size,
    }
}

pub(crate) fn create_texture_impl(d: &Device, t: &TextureDescriptor<'_>) -> Texture {
    assert!(
        t.size.width >= 1 && t.size.height >= 1,
        "entry 断言: 纹理宽高须≥1（field=size, label={:?}）",
        t.label
    );
    assert!(
        t.size.depth_or_array_layers >= 1,
        "entry 断言: depth_or_array_layers 须≥1（field=size, label={:?}）",
        t.label
    );
    assert!(
        t.mip_level_count >= 1,
        "entry 断言: mip_level_count 须≥1（field=mip_level_count={}, label={:?}）",
        t.mip_level_count,
        t.label
    );
    assert!(
        t.sample_count == 1 || t.sample_count == 4,
        "entry 断言: sample_count 仅 1/4（field=sample_count={}, label={:?}）",
        t.sample_count,
        t.label
    );
    assert_eq!(
        t.usage.0 & !TEXTURE_USAGE_MASK,
        0,
        "entry 断言: 纹理 usage 含非支持位（field=usage={:#x}, label={:?}）",
        t.usage.0,
        t.label
    );
    for vf in t.view_formats {
        assert!(
            same_family(*vf, t.format),
            "entry 断言: view_format 与主格式非同族（field=view_formats, label={:?}）",
            t.label
        );
    }
    let params = formats::to_gl_params(t, t.format);
    let target = formats::target_of(t);
    let layers = t.size.depth_or_array_layers;
    let h = crate::objects_inner::create_direct(DirectKind::Texture);
    let name = h.name().expect("entry 断言: 新纹理无名");
    crate::context_inner::with_gl(|gl| unsafe {
        gl.bind_texture(target, Some(glow::NativeTexture(nz(name))));
        match target {
            GL_TEXTURE_3D | GL_TEXTURE_2D_ARRAY | GL_TEXTURE_CUBE_MAP_ARRAY => {
                gl.tex_storage_3d(
                    target,
                    t.mip_level_count as i32,
                    params.internal_format,
                    t.size.width as i32,
                    t.size.height as i32,
                    layers as i32,
                );
            }
            _ => {
                gl.tex_storage_2d(
                    target,
                    t.mip_level_count as i32,
                    params.internal_format,
                    t.size.width as i32,
                    t.size.height as i32,
                );
            }
        }
    });
    // 旁路：建后直绑当前 active 单元（tex_storage 需绑定态）——失效纹理槽水位，令下次 flush 重发镜像期望复位。
    {
        let mut st = d._inner.state.lock().unwrap_or_else(|e| e.into_inner());
        crate::state_inner::invalidate_texture_water(&mut st);
    }
    Texture {
        _h: h,
        params,
        _target: target,
        _dim: t.dim,
        _layers: layers,
    }
}

/// 建纹理视图（唯一 @public 视图入口）：维度相容校验 + 视图目标覆盖（Cube/CubeArray）。
pub(crate) fn create_view_impl(tex: &Texture, d: &TextureViewDescriptor<'_>) -> TextureView {
    use crate::types::TextureViewDimension as VD;
    let ok = match (tex._dim, d.dimension) {
        (TextureDimension::D1, VD::D1) => true,
        (TextureDimension::D2, VD::D2 | VD::D2Array) => true,
        (TextureDimension::D2, VD::Cube) => tex._layers == 6,
        (TextureDimension::D2, VD::CubeArray) => tex._layers % 6 == 0,
        (TextureDimension::D3, VD::D3) => true,
        _ => false,
    };
    assert!(
        ok,
        "entry 断言: 视图维度与纹理维度不相容（field=dimension, label={:?}）",
        d.label
    );
    let target = match d.dimension {
        VD::Cube => GL_TEXTURE_CUBE_MAP,
        VD::CubeArray => GL_TEXTURE_CUBE_MAP_ARRAY,
        _ => tex._target,
    };
    TextureView {
        _tex: Some(tex.clone()),
        _target: target,
        _dim: d.dimension,
    }
}

/// 主格式族：含/不含 sRGB 视为同族（兼容子集；深度同值）。
fn same_family(a: TextureFormat, b: TextureFormat) -> bool {
    fn strip(f: TextureFormat) -> TextureFormat {
        match f {
            TextureFormat::Rgba8UnormSrgb => TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8UnormSrgb => TextureFormat::Bgra8Unorm,
            other => other,
        }
    }
    strip(a) == strip(b)
}

pub(crate) fn create_sampler_impl(_d: &Device, s: &SamplerDescriptor<'_>) -> Sampler {
    if let Some(an) = s.anisotropy_clamp {
        let v = an.get();
        assert!(
            matches!(v, 1 | 2 | 4 | 8 | 16),
            "entry 断言: anisotropy_clamp 仅 1/2/4/8/16（field=anisotropy_clamp={v}, label={:?}）",
            s.label
        );
    }
    let h = crate::objects_inner::create_direct(DirectKind::Sampler);
    let name = h.name().expect("entry 断言: 新采样器无名");
    crate::context_inner::with_gl(|gl| unsafe {
        let sm = glow::NativeSampler(nz(name));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_MIN_FILTER, filter_gl(s.min_filter));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_MAG_FILTER, filter_gl(s.mag_filter));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_S, addr_gl(s.address_mode_u));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_T, addr_gl(s.address_mode_v));
        if let Some(cf) = s.compare {
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_MODE, GL_COMPARE_REF_TO_TEXTURE);
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_FUNC, compare_gl(cf));
        }
    });
    Sampler { _h: h }
}

fn filter_gl(f: FilterMode) -> i32 {
    match f {
        FilterMode::Nearest => 0x2600,
        FilterMode::Linear => 0x2601,
    }
}
fn addr_gl(a: AddressMode) -> i32 {
    match a {
        AddressMode::ClampToEdge => 0x812F,
        AddressMode::Repeat => 0x2901,
        AddressMode::MirroredRepeat => 0x8370,
    }
}
fn compare_gl(c: CompareFunction) -> i32 {
    match c {
        CompareFunction::Never => 0x0200,
        CompareFunction::Less => 0x0201,
        CompareFunction::Equal => 0x0202,
        CompareFunction::LessEqual => 0x0203,
        CompareFunction::Greater => 0x0204,
        CompareFunction::NotEqual => 0x0205,
        CompareFunction::GreaterEqual => 0x0206,
        CompareFunction::Always => 0x0207,
    }
}

pub(crate) fn create_shader_impl(_d: &Device, s: ShaderModuleDescriptor<'_>) -> ShaderModule {
    let blob = match shader::compile_to_es300(&s.source, s.label) {
        Ok(b) => b,
        Err(code) => {
            log::error!("entry shader 转译失败 code={code:?} label={:?}", s.label);
            CompiledEs300 {
                hash: 0,
                modules: Vec::new(),
            }
        }
    };
    ShaderModule {
        _refl: crate::shader::ReflectionTable::default(),
        _compiled: blob,
    }
}

pub(crate) fn create_bgl_impl(_d: &Device, b: &BindGroupLayoutDescriptor<'_>) -> BindGroupLayout {
    for e in b.entries {
        assert!(
            e.visibility.0 != 0,
            "entry 断言: 布局条目 visibility 为空（field=visibility, binding={}, label={:?}）",
            e.binding,
            b.label
        );
        assert!(
            e.count.map_or(true, |c| c.get() > 0),
            "entry 断言: 布局条目数组长度须>0（field=count, binding={}, label={:?}）",
            e.binding,
            b.label
        );
        assert!(
            e.binding < 65_536,
            "entry 断言: 布局条目 binding 域超 16 位（field=binding={}, label={:?}）",
            e.binding,
            b.label
        );
    }
    BindGroupLayout {
        _entries: b.entries.to_vec(),
    }
}

pub(crate) fn create_bg_impl(_d: &Device, b: &BindGroupDescriptor<'_>) -> BindGroup {
    let mut bound: Vec<crate::binds::BoundEntry> = Vec::with_capacity(b.entries.len());
    for e in b.entries {
        let le = b
            .layout
            ._entries
            .iter()
            .find(|l| l.binding == e.binding)
            .unwrap_or_else(|| {
                panic!(
                    "entry 断言: 绑定组条目不在布局中（field=binding={}, label={:?}）",
                    e.binding, b.label
                )
            });
        let target = match e.resource {
            BindingResource::TextureView(tv) => tv._target,
            _ => 0,
        };
        match (&le.ty, &e.resource) {
            (BindingType::Buffer { .. }, BindingResource::Buffer(_)) => {}
            (BindingType::Sampler(_), BindingResource::Sampler(_)) => {}
            (BindingType::Texture { .. }, BindingResource::TextureView(_)) => {}
            _ => panic!(
                "entry 断言: 绑定资源类型与布局不符（field=ty, binding={}, label={:?}）",
                e.binding, b.label
            ),
        }
        let (domain, handle, offset, size, dynamic) = match e.resource {
            BindingResource::Buffer(bb) => {
                assert!(
                    bb.offset % COPY_BUFFER_ALIGNMENT == 0,
                    "entry 断言: 缓冲绑定偏移须 4 对齐（field=offset={}, binding={}）",
                    bb.offset,
                    e.binding
                );
                let sz = bb.size.map(|z| z.get() as u32);
                if let Some(z) = bb.size {
                    assert!(
                        bb.offset + z.get() <= bb.buffer.size,
                        "entry 断言: 缓冲绑定越界（field=offset+size, binding={}）",
                        e.binding
                    );
                }
                let dyn_flag = matches!(
                    le.ty,
                    BindingType::Buffer {
                        has_dynamic_offset: true,
                        ..
                    }
                );
                (
                    crate::binds::BindDomain::Uniform,
                    bb.buffer._h.clone(),
                    bb.offset as u32,
                    sz,
                    dyn_flag,
                )
            }
            BindingResource::TextureView(tv) => {
                let t = tv._tex.as_ref().unwrap_or_else(|| {
                    panic!(
                        "entry 断言: 纹理视图无宿主（帧视图不可绑定，binding={}）",
                        e.binding
                    )
                });
                (
                    crate::binds::BindDomain::Texture,
                    t._h.clone(),
                    0,
                    None,
                    false,
                )
            }
            BindingResource::Sampler(sm) => (
                crate::binds::BindDomain::Sampler,
                sm._h.clone(),
                0,
                None,
                false,
            ),
        };
        assert!(
            handle.name().is_some(),
            "entry 断言: 绑定资源句柄已死（field=resource, binding={}, label={:?}）",
            e.binding,
            b.label
        );
        bound.push(crate::binds::BoundEntry {
            binding: e.binding,
            domain,
            handle,
            offset,
            size,
            dynamic,
            target,
        });
    }
    BindGroup { _bound: bound }
}

pub(crate) fn create_pl_impl(_d: &Device, p: &PipelineLayoutDescriptor<'_>) -> PipelineLayout {
    // desc-align-wgpu25：推送范围形在场、法恒拒止（GL 路无 push constants；能力位恒 0/上限恒 0）。
    assert!(
        p.push_constant_ranges.is_empty(),
        "entry 断言 ErrPushRange: GL 路恒拒止（label={:?}）",
        p.label
    );
    assert!(
        p.bind_group_layouts.len() <= 4,
        "entry 断言: 管线布局成员组数超上限 4（field=bind_group_layouts, label={:?}）",
        p.label
    );
    for i in 0..p.bind_group_layouts.len() {
        for j in (i + 1)..p.bind_group_layouts.len() {
            assert!(
                !std::ptr::eq(p.bind_group_layouts[i], p.bind_group_layouts[j]),
                "entry 断言: 管线布局成员重复（field=bind_group_layouts, label={:?}）",
                p.label
            );
        }
    }
    PipelineLayout {
        _bgls: p
            .bind_group_layouts
            .iter()
            .map(|b| BindGroupLayout {
                _entries: b._entries.clone(),
            })
            .collect(),
    }
}

pub(crate) fn create_rp_impl(d: &Device, p: &RenderPipelineDescriptor<'_>) -> RenderPipeline {
    // desc-align-wgpu25：形照齐、法收窄——以下违约=短码断言含 label，无半建管线（前置拒止先于一切分配）。
    // entry_point=None=实现批落地：透传出码面自动选唯一入口（缺失/歧义=Err(NoEntry)→出码失败断言路径）。
    assert!(
        p.layout.is_some(),
        "entry 断言 ErrNoLayout: auto-layout 不提供（label={:?})",
        p.label
    );

    let opts_default = |o: &crate::types::PipelineCompilationOptions<'_>| {
        o.constants.is_empty() && o.zero_initialize_workgroup_memory
    };
    assert!(
        opts_default(&p.vertex.compilation_options),
        "entry 断言 ErrCompileOpts（stage=vertex, label={:?}）",
        p.label
    );
    assert!(
        p.fragment
            .as_ref()
            .map_or(true, |f| opts_default(&f.compilation_options)),
        "entry 断言 ErrCompileOpts（stage=fragment, label={:?}）",
        p.label
    );
    assert!(
        p.multiview.is_none(),
        "entry 断言 ErrMultiview: GL 路不支持（label={:?}）",
        p.label
    );
    assert!(
        p.cache.is_none(),
        "entry 断言 ErrCache: 占位类型无构造面、此项不可达（label={:?}）",
        p.label
    );
    for vb in p.vertex.buffers {
        assert!(
            vb.array_stride % VERTEX_STRIDE_ALIGNMENT == 0,
            "entry 断言: 顶点步长须 4 对齐（field=array_stride={}, label={:?}）",
            vb.array_stride,
            p.label
        );
        for a in vb.attributes {
            assert!(
                a.offset % VERTEX_STRIDE_ALIGNMENT == 0,
                "entry 断言: 顶点属性偏移须 4 对齐（field=offset={}, label={:?}）",
                a.offset,
                p.label
            );
        }
    }
    {
        let mut slots = d._inner.slots.lock().unwrap_or_else(|e| e.into_inner());
        binds::alloc_slots(p.layout.expect("ErrNoLayout 已前置断言"), &mut slots);
    }
    let (vs, fs) = {
        let slots = d._inner.slots.lock().unwrap_or_else(|e| e.into_inner());
        let vs = shader::emit_stage(
            &p.vertex.module._compiled,
            naga::ShaderStage::Vertex,
            p.vertex.entry_point,
            &slots,
        )
        .unwrap_or_else(|e| panic!("entry 断言: 顶点出码失败 code={e:?} label={:?}", p.label));
        let fs = p.fragment.as_ref().map(|f| {
            shader::emit_stage(
                &f.module._compiled,
                naga::ShaderStage::Fragment,
                f.entry_point,
                &slots,
            )
            .unwrap_or_else(|e| panic!("entry 断言: 片元出码失败 code={e:?} label={:?}", p.label))
        });
        (vs, fs)
    };
    // 复用键=顶点+片元转译哈希合成（换任一段不得命中旧程序；形制对齐时由单 vertex 键修正为合成键）。
    let hash = {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        p.vertex.module._compiled.hash.hash(&mut h);
        if let Some(f) = p.fragment.as_ref() {
            f.module._compiled.hash.hash(&mut h);
        }
        h.finish()
    };
    let prog = {
        let mut cache = d._inner.cache.lock().unwrap_or_else(|e| e.into_inner());
        let prog = objects::ensure_reuse(ReuseKey::Program(hash), &mut cache);
        if prog.refcount.get() == 1 {
            link_program(
                prog.handle.expect("entry 断言: 新程序无句柄"),
                &vs,
                fs.as_deref(),
            );
        }
        prog
    };
    let vertex: Vec<crate::draw::VertexLayout> = p
        .vertex
        .buffers
        .iter()
        .map(|vb| crate::draw::VertexLayout {
            array_stride: vb.array_stride,
            step_mode: vb.step_mode,
            attributes: vb.attributes.to_vec(),
        })
        .collect();
    let vao = {
        let mut cache = d._inner.cache.lock().unwrap_or_else(|e| e.into_inner());
        objects::ensure_reuse(
            ReuseKey::Vao(crate::draw_inner::layout_hash(&vertex)),
            &mut cache,
        )
    };
    RenderPipeline {
        _program: prog,
        _vao: vao,
        _vertex: vertex,
        _topology: p.primitive.topology,
    }
}

/// 把 GLSL ES 3.0 源编译+链接进给定程序对象（仅新程序路径；复用命中不重链）。
fn link_program(name: GlHandle, vs: &str, fs: Option<&str>) {
    crate::context_inner::with_gl(|gl| unsafe {
        let prog = glow::NativeProgram(nz(name));
        let vsh = gl
            .create_shader(GL_VERTEX_SHADER)
            .unwrap_or_else(|e| panic!("entry 断言: 顶点着色器对象创建失败 {e}"));
        gl.shader_source(vsh, vs);
        gl.compile_shader(vsh);
        let cvs = gl.get_shader_compile_status(vsh);
        if !cvs {
            log::error!(
                "entry 顶点着色器编译失败 log={}",
                gl.get_shader_info_log(vsh)
            );
        }
        gl.attach_shader(prog, vsh);
        let mut fsh_opt = None;
        if let Some(src) = fs {
            let fsh = gl
                .create_shader(GL_FRAGMENT_SHADER)
                .unwrap_or_else(|e| panic!("entry 断言: 片元着色器对象创建失败 {e}"));
            gl.shader_source(fsh, src);
            gl.compile_shader(fsh);
            let cfs = gl.get_shader_compile_status(fsh);
            if !cfs {
                log::error!(
                    "entry 片元着色器编译失败 log={}",
                    gl.get_shader_info_log(fsh)
                );
            }
            gl.attach_shader(prog, fsh);
            fsh_opt = Some(fsh);
        }
        gl.link_program(prog);
        let lk = gl.get_program_link_status(prog);
        if !lk {
            log::error!("entry 程序链接失败 log={}", gl.get_program_info_log(prog));
        }
        gl.detach_shader(prog, vsh);
        gl.delete_shader(vsh);
        if let Some(fsh) = fsh_opt {
            gl.detach_shader(prog, fsh);
            gl.delete_shader(fsh);
        }
    });
}

pub(crate) fn create_encoder_impl(d: &Device, _c: &CommandEncoderDescriptor<'_>) -> CommandEncoder {
    CommandEncoder {
        _opened_pass: false,
        _inner: d._inner.clone(),
    }
}

/// 能力查询。登记差异：DeviceInner 未存创建时快照，此处现探（单适配器、建后未重探时等价创建快照）。
pub(crate) fn device_features(_d: &Device) -> Features {
    caps::fill_features(&caps::probe())
}
/// 上限查询。登记差异同 device_features。
pub(crate) fn device_limits(_d: &Device) -> Limits {
    limits_of(&caps::probe())
}

pub(crate) fn device_reset_state(d: &Device) {
    let mut st = d._inner.state.lock().unwrap_or_else(|e| e.into_inner());
    crate::state_inner::reset_all(&mut st);
}

pub(crate) fn encoder_finish(_e: CommandEncoder) -> CommandBuffer {
    CommandBuffer
}
