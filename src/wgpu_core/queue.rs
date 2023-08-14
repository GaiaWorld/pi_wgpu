use crate::{
    wgpu_hal as hal, Buffer, BufferAddress, BufferSize, CommandBuffer, Extent3d, ImageDataLayout,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

/// Handle to a command queue on a device.
///
/// A `Queue` executes recorded [`CommandBuffer`] objects and provides convenience methods
/// for writing to [buffers](Queue::write_buffer) and [textures](Queue::write_texture).
/// It can be created along with a [`Device`] by calling [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUQueue`](https://gpuweb.github.io/gpuweb/#gpu-queue).
#[derive(Debug)]
pub struct Queue {
    inner: <hal::GL as hal::Api>::Queue,
}

static_assertions::assert_impl_all!(Queue: Send, Sync);

impl Queue {
    /// Schedule a data write into `buffer` starting at `offset`.
    ///
    /// This method is intended to have low performance costs.
    /// As such, the write is not immediately submitted, and instead enqueued
    /// internally to happen at the start of the next `submit()` call.
    ///
    /// This method fails if `data` overruns the size of `buffer` starting at `offset`.
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        unimplemented!("Queue::write_buffer is not implemented")
    }

    /// Schedule a data write into `buffer` starting at `offset` via the returned
    /// [`QueueWriteBufferView`].
    ///
    /// Reading from this buffer is slow and will not yield the actual contents of the buffer.
    ///
    /// This method is intended to have low performance costs.
    /// As such, the write is not immediately submitted, and instead enqueued
    /// internally to happen at the start of the next `submit()` call.
    ///
    /// This method fails if `size` is greater than the size of `buffer` starting at `offset`.
    #[must_use]
    pub fn write_buffer_with<'a>(
        &'a self,
        buffer: &'a Buffer,
        offset: BufferAddress,
        size: BufferSize,
    ) -> Option<QueueWriteBufferView<'a>> {
        unimplemented!("Queue::write_buffer_with is not implemented")
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
        texture: crate::ImageCopyTexture,
        data: &[u8],
        data_layout: ImageDataLayout,
        size: Extent3d,
    ) {
        unimplemented!("Queue::write_texture is not implemented")
    }

    /// Schedule a copy of data from `image` into `texture`.
    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    pub fn copy_external_image_to_texture(
        &self,
        source: &wgt::ImageCopyExternalImage,
        dest: ImageCopyTextureTagged,
        size: Extent3d,
    ) {
        unimplemented!("Queue::copy_external_image_to_texture is not implemented")
    }

    /// Submits a series of finished command buffers for execution.
    pub fn submit<I: IntoIterator<Item = CommandBuffer>>(
        &self,
        command_buffers: I,
    ) -> SubmissionIndex {
        unimplemented!("Queue::submit is not implemented")
    }

    /// Gets the amount of nanoseconds each tick of a timestamp query represents.
    ///
    /// Returns zero if timestamp queries are unsupported.
    pub fn get_timestamp_period(&self) -> f32 {
        unimplemented!("Queue::get_timestamp_period is not implemented")
    }

    /// Registers a callback when the previous call to submit finishes running on the gpu. This callback
    /// being called implies that all mapped buffer callbacks attached to the same submission have also
    /// been called.
    ///
    /// For the callback to complete, either `queue.submit(..)`, `instance.poll_all(..)`, or `device.poll(..)`
    /// must be called elsewhere in the runtime, possibly integrated into an event loop or run on a separate thread.
    ///
    /// The callback will be called on the thread that first calls the above functions after the gpu work
    /// has completed. There are no restrictions on the code you can run in the callback, however on native the
    /// call to the function will not complete until the callback returns, so prefer keeping callbacks short
    /// and used to set flags, send messages, etc.
    pub fn on_submitted_work_done(&self, callback: impl FnOnce() + Send + 'static) {
        unimplemented!("Queue::on_submitted_work_done is not implemented")
    }
}

/// Identifier for a particular call to [`Queue::submit`]. Can be used
/// as part of an argument to [`Device::poll`] to block for a particular
/// submission to finish.
///
/// This type is unique to the Rust API of `wgpu`.
/// There is no analogue in the WebGPU specification.
#[derive(Debug, Clone)]
pub struct SubmissionIndex(Arc<crate::Data>);
static_assertions::assert_impl_all!(SubmissionIndex: Send, Sync);

/// A read-only view into a staging buffer.
///
/// Reading into this buffer won't yield the contents of the buffer from the
/// GPU and is likely to be slow. Because of this, although [`AsMut`] is
/// implemented for this type, [`AsRef`] is not.
pub struct QueueWriteBufferView<'a> {
    _data: PhantomData<&'a ()>,
}
static_assertions::assert_impl_all!(QueueWriteBufferView: Send, Sync);

impl Deref for QueueWriteBufferView<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unimplemented!("QueueWriteBufferView::deref is not implemented")
    }
}

impl DerefMut for QueueWriteBufferView<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unimplemented!("QueueWriteBufferView::deref_mut is not implemented")
    }
}

impl<'a> AsMut<[u8]> for QueueWriteBufferView<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        unimplemented!("QueueWriteBufferView::as_mut is not implemented")
    }
}

impl<'a> Drop for QueueWriteBufferView<'a> {
    fn drop(&mut self) {
        unimplemented!("QueueWriteBufferView::drop is not implemented")
    }
}
