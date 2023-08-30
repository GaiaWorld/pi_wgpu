//!
//! + Buffer, BufferSlice<'a>
//! + BufferBinding<'a>, BindingResource<'a>
//!

use std::ops::RangeBounds;

use crate::{hal, BindingResource, BufferAddress, BufferSize, BufferUsages, Label};

/// Describes a [`Buffer`].
///
/// For use with [`Device::create_buffer`].
///
/// Corresponds to [WebGPU `GPUBufferDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubufferdescriptor).
pub type BufferDescriptor<'a> = crate::wgt::BufferDescriptor<Label<'a>>;

/// Handle to a GPU-accessible buffer.
///
/// Created with [`Device::create_buffer`] or
/// [`DeviceExt::create_buffer_init`](util::DeviceExt::create_buffer_init).
///
/// Corresponds to [WebGPU `GPUBuffer`](https://gpuweb.github.io/gpuweb/#buffer-interface).
#[derive(Debug)]
pub struct Buffer {
    pub(crate) usage: BufferUsages,
    pub(crate) size: BufferAddress,

    pub(crate) inner: hal::Buffer,
}

impl Buffer {
    #[inline]
    pub(crate) fn from_hal(
        inner: crate::hal::Buffer,
        usage: BufferUsages,
        size: BufferAddress,
    ) -> Self {
        Self { inner, usage, size }
    }
}

impl Buffer {
    /// Returns the length of the buffer allocation in bytes.
    ///
    /// This is always equal to the `size` that was specified when creating the buffer.
    #[inline]
    pub fn size(&self) -> BufferAddress {
        self.size
    }

    /// Returns the allowed usages for this `Buffer`.
    ///
    /// This is always equal to the `usage` that was specified when creating the buffer.
    #[inline]
    pub fn usage(&self) -> BufferUsages {
        self.usage
    }

    /// Return the binding view of the entire buffer.
    #[inline]
    pub fn as_entire_binding(&self) -> BindingResource {
        BindingResource::Buffer(self.as_entire_buffer_binding())
    }

    /// Return the binding view of the entire buffer.
    #[inline]
    pub fn as_entire_buffer_binding(&self) -> BufferBinding {
        BufferBinding {
            buffer: self,
            offset: 0,
            size: None,
        }
    }

    /// Use only a portion of this Buffer for a given operation. Choosing a range with no end
    /// will use the rest of the buffer. Using a totally unbounded range will use the entire buffer.
    #[inline]
    pub fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice {
        let s = bounds.start_bound();
        let e = bounds.end_bound();

        let offset = match s {
            std::ops::Bound::Included(s) => *s,
            std::ops::Bound::Excluded(s) => *s,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match e {
            std::ops::Bound::Included(e) => *e,
            std::ops::Bound::Excluded(e) => *e,
            std::ops::Bound::Unbounded => self.size,
        };

        let size = std::num::NonZeroU64::try_from(end - offset).ok();

        BufferSlice {
            buffer: &self,
            offset,
            size,
        }
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
    pub(crate) buffer: &'a Buffer,
    pub(crate) offset: BufferAddress,
    pub(crate) size: Option<BufferSize>,
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
