use super::super::wgt;

/// Describes a [Buffer](super::super::Buffer) when allocating.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferInitDescriptor<'a> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: super::super::Label<'a>,
    /// Contents of a buffer on creation.
    pub contents: &'a [u8],
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: super::super::BufferUsages,
}

/// Utility methods not meant to be in the main API.
pub trait DeviceExt {
    /// Creates a [Buffer](super::super::Buffer) with data to initialize it.
    fn create_buffer_init(&self, desc: &BufferInitDescriptor) -> super::super::Buffer;

    /// Upload an entire texture and its mipmaps from a source buffer.
    ///
    /// Expects all mipmaps to be tightly packed in the data buffer.
    ///
    /// If the texture is a 2DArray texture, uploads each layer in order, expecting
    /// each layer and its mips to be tightly packed.
    ///
    /// Example:
    /// Layer0Mip0 Layer0Mip1 Layer0Mip2 ... Layer1Mip0 Layer1Mip1 Layer1Mip2 ...
    ///
    /// Implicitly adds the `COPY_DST` usage if it is not present in the descriptor,
    /// as it is required to be able to upload the data to the gpu.
    fn create_texture_with_data(
        &self,
        queue: &super::super::Queue,
        desc: &super::super::TextureDescriptor,
        data: &[u8],
    ) -> super::super::Texture;
}

impl DeviceExt for super::super::Device {
    fn create_buffer_init(&self, descriptor: &BufferInitDescriptor<'_>) -> super::super::Buffer {
        // Skip mapping if the buffer is zero sized
        if descriptor.contents.is_empty() {
            panic!("can't create buffer with zero conent!");
        } else {
            let size = descriptor.contents.len() as crate::BufferAddress;
            let wgt_descriptor = crate::BufferDescriptor {
                label: descriptor.label,
                size,
                usage: descriptor.usage,
                mapped_at_creation: true,
            };

            let buffer = self.create_buffer(&wgt_descriptor);
            buffer.inner.write_buffer(0, &descriptor.contents[..]);
            buffer
        }
    }

    fn create_texture_with_data(
        &self,
        queue: &super::super::Queue,
        desc: &super::super::TextureDescriptor,
        data: &[u8],
    ) -> super::super::Texture {
        // Implicitly add the COPY_DST usage
        let mut desc = desc.to_owned();
        desc.usage |= crate::TextureUsages::COPY_DST;
        let texture = self.create_texture(&desc);

        let format_info = desc.format.describe();
        let layer_iterations = desc.array_layer_count();

        let mut binary_offset = 0;
        for layer in 0..layer_iterations {
            for mip in 0..desc.mip_level_count {
                let mut mip_size = desc.mip_level_size(mip).unwrap();
                // copying layers separately
                if desc.dimension != wgt::TextureDimension::D3 {
                    mip_size.depth_or_array_layers = 1;
                }

                // When uploading mips of compressed textures and the mip is supposed to be
                // a size that isn't a multiple of the block size, the mip needs to be uploaded
                // as its "physical size" which is the size rounded up to the nearest block size.
                let mip_physical = mip_size.physical_size(desc.format);

                // All these calculations are performed on the physical size as that's the
                // data that exists in the buffer.
                let width_blocks = mip_physical.width / format_info.block_dimensions.0 as u32;
                let height_blocks = mip_physical.height / format_info.block_dimensions.1 as u32;

                let bytes_per_row = width_blocks * format_info.block_size as u32;
                let data_size = bytes_per_row * height_blocks * mip_size.depth_or_array_layers;

                let end_offset = binary_offset + data_size as usize;

                queue.write_texture(
                    crate::ImageCopyTexture {
                        texture: &texture,
                        mip_level: mip,
                        origin: crate::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgt::TextureAspect::All,
                    },
                    &data[binary_offset..end_offset],
                    crate::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row),
                        rows_per_image: Some(height_blocks),
                    },
                    mip_physical,
                );

                binary_offset = end_offset;
            }
        }

        texture
    }
}
