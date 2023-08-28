use std::sync::Arc;

use crate::wgt;

#[derive(Debug)]
pub struct BindGroupLayout {
    entries: Arc<[wgt::BindGroupLayoutEntry]>,
}

#[derive(Debug)]
pub(crate) struct BindGroup {
    contents: Box<[RawBinding]>,
}

#[derive(Debug)]
pub(crate) struct BindGroupLayoutInfo {
    pub(crate) entries: Arc<[wgt::BindGroupLayoutEntry]>,
    
    /// Mapping of resources, indexed by `binding`, into the whole layout space.
    /// For texture resources, the value is the texture slot index.
    /// For sampler resources, the value is the index of the sampler in the whole layout.
    /// For buffers, the value is the uniform or storage slot index.
    /// For unused bindings, the value is `!0`
    pub(crate) binding_to_slot: Box<[u8]>,
}

#[derive(Debug)]
enum RawBinding {
    Buffer {
        raw: glow::Buffer,
        offset: i32,
        size: i32,
    },
    Texture {
        raw: glow::Texture,
        target: super::BindTarget,
        //TODO: mip levels, array layers
    },
    Image(ImageBinding),
    Sampler(glow::Sampler),
}

#[derive(Clone, Debug)]
struct ImageBinding {
    raw: glow::Texture,
    mip_level: u32,
    array_layer: Option<u32>,
    access: u32,
    format: u32,
}