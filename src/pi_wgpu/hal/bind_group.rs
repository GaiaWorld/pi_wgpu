use std::sync::atomic::AtomicU32;

use pi_share::Share;
use derive_more::Debug;

use super::super::wgt;

#[derive(Debug)]
pub struct BindGroupLayout {
    #[debug("Share::new({entries:?})")]
    pub(crate) entries: Share<[wgt::BindGroupLayoutEntry]>,
	pub(crate) id: u32,
}

impl BindGroupLayout {
    pub fn new(
        desc: &super::super::BindGroupLayoutDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        profiling::scope!("hal::BindGroupLayout::new");

        let entries = desc.entries.to_vec().into();
        
        Ok(Self { entries, id: GROUP_AROM.fetch_add(1, std::sync::atomic::Ordering::Relaxed) })
    }
}

#[derive(Debug)]
pub(crate) struct BindGroup {
    pub(crate) layout: Share<[wgt::BindGroupLayoutEntry]>,
    pub(crate) contents: Box<[RawBinding]>,
	pub(crate) id: u32, // id用于跟踪调试
}

impl BindGroup {
    pub fn new(
        desc: &super::super::BindGroupDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        profiling::scope!("hal::BindGroup::new");

        let layout = desc.layout.inner.entries.as_ref();

        let mut next_dynamic_offset = -1;

        let contents = desc
            .entries
            .iter()
            .enumerate()
            .map(|(index, v)| {
                let layout = &layout[index];
                assert!(v.binding == layout.binding);

                let binding = match &v.resource {
                    super::super::BindingResource::Buffer(b) => {
                        let has_dynamic_offset = match &layout.ty {
                            super::super::BindingType::Buffer {
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
                    super::super::BindingResource::Sampler(s) => {
                        match &layout.ty {
                            super::super::BindingType::Sampler { .. } => {}
                            _ => panic!("mis match Sampler type"),
                        }

                        RawBinding::Sampler(s.inner.clone())
                    }
                    super::super::BindingResource::TextureView(view) => {
                        match &layout.ty {
                            super::super::BindingType::Texture { .. } => {}
                            _ => panic!("mis match Texture type"),
                        }

                        RawBinding::Texture(view.inner.clone())
                    }
                    super::super::BindingResource::BufferArray(_) => unimplemented!(),
                    super::super::BindingResource::SamplerArray(_) => unimplemented!(),
                    super::super::BindingResource::TextureViewArray(_) => unimplemented!(),
                };

                binding
            })
            .collect();

        let layout = desc.layout.inner.entries.clone();
        Ok(Self { contents, layout, id: GROUP_AROM.fetch_add(1, std::sync::atomic::Ordering::Relaxed), })
    }
}

lazy_static! {
    static ref GROUP_AROM: AtomicU32 = AtomicU32::new(1);
	static ref GROUP_LAYOUT_AROM: AtomicU32 = AtomicU32::new(1);
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
