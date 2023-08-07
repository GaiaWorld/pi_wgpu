use std::{
    borrow::Cow,
    fmt,
    num::{NonZeroU32, NonZeroU8},
    ops::{Range, RangeInclusive},
    ptr::NonNull,
};

use bitflags::bitflags;
use thiserror::Error;

use super::Api;

pub const MAX_ANISOTROPY: u8 = 16;
pub const MAX_BIND_GROUPS: usize = 8;
pub const MAX_VERTEX_BUFFERS: usize = 16;
pub const MAX_COLOR_ATTACHMENTS: usize = 8;
pub const MAX_MIP_LEVELS: u32 = 16;
/// Size of a single occlusion/timestamp query, when copied into a buffer, in bytes.
pub const QUERY_SIZE: wgt::BufferAddress = 8;

pub type Label<'a> = Option<&'a str>;
pub type MemoryRange = Range<wgt::BufferAddress>;
pub type FenceValue = u64;

/// Drop guard to signal wgpu-hal is no longer using an externally created object.
pub type DropGuard = Box<dyn std::any::Any + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum DeviceError {
    #[error("out of memory")]
    OutOfMemory,
    #[error("device is lost")]
    Lost,
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ShaderError {
    #[error("compilation failed: {0:?}")]
    Compilation(String),
    #[error(transparent)]
    Device(#[from] DeviceError),
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum PipelineError {
    #[error("linkage failed for stage {0:?}: {1}")]
    Linkage(wgt::ShaderStages, String),
    #[error("entry point for stage {0:?} is invalid")]
    EntryPoint(naga::ShaderStage),
    #[error(transparent)]
    Device(#[from] DeviceError),
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum SurfaceError {
    #[error("surface is lost")]
    Lost,
    #[error("surface is outdated, needs to be re-created")]
    Outdated,
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("other reason: {0}")]
    Other(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("Not supported")]
pub struct InstanceError;

bitflags!(
    /// Instance initialization flags.
    #[derive(Clone, Copy, Debug)]
    pub struct InstanceFlags: u32 {
        /// Generate debug information in shaders and objects.
        const DEBUG = 1 << 0;
        /// Enable validation, if possible.
        const VALIDATION = 1 << 1;
    }
);

bitflags!(
    /// Pipeline layout creation flags.
    #[derive(Clone, Copy, Debug)]
    pub struct PipelineLayoutFlags: u32 {
        /// Include support for base vertex/instance drawing.
        const BASE_VERTEX_INSTANCE = 1 << 0;
        /// Include support for num work groups builtin.
        const NUM_WORK_GROUPS = 1 << 1;
    }
);

bitflags!(
    /// Pipeline layout creation flags.
    #[derive(Clone, Copy, Debug)]
    pub struct BindGroupLayoutFlags: u32 {
        /// Allows for bind group binding arrays to be shorter than the array in the BGL.
        const PARTIALLY_BOUND = 1 << 0;
    }
);

bitflags!(
    /// Texture format capability flags.
    #[derive(Clone, Copy, Debug)]
    pub struct TextureFormatCapabilities: u32 {
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

bitflags!(
    /// Texture format capability flags.
    #[derive(Clone, Copy, Debug)]
    pub struct FormatAspects: u8 {
        const COLOR = 1 << 0;
        const DEPTH = 1 << 1;
        const STENCIL = 1 << 2;
    }
);

impl From<wgt::TextureAspect> for FormatAspects {
    fn from(aspect: wgt::TextureAspect) -> Self {
        match aspect {
            wgt::TextureAspect::All => Self::all(),
            wgt::TextureAspect::DepthOnly => Self::DEPTH,
            wgt::TextureAspect::StencilOnly => Self::STENCIL,
        }
    }
}

impl From<wgt::TextureFormat> for FormatAspects {
    fn from(format: wgt::TextureFormat) -> Self {
        match format {
            wgt::TextureFormat::Stencil8 => Self::STENCIL,
            wgt::TextureFormat::Depth16Unorm => Self::DEPTH,
            wgt::TextureFormat::Depth32Float | wgt::TextureFormat::Depth24Plus => Self::DEPTH,
            wgt::TextureFormat::Depth32FloatStencil8 | wgt::TextureFormat::Depth24PlusStencil8 => {
                Self::DEPTH | Self::STENCIL
            }
            _ => Self::COLOR,
        }
    }
}

bitflags!(
    #[derive(Clone, Copy, Debug)]
    pub struct MemoryFlags: u32 {
        const TRANSIENT = 1 << 0;
        const PREFER_COHERENT = 1 << 1;
    }
);

//TODO: it's not intuitive for the backends to consider `LOAD` being optional.

bitflags!(
    #[derive(Clone, Copy, Debug)]
    pub struct AttachmentOps: u8 {
        const LOAD = 1 << 0;
        const STORE = 1 << 1;
    }
);

bitflags::bitflags! {
    /// Similar to `wgt::BufferUsages` but for internal use.
    #[derive(Clone, Copy, Debug)]
    pub struct BufferUses: u16 {
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
        const INCLUSIVE = Self::MAP_READ.bits() | Self::COPY_SRC.bits() |
            Self::INDEX.bits() | Self::VERTEX.bits() | Self::UNIFORM.bits() |
            Self::STORAGE_READ.bits() | Self::INDIRECT.bits();
        /// The combination of states that a buffer must exclusively be in.
        const EXCLUSIVE = Self::MAP_WRITE.bits() | Self::COPY_DST.bits() | Self::STORAGE_READ_WRITE.bits();
        /// The combination of all usages that the are guaranteed to be be ordered by the hardware.
        /// If a usage is ordered, then if the buffer state doesn't change between draw calls, there
        /// are no barriers needed for synchronization.
        const ORDERED = Self::INCLUSIVE.bits() | Self::MAP_WRITE.bits();
    }
}

bitflags::bitflags! {
    /// Similar to `wgt::TextureUsages` but for internal use.
    #[derive(Clone, Copy, Debug)]
    pub struct TextureUses: u16 {
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
        const INCLUSIVE = Self::COPY_SRC.bits() | Self::RESOURCE.bits() | Self::DEPTH_STENCIL_READ.bits();
        /// The combination of states that a texture must exclusively be in.
        const EXCLUSIVE = Self::COPY_DST.bits() | Self::COLOR_TARGET.bits() | Self::DEPTH_STENCIL_WRITE.bits() | Self::STORAGE_READ.bits() | Self::STORAGE_READ_WRITE.bits() | Self::PRESENT.bits();
        /// The combination of all usages that the are guaranteed to be be ordered by the hardware.
        /// If a usage is ordered, then if the texture state doesn't change between draw calls, there
        /// are no barriers needed for synchronization.
        const ORDERED = Self::INCLUSIVE.bits() | Self::COLOR_TARGET.bits() | Self::DEPTH_STENCIL_WRITE.bits() | Self::STORAGE_READ.bits();

        /// Flag used by the wgpu-core texture tracker to say a texture is in different states for every sub-resource
        const COMPLEX = 1 << 10;
        /// Flag used by the wgpu-core texture tracker to say that the tracker does not know the state of the sub-resource.
        /// This is different from UNINITIALIZED as that says the tracker does know, but the texture has not been initialized.
        const UNKNOWN = 1 << 11;
    }
}

#[derive(Clone, Debug)]
pub struct InstanceDescriptor<'a> {
    pub name: &'a str,
    pub flags: InstanceFlags,
    pub dx12_shader_compiler: wgt::Dx12Compiler,
}

#[derive(Clone, Debug)]
pub struct Alignments {
    /// The alignment of the start of the buffer used as a GPU copy source.
    pub buffer_copy_offset: wgt::BufferSize,
    /// The alignment of the row pitch of the texture data stored in a buffer that is
    /// used in a GPU copy operation.
    pub buffer_copy_pitch: wgt::BufferSize,
}

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub limits: wgt::Limits,
    pub alignments: Alignments,
    pub downlevel: wgt::DownlevelCapabilities,
}

#[derive(Debug)]
pub struct ExposedAdapter<A: Api> {
    pub adapter: A::Adapter,
    pub info: wgt::AdapterInfo,
    pub features: wgt::Features,
    pub capabilities: Capabilities,
}

/// Describes information about what a `Surface`'s presentation capabilities are.
/// Fetch this with [Adapter::surface_capabilities].
#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    /// List of supported texture formats.
    ///
    /// Must be at least one.
    pub formats: Vec<wgt::TextureFormat>,

    /// List of supported V-sync modes.
    ///
    /// Must be at least one.
    pub present_modes: Vec<wgt::PresentMode>,

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

    /// List of supported alpha composition modes.
    ///
    /// Must be at least one.
    pub composite_alpha_modes: Vec<wgt::CompositeAlphaMode>,
}

#[derive(Debug)]
pub struct AcquiredSurfaceTexture<A: Api> {
    pub texture: A::SurfaceTexture,
    /// The presentation configuration no longer matches
    /// the surface properties exactly, but can still be used to present
    /// to the surface successfully.
    pub suboptimal: bool,
}

#[derive(Debug)]
pub struct OpenDevice<A: Api> {
    pub device: A::Device,
    pub queue: A::Queue,
}

#[derive(Clone, Debug)]
pub struct BufferMapping {
    pub ptr: NonNull<u8>,
    pub is_coherent: bool,
}

#[derive(Clone, Debug)]
pub struct BufferDescriptor<'a> {
    pub label: Label<'a>,
    pub size: wgt::BufferAddress,
    pub usage: BufferUses,
    pub memory_flags: MemoryFlags,
}

#[derive(Clone, Debug)]
pub struct TextureDescriptor<'a> {
    pub label: Label<'a>,
    pub size: wgt::Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: wgt::TextureDimension,
    pub format: wgt::TextureFormat,
    pub usage: TextureUses,
    pub memory_flags: MemoryFlags,
    /// Allows views of this texture to have a different format
    /// than the texture does.
    pub view_formats: Vec<wgt::TextureFormat>,
}

/// TextureView descriptor.
///
/// Valid usage:
///. - `format` has to be the same as `TextureDescriptor::format`
///. - `dimension` has to be compatible with `TextureDescriptor::dimension`
///. - `usage` has to be a subset of `TextureDescriptor::usage`
///. - `range` has to be a subset of parent texture
#[derive(Clone, Debug)]
pub struct TextureViewDescriptor<'a> {
    pub label: Label<'a>,
    pub format: wgt::TextureFormat,
    pub dimension: wgt::TextureViewDimension,
    pub usage: TextureUses,
    pub range: wgt::ImageSubresourceRange,
}

#[derive(Clone, Debug)]
pub struct SamplerDescriptor<'a> {
    pub label: Label<'a>,
    pub address_modes: [wgt::AddressMode; 3],
    pub mag_filter: wgt::FilterMode,
    pub min_filter: wgt::FilterMode,
    pub mipmap_filter: wgt::FilterMode,
    pub lod_clamp: Option<Range<f32>>,
    pub compare: Option<wgt::CompareFunction>,
    pub anisotropy_clamp: Option<NonZeroU8>,
    pub border_color: Option<wgt::SamplerBorderColor>,
}

/// BindGroupLayout descriptor.
///
/// Valid usage:
/// - `entries` are sorted by ascending `wgt::BindGroupLayoutEntry::binding`
#[derive(Clone, Debug)]
pub struct BindGroupLayoutDescriptor<'a> {
    pub label: Label<'a>,
    pub flags: BindGroupLayoutFlags,
    pub entries: &'a [wgt::BindGroupLayoutEntry],
}

#[derive(Clone, Debug)]
pub struct PipelineLayoutDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    pub flags: PipelineLayoutFlags,
    pub bind_group_layouts: &'a [&'a A::BindGroupLayout],
    pub push_constant_ranges: &'a [wgt::PushConstantRange],
}

#[derive(Debug)]
pub struct BufferBinding<'a, A: Api> {
    /// The buffer being bound.
    pub buffer: &'a A::Buffer,

    /// The offset at which the bound region starts.
    ///
    /// This must be less than the size of the buffer. Some back ends
    /// cannot tolerate zero-length regions; for example, see
    /// [VUID-VkDescriptorBufferInfo-offset-00340][340] and
    /// [VUID-VkDescriptorBufferInfo-range-00341][341], or the
    /// documentation for GLES's [glBindBufferRange][bbr].
    ///
    /// [340]: https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-VkDescriptorBufferInfo-offset-00340
    /// [341]: https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-VkDescriptorBufferInfo-range-00341
    /// [bbr]: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/glBindBufferRange.xhtml
    pub offset: wgt::BufferAddress,

    /// The size of the region bound, in bytes.
    ///
    /// If `None`, the region extends from `offset` to the end of the
    /// buffer. Given the restrictions on `offset`, this means that
    /// the size is always greater than zero.
    pub size: Option<wgt::BufferSize>,
}

// Rust gets confused about the impl requirements for `A`
impl<A: Api> Clone for BufferBinding<'_, A> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer,
            offset: self.offset,
            size: self.size,
        }
    }
}

#[derive(Debug)]
pub struct TextureBinding<'a, A: Api> {
    pub view: &'a A::TextureView,
    pub usage: TextureUses,
}

// Rust gets confused about the impl requirements for `A`
impl<A: Api> Clone for TextureBinding<'_, A> {
    fn clone(&self) -> Self {
        Self {
            view: self.view,
            usage: self.usage,
        }
    }
}

#[derive(Clone, Debug)]
pub struct BindGroupEntry {
    pub binding: u32,
    pub resource_index: u32,
    pub count: u32,
}

/// BindGroup descriptor.
///
/// Valid usage:
///. - `entries` has to be sorted by ascending `BindGroupEntry::binding`
///. - `entries` has to have the same set of `BindGroupEntry::binding` as `layout`
///. - each entry has to be compatible with the `layout`
///. - each entry's `BindGroupEntry::resource_index` is within range
///    of the corresponding resource array, selected by the relevant
///    `BindGroupLayoutEntry`.
#[derive(Clone, Debug)]
pub struct BindGroupDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    pub layout: &'a A::BindGroupLayout,
    pub buffers: &'a [BufferBinding<'a, A>],
    pub samplers: &'a [&'a A::Sampler],
    pub textures: &'a [TextureBinding<'a, A>],
    pub entries: &'a [BindGroupEntry],
}

#[derive(Clone, Debug)]
pub struct CommandEncoderDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    pub queue: &'a A::Queue,
}

/// Naga shader module.
pub struct NagaShader {
    /// Shader module IR.
    pub module: Cow<'static, naga::Module>,
    
    /// Analysis information of the module.
    pub info: naga::valid::ModuleInfo,
}

// Custom implementation avoids the need to generate Debug impl code
// for the whole Naga module and info.
impl fmt::Debug for NagaShader {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Naga shader")
    }
}

/// Shader input.
#[allow(clippy::large_enum_variant)]
pub enum ShaderInput<'a> {
    Naga(NagaShader),
    SpirV(&'a [u32]),
}

pub struct ShaderModuleDescriptor<'a> {
    pub label: Label<'a>,
    pub runtime_checks: bool,
}

/// Describes a programmable pipeline stage.
#[derive(Debug)]
pub struct ProgrammableStage<'a, A: Api> {
    /// The compiled shader module for this stage.
    pub module: &'a A::ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    ///  in the shader.
    pub entry_point: &'a str,
}

// Rust gets confused about the impl requirements for `A`
impl<A: Api> Clone for ProgrammableStage<'_, A> {
    fn clone(&self) -> Self {
        Self {
            module: self.module,
            entry_point: self.entry_point,
        }
    }
}

/// Describes a compute pipeline.
#[derive(Clone, Debug)]
pub struct ComputePipelineDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    pub layout: &'a A::PipelineLayout,
    /// The compiled compute stage and its entry point.
    pub stage: ProgrammableStage<'a, A>,
}

/// Describes how the vertex buffer is interpreted.
#[derive(Clone, Debug)]
pub struct VertexBufferLayout<'a> {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: wgt::BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: wgt::VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: &'a [wgt::VertexAttribute],
}

/// Describes a render (graphics) pipeline.
#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    pub layout: &'a A::PipelineLayout,
    /// The format of any vertex buffers used with this pipeline.
    pub vertex_buffers: &'a [VertexBufferLayout<'a>],
    /// The vertex stage for this pipeline.
    pub vertex_stage: ProgrammableStage<'a, A>,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: wgt::PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<wgt::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: wgt::MultisampleState,
    /// The fragment stage for this pipeline.
    pub fragment_stage: Option<ProgrammableStage<'a, A>>,
    /// The effect of draw calls on the color aspect of the output target.
    pub color_targets: &'a [Option<wgt::ColorTargetState>],
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
}

#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
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

#[derive(Debug, Clone)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

#[derive(Debug, Clone)]
pub struct BufferBarrier<'a, A: Api> {
    pub buffer: &'a A::Buffer,
    pub usage: Range<BufferUses>,
}

#[derive(Debug, Clone)]
pub struct TextureBarrier<'a, A: Api> {
    pub texture: &'a A::Texture,
    pub range: wgt::ImageSubresourceRange,
    pub usage: Range<TextureUses>,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferCopy {
    pub src_offset: wgt::BufferAddress,
    pub dst_offset: wgt::BufferAddress,
    pub size: wgt::BufferSize,
}

#[derive(Clone, Debug)]
pub struct TextureCopyBase {
    pub mip_level: u32,
    pub array_layer: u32,
    /// Origin within a texture.
    /// Note: for 1D and 2D textures, Z must be 0.
    pub origin: wgt::Origin3d,
    pub aspect: FormatAspects,
}

#[derive(Clone, Copy, Debug)]
pub struct CopyExtent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Clone, Debug)]
pub struct TextureCopy {
    pub src_base: TextureCopyBase,
    pub dst_base: TextureCopyBase,
    pub size: CopyExtent,
}

#[derive(Clone, Debug)]
pub struct BufferTextureCopy {
    pub buffer_layout: wgt::ImageDataLayout,
    pub texture_base: TextureCopyBase,
    pub size: CopyExtent,
}

#[derive(Debug)]
pub struct Attachment<'a, A: Api> {
    pub view: &'a A::TextureView,
    /// Contains either a single mutating usage as a target,
    /// or a valid combination of read-only usages.
    pub usage: TextureUses,
}

// Rust gets confused about the impl requirements for `A`
impl<A: Api> Clone for Attachment<'_, A> {
    fn clone(&self) -> Self {
        Self {
            view: self.view,
            usage: self.usage,
        }
    }
}

#[derive(Debug)]
pub struct ColorAttachment<'a, A: Api> {
    pub target: Attachment<'a, A>,
    pub resolve_target: Option<Attachment<'a, A>>,
    pub ops: AttachmentOps,
    pub clear_value: wgt::Color,
}

// Rust gets confused about the impl requirements for `A`
impl<A: Api> Clone for ColorAttachment<'_, A> {
    fn clone(&self) -> Self {
        Self {
            target: self.target.clone(),
            resolve_target: self.resolve_target.clone(),
            ops: self.ops,
            clear_value: self.clear_value,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DepthStencilAttachment<'a, A: Api> {
    pub target: Attachment<'a, A>,
    pub depth_ops: AttachmentOps,
    pub stencil_ops: AttachmentOps,
    pub clear_value: (f32, u32),
}

#[derive(Clone, Debug)]
pub struct RenderPassDescriptor<'a, A: Api> {
    pub label: Label<'a>,
    pub extent: wgt::Extent3d,
    pub sample_count: u32,
    pub color_attachments: &'a [Option<ColorAttachment<'a, A>>],
    pub depth_stencil_attachment: Option<DepthStencilAttachment<'a, A>>,
    pub multiview: Option<NonZeroU32>,
}

#[derive(Clone, Debug)]
pub struct ComputePassDescriptor<'a> {
    pub label: Label<'a>,
}
