use std::ops::Range;

use crate::wgt;

use super::{CopyExtent, TextureFormatDesc};

pub(crate) type ShaderID = u64;
pub(crate) type TextureID = u64;

#[derive(Debug)]
pub(crate) struct Texture {
    pub inner: TextureInner,
    pub mip_level_count: u32,
    pub array_layer_count: u32,
    pub format: wgt::TextureFormat,
    
    #[allow(unused)]
    pub format_desc: TextureFormatDesc,
    
    pub copy_size: CopyExtent,
    pub is_cubemap: bool,
}

#[derive(Clone, Debug)]
pub struct TextureView {
    pub(crate) inner: TextureInner,
    pub(crate) sample_type: wgt::TextureSampleType,
    pub(crate) aspects: super::FormatAspects,
    pub(crate) mip_levels: Range<u32>,
    pub(crate) array_layers: Range<u32>,
    pub(crate) format: wgt::TextureFormat,
}

#[derive(Clone, Debug)]
pub(crate) enum TextureInner {
    Renderbuffer {
        raw: glow::Renderbuffer,
    },
    DefaultRenderbuffer,
    Texture {
        raw: glow::Texture,
        target: super::BindTarget,
    },
    #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
    ExternalFramebuffer {
        inner: web_sys::WebGlFramebuffer,
    },
}

// SAFE: WASM doesn't have threads
#[cfg(target_arch = "wasm32")]
unsafe impl Send for TextureInner {}
#[cfg(target_arch = "wasm32")]
unsafe impl Sync for TextureInner {}

impl TextureInner {
    fn as_native(&self) -> (glow::Texture, super::BindTarget) {
        match *self {
            Self::Renderbuffer { .. } | Self::DefaultRenderbuffer => {
                panic!("Unexpected renderbuffer");
            }
            Self::Texture { raw, target } => (raw, target),
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            Self::ExternalFramebuffer { .. } => panic!("Unexpected external framebuffer"),
        }
    }
}
