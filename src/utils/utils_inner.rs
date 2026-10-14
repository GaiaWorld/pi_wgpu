// 设计：pi-wgpu-rebuild
//! utils 芯：便捷构造与环境读取。组合面零新契约——失败/断言=被组合双方原样成立。
use crate::entry::{Buffer, Device, Queue, Texture};
use crate::types::{Backends, BufferDescriptor, ImageCopyTexture, ImageDataLayout, MemoryHints, Origin3d, PowerPreference, TextureAspect, TextureDataOrder, TextureDescriptor};
use crate::utils::BufferInitDescriptor;

pub(crate) fn buffer_init_impl(dev: &Device, d: &BufferInitDescriptor<'_>) -> Buffer {
    let b = dev.create_buffer(&BufferDescriptor {
        label: d.label,
        usage: d.usage,
        size: d.contents.len() as u64,
        mapped_at_creation: false,
        memory_hints: MemoryHints::default(),
    });
    let q = Queue { _dev: Some(Device { _inner: dev._inner.clone() }) };
    q.write_buffer(&b, 0, d.contents);
    b
}

pub(crate) fn tex_init_impl(dev: &Device, q: &Queue, d: &TextureDescriptor<'_>, _order: TextureDataOrder, data: &[u8]) -> Texture {
    let t = dev.create_texture(d);
    let dst = ImageCopyTexture { texture: &t, mip_level: 0, origin: Origin3d::default(), aspect: TextureAspect::All };
    let layout = ImageDataLayout { offset: 0, bytes_per_row: None, rows_per_image: None };
    q.write_texture(&dst, data, &layout, d.size);
    t
}

pub(crate) fn power_env_impl() -> PowerPreference {
    match std::env::var("WGPU_POWER_PREF").ok().as_deref() {
        Some("low") => PowerPreference::LowPower,
        Some("high") => PowerPreference::HighPerformance,
        _ => PowerPreference::default(),
    }
}

pub(crate) fn backend_bits_impl() -> Option<Backends> { Some(Backends::GL) }

pub(crate) fn parse_impl(list: &str) -> Backends {
    for tok in list.split(',') {
        if tok.trim().eq_ignore_ascii_case("gl") {
            return Backends::GL;
        }
    }
    Backends::EMPTY
}
