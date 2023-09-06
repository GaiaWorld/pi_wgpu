use super::super::{hal, Buffer, BufferAddress, CommandBuffer, Extent3d, ImageDataLayout};

/// Handle to a command queue on a device.
///
/// A `Queue` executes recorded [`CommandBuffer`] objects and provides convenience methods
/// for writing to [buffers](Queue::write_buffer) and [textures](Queue::write_texture).
/// It can be created along with a [`Device`] by calling [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUQueue`](https://gpuweb.github.io/gpuweb/#gpu-queue).
#[derive(Debug)]
pub struct Queue {
    pub(crate) inner: hal::Queue,
}

impl Queue {
    /// Schedule a data write into `buffer` starting at `offset`.
    ///
    /// This method is intended to have low performance costs.
    /// As such, the write is not immediately submitted, and instead enqueued
    /// internally to happen at the start of the next `submit()` call.
    ///
    /// This method fails if `data` overruns the size of `buffer` starting at `offset`.
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        buffer.inner.write_buffer(offset as i32, data);
    }

    /// Schedule a write of some data into a texture.
    ///
    /// * `data` contains the texels to be written, which must be in
    ///   [the same format as the texture](TextureFormat).
    /// * `data_layout` describes the memory layout of `data`, which does not necessarily
    ///   have to have tightly packed rows.
    /// * `texture` specifies the texture to write into, and the location within the
    ///   texture (coordinate offset, mip level) that will be overwritten.
    /// * `size` is the size, in texels, of the region to be written.
    ///
    /// This method is intended to have low performance costs.
    /// As such, the write is not immediately submitted, and instead enqueued
    /// internally to happen at the start of the next `submit()` call.
    /// However, `data` will be immediately copied into staging memory; so the caller may
    /// discard it any time after this call completes.
    ///
    /// This method fails if `size` overruns the size of `texture`, or if `data` is too short.
    pub fn write_texture(
        &self,
        texture: super::super::ImageCopyTexture,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d,
    ) {
        hal::Texture::write_data(texture, data, data_layout, size);
    }

    /// Submits a series of finished command buffers for execution.
    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(
        &self,
        command_buffers: I,
    ) -> SubmissionIndex {
        SubmissionIndex
    }
}

/// Identifier for a particular call to [`Queue::submit`]. Can be used
/// as part of an argument to [`Device::poll`] to block for a particular
/// submission to finish.
///
/// This type is unique to the Rust API of `wgpu`.
/// There is no analogue in the WebGPU specification.
#[derive(Debug, Clone)]
pub struct SubmissionIndex;
