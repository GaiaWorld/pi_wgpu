use pi_share::Share;

use crate::wgt;

#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) entries: Share<[wgt::BindGroupLayoutEntry]>,
}

impl BindGroupLayout {
    pub fn new(desc: &crate::BindGroupLayoutDescriptor) -> Result<Self, crate::DeviceError> {
        profiling::scope!("hal::BindGroupLayout::new");
        let entries = desc.entries.to_vec().into();
        Ok(Self { entries })
    }
}

#[derive(Debug)]
pub(crate) struct BindGroup {
    pub(crate) contents: Box<[RawBinding]>,
}

impl BindGroup {
    pub fn new(desc: &crate::BindGroupDescriptor) -> Result<Self, crate::DeviceError> {
        profiling::scope!("hal::BindGroup::new");

        let contents = desc
            .entries
            .iter()
            .map(|v| match &v.resource {
                crate::BindingResource::Buffer(b) => {
                    let size: i32 = match b.size {
                        Some(size) => u64::from(size) as i32,
                        None => b.buffer.size as i32,
                    };
                    RawBinding::Buffer {
                        raw: b.buffer.inner.clone(),
                        offset: b.offset as i32,
                        size,
                    }
                }
                crate::BindingResource::Sampler(s) => RawBinding::Sampler(s.inner.clone()),
                crate::BindingResource::TextureView(view) => {
                    RawBinding::Texture(view.inner.clone())
                }
                crate::BindingResource::BufferArray(_) => unimplemented!(),
                crate::BindingResource::SamplerArray(_) => unimplemented!(),
                crate::BindingResource::TextureViewArray(_) => unimplemented!(),
            })
            .collect();

        Ok(Self { contents })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RawBinding {
    Buffer {
        raw: super::Buffer,
        offset: i32,
        size: i32,
    },
    Texture(super::TextureView),
    Sampler(super::Sampler),
}
