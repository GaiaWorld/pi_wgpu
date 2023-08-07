//!
//! + Buffer
//! + BufferSlice<'a>, BufferView<'a>, BufferViewMut<'a>
//! + BufferBinding<'a>, BufferBinding<'a>, BindingResource<'a>
//! + MapMode, BufferAsyncError
//!

use std::ops::RangeBounds;

use crate::{BindingResource, Label, BufferAddress, BufferUsages, BufferSize};

/// Describes a [`Buffer`].
///
/// For use with [`Device::create_buffer`].
///
/// Corresponds to [WebGPU `GPUBufferDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubufferdescriptor).
pub type BufferDescriptor<'a> = wgt::BufferDescriptor<Label<'a>>;

static_assertions::assert_impl_all!(BufferDescriptor: Send, Sync);

/// Handle to a GPU-accessible buffer.
///
/// Created with [`Device::create_buffer`] or
/// [`DeviceExt::create_buffer_init`](util::DeviceExt::create_buffer_init).
///
/// Corresponds to [WebGPU `GPUBuffer`](https://gpuweb.github.io/gpuweb/#buffer-interface).
#[derive(Debug)]
pub struct Buffer {
    // Todo: missing map_state https://www.w3.org/TR/webgpu/#dom-gpubuffer-mapstate
}

static_assertions::assert_impl_all!(Buffer: Send, Sync);

impl Drop for Buffer {
    fn drop(&mut self) {
        unimplemented!("Buffer::drop is not implemented")
    }
}

impl Buffer {
    /// Return the binding view of the entire buffer.
    pub fn as_entire_binding(&self) -> BindingResource {
        unimplemented!("Buffer::as_entire_binding is not implemented")
    }

    /// Return the binding view of the entire buffer.
    pub fn as_entire_buffer_binding(&self) -> BufferBinding {
        unimplemented!("Buffer::as_entire_buffer_binding is not implemented")
    }

    /// Use only a portion of this Buffer for a given operation. Choosing a range with no end
    /// will use the rest of the buffer. Using a totally unbounded range will use the entire buffer.
    pub fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice {
        unimplemented!("Buffer::slice is not implemented")
    }

    /// Flushes any pending write operations and unmaps the buffer from host memory.
    pub fn unmap(&self) {
        unimplemented!("Buffer::unmap is not implemented")
    }

    /// Destroy the associated native resources as soon as possible.
    pub fn destroy(&self) {
        unimplemented!("Buffer::destroy is not implemented")
    }

    /// Returns the length of the buffer allocation in bytes.
    ///
    /// This is always equal to the `size` that was specified when creating the buffer.
    pub fn size(&self) -> BufferAddress {
        unimplemented!("Buffer::size is not implemented")
    }

    /// Returns the allowed usages for this `Buffer`.
    ///
    /// This is always equal to the `usage` that was specified when creating the buffer.
    pub fn usage(&self) -> BufferUsages {
        unimplemented!("Buffer::usage is not implemented")
    }
}

/// Slice into a [`Buffer`].
///
/// It can be created with [`Buffer::slice`]. To use the whole buffer, call with unbounded slice:
///
/// `buffer.slice(..)`
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// an offset and size are specified as arguments to each call working with the [`Buffer`], instead.
#[derive(Copy, Clone, Debug)]
pub struct BufferSlice<'a> {
    buffer: &'a Buffer,
    offset: BufferAddress,
    size: Option<BufferSize>,
}

static_assertions::assert_impl_all!(BufferSlice: Send, Sync);

impl<'a> BufferSlice<'a> {
    /// Map the buffer. Buffer is ready to map once the callback is called.
    ///
    /// For the callback to complete, either `queue.submit(..)`, `instance.poll_all(..)`, or `device.poll(..)`
    /// must be called elsewhere in the runtime, possibly integrated into an event loop or run on a separate thread.
    ///
    /// The callback will be called on the thread that first calls the above functions after the gpu work
    /// has completed. There are no restrictions on the code you can run in the callback, however on native the
    /// call to the function will not complete until the callback returns, so prefer keeping callbacks short
    /// and used to set flags, send messages, etc.
    pub fn map_async(
        &self,
        mode: MapMode,
        callback: impl FnOnce(Result<(), BufferAsyncError>) + Send + 'static,
    ) {
        unimplemented!("BufferSlice::map_async is not implemented")
    }

    /// Synchronously and immediately map a buffer for reading. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn get_mapped_range(&self) -> BufferView<'a> {
        unimplemented!("BufferSlice::get_mapped_range is not implemented")
    }

    /// Synchronously and immediately map a buffer for writing. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn get_mapped_range_mut(&self) -> BufferViewMut<'a> {
        unimplemented!("BufferSlice::get_mapped_range_mut is not implemented")
    }
}

/// Read only view into a mapped buffer.
#[derive(Debug)]
pub struct BufferView<'a> {
    slice: BufferSlice<'a>,
    data: Box<dyn BufferMappedRange>,
}

/// Write only view into mapped buffer.
///
/// It is possible to read the buffer using this view, but doing so is not
/// recommended, as it is likely to be slow.
#[derive(Debug)]
pub struct BufferViewMut<'a> {
    slice: BufferSlice<'a>,
    data: Box<dyn BufferMappedRange>,
    readable: bool,
}

pub trait BufferMappedRange: std::fmt::Debug {
    fn slice(&self) -> &[u8];

    fn slice_mut(&mut self) -> &mut [u8];
}

/// Describes the segment of a buffer to bind.
///
/// Corresponds to [WebGPU `GPUBufferBinding`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubufferbinding).
#[derive(Clone, Debug)]
pub struct BufferBinding<'a> {
    /// The buffer to bind.
    pub buffer: &'a Buffer,
    /// Base offset of the buffer. For bindings with `dynamic == true`, this offset
    /// will be added to the dynamic offset provided in [`RenderPass::set_bind_group`].
    ///
    /// The offset has to be aligned to [`Limits::min_uniform_buffer_offset_alignment`]
    /// or [`Limits::min_storage_buffer_offset_alignment`] appropriately.
    pub offset: BufferAddress,
    /// Size of the binding, or `None` for using the rest of the buffer.
    pub size: Option<BufferSize>,
}
static_assertions::assert_impl_all!(BufferBinding: Send, Sync);

/// Error occurred when trying to async map a buffer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BufferAsyncError;
static_assertions::assert_impl_all!(BufferAsyncError: Send, Sync);

impl std::fmt::Display for BufferAsyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error occurred when trying to async map a buffer")
    }
}

impl std::error::Error for BufferAsyncError {}

/// Type of buffer mapping.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MapMode {
    /// Map only for reading
    Read,
    /// Map only for writing
    Write,
}
static_assertions::assert_impl_all!(MapMode: Send, Sync);
