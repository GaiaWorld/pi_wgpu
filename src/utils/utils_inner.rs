// 设计：pi-wgpu-rebuild
//! utils 芯：便捷构造与环境读取。组合面零新契约——失败/断言=被组合双方原样成立。
use crate::entry::{Buffer, Device, Queue, Texture};
use crate::types::{
    Backends, BufferDescriptor, Origin3d, PowerPreference, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDataOrder, TextureDescriptor,
};
use crate::utils::BufferInitDescriptor;

pub(crate) fn buffer_init_impl(dev: &Device, d: &BufferInitDescriptor<'_>) -> Buffer {
    let b = dev.create_buffer(&BufferDescriptor {
        label: d.label,
        usage: d.usage,
        size: d.contents.len() as u64,
        mapped_at_creation: false,
    });
    let q = Queue {
        _dev: Some(Device {
            _inner: dev._inner.clone(),
        }),
    };
    q.write_buffer(&b, 0, d.contents);
    b
}

pub(crate) fn tex_init_impl(
    dev: &Device,
    q: &Queue,
    d: &TextureDescriptor<'_>,
    _order: TextureDataOrder,
    data: &[u8],
) -> Texture {
    let t = dev.create_texture(d);
    let dst = TexelCopyTextureInfo {
        texture: &t,
        mip_level: 0,
        origin: Origin3d::default(),
        aspect: TextureAspect::All,
    };
    let layout = TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: None,
        rows_per_image: None,
    };
    q.write_texture(dst, data, layout, d.size);
    t
}

pub(crate) fn power_env_impl() -> PowerPreference {
    match std::env::var("WGPU_POWER_PREF").ok().as_deref() {
        Some("low") => PowerPreference::LowPower,
        Some("high") => PowerPreference::HighPerformance,
        _ => PowerPreference::default(),
    }
}

pub(crate) fn backend_bits_impl() -> Option<Backends> {
    Some(Backends::GL)
}

pub(crate) fn parse_impl(list: &str) -> Backends {
    for tok in list.split(',') {
        if tok.trim().eq_ignore_ascii_case("gl") {
            return Backends::GL;
        }
    }
    Backends::empty()
}

/// 压缩纹 js 段直建芯（Web 专面）：绑定专用单元 jsobj 零拷贝灌段（ArrayBufferView 原对象进
/// compressedTexSubImage2D——JS 侧无字节复制）。层形合同：单层 2D（1 层）/立方体（逐面）；
/// 数组/3D=CompressedArray 留痕（ES 无多层压缩形——上传芯同源约定）。尺寸=块公式 level 尺寸
/// （max(块, roundup(基>>mip, 块))）——与上传守门同源；段字节=块数×block_bytes，形不符=留痕跳段。
#[cfg(target_arch = "wasm32")]
pub(crate) fn compress_jsdata_impl(
    dev: &Device,
    q: &Queue,
    d: &TextureDescriptor<'_>,
    order: TextureDataOrder,
    data: &[js_sys::Object],
) -> Texture {
    use crate::upload::UploadErr;
    use glow::HasContext as _;
    // GL 目标/单元常量与上传芯同源规范值（独立文件不复用 import 面）。
    const TEX2D: u32 = 0x0DE1;
    const CUBE: u32 = 0x8513;
    const CUBE_PX: u32 = 0x8515;
    const UNIT0: u32 = 0x84C0;
    const UPLOAD_UNIT: u32 = crate::upload_inner::UPLOAD_TEX_UNIT;
    let t = dev.create_texture(d);
    let p = &t.params;
    if p.block == (1, 1) {
        log::error!("utils {:?} fmt={:?} 非压缩条目误用 jsdata 面", UploadErr::NotBitmap, d.format);
    }
    let bw = p.block.0.max(1) as u64;
    let bh = p.block.1.max(1) as u64;
    let bb = p.block_bytes as u64;
    let restore: u32 = match q._dev.as_ref() {
        Some(dd) => dd
            ._inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active_texture_unit
            .unwrap_or(0),
        None => 0,
    };
    let mips = d.mip_level_count;
    let layers = d.size.depth_or_array_layers;
    let mut cur: usize = 0;
    for mi in 0..mips {
        for ly in 0..layers {
            let (m, l) = match order {
                TextureDataOrder::MipMajor => (mi, ly),
                TextureDataOrder::LayerMajor => (ly, mi),
            };
            let lw = (((d.size.width >> m) as u64 + bw - 1) / bw * bw).max(bw);
            let lh = (((d.size.height >> m) as u64 + bh - 1) / bh * bh).max(bh);
            let need = (lw / bw) * (lh / bh) * bb;
            let obj: &js_sys::Object = match data.get(cur) {
                Some(o) if !data.is_empty() => o,
                _ => {
                    log::error!("utils {:?} mip={m} layer={l} 段缺失 len={}", UploadErr::OutOfRange, data.len());
                    cur += 1;
                    continue;
                }
            };
            let Some(view) = wasm_bindgen::JsCast::dyn_ref::<js_sys::Uint8Array>(obj) else {
                log::error!("utils {:?} 段非 u8 ArrayBufferView mip={m} layer={l}", UploadErr::NotBitmap);
                cur += 1;
                continue;
            };
            if (view.byte_length() as u64) != need {
                log::error!("utils {:?} 段字节={} 应={} mip={m} layer={l}", UploadErr::OutOfRange, view.byte_length() as u64, need);
                cur += 1;
                continue;
            }
            let target: u32 = match t._target {
                TEX2D if l == 0 => TEX2D,
                CUBE if l < t._layers.min(6) => CUBE_PX + l,
                other => {
                    log::error!("utils {:?} target={other:#x} mip={m} layer={l}", UploadErr::CompressedArray);
                    cur += 1;
                    continue;
                }
            };
            let name = match t._h.name() {
                Some(n) if n.0 != 0 => n,
                _ => {
                    log::error!("utils {:?} 纹名失效", UploadErr::DeadHandle);
                    return t;
                }
            };
            let obj2: js_sys::Object = obj.clone();
            let internal = p.internal_format;
            crate::context_inner::with_gl(|gl| unsafe {
                gl.active_texture(UNIT0 + UPLOAD_UNIT);
                gl.bind_texture(target, Some(crate::gid::texture_from(name.0)));
                gl.compressed_tex_sub_image_2d_jsobj(
                    target,
                    m as i32,
                    0,
                    0,
                    lw as i32,
                    lh as i32,
                    internal,
                    &obj2,
                );
                gl.active_texture(UNIT0 + restore);
            });
            cur += 1;
        }
    }
    t
}
