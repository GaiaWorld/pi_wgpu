use std::sync::Arc;

use crate::wgt;

use super::GLState;

#[derive(Debug)]
pub struct BindGroupLayout {
    state: GLState,
    entries: Arc<[wgt::BindGroupLayoutEntry]>,
}


impl BindGroupLayout {
    pub fn new(
        state: GLState,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> Result<Self, crate::DeviceError> {
    }
}

#[derive(Debug)]
pub(crate) struct BindGroup {
    state: GLState,
    contents: Box<[RawBinding]>,
}

impl BindGroup {
    pub fn new(
        state: GLState,
        desc: &crate::BindGroupDescriptor,
    ) -> Result<Self, crate::DeviceError> {
    }
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