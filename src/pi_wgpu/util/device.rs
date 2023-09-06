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
        todo!();
    }

    fn create_texture_with_data(
        &self,
        queue: &super::super::Queue,
        desc: &super::super::TextureDescriptor,
        data: &[u8],
    ) -> super::super::Texture {
        todo!();
    }
}