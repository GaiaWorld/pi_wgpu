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
    pub(crate) layout: Share<[wgt::BindGroupLayoutEntry]>,
    pub(crate) contents: Box<[RawBinding]>,
}

impl BindGroup {
    pub fn new(desc: &crate::BindGroupDescriptor) -> Result<Self, crate::DeviceError> {
        profiling::scope!("hal::BindGroup::new");

        let layout = desc.layout.inner.entries.as_ref();

        let next_dynamic_offset = -1;

        let contents = desc
            .entries
            .iter()
            .enumerate()
            .map(|(index, v)| {
                let layout = &layout[index];
                assert!(v.binding == layout.binding);

                let binding = match &v.resource {
                    crate::BindingResource::Buffer(b) => {
                        let has_dynamic_offset = match &layout.ty {
                            crate::BindingType::Buffer {
                                has_dynamic_offset, ..
                            } => *has_dynamic_offset,
                            _ => panic!("mis match Buffer type"),
                        };

                        let dynamic_offset = if has_dynamic_offset {
                            next_dynamic_offset += 1;
                            next_dynamic_offset
                        } else {
                            -1
                        };

                        let size: i32 = match b.size {
                            Some(size) => u64::from(size) as i32,
                            None => b.buffer.size as i32,
                        };

                        RawBinding::Buffer {
                            dynamic_offset,
                            raw: b.buffer.inner.clone(),
                            offset: b.offset as i32,
                            size,
                        }
                    }
                    crate::BindingResource::Sampler(s) => {
                        match &layout.ty {
                            crate::BindingType::Sampler { .. } => {}
                            _ => panic!("mis match Sampler type"),
                        }

                        RawBinding::Sampler(s.inner.clone())
                    }
                    crate::BindingResource::TextureView(view) => {
                        match &layout.ty {
                            crate::BindingType::Texture { .. } => {}
                            _ => panic!("mis match Texture type"),
                        }

                        RawBinding::Texture(view.inner.clone())
                    }
                    crate::BindingResource::BufferArray(_) => unimplemented!(),
                    crate::BindingResource::SamplerArray(_) => unimplemented!(),
                    crate::BindingResource::TextureViewArray(_) => unimplemented!(),
                };

                binding
            })
            .collect();

        let layout = desc.layout.inner.entries.clone();
        Ok(Self { contents, layout })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RawBinding {
    Buffer {
        raw: super::Buffer,
        dynamic_offset: i32, // 如果没有，等于 -1
        offset: i32,
        size: i32,
    },
    Texture(super::TextureView),
    Sampler(super::Sampler),
}
