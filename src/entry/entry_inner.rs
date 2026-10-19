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
const GL_TEXTURE_MIN_LOD: u32 = 0x8130;
const GL_TEXTURE_MAX_LOD: u32 = 0x8131;

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TEXTURE_COMPARE_MODE: u32 = 0x884C;
const GL_TEXTURE_COMPARE_FUNC: u32 = 0x884D;
const GL_COMPARE_REF_TO_TEXTURE: i32 = 0x884E;
/// GL_COPY_WRITE_BUFFER（通用子写/分配目标；与 upload 芯同源）。
const GL_COPY_WRITE_BUFFER: u32 = 0x8F3F;
/// GL_DYNAMIC_DRAW（存储用途提示）。
const GL_DYNAMIC_DRAW: u32 = 0x88E8;
/// 缓冲支持域（INDEX|VERTEX|UNIFORM|COPY_DST 按上游位值合成；MAP/COPY_SRC/STORAGE 词位在场但域外拒止）。
const BUFFER_USAGE_MASK: crate::types::BufferUsages = crate::types::BufferUsages::COPY_DST
    .union(crate::types::BufferUsages::INDEX)
    .union(crate::types::BufferUsages::VERTEX)
    .union(crate::types::BufferUsages::UNIFORM);
/// 纹理支持域（COPY_SRC|COPY_DST|TEXTURE_BINDING|RENDER_ATTACHMENT；STORAGE_BINDING 词位在场但域外拒止）。
/// COPY_SRC 只作声明位：本层无 copy_texture_to_* 命令族（纹理写入唯经 Queue::write_texture CPU 通路），
/// 该位无命令消费=不引入行为，纯描述符兼容面（上游 RenderTexture 构造恒带此位，全引擎渲染目标必经）。
/// STORAGE_BINDING 需 image load/store 绑定，本层未接=仍拒。
const TEXTURE_USAGE_MASK: crate::types::TextureUsages = crate::types::TextureUsages::COPY_SRC
    .union(crate::types::TextureUsages::COPY_DST)
    .union(crate::types::TextureUsages::TEXTURE_BINDING)
    .union(crate::types::TextureUsages::RENDER_ATTACHMENT);
/// 阶段支持域（VERTEX|FRAGMENT；COMPUTE/TASK/MESH 词位在场但本层无对应管线=域外拒止）。
const STAGE_MASK: crate::types::ShaderStages =
    crate::types::ShaderStages::VERTEX.union(crate::types::ShaderStages::FRAGMENT);

/// 顶点格式兼容域（无平台差异口径）：ES3.0/WebGL2 与桌面 GL 核心**共有**的 40 型合法顶点格式。
/// 域外=桌面专有（Float64 系核心、Unorm8x4Bgra 扩展）——WebGL2 无此能力，按「无平台差异」拒止。
/// 非归一化整型（Uint*/Sint*）经 draw 侧 glVertexAttribIPointer 通路，其余走 f32 变体。
fn vtx_format_in_domain(f: crate::types::VertexFormat) -> bool {
    use crate::types::VertexFormat::*;
    matches!(
        f,
        Uint8
            | Uint8x2
            | Uint8x4
            | Sint8
            | Sint8x2
            | Sint8x4
            | Unorm8
            | Unorm8x2
            | Unorm8x4
            | Snorm8
            | Snorm8x2
            | Snorm8x4
            | Uint16
            | Uint16x2
            | Uint16x4
            | Sint16
            | Sint16x2
            | Sint16x4
            | Unorm16
            | Unorm16x2
            | Unorm16x4
            | Snorm16
            | Snorm16x2
            | Snorm16x4
            | Float16
            | Float16x2
            | Float16x4
            | Float32
            | Float32x2
            | Float32x3
            | Float32x4
            | Uint32
            | Uint32x2
            | Uint32x3
            | Uint32x4
            | Sint32
            | Sint32x2
            | Sint32x3
            | Sint32x4
            | Unorm10_10_10_2
    )
}

/// 句柄名→非零包装（零名=非法断言）。
fn nz(name: GlHandle) -> NonZeroU32 {
    NonZeroU32::new(name.0).unwrap_or_else(|| panic!("entry 断言: 非法零句柄名"))
}

/// 能力快照→上限全表（36 字段静态保底拍板=宁低报不虚高，不逐项真探；字段值表住 types::Limits）。
fn limits_of(_c: &caps::Caps) -> Limits {
    Limits::default()
}

pub(crate) fn new_impl(d: &InstanceDescriptor) -> Instance {
    let bad = d.backends.difference(Backends::GL);
    assert!(
        bad.is_empty(),
        "entry 断言: descriptor.backends 含不支持位（field=backends, bits={bad:?}）"
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
        _cfg: pi_share::ShareCell::new(None),
        _attach: Some(slot),
        _creation: pi_share::ShareCell::new(0),
    })
}

pub(crate) fn request_adapter_impl(
    _o: &crate::types::RequestAdapterOptions<&crate::surface::Surface<'_>>,
) -> Option<Adapter> {
    context::attach()?;
    let c = caps::probe();
    Some(Adapter { _caps: c })
}

/// required 与快照逐字段对照（desc-align-wgpu25 Limits 全表面：36 项任一超出=true）。
fn over_limits(r: &Limits, cap: &Limits) -> bool {
    r.max_texture_dimension_1d > cap.max_texture_dimension_1d
        || r.max_texture_dimension_2d > cap.max_texture_dimension_2d
        || r.max_texture_dimension_3d > cap.max_texture_dimension_3d
        || r.max_texture_array_layers > cap.max_texture_array_layers
        || r.max_bind_groups > cap.max_bind_groups
        || r.max_bindings_per_bind_group > cap.max_bindings_per_bind_group
        || r.max_dynamic_uniform_buffers_per_pipeline_layout
            > cap.max_dynamic_uniform_buffers_per_pipeline_layout
        || r.max_dynamic_storage_buffers_per_pipeline_layout
            > cap.max_dynamic_storage_buffers_per_pipeline_layout
        || r.max_sampled_textures_per_shader_stage > cap.max_sampled_textures_per_shader_stage
        || r.max_samplers_per_shader_stage > cap.max_samplers_per_shader_stage
        || r.max_storage_buffers_per_shader_stage > cap.max_storage_buffers_per_shader_stage
        || r.max_storage_textures_per_shader_stage > cap.max_storage_textures_per_shader_stage
        || r.max_uniform_buffers_per_shader_stage > cap.max_uniform_buffers_per_shader_stage
        || r.max_binding_array_elements_per_shader_stage
            > cap.max_binding_array_elements_per_shader_stage
        || r.max_binding_array_sampler_elements_per_shader_stage
            > cap.max_binding_array_sampler_elements_per_shader_stage
        || r.max_uniform_buffer_binding_size > cap.max_uniform_buffer_binding_size
        || r.max_storage_buffer_binding_size > cap.max_storage_buffer_binding_size
        || r.max_vertex_buffers > cap.max_vertex_buffers
        || r.max_buffer_size > cap.max_buffer_size
        || r.max_vertex_attributes > cap.max_vertex_attributes
        || r.max_vertex_buffer_array_stride > cap.max_vertex_buffer_array_stride
        || r.min_uniform_buffer_offset_alignment > cap.min_uniform_buffer_offset_alignment
        || r.min_storage_buffer_offset_alignment > cap.min_storage_buffer_offset_alignment
        || r.max_inter_stage_shader_components > cap.max_inter_stage_shader_components
        || r.max_color_attachments > cap.max_color_attachments
        || r.max_color_attachment_bytes_per_sample > cap.max_color_attachment_bytes_per_sample
        || r.max_compute_workgroup_storage_size > cap.max_compute_workgroup_storage_size
        || r.max_compute_invocations_per_workgroup > cap.max_compute_invocations_per_workgroup
        || r.max_compute_workgroup_size_x > cap.max_compute_workgroup_size_x
        || r.max_compute_workgroup_size_y > cap.max_compute_workgroup_size_y
        || r.max_compute_workgroup_size_z > cap.max_compute_workgroup_size_z
        || r.max_compute_workgroups_per_dimension > cap.max_compute_workgroups_per_dimension
        || r.min_subgroup_size > cap.min_subgroup_size
        || r.max_subgroup_size > cap.max_subgroup_size
        || r.max_push_constant_size > cap.max_push_constant_size
        || r.max_non_sampler_bindings > cap.max_non_sampler_bindings
}

pub(crate) fn request_device_impl(
    a: &Adapter,
    d: &DeviceDescriptor<'_>,
    _trace_path: Option<&std::path::Path>,
) -> std::result::Result<(Device, Queue), RequestDeviceError> {
    // trace 词位=纯占位（用户裁决）：DeviceDescriptor.trace 与 trace_path 双参形仅保持形制，
    // 本层无产出面也不拒止——赋值无效果，参数不消费。
    let avail = caps::fill_features(&a._caps);
    if !avail.contains(d.required_features) {
        log::error!(
            "request_device 拒绝: features 不足 missing={:?}",
            d.required_features & !avail
        );
        return Err(RequestDeviceError);
    }
    let cap = limits_of(&a._caps);
    if over_limits(&d.required_limits, &cap) {
        log::error!("request_device 拒绝: limits 超限");
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
    let (driver, name, driver_info) = crate::context_inner::with_gl(|gl| unsafe {
        (
            gl.get_parameter_string(GL_VENDOR),
            gl.get_parameter_string(GL_RENDERER),
            gl.get_parameter_string(GL_VERSION),
        )
    });
    AdapterInfo {
        name,
        vendor: 0,
        device: 0,
        device_type: DeviceType::Other,
        driver,
        driver_info,
        backend: Backend::Gl,
    }
}
pub(crate) fn adapter_downlevel(a: &Adapter) -> DownlevelCapabilities {
    DownlevelCapabilities {
        flags: DownlevelFlags::empty(),
        limits: crate::types::DownlevelLimits::default(),
        shader_model: ShaderModel::Sm5,
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
        TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT
    } else {
        TextureUsages::empty()
    };
    TextureFormatFeatures {
        allowed_usages: allowed,
        flags: TextureFormatFeatureFlags::empty(),
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
        !b.usage.is_empty(),
        "entry 断言: buffer usage 为空（field=usage={:?}, label={:?}）",
        b.usage,
        b.label
    );
    assert!(
        BUFFER_USAGE_MASK.contains(b.usage),
        "entry 断言: buffer usage 含非支持位（field=usage={:?}, label={:?}）",
        b.usage,
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
        _usage: b.usage,
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
    assert!(
        TEXTURE_USAGE_MASK.contains(t.usage),
        "entry 断言: 纹理 usage 含非支持位（field=usage={:?}, label={:?}）",
        t.usage,
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
    crate::crash_diag::op(&format!(
        "create_texture target={target:#x} {}x{} layers={layers} mips={} fmt={:?} ifmt={:#x} samples={}",
        t.size.width, t.size.height, t.mip_level_count, t.format, params.internal_format, t.sample_count
    ));
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
        _dim: t.dimension,
        _layers: layers,
        _size: t.size,
        _mip: t.mip_level_count,
        _samples: t.sample_count,
        _format: t.format,
        _usage: t.usage,
    }
}

/// 纹理显式销毁：句柄法死名（与末引用 Drop 同通道，先置 None 防重入；物理删除+死名登记）。
pub(crate) fn texture_destroy_impl(tex: &Texture) {
    tex._h.mark_dead();
}

/// 建纹理视图（唯一 @public 视图入口）：维度相容校验 + 视图目标覆盖（Cube/CubeArray）。
pub(crate) fn create_view_impl(tex: &Texture, d: &TextureViewDescriptor<'_>) -> TextureView {
    use crate::types::TextureViewDimension as VD;
    // desc-align-wgpu25：dimension=None=按宿主维度/层数推定（上游语义）；本层「取整张」。
    let dim = d.dimension.unwrap_or_else(|| match tex._dim {
        TextureDimension::D1 => VD::D1,
        TextureDimension::D3 => VD::D3,
        TextureDimension::D2 => match tex._layers {
            1 => VD::D2,
            6 => VD::Cube,
            _ => VD::D2Array,
        },
    });
    let ok = match (tex._dim, dim) {
        (TextureDimension::D1, VD::D1) => true,
        (TextureDimension::D2, VD::D2 | VD::D2Array) => true,
        (TextureDimension::D2, VD::Cube) => tex._layers == 6,
        (TextureDimension::D2, VD::CubeArray) => tex._layers % 6 == 0,
        (TextureDimension::D3, VD::D3) => true,
        _ => false,
    };
    assert!(
        ok,
        "entry 断言: 视图维度与纹理维度不相容（field=dimension/dim_inferred, label={:?}）",
        d.label
    );
    let target = match dim {
        VD::Cube => GL_TEXTURE_CUBE_MAP,
        VD::CubeArray => GL_TEXTURE_CUBE_MAP_ARRAY,
        _ => tex._target,
    };
    TextureView {
        _tex: Some(tex.clone()),
        _target: target,
        _dim: dim,
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
    // desc-align 补齐：上游合同——anisotropy ≥1，非 1 时三过滤须全 Linear（旧 2 的幂集校验随形退役）。
    assert!(
        s.anisotropy_clamp >= 1,
        "entry 断言: anisotropy_clamp 须 ≥1（field=anisotropy_clamp={}, label={:?}）",
        s.anisotropy_clamp,
        s.label
    );
    if s.anisotropy_clamp != 1 {
        assert!(
            matches!(s.mag_filter, crate::types::FilterMode::Linear)
                && matches!(s.min_filter, crate::types::FilterMode::Linear)
                && matches!(s.mipmap_filter, crate::types::FilterMode::Linear),
            "entry 断言: 各向异性非 1 须三过滤全 Linear（field=anisotropy_clamp={}, label={:?}）",
            s.anisotropy_clamp,
            s.label
        );
    }
    let h = crate::objects_inner::create_direct(DirectKind::Sampler);
    let name = h.name().expect("entry 断言: 新采样器无名");
    crate::context_inner::with_gl(|gl| unsafe {
        let sm = glow::NativeSampler(nz(name));
        gl.sampler_parameter_i32(
            sm,
            GL_TEXTURE_MIN_FILTER,
            min_filter_gl(s.min_filter, s.mipmap_filter),
        );
        gl.sampler_parameter_i32(sm, GL_TEXTURE_MAG_FILTER, filter_gl(s.mag_filter));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_S, addr_gl(s.address_mode_u));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_T, addr_gl(s.address_mode_v));
        if let Some(cf) = s.compare {
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_MODE, GL_COMPARE_REF_TO_TEXTURE);
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_FUNC, compare_gl(cf));
        }
        // MIP 夹值直通：上游字段语义（缺省 -1000/32 与 GL 内部约定同值；实际采样层
        // 硬件再按 [min, min(max, levels-1)] 收拢——25 形无 lod_bias 字段，GL 侧不掷偏置。
        gl.sampler_parameter_f32(sm, GL_TEXTURE_MIN_LOD, s.lod_min_clamp);
        gl.sampler_parameter_f32(sm, GL_TEXTURE_MAX_LOD, s.lod_max_clamp);
        // 各向异性：EXT_texture_filter_anisotropic——能力顶=查询可得值；无扩展=降级不掷
        // （warn 留痕，请求不生效=能力面实话），有则取请求与硬件顶的较小值。
        if s.anisotropy_clamp > 1 {
            let cap = aniso_cap(gl);
            if cap > 1.0 {
                let v: f32 = (s.anisotropy_clamp as f32).min(cap);
                gl.sampler_parameter_f32(sm, GL_TEXTURE_MAX_ANISOTROPY_EXT, v);
            } else {
                log::warn!(
                    "entry: 本形态无各向异性扩展，请求 {} 被降级",
                    s.anisotropy_clamp
                );
            }
        }
    });
    Sampler { _h: h }
}

/// min 过滤合成值：单级值×mipmap 组合（GL 0x2700-0x2703 四档）——上游 mipmap_filter 词位真消费。
fn min_filter_gl(min: FilterMode, mip: FilterMode) -> i32 {
    use crate::types::FilterMode as F;
    match (min, mip) {
        (F::Nearest, F::Nearest) => 0x2700,
        (F::Linear, F::Nearest) => 0x2701,
        (F::Nearest, F::Linear) => 0x2702,
        (F::Linear, F::Linear) => 0x2703,
    }
}

/// 各向异性能力顶（EXT 全局查询；无条目/枚举不认=返回 1 即不支持——先清残留 error 再看新鲜面）。
fn aniso_cap(gl: &glow::Context) -> f32 {
    unsafe {
        while gl.get_error() != 0 {}
        let v = gl.get_parameter_f32(GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT);
        let fresh = gl.get_error();
        if fresh == 0 && v > 1.0 {
            v
        } else {
            1.0
        }
    }
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
        AddressMode::MirrorRepeat => 0x8370,
        // 边框色状态本层未维护——取驱动默认黑边透明语义（词汇收录位）。
        AddressMode::ClampToBorder => 0x812D,
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
            !e.visibility.is_empty(),
            "entry 断言: 布局条目 visibility 为空（field=visibility, binding={}, label={:?}）",
            e.binding,
            b.label
        );
        assert!(
            STAGE_MASK.contains(e.visibility),
            "entry 断言: 布局条目 visibility 含非支持阶段（field=visibility={:?}, binding={}, label={:?}）",
            e.visibility,
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
            // 无平台差异口径：两形态核心共有的合法顶点格式全支持，域外=桌面专有（拒止）。
            assert!(
                vtx_format_in_domain(a.format),
                "entry 断言 ErrVtxFormat: 顶点格式在本层兼容域外（field=format={:?}, label={:?}）",
                a.format,
                p.label
            );
        }
    }
    if let Some(frag) = p.fragment.as_ref() {
        for t in frag.targets.iter().flatten() {
            if let Some(bs) = t.blend {
                // desc-align 法窄：双源混合因子 WebGL2 无法能力=创建拒止。
                assert!(
                    !bs.color.src_factor.ref_second_blend_source()
                        && !bs.color.dst_factor.ref_second_blend_source()
                        && !bs.alpha.src_factor.ref_second_blend_source()
                        && !bs.alpha.dst_factor.ref_second_blend_source(),
                    "entry 断言 ErrBlendDual: 双源混合因子不提供（label={:?}）",
                    p.label
                );
            }
        }
    }
    // 槽表=本管线专属（create 期定格并随管线下发）：设备级槽表只作「最近一次分配」的查询面，
    // 若绘制期读它，则「后建的任何管线」都会把当前管线的编号覆盖掉=绑定错配。故此处既写设备级
    // （兼容未设管线路径/探针）、也把同一份拷进管线，出码与查槽全走管线自带那份，天然同源。
    let own_slots = {
        let mut slots = d._inner.slots.lock().unwrap_or_else(|e| e.into_inner());
        binds::alloc_slots(p.layout.expect("ErrNoLayout 已前置断言"), &mut slots);
        slots.clone()
    };
    let (vs, fs) = {
        let slots = &own_slots;
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
        if prog.refcount.load(std::sync::atomic::Ordering::SeqCst) == 1 {
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
        _slots: own_slots,
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

/// RangeBounds→字节区间解算（上游 check_buffer_bounds 同口径）：开界=到末尾；
/// 空段/越界/倒置界=断言短码 ErrSlice（长度≥1 即 BufferSize 构造必成）。
pub(crate) fn slice_bounds_impl<S: std::ops::RangeBounds<crate::types::BufferAddress>>(
    total: crate::types::BufferAddress,
    bounds: S,
) -> (
    crate::types::BufferAddress,
    Option<crate::types::BufferSize>,
) {
    use std::ops::Bound;
    let start = match bounds.start_bound() {
        Bound::Included(v) => *v,
        Bound::Excluded(v) => v + 1,
        Bound::Unbounded => 0,
    };
    let end = match bounds.end_bound() {
        Bound::Included(v) => v + 1,
        Bound::Excluded(v) => *v,
        Bound::Unbounded => total,
    };
    assert!(
        start < end && end <= total,
        "entry 断言 ErrSlice: 片段空或越界（start={start} end={end} total={total}）"
    );
    (start, crate::types::BufferSize::new(end - start))
}

#[cfg(test)]
mod send_sync_contract_tests {
    //! 公开句柄全面的 Send+Sync 合同断言（上游 static_assert_impl_all! 的本层等价物——
    //! 类型界即证据，消费方泛型可依赖；GL 调用线程安全仍由附着门控守护）。
    fn assert_send_sync<T: Send + Sync>() {}
    #[test]
    fn public_handles_are_send_sync_test() {
        use crate::entry::*;
        assert_send_sync::<Instance>();
        assert_send_sync::<Adapter>();
        assert_send_sync::<Device>();
        assert_send_sync::<Queue>();
        assert_send_sync::<Buffer>();
        assert_send_sync::<Texture>();
        assert_send_sync::<TextureView>();
        assert_send_sync::<Sampler>();
        assert_send_sync::<BindGroupLayout>();
        assert_send_sync::<BindGroup>();
        assert_send_sync::<PipelineLayout>();
        assert_send_sync::<ShaderModule>();
        assert_send_sync::<RenderPipeline>();
        assert_send_sync::<CommandEncoder>();
        assert_send_sync::<CommandBuffer>();
        assert_send_sync::<crate::surface::Surface<'static>>();
        assert_send_sync::<crate::surface::SurfaceTexture>();
    }
}

#[cfg(test)]
mod sampler_map_tests {
    use super::*;
    use crate::types::FilterMode as FM;

    /// min×mipmap 组合映射四档（GL 标准名值；纯函数无 GL 环境）。
    #[test]
    fn min_filter_gl_table_test() {
        assert_eq!(min_filter_gl(FM::Nearest, FM::Nearest), 0x2700);
        assert_eq!(min_filter_gl(FM::Linear, FM::Nearest), 0x2701);
        assert_eq!(min_filter_gl(FM::Nearest, FM::Linear), 0x2702);
        assert_eq!(min_filter_gl(FM::Linear, FM::Linear), 0x2703);
    }
}
