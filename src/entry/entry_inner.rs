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
/// GL_MAX_SAMPLES（多重采样能力顶查询，桌面 3.2+/ES3.1 EXT）。
const GL_MAX_SAMPLES: u32 = 0x8D57;
// LOD clamps are sampler-object params; 0x813A/0x813B are the spec values
// (0x8130/0x8131 are not valid pnames -> INVALID_ENUM 0x500 on every call).
const GL_TEXTURE_MIN_LOD: u32 = 0x813A;
const GL_TEXTURE_MAX_LOD: u32 = 0x813B;

const GL_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FE;
const GL_MAX_TEXTURE_MAX_ANISOTROPY_EXT: u32 = 0x84FF;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_TEXTURE_COMPARE_MODE: u32 = 0x884C;
const GL_TEXTURE_COMPARE_FUNC: u32 = 0x884D;
const GL_COMPARE_REF_TO_TEXTURE: i32 = 0x884E;
/// GL_COPY_WRITE_BUFFER（通用子写/分配目标；与 upload 芯同源）。
// COPY_WRITE_BUFFER is 0x8F37 (0x8F3F belongs to the copy-buffer binding group).
const GL_COPY_WRITE_BUFFER: u32 = 0x8F37;
/// GL_DYNAMIC_DRAW（存储用途提示）。
const GL_DYNAMIC_DRAW: u32 = 0x88E8;
/// 缓冲支持域（INDEX|VERTEX|UNIFORM|COPY_DST|COPY_SRC 按上游位值合成；MAP/STORAGE 词位在场但域外拒止——
/// COPY_SRC 实消费=b2t 源读取/get_sub_data 读回路，无 map 家族）。
const BUFFER_USAGE_MASK: crate::types::BufferUsages = crate::types::BufferUsages::COPY_DST
    .union(crate::types::BufferUsages::COPY_SRC)
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
            error_state: ShareMutex::new(crate::error_inner::ErrorState::default()),
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
    // 深度族=渲染/采样可用但 copy 路硬拒（readPixels DEPTH_COMPONENT 本机 0x502 实测定论，
    // upload ErrCopyDepth）——能力面如实报 RENDER_ATTACHMENT only，不虚报 COPY_DST。
    let depth_family = matches!(
        f,
        TextureFormat::Depth24PlusStencil8 | TextureFormat::Depth32Float
    );
    let allowed = if depth_family {
        TextureUsages::RENDER_ATTACHMENT
    } else if supported {
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
    if t.sample_count > 1 {
        assert!(
            matches!(t.dimension, TextureDimension::D2)
                && t.size.depth_or_array_layers == 1
                && t.mip_level_count == 1,
            "entry 断言 ErrMsShape: 多重采样仅 D2 单层无 mip 链（dim={:?} layers={} mips={} label={:?}）",
            t.dimension,
            t.size.depth_or_array_layers,
            t.mip_level_count,
            t.label
        );
    }
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
        // 存储新鲜度：先清可能遗留的前序操作错误（如上传 RowPitch 非对齐），
        // 防粘进本纹理错误探测造成假「存储被拒」误诊。
        let mut stale_codes: Vec<u32> = Vec::new();
        loop {
            let code = gl.get_error() as u32;
            if code == 0 {
                break;
            }
            stale_codes.push(code);
            if stale_codes.len() > 8 {
                break;
            }
        }
        gl.bind_texture(target, Some(glow::NativeTexture(nz(name))));
        let e_bind = gl.get_error() as u32;
        if e_bind != 0 || !stale_codes.is_empty() {
            log::error!(
                "entry: 绑定/前序 err bind={:#x} stale={:?} name={} target={:#x}（label={:?}）",
                e_bind, stale_codes, name.0, target, t.label
            );
        }
        match target {
            GL_TEXTURE_3D | GL_TEXTURE_2D_ARRAY | GL_TEXTURE_CUBE_MAP_ARRAY => {
                if crate::draw_inner::is_depth_format(t.format) {
                    // 深度族数组/3D：texStorage3D 在本上下文对深度 internal 直接 0x502
                    // （图集深度纹理实锤）——改经典 texImage3D 逐 mip 置存储（全代通用）。
                    for lvl in 0..t.mip_level_count {
                        gl.tex_image_3d(
                            target,
                            lvl as i32,
                            params.internal_format as i32,
                            (t.size.width >> lvl).max(1) as i32,
                            (t.size.height >> lvl).max(1) as i32,
                            layers as i32,
                            0,
                            params.format,
                            params.pixel_type,
                            None::<&[u8]>,
                        );
                    }
                } else {
                    gl.tex_storage_3d(
                        target,
                        t.mip_level_count as i32,
                        params.internal_format,
                        t.size.width as i32,
                        t.size.height as i32,
                        layers as i32,
                    );
                }
                let serr = gl.get_error() as u32;
                if serr != 0 {
                    log::error!(
                        "entry: 纹理存储被拒(数组) fmt={:?} ifmt={:#x} err={:#x} layers={}（label={:?}）",
                        t.format,
                        params.internal_format,
                        serr,
                        layers,
                        t.label
                    );
                }
            }
            _ => {
                if t.sample_count > 1 {
                    let maxs = gl.get_parameter_i32(GL_MAX_SAMPLES);
                    assert!(
                        maxs >= t.sample_count as i32,
                        "entry 断言 ErrMsCap: 设备采样顶 {} 低于请求 {}（label={:?}）",
                        maxs,
                        t.sample_count,
                        t.label
                    );
                    gl.tex_storage_2d_multisample(
                        target,
                        t.sample_count as i32,
                        params.internal_format,
                        t.size.width as i32,
                        t.size.height as i32,
                        true,
                    );
                } else if matches!(
                    t.format,
                    crate::types::TextureFormat::Depth16Unorm
                        | crate::types::TextureFormat::Depth24Plus
                        | crate::types::TextureFormat::Depth32Float
                        | crate::types::TextureFormat::Depth24PlusStencil8
                        | crate::types::TextureFormat::Depth32FloatStencil8
                ) {
                    // 深度族=经典 texImage 路（本上下文 texStorage 对深度 internal 实测无效
                    // 置存储 err0/internal0 三证定论 2026-10-19；texImage 全代通用含 ES2）。
                    gl.tex_image_2d(
                        target,
                        0,
                        params.internal_format as i32,
                        t.size.width as i32,
                        t.size.height as i32,
                        0,
                        params.format,
                        params.pixel_type,
                        None::<&[u8]>,
                    );
                } else {
                    while gl.get_error() != 0 {}
                    gl.tex_storage_2d(
                        target,
                        t.mip_level_count as i32,
                        params.internal_format,
                        t.size.width as i32,
                        t.size.height as i32,
                    );
                }
                let serr = gl.get_error() as u32;
                if serr != 0 {
                    log::error!(
                        "entry: 纹理存储被拒 fmt={:?} ifmt={:#x} err={:#x} target={:#x} {}x{} layers={} mips={} samples={} tid={:?}（label={:?}）",
                        t.format,
                        params.internal_format,
                        serr,
                        target,
                        t.size.width,
                        t.size.height,
                        layers,
                        t.mip_level_count,
                        t.sample_count,
                        std::thread::current().id(),
                        t.label
                    );
                }
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
        _surface: false,
    }
}

/// 纹理显式销毁：句柄法死名（与末引用 Drop 同通道，先置 None 防重入；物理删除+死名登记）。
pub(crate) fn texture_destroy_impl(tex: &Texture) {
    tex._h.mark_dead();
}

/// 建纹理视图（唯一 @public 视图入口）：维度相容校验 + 视图目标覆盖（Cube/CubeArray）。
pub(crate) fn create_view_impl(tex: &Texture, d: &TextureViewDescriptor<'_>) -> TextureView {
    crate::state_inner::check_err("create_view");
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
    // Surface backbuffer: no GL texture object (placeholder name 0); view must stay a frame view
    // (same shape as SurfaceTexture::view), otherwise the attachment gate reports a dead handle.
    if tex._surface {
        return TextureView {
            _tex: None,
            _target: 0,
            _dim: dim,
        };
    }
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
    crate::state_inner::check_err("create_sampler");
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
        crate::state_inner::check_err("s_min_filter");
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_S, addr_gl(s.address_mode_u));
        gl.sampler_parameter_i32(sm, GL_TEXTURE_WRAP_T, addr_gl(s.address_mode_v));
        crate::state_inner::check_err("s_wrap");
        if let Some(cf) = s.compare {
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_MODE, GL_COMPARE_REF_TO_TEXTURE);
            gl.sampler_parameter_i32(sm, GL_TEXTURE_COMPARE_FUNC, compare_gl(cf));
        }
        // MIP 夹值直通：上游字段语义（缺省 -1000/32 与 GL 内部约定同值；实际采样层
        // 硬件再按 [min, min(max, levels-1)] 收拢——25 形无 lod_bias 字段，GL 侧不掷偏置。
        gl.sampler_parameter_f32(sm, GL_TEXTURE_MIN_LOD, s.lod_min_clamp);
        gl.sampler_parameter_f32(sm, GL_TEXTURE_MAX_LOD, s.lod_max_clamp);
        crate::state_inner::check_err("s_lod");
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
    crate::state_inner::check_err("s_return");
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
    crate::state_inner::check_err("create_bind_group");
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
    crate::state_inner::check_err("create_pipeline");
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
    let mut own_slots = {
        let mut slots = d._inner.slots.lock().unwrap_or_else(|e| e.into_inner());
        binds::alloc_slots(p.layout.expect("ErrNoLayout 已前置断言"), &mut slots);
        slots.clone()
    };
    // 配对表随管线定格（出码期自 IR 收集，与应用期同源）。
    own_slots.alias = {
        let mut v = crate::shader::sampler_pairs(&p.vertex.module._compiled);
        if let Some(f) = p.fragment.as_ref() {
            for pr in crate::shader::sampler_pairs(&f.module._compiled) {
                if !v.contains(&pr) {
                    v.push(pr);
                }
            }
        }
        v
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
    // 采样入口指位表（设计：sampler-unit-assign）：候选=反射里的纹理类绑定（naga 对 ES300
    // 不下 sampler 全局，合并后的 sampler2D 即纹理全局）；位置查询要触 GL，故在创建期一次收齐，
    // 切换期只发整数。表随管线定格，切换时经镜像与水位成对下发。
    let sampler_uniforms = {
        // 反射源自产物 IR 现取（模块 `_refl` 只在测试里填过；生产路径以 IR 为准）。
        let mut targets: Vec<(u32, u32)> = Vec::new();
        for blob in core::iter::once(&p.vertex.module._compiled)
            .chain(p.fragment.as_ref().map(|f| &f.module._compiled))
        {
            match shader::reflection_of(blob) {
                Ok(refl) => {
                    for t in shader::sampler_targets(&refl) {
                        if !targets.contains(&t) {
                            targets.push(t);
                        }
                    }
                }
                Err(e) => log::error!("shader {e:?}: 反射收集失败（采样指位降级为空表）"),
            }
        }
        targets.sort_unstable();
        targets.dedup();
        collect_sampler_uniforms(
            prog.handle.expect("entry 断言: 新程序无句柄"),
            &own_slots,
            &targets,
        )
    };
    // IR 侧「类型名+阶段 → (set,binding)」：GL 块名由类型名派生（registry naga 命名），
    // 用它把块索引反查回绑定点。
    let mut ir_keys: Vec<(String, (u32, u32))> = Vec::new();
    for (blob, tag) in core::iter::once((&p.vertex.module._compiled, "Vertex"))
        .chain(p.fragment.as_ref().map(|f| (&f.module._compiled, "Fragment")))
    {
        for ir in blob.modules.iter() {
            // 复算 naga Namer 的类型名：匿名类型走 "type"/"type_N" 序号（与出码同名）。
            let mut ty_names: Vec<(naga::Handle<naga::Type>, String)> = Vec::new();
            let mut anon: u32 = 0;
            for (h, ty) in ir.module.types.iter() {
                let nm = match &ty.name {
                    Some(n) => n.clone(),
                    None => {
                        let s = if anon == 0 {
                            String::from("type")
                        } else {
                            format!("type_{anon}")
                        };
                        anon += 1;
                        s
                    }
                };
                ty_names.push((h, nm));
            }
            for (_h, gv) in ir.module.global_variables.iter() {
                if gv.space != naga::AddressSpace::Uniform {
                    continue;
                }
                let Some(rb) = gv.binding else { continue };
                let Some((_, tn)) = ty_names.iter().find(|(h, _)| *h == gv.ty) else {
                    continue;
                };
                ir_keys.push((format!("{tn}_{tag}"), (rb.group, rb.binding)));
            }
        }
    }
    // UBO 块指位：ES300 无 `binding=` 限定符，块绑定必须由程序态指定（否则全部停在 0 号）。
    let block_bindings =
        collect_block_bindings(prog.handle.expect("entry 断言: 新程序无句柄"), &own_slots, &ir_keys);
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
        _depth_stencil: p.depth_stencil.clone(),
        _primitive: p.primitive.clone(),
        _slots: own_slots,
        _sampler_uniforms: sampler_uniforms,
        _block_bindings: block_bindings,
    }
}

/// 资源类全局名解析：naga 出码名为 `_group_{G}_binding_{B}_{stage}`（stage 后缀因阶段而异，
/// 同一绑定在 VS/FS 各出一条），故解析取前两段、容忍后缀；非资源类名= None。
fn parse_group_binding(name: &str) -> Option<(u32, u32)> {
    let rest = name.strip_prefix("_group_")?;
    let (g, rest) = rest.split_once("_binding_")?;
    let b = rest.split_once('_').map(|(head, _)| head).unwrap_or(rest);
    Some((g.parse().ok()?, b.parse().ok()?))
}

/// 采样入口 → 驱动侧指位表（设计：sampler-unit-assign 切片 3；契约见该设计 07-design/api.md）。
/// 只收 targets 命中的名字，按**名字**逐条入表（不按 (group,binding) 去重：VS/FS 同采一图
/// 是两条 uniform、两个 location、同一 unit）；单条不可得记短码后跳过，整体不失败。
fn collect_sampler_uniforms(
    program: GlHandle,
    table: &crate::binds::SlotTable,
    targets: &[(u32, u32)],
) -> Vec<crate::binds::SamplerUniform> {
    if targets.is_empty() {
        return Vec::new();
    }

    crate::context_inner::with_gl(|gl| unsafe {
        let prog = glow::NativeProgram(nz(program));
        let n = gl.get_active_uniforms(prog);
        let mut out: Vec<crate::binds::SamplerUniform> = Vec::with_capacity(n as usize);
        for i in 0..n {
            let Some(au) = gl.get_active_uniform(prog, i) else {
                continue;
            };
            let Some((g, b)) = parse_group_binding(&au.name) else {
                continue;
            };
            if !targets.contains(&(g, b)) {
                continue;
            }
            let Some(unit) = crate::binds::unit_of(table, g, b) else {
                log::error!("SamplerUnitMissing set={g} binding={b}");
                continue;
            };
            let Some(loc) = gl.get_uniform_location(prog, &au.name) else {
                continue;
            };

            out.push(crate::binds::SamplerUniform {
                location: loc.0 as i32,
                unit,
            });
        }
        out
    })
}

/// UBO 块 → 驱动侧块指位表：按出码期带回的「(set,binding) → 产物名」名表逐个向程序查块索引，
/// 绑定点取本管线槽表（与 apply_impl 同源）。ES300 无 `binding=`，块绑定只能经此程序态指定，
/// 否则所有块停在 0 号绑定点（set>0 的材质块读到垃圾）。整体不失败。
fn collect_block_bindings(
    program: GlHandle,
    table: &crate::binds::SlotTable,
    ir_keys: &[(String, (u32, u32))],
) -> Vec<crate::binds::BlockBinding> {
    crate::context_inner::with_gl(|gl| unsafe {
        let prog = glow::NativeProgram(nz(program));
        let mut out: Vec<crate::binds::BlockBinding> = Vec::new();
        for i in 0..64u32 {
            let name = gl.get_active_uniform_block_name(prog, i);
            if name.is_empty() {
                break;
            }
            // 两种名形：`_group_G_binding_B_<stage>`（本地 naga）或 `<类型名>_block_<id><Stage>`
            // （registry naga）。后者按「类型名 + 阶段」回查 IR 的 (set,binding)。
            let key = parse_group_binding(&name).or_else(|| {
                let at = name.rfind("_block_")?;
                let prefix = &name[..at];
                let tail = &name[at + 7..];
                let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
                let stage = &tail[digits.len()..];
                let want = format!("{prefix}_{stage}");
                ir_keys
                    .iter()
                    .find(|(k, _)| *k == want)
                    .map(|(_, v)| *v)
            });
            let Some((g, b)) = key else {
                log::error!("BlockBindingUnparsed index={i} name={name}");
                continue;
            };
            let Some(binding) = crate::binds::block_binding(table, g, b) else {
                log::error!("BlockBindingMissing set={g} binding={b} name={name}");
                continue;
            };

            out.push(crate::binds::BlockBinding { index: i, binding });
        }
        out
    })
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
        _commands: Vec::new(),
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

pub(crate) fn encoder_finish(mut e: CommandEncoder) -> CommandBuffer {
    CommandBuffer {
        _commands: std::mem::take(&mut e._commands),
    }
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

/// 注册未捕获错误回调（Device::on_uncaptured_error 芯）。
pub(crate) fn on_uncaptured_error_impl(
    dev: &crate::entry::Device,
    handler: Box<dyn crate::error::UncapturedErrorHandler>,
) {
    let mut st = dev
        ._inner
        .error_state
        .lock()
        .unwrap_or_else(|x| x.into_inner());
    st.uncaptured = Some(handler);
}

/// 开启错误作用域（Device::push_error_scope 芯）。
pub(crate) fn push_error_scope_impl(dev: &crate::entry::Device, filter: crate::error::ErrorFilter) {
    let mut st = dev
        ._inner
        .error_state
        .lock()
        .unwrap_or_else(|x| x.into_inner());
    st.push(filter);
}

/// 关闭最近作用域取回首条捕获（Device::pop_error_scope 芯，ready future=上游 async 形）。
pub(crate) fn pop_error_scope_impl(
    dev: &crate::entry::Device,
) -> impl std::future::Future<Output = Option<crate::error::Error>> + Send {
    let inner = dev._inner.clone();
    async move {
        let mut st = inner.error_state.lock().unwrap_or_else(|x| x.into_inner());
        st.pop()
    }
}

/// 拒绝/降级错误上报通道（内部 helper）：最内匹配 scope 捕获；无捕获走 uncaptured 回调；
/// 再无=默认留痕降级（不 panic=本层纪律差异）。返回 true=通道或回调接管。
pub(crate) fn report_error(inner: &crate::entry::DeviceInner, err: crate::error::Error) -> bool {
    let mut st = inner.error_state.lock().unwrap_or_else(|x| x.into_inner());
    if st.capture(&err) {
        return true;
    }
    if let Some(h) = st.uncaptured.as_ref() {
        h(err);
        return true;
    }
    drop(st);
    log::error!("pi_wgpu 未捕获错误（通道空，降级留痕）: {err}");
    false
}

impl crate::entry::DeviceInner {
    /// 错误态短临界访问（通道内部/测试用）。
    pub(crate) fn with_error<R>(
        &self,
        f: impl FnOnce(&mut crate::error_inner::ErrorState) -> R,
    ) -> R {
        let mut st = self.error_state.lock().unwrap_or_else(|x| x.into_inner());
        f(&mut st)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::parse_group_binding;
    use glow::HasContext;

    /// 资源名解析：带/不带 stage 后缀均可；非资源名= None（collect 的静默跳过路径）。
    #[test]
    fn parse_group_binding_test() {
        assert_eq!(parse_group_binding("_group_0_binding_1_fragment"), Some((0, 1)));
        assert_eq!(parse_group_binding("_group_2_binding_13_vertex"), Some((2, 13)));
        assert_eq!(parse_group_binding("_group_0_binding_3"), Some((0, 3)), "无后缀也接受");
        assert_eq!(parse_group_binding("color"), None, "普通 uniform 不进表");
        assert_eq!(parse_group_binding("_group_x_binding_1_vertex"), None, "非数字=跳过");
        assert_eq!(parse_group_binding("_push_constant_binding_vertex"), None);
    }

    /// 切片 3 真 GL（由全链夹具调用；本进程只允许一次 attach，勿另立 #[test]）：VS/FS 同采一图 →
    /// 两条 location、同一 unit（关 L9 关键坑 1 与 L0 待验证动态面）。
    pub(crate) fn gl_two_stage_collect_check() {
        use crate::types::{
            BindGroupLayoutEntry, BindingType, SamplerBindingType, ShaderStages, TextureSampleType,
            TextureViewDimension,
        };
        // 布局：set0 = 纹理 binding0 + 采样器 binding1（纹理占单元 0）。
        let layout = crate::entry::PipelineLayout {
            _bgls: vec![crate::entry::BindGroupLayout {
                _entries: vec![
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    },
                ],
            }],
        };
        let mut table = crate::binds::SlotTable::default();
        crate::binds::alloc_slots(&layout, &mut table);
        let targets = vec![(0u32, 0u32)];
        let mut cache = crate::objects::HandleCache::default();
        let prog =
            crate::objects::ensure_reuse(crate::objects::ReuseKey::Program(0x53_0000_01), &mut cache);
        let name = prog.handle.expect("新程序应有句柄");
        let vs_src: String = [
            "#version 300 es",
            "in vec2 a_uv;",
            "uniform sampler2D _group_0_binding_0_vertex;",
            "void main() {",
            "    vec4 t = textureLod(_group_0_binding_0_vertex, a_uv, 0.0);",
            "    gl_Position = vec4(a_uv + t.xy * 0.001, 0.0, 1.0);",
            "}",
        ]
        .join("\n");
        let fs_src: String = [
            "#version 300 es",
            "precision highp float;",
            "uniform sampler2D _group_0_binding_0_fragment;",
            "out vec4 o;",
            "void main() { o = texture(_group_0_binding_0_fragment, vec2(0.5)); }",
        ]
        .join("\n");
        super::link_program(name, &vs_src, Some(&fs_src));
        let got = super::collect_sampler_uniforms(name, &table, &targets);
        let unit = crate::binds::unit_of(&table, 0, 0).expect("纹理在槽表");
        assert_eq!(got.len(), 2, "VS/FS 各一条（不按 (group,binding) 去重）");
        assert!(
            got.iter().all(|u: &crate::binds::SamplerUniform| u.unit == unit),
            "同一 (g,b) 的两条 uniform 指向同一单元"
        );
        assert!(got
            .iter()
            .all(|u: &crate::binds::SamplerUniform| u.location >= 0));
        assert_ne!(got[0].location, got[1].location, "两阶段各得自己的位置");
    }

    /// 切片 5 真 GL 端到端（由全链夹具调用）：指位（collect 产物）经 `StateMirror` 水位真实下发后，
    /// 采样确实读到目标单元——1×1 洋红纹理 + 全屏三角，回读中心=源色（容差 0）。
    pub(crate) fn gl_sampler_unit_check() {
        use crate::binds::{alloc_slots, SlotTable};
        use crate::types::{
            BindGroupLayoutEntry, BindingType, SamplerBindingType, ShaderStages, TextureSampleType,
            TextureViewDimension,
        };
        const TEX0: u32 = 0x84C0;
        const TEX2D: u32 = 0x0DE1;
        const RGBA8: i32 = 0x8058;
        const RGBA: u32 = 0x1908;
        const UBYTE: u32 = 0x1401;
        const NEAREST: i32 = 0x2600;
        const MINF: u32 = 0x2801;
        const MAGF: u32 = 0x2800;
        const WRAPS: u32 = 0x2802;
        const WRAPT: u32 = 0x2803;
        const CLAMP: i32 = 0x812F;
        const FB: u32 = 0x8D40;
        const COLOR0: u32 = 0x8CE0;
        const TRIANGLES: u32 = 0x0004;
        const COLOR_BIT: u32 = 0x4000;

        let layout = crate::entry::PipelineLayout {
            _bgls: vec![crate::entry::BindGroupLayout {
                _entries: vec![
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        count: None,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    },
                ],
            }],
        };
        let mut table = SlotTable::default();
        alloc_slots(&layout, &mut table);
        let unit = crate::binds::unit_of(&table, 0, 0).expect("纹理应在槽表");

        let mut cache = crate::objects::HandleCache::default();
        let prog = crate::objects::ensure_reuse(
            crate::objects::ReuseKey::Program(0x53_0000_05),
            &mut cache,
        );
        let name = prog.handle.expect("程序句柄");
        let vs_src: String = [
            "#version 300 es",
            "void main() {",
            "    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));",
            "    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);",
            "}",
        ]
        .join("\n");
        let fs_src: String = [
            "#version 300 es",
            "precision highp float;",
            "uniform sampler2D _group_0_binding_0_fragment;",
            "out vec4 o;",
            "void main() { o = texture(_group_0_binding_0_fragment, vec2(0.5)); }",
        ]
        .join("\n");
        super::link_program(name, &vs_src, Some(&fs_src));

        let got = super::collect_sampler_uniforms(name, &table, &[(0u32, 0u32)]);
        assert_eq!(got.len(), 1, "本程序单一采样入口");
        assert_eq!(got[0].unit, unit, "collect 的单元=槽表算出的单元");

        // 先走镜像水位下发（flush 会按空镜像把 16 单元解绑，早于手工绑定执行才不互相冲掉），
        // 再备资源：期望态进镜像 → 程序真换 → useProgram + 全量指位。
        // 与产品同路径：期望态进镜像 → 水位差异下发 useProgram + 全量指位。
        let mut m = crate::state::StateMirror::default();
        m.program = Some(crate::objects::GlHandle(name.0));
        m.sampler_uniforms = got.clone();
        m.flush_if_changed();
        assert_eq!(m.water.program, Some(name.0), "程序水位推进=useProgram+指位已发");

        // 资源准备（源纹理与采样器落在目标单元；渲染目标另占末端单元，免覆盖源）。
        let (tex, smp, rt, fb, vao) = crate::context_inner::with_gl(|gl| unsafe {
            let tex = gl.create_texture().expect("源纹理");
            let smp = gl.create_sampler().expect("采样器");
            let rt = gl.create_texture().expect("渲染目标");
            let fb = gl.create_framebuffer().expect("FBO");
            let vao = gl.create_vertex_array().expect("VAO");
            // 前序上传探针可能遗留 PBO/对齐态：不复位会让 texImage2D 从缓冲取数（上传静默变黑）。
            while gl.get_error() != 0 {}
            gl.bind_buffer(0x88EC, None); // PIXEL_UNPACK_BUFFER
            gl.bind_buffer(0x88EB, None); // PIXEL_PACK_BUFFER
            gl.pixel_store_i32(0x0CF5, 1); // UNPACK_ALIGNMENT
            gl.active_texture(TEX0 + 15);
            gl.bind_texture(TEX2D, Some(rt));
            gl.tex_image_2d(TEX2D, 0, RGBA8, 1, 1, 0, RGBA, UBYTE, None);
            gl.tex_parameter_i32(TEX2D, MINF, NEAREST);
            gl.active_texture(TEX0 + unit);
            gl.bind_texture(TEX2D, Some(tex));
            gl.tex_image_2d(TEX2D, 0, RGBA8, 1, 1, 0, RGBA, UBYTE, Some(&[255, 0, 255, 255]));
            gl.tex_parameter_i32(TEX2D, MINF, NEAREST);
            gl.tex_parameter_i32(TEX2D, MAGF, NEAREST);
            gl.sampler_parameter_i32(smp, MINF, NEAREST);
            gl.sampler_parameter_i32(smp, MAGF, NEAREST);
            gl.sampler_parameter_i32(smp, WRAPS, CLAMP);
            gl.sampler_parameter_i32(smp, WRAPT, CLAMP);
            gl.bind_sampler(unit, Some(smp));
            gl.bind_framebuffer(FB, Some(fb));
            gl.framebuffer_texture_2d(FB, COLOR0, TEX2D, Some(rt), 0);
            let e = gl.get_error();
            assert_eq!(e, 0, "纹理/FBO 准备期驱动 error {e:#x}");
            (tex, smp, rt, fb, vao)
        });

        let px: [u8; 4] = crate::context_inner::with_gl(|gl| unsafe {
            // 探针自带状态复位：前序探针留下的裁剪/掩码/丢弃态会吞掉绘制。
            gl.disable(0x0C11); // SCISSOR_TEST
            gl.disable(0x0B71); // DEPTH_TEST
            gl.disable(0x0BE2); // BLEND
            gl.disable(0x0B44); // CULL_FACE
            gl.disable(0x8C89); // RASTERIZER_DISCARD
            gl.color_mask(true, true, true, true);
            gl.use_program(Some(glow::NativeProgram(super::nz(name))));
            gl.bind_framebuffer(FB, Some(fb));
            assert_eq!(gl.check_framebuffer_status(FB), 0x8CD5, "FBO 应完整");
            gl.viewport(0, 0, 1, 1);
            gl.clear_color(0.0, 1.0, 0.0, 1.0); // 绿清屏：与源色洋红区分「没画到」与「采到黑」
            gl.clear(COLOR_BIT);
            gl.bind_vertex_array(Some(vao));
            gl.draw_arrays(TRIANGLES, 0, 3);
            let mut out = [0u8; 4];
            gl.read_pixels(0, 0, 1, 1, RGBA, UBYTE, glow::PixelPackData::Slice(&mut out));
            gl.bind_framebuffer(FB, None);
            gl.bind_vertex_array(None);
            gl.bind_sampler(unit, None);
            gl.delete_vertex_array(vao);
            gl.delete_framebuffer(fb);
            gl.delete_texture(rt);
            gl.delete_texture(tex);
            gl.delete_sampler(smp);
            out
        });
        assert_eq!(px, [255, 0, 255, 255], "回读中心=源色：采样器读到的正是指位单元");
    }

    /// 块内数值 uniform 数组（批渲形态）：一个 UBO 块持 4 个材质色，4 个实例各按
    /// gl_InstanceID 取自己那格 → 同一次绘制回读 4 列=4 种颜色。
    /// 同时覆盖：绑定点非 0（set2→8）、子区间绑定、std140 数组步进。
    pub(crate) fn gl_ubo_array_check() {
        use glow::HasContext;
        const FB: u32 = 0x8D40;
        const COLOR0: u32 = 0x8CE0;
        const RGBA8: i32 = 0x8058;
        const RGBA: u32 = 0x1908;
        const UBYTE: u32 = 0x1401;
        const TRI_STRIP: u32 = 0x0005;
        const COLOR_BIT: u32 = 0x4000;
        const UNIFORM_BUFFER: u32 = 0x8A11;
        const STATIC_DRAW: u32 = 0x88E4;
        const TEX2D: u32 = 0x0DE1;
        const TEX0: u32 = 0x84C0;
        const NEAREST: i32 = 0x2600;
        const MINF: u32 = 0x2801;
        let layout = crate::entry::PipelineLayout {
            _bgls: vec![
                crate::entry::BindGroupLayout { _entries: Vec::new() },
                crate::entry::BindGroupLayout { _entries: Vec::new() },
                crate::entry::BindGroupLayout {
                    _entries: vec![crate::types::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: crate::types::ShaderStages::VERTEX,
                        count: None,
                        ty: crate::types::BindingType::Buffer {
                            ty: crate::types::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                    }],
                },
            ],
        };
        let mut table = crate::binds::SlotTable::default();
        crate::binds::alloc_slots(&layout, &mut table);
        let binding_point = crate::binds::block_binding(&table, 2, 0).expect("set2/bind0 应在 UBO 槽表");
        assert_eq!(binding_point, 0, "紧凑编号：本布局唯一 UBO 落 0 号绑定点（无 set 基址跳号）");

        let vs: String = [
            "#version 300 es",
            "uniform MatParams { vec4 colors[4]; } mp;",
            "out vec4 v;",
            "void main() {",
            "    float cx = (float(gl_InstanceID) + 0.5) / 4.0;",
            "    vec2 p = vec2(float(gl_VertexID & 1), float((gl_VertexID >> 1) & 1));",
            "    gl_Position = vec4((p.x - 0.5) * 0.24 + cx * 2.0 - 1.0, (p.y - 0.5) * 1.6, 0.0, 1.0);",
            "    v = mp.colors[gl_InstanceID];",
            "}",
        ]
        .join("\n");
        let fs: String = [
            "#version 300 es",
            "precision highp float;",
            "in vec4 v;",
            "out vec4 o;",
            "void main() { o = v; }",
        ]
        .join("\n");
        let mut cache = crate::objects::HandleCache::default();
        let prog = crate::objects::ensure_reuse(
            crate::objects::ReuseKey::Program(0x53_0000_07),
            &mut cache,
        );
        let name = prog.handle.expect("程序句柄");
        super::link_program(name, &vs, Some(&fs));

        let px: Vec<[u8; 4]> = crate::context_inner::with_gl(|gl| unsafe {
            while gl.get_error() != 0 {}
            gl.bind_buffer(0x88EC, None);
            gl.bind_buffer(0x88EB, None);
            gl.disable(0x0C11);
            gl.disable(0x0B71);
            gl.disable(0x0BE2);
            gl.disable(0x0B44);
            gl.disable(0x8C89);
            gl.color_mask(true, true, true, true);
            // 材质数据：4 个实例各自一格
            // 4 个 vec4 = 16 个 f32（std140 步长 16 字节），按实例索引各取一格。
            let vals: [f32; 16] = [
                1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0,
            ];
            let data: Vec<u8> = vals.iter().flat_map(|v| v.to_ne_bytes()).collect();
            let ubuf = gl.create_buffer().expect("UBO");
            gl.bind_buffer(UNIFORM_BUFFER, Some(ubuf));
            gl.buffer_data_u8_slice(UNIFORM_BUFFER, &data, STATIC_DRAW);
            gl.bind_buffer_range(UNIFORM_BUFFER, binding_point, Some(ubuf), 0, 64);
            let prog_gl = glow::NativeProgram(super::nz(name));
            gl.use_program(Some(prog_gl));
            let block = gl
                .get_uniform_block_index(prog_gl, "MatParams")
                .expect("MatParams 块应存在");
            gl.uniform_block_binding(prog_gl, block, binding_point);

            // 4×4 目标
            let rt = gl.create_texture().expect("目标纹理");
            gl.active_texture(TEX0 + 15);
            gl.bind_texture(TEX2D, Some(rt));
            gl.tex_image_2d(TEX2D, 0, RGBA8, 4, 4, 0, RGBA, UBYTE, None);
            gl.tex_parameter_i32(TEX2D, MINF, NEAREST);
            let fb = gl.create_framebuffer().expect("FBO");
            gl.bind_framebuffer(FB, Some(fb));
            gl.framebuffer_texture_2d(FB, COLOR0, TEX2D, Some(rt), 0);
            assert_eq!(gl.check_framebuffer_status(FB), 0x8CD5, "FBO 应完整");
            gl.viewport(0, 0, 4, 4);
            gl.clear_color(0.0, 0.0, 0.0, 1.0);
            gl.clear(COLOR_BIT);

            let vao = gl.create_vertex_array().expect("VAO");
            gl.bind_vertex_array(Some(vao));
            gl.draw_arrays_instanced(TRI_STRIP, 0, 4, 4);
            let mut out: Vec<[u8; 4]> = Vec::new();
            for x in 0..4 {
                let mut one = [0u8; 4];
                gl.read_pixels(x, 2, 1, 1, RGBA, UBYTE, glow::PixelPackData::Slice(&mut one));
                out.push(one);
            }
            gl.bind_framebuffer(FB, None);
            gl.bind_vertex_array(None);
            gl.delete_vertex_array(vao);
            gl.delete_framebuffer(fb);
            gl.delete_texture(rt);
            gl.delete_buffer(ubuf);
            out
        });
        assert_eq!(px.len(), 4);
        assert_eq!(px[0], [255, 0, 0, 255], "实例0 取数组第0格（红）");
        assert_eq!(px[1], [0, 255, 0, 255], "实例1 取第1格（绿）");
        assert_eq!(px[2], [0, 0, 255, 255], "实例2 取第2格（蓝）");
        assert_eq!(px[3], [255, 255, 0, 255], "实例3 取第3格（黄）");
    }
}
