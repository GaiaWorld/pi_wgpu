use thiserror::Error;

pub mod api;

mod egl_impl;
mod gles;
mod gl_state;

mod instance;

mod adapter;

mod surface;

mod device;

mod queue;

mod command;

mod buffer;

mod texture;

mod sampler;

mod query_set;

mod fence;

mod bind_group;

mod shader_module;

mod pipeline;

pub use api::*;

pub(crate) use adapter::*;
pub(crate) use device::*;
pub(crate) use egl_impl::*;
pub(crate) use gles::*;
pub(crate) use instance::*;
pub(crate) use queue::*;
pub(crate) use surface::*;
pub(crate) use command::*;
pub(crate) use buffer::*;
pub(crate) use texture::*;
pub(crate) use sampler::*;
pub(crate) use query_set::*;
pub(crate) use fence::*;
pub(crate) use bind_group::*;
pub(crate) use shader_module::*;
pub(crate) use pipeline::*;


use std::{ops::{RangeInclusive, Range}, sync::atomic::AtomicBool, ptr::NonNull};

use bitflags::bitflags;

use crate::{wgt, Api, Label};

#[derive(Debug)]
pub struct ExposedAdapter<A: Api> {
    pub adapter: A::Adapter,
    pub info: wgt::AdapterInfo,
    pub features: wgt::Features,
    pub capabilities: Capabilities,
}

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub limits: wgt::Limits,
    pub alignments: Alignments,
    pub downlevel: wgt::DownlevelCapabilities,
}

#[derive(Clone, Debug)]
pub struct Alignments {
    /// The alignment of the start of the buffer used as a GPU copy source.
    pub buffer_copy_offset: wgt::BufferSize,
    /// The alignment of the row pitch of the texture data stored in a buffer that is
    /// used in a GPU copy operation.
    pub buffer_copy_pitch: wgt::BufferSize,
}

pub(crate) type MemoryRange = Range<wgt::BufferAddress>;
pub(crate) type FenceValue = u64;

pub(crate) const MAX_ANISOTROPY: u8 = 16;
pub(crate) const MAX_BIND_GROUPS: usize = 8;
pub(crate) const MAX_VERTEX_BUFFERS: usize = 16;
pub(crate) const MAX_COLOR_ATTACHMENTS: usize = 8;
pub(crate) const MAX_MIP_LEVELS: u32 = 16;

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub(crate) enum ShaderError {
    #[error("compilation failed: {0:?}")]
    Compilation(String),
    #[error(transparent)]
    Device(#[from] DeviceError),
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub(crate) enum PipelineError {
    #[error("linkage failed for stage {0:?}: {1}")]
    Linkage(wgt::ShaderStages, String),
    #[error("entry point for stage {0:?} is invalid")]
    EntryPoint(naga::ShaderStage),
    #[error(transparent)]
    Device(#[from] DeviceError),
}

#[derive(Debug)]
pub(crate) struct AcquiredSurfaceTexture<A: Api> {
    pub texture: A::SurfaceTexture,
    /// The presentation configuration no longer matches
    /// the surface properties exactly, but can still be used to present
    /// to the surface successfully.
    pub suboptimal: bool,
}

/// Stores if any API validation error has occurred in this process
/// since it was last reset.
///
/// This is used for internal wgpu testing only and _must not_ be used
/// as a way to check for errors.
///
/// This works as a static because `cargo nextest` runs all of our
/// tests in separate processes, so each test gets its own canary.
///
/// This prevents the issue of one validation error terminating the
/// entire process.
pub(crate) static VALIDATION_CANARY: ValidationCanary = ValidationCanary {
    inner: AtomicBool::new(false),
};

#[derive(Debug, Clone)]
pub(crate) struct BufferBarrier<'a, A: Api> {
    pub buffer: &'a A::Buffer,
    pub usage: Range<BufferUses>,
}

#[derive(Debug, Clone)]
pub(crate) struct TextureBarrier<'a, A: Api> {
    pub texture: &'a A::Texture,
    pub range: wgt::ImageSubresourceRange,
    pub usage: Range<TextureUses>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BufferCopy {
    pub src_offset: wgt::BufferAddress,
    pub dst_offset: wgt::BufferAddress,
    pub size: wgt::BufferSize,
}

#[derive(Clone, Debug)]
pub(crate) struct TextureCopyBase {
    pub mip_level: u32,
    pub array_layer: u32,
    /// Origin within a texture.
    /// Note: for 1D and 2D textures, Z must be 0.
    pub origin: wgt::Origin3d,
    pub aspect: FormatAspects,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CopyExtent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct TextureCopy {
    pub src_base: TextureCopyBase,
    pub dst_base: TextureCopyBase,
    pub size: CopyExtent,
}

#[derive(Clone, Debug)]
pub(crate) struct BufferTextureCopy {
    pub buffer_layout: wgt::ImageDataLayout,
    pub texture_base: TextureCopyBase,
    pub size: CopyExtent,
}

/// Flag for internal testing.
pub(crate) struct ValidationCanary {
    inner: AtomicBool,
}

impl ValidationCanary {
    #[allow(dead_code)] // in some configurations this function is dead
    pub(crate) fn set(&self) {
        self.inner.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Returns true if any API validation error has occurred in this process
    /// since the last call to this function.
    pub(crate) fn get_and_reset(&self) -> bool {
        self.inner.swap(false, std::sync::atomic::Ordering::SeqCst)
    }
}

/// Describes information about what a `Surface`'s presentation capabilities are.
/// Fetch this with [Adapter::surface_capabilities].
#[derive(Debug, Clone)]
pub(crate) struct SurfaceCapabilities {
    /// List of supported texture formats.
    ///
    /// Must be at least one.
    pub formats: Vec<wgt::TextureFormat>,

    /// Range for the swap chain sizes.
    ///
    /// - `swap_chain_sizes.start` must be at least 1.
    /// - `swap_chain_sizes.end` must be larger or equal to `swap_chain_sizes.start`.
    pub swap_chain_sizes: RangeInclusive<u32>,

    /// Current extent of the surface, if known.
    pub current_extent: Option<wgt::Extent3d>,

    /// Range of supported extents.
    ///
    /// `current_extent` must be inside this range.
    pub extents: RangeInclusive<wgt::Extent3d>,

    /// Supported texture usage flags.
    ///
    /// Must have at least `TextureUses::COLOR_TARGET`
    pub usage: TextureUses,

    /// List of supported V-sync modes.
    ///
    /// Must be at least one.
    pub present_modes: Vec<wgt::PresentMode>,

    /// List of supported alpha composition modes.
    ///
    /// Must be at least one.
    pub composite_alpha_modes: Vec<wgt::CompositeAlphaMode>,
}

#[derive(Debug, Clone)]
pub(crate) struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SrgbFrameBufferKind {
    /// No support for SRGB surface
    None,
    /// Using EGL 1.5's support for colorspaces
    Core,
    /// Using EGL_KHR_gl_colorspace
    Khr,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum DeviceError {
    #[error("out of memory")]
    OutOfMemory,
    #[error("device is lost")]
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub(crate) enum SurfaceError {
    #[error("surface is lost")]
    Lost,
    #[error("surface is outdated, needs to be re-created")]
    Outdated,
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("other reason: {0}")]
    Other(&'static str),
}

#[derive(Clone, Debug)]
pub(crate) struct BufferMapping {
    pub ptr: NonNull<u8>,
    pub is_coherent: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct BufferDescriptor<'a> {
    pub label: Label<'a>,
    pub size: wgt::BufferAddress,
    pub usage: BufferUses,
    pub memory_flags: MemoryFlags,
}

#[derive(Debug, Clone)]
pub(crate) struct SurfaceConfiguration {
    /// Number of textures in the swap chain. Must be in
    /// `SurfaceCapabilities::swap_chain_size` range.
    pub swap_chain_size: u32,
    /// Vertical synchronization mode.
    pub present_mode: wgt::PresentMode,
    /// Alpha composition mode.
    pub composite_alpha_mode: wgt::CompositeAlphaMode,
    /// Format of the surface textures.
    pub format: wgt::TextureFormat,
    /// Requested texture extent. Must be in
    /// `SurfaceCapabilities::extents` range.
    pub extent: wgt::Extent3d,
    /// Allowed usage of surface textures,
    pub usage: TextureUses,
    /// Allows views of swapchain texture to have a different format
    /// than the texture does.
    pub view_formats: Vec<wgt::TextureFormat>,
}

bitflags!(
    pub(crate) struct MemoryFlags: u32 {
        const TRANSIENT = 1 << 0;
        const PREFER_COHERENT = 1 << 1;
    }
);

bitflags! (
    /// Similar to `wgt::TextureUsages` but for internal use.
    pub(crate) struct TextureUses: u16 {
        /// The texture is in unknown state.
        const UNINITIALIZED = 1 << 0;
        /// Ready to present image to the surface.
        const PRESENT = 1 << 1;
        /// The source of a hardware copy.
        const COPY_SRC = 1 << 2;
        /// The destination of a hardware copy.
        const COPY_DST = 1 << 3;
        /// Read-only sampled or fetched resource.
        const RESOURCE = 1 << 4;
        /// The color target of a renderpass.
        const COLOR_TARGET = 1 << 5;
        /// Read-only depth stencil usage.
        const DEPTH_STENCIL_READ = 1 << 6;
        /// Read-write depth stencil usage
        const DEPTH_STENCIL_WRITE = 1 << 7;
        /// Read-only storage buffer usage. Corresponds to a UAV in d3d, so is exclusive, despite being read only.
        const STORAGE_READ = 1 << 8;
        /// Read-write or write-only storage buffer usage.
        const STORAGE_READ_WRITE = 1 << 9;
        /// The combination of states that a texture may be in _at the same time_.
        const INCLUSIVE = Self::COPY_SRC.bits | Self::RESOURCE.bits | Self::DEPTH_STENCIL_READ.bits;
        /// The combination of states that a texture must exclusively be in.
        const EXCLUSIVE = Self::COPY_DST.bits | Self::COLOR_TARGET.bits | Self::DEPTH_STENCIL_WRITE.bits | Self::STORAGE_READ.bits | Self::STORAGE_READ_WRITE.bits | Self::PRESENT.bits;
        /// The combination of all usages that the are guaranteed to be be ordered by the hardware.
        /// If a usage is ordered, then if the texture state doesn't change between draw calls, there
        /// are no barriers needed for synchronization.
        const ORDERED = Self::INCLUSIVE.bits | Self::COLOR_TARGET.bits | Self::DEPTH_STENCIL_WRITE.bits | Self::STORAGE_READ.bits;

        /// Flag used by the wgpu-core texture tracker to say a texture is in different states for every sub-resource
        const COMPLEX = 1 << 10;
        /// Flag used by the wgpu-core texture tracker to say that the tracker does not know the state of the sub-resource.
        /// This is different from UNINITIALIZED as that says the tracker does know, but the texture has not been initialized.
        const UNKNOWN = 1 << 11;
    }
);

bitflags!(
    /// Texture format capability flags.
    pub(crate) struct TextureFormatCapabilities: u32 {
        /// Format can be sampled.
        const SAMPLED = 1 << 0;
        /// Format can be sampled with a linear sampler.
        const SAMPLED_LINEAR = 1 << 1;
        /// Format can be sampled with a min/max reduction sampler.
        const SAMPLED_MINMAX = 1 << 2;

        /// Format can be used as storage with write-only access.
        const STORAGE = 1 << 3;
        /// Format can be used as storage with read and read/write access.
        const STORAGE_READ_WRITE = 1 << 4;
        /// Format can be used as storage with atomics.
        const STORAGE_ATOMIC = 1 << 5;

        /// Format can be used as color and input attachment.
        const COLOR_ATTACHMENT = 1 << 6;
        /// Format can be used as color (with blending) and input attachment.
        const COLOR_ATTACHMENT_BLEND = 1 << 7;
        /// Format can be used as depth-stencil and input attachment.
        const DEPTH_STENCIL_ATTACHMENT = 1 << 8;

        /// Format can be multisampled by x2.
        const MULTISAMPLE_X2   = 1 << 9;
        /// Format can be multisampled by x4.
        const MULTISAMPLE_X4   = 1 << 10;
        /// Format can be multisampled by x8.
        const MULTISAMPLE_X8   = 1 << 11;

        /// Format can be used for render pass resolve targets.
        const MULTISAMPLE_RESOLVE = 1 << 12;

        /// Format can be copied from.
        const COPY_SRC = 1 << 13;
        /// Format can be copied to.
        const COPY_DST = 1 << 14;
    }
);

bitflags::bitflags! {
    /// Similar to `wgt::BufferUsages` but for internal use.
    pub(crate) struct BufferUses: u16 {
        /// The argument to a read-only mapping.
        const MAP_READ = 1 << 0;
        /// The argument to a write-only mapping.
        const MAP_WRITE = 1 << 1;
        /// The source of a hardware copy.
        const COPY_SRC = 1 << 2;
        /// The destination of a hardware copy.
        const COPY_DST = 1 << 3;
        /// The index buffer used for drawing.
        const INDEX = 1 << 4;
        /// A vertex buffer used for drawing.
        const VERTEX = 1 << 5;
        /// A uniform buffer bound in a bind group.
        const UNIFORM = 1 << 6;
        /// A read-only storage buffer used in a bind group.
        const STORAGE_READ = 1 << 7;
        /// A read-write or write-only buffer used in a bind group.
        const STORAGE_READ_WRITE = 1 << 8;
        /// The indirect or count buffer in a indirect draw or dispatch.
        const INDIRECT = 1 << 9;
        /// The combination of states that a buffer may be in _at the same time_.
        const INCLUSIVE = Self::MAP_READ.bits | Self::COPY_SRC.bits |
            Self::INDEX.bits | Self::VERTEX.bits | Self::UNIFORM.bits |
            Self::STORAGE_READ.bits | Self::INDIRECT.bits;
        /// The combination of states that a buffer must exclusively be in.
        const EXCLUSIVE = Self::MAP_WRITE.bits | Self::COPY_DST.bits | Self::STORAGE_READ_WRITE.bits;
        /// The combination of all usages that the are guaranteed to be be ordered by the hardware.
        /// If a usage is ordered, then if the buffer state doesn't change between draw calls, there
        /// are no barriers needed for synchronization.
        const ORDERED = Self::INCLUSIVE.bits | Self::MAP_WRITE.bits;
    }
}


bitflags!(
    /// Texture format capability flags.
    pub struct FormatAspects: u8 {
        const COLOR = 1 << 0;
        const DEPTH = 1 << 1;
        const STENCIL = 1 << 2;
    }
);