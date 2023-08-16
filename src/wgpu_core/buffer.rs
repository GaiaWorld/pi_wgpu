//!
//! + Buffer
//! + BufferSlice<'a>, BufferView<'a>, BufferViewMut<'a>
//! + BufferBinding<'a>, BufferBinding<'a>, BindingResource<'a>
//! + MapMode, BufferAsyncError
//!

use std::ops::RangeBounds;

use crate::{
    wgpu_hal::{self as hal, Device},
    BindingResource, BufferAddress, BufferSize, BufferUsages, Label,
};

/// Describes a [`Buffer`].
///
/// For use with [`Device::create_buffer`].
///
/// Corresponds to [WebGPU `GPUBufferDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubufferdescriptor).
pub type BufferDescriptor<'a> = crate::wgpu_types::BufferDescriptor<Label<'a>>;

/// Handle to a GPU-accessible buffer.
///
/// Created with [`Device::create_buffer`] or
/// [`DeviceExt::create_buffer_init`](util::DeviceExt::create_buffer_init).
///
/// Corresponds to [WebGPU `GPUBuffer`](https://gpuweb.github.io/gpuweb/#buffer-interface).
#[derive(Debug)]
pub struct Buffer {
    // Todo: missing map_state https://www.w3.org/TR/webgpu/#dom-gpubuffer-mapstate
    pub(crate) inner: Option<<hal::GL as hal::Api>::Buffer>,

    pub(crate) usage: BufferUsages,
    pub(crate) size: BufferAddress,

    pub(crate) device: crate::Device,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        profiling::scope!("wgpu_core::Buffer::drop");

        // TODO: 这里是单线程结构，没有考虑到 多线程 和 录制需求；
        if let Some(buffer) = self.inner.take() {
            unsafe { self.device.inner.destroy_buffer(buffer) }
        }
    }
}

impl Buffer {
    /// Returns the length of the buffer allocation in bytes.
    ///
    /// This is always equal to the `size` that was specified when creating the buffer.
    pub fn size(&self) -> BufferAddress {
        self.size
    }

    /// Returns the allowed usages for this `Buffer`.
    ///
    /// This is always equal to the `usage` that was specified when creating the buffer.
    pub fn usage(&self) -> BufferUsages {
        self.usage
    }

    /// Return the binding view of the entire buffer.
    pub fn as_entire_binding(&self) -> BindingResource {
        BindingResource::Buffer(self.as_entire_buffer_binding())
    }

    /// Return the binding view of the entire buffer.
    pub fn as_entire_buffer_binding(&self) -> BufferBinding {
        BufferBinding {
            buffer: self,
            offset: 0,
            size: None,
        }
    }

    /// Use only a portion of this Buffer for a given operation. Choosing a range with no end
    /// will use the rest of the buffer. Using a totally unbounded range will use the entire buffer.
    // TODO: map, unmap 请使用 Queue::write_buffer
    pub fn slice<S: RangeBounds<BufferAddress>>(&self, _bounds: S) -> BufferSlice {
        unimplemented!("wgpu_core::Buffer::slice is not implemented")
    }

    /// Flushes any pending write operations and unmaps the buffer from host memory.
    // TODO: map, unmap 请使用 Queue::write_buffer
    pub fn unmap(&self) {
        unimplemented!("wgpu_core::Buffer::unmap is not implemented")
    }

    /// Destroy the associated native resources as soon as possible.
    // TODO: 只要高层不握住，自然调用 Drop 释放Buffer；此处不明白为什么要写这个函数？
    pub fn destroy(&self) {
        unimplemented!("wgpu_core::Buffer::destroy is not implemented")
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
    _buffer: &'a Buffer,
    _offset: BufferAddress,
    _size: Option<BufferSize>,
}

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
        _mode: MapMode,
        _callback: impl FnOnce(Result<(), BufferAsyncError>) + Send + 'static,
    ) {
        unimplemented!("wgpu_core::BufferSlice::map_async is not implemented")
    }

    /// Synchronously and immediately map a buffer for reading. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn get_mapped_range(&self) -> BufferView<'a> {
        unimplemented!("wgpu_core::BufferSlice::get_mapped_range is not implemented")
    }

    /// Synchronously and immediately map a buffer for writing. If the buffer is not immediately mappable
    /// through [`BufferDescriptor::mapped_at_creation`] or [`BufferSlice::map_async`], will panic.
    pub fn get_mapped_range_mut(&self) -> BufferViewMut<'a> {
        unimplemented!("wgpu_core::BufferSlice::get_mapped_range_mut is not implemented")
    }
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

/// Read only view into a mapped buffer.
#[derive(Debug)]
pub struct BufferView<'a> {
    _slice: BufferSlice<'a>,
    _data: Box<dyn BufferMappedRange>,
}

/// Write only view into mapped buffer.
///
/// It is possible to read the buffer using this view, but doing so is not
/// recommended, as it is likely to be slow.
#[derive(Debug)]
pub struct BufferViewMut<'a> {
    _slice: BufferSlice<'a>,
    _data: Box<dyn BufferMappedRange>,
    _readable: bool,
}

pub trait BufferMappedRange: std::fmt::Debug {
    fn slice(&self) -> &[u8];

    fn slice_mut(&mut self) -> &mut [u8];
}

/// Error occurred when trying to async map a buffer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BufferAsyncError;

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
