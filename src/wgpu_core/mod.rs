mod api;

mod adapter;
mod device;
mod instance;
mod surface;

mod command;
mod queue;

mod buffer;
mod sampler;
mod texture;

mod query_set;
mod shader_module;

mod bind_group;
mod bind_group_layout;
mod pipeline_layout;

mod compute_pipeline;
mod render_pipeline;

use std::any::Any;

pub use adapter::*;
pub use device::*;
pub use instance::*;
pub use surface::*;

pub use command::*;
pub use queue::*;

pub use buffer::*;
pub use sampler::*;
pub use texture::*;

pub use query_set::*;
pub use shader_module::*;

pub use bind_group::*;
pub use bind_group_layout::*;
pub use pipeline_layout::*;

pub use compute_pipeline::*;
pub use render_pipeline::*;

pub use wgt::{
    AdapterInfo, AddressMode, AstcBlock, AstcChannel, Backend, Backends, BindGroupLayoutEntry,
    BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, BufferAddress,
    BufferBindingType, BufferSize, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandBufferDescriptor, CompareFunction, CompositeAlphaMode, DepthBiasState,
    DepthStencilState, DeviceType, DownlevelCapabilities, DownlevelFlags, Dx12Compiler,
    DynamicOffset, Extent3d, Face, Features, FilterMode, FrontFace, ImageDataLayout,
    ImageSubresourceRange, IndexFormat, InstanceDescriptor, Limits, MultisampleState, Origin2d,
    Origin3d, PipelineStatisticsTypes, PolygonMode, PowerPreference, PredefinedColorSpace,
    PresentMode, PresentationTimestamp, PrimitiveState, PrimitiveTopology, PushConstantRange,
    QueryType, RenderBundleDepthStencil, SamplerBindingType, SamplerBorderColor, ShaderLocation,
    ShaderModel, ShaderStages, StencilFaceState, StencilOperation, StencilState,
    StorageTextureAccess, SurfaceCapabilities, SurfaceStatus, TextureAspect, TextureDimension,
    TextureFormat, TextureFormatFeatureFlags, TextureFormatFeatures, TextureSampleType,
    TextureUsages, TextureViewDimension, VertexAttribute, VertexFormat, VertexStepMode,
    COPY_BUFFER_ALIGNMENT, COPY_BYTES_PER_ROW_ALIGNMENT, MAP_ALIGNMENT, PUSH_CONSTANT_ALIGNMENT,
    QUERY_RESOLVE_BUFFER_ALIGNMENT, QUERY_SET_MAX_QUERIES, QUERY_SIZE, VERTEX_STRIDE_ALIGNMENT,
};

// The underlying types are also exported so that documentation shows up for them

/// Object debugging label.
pub type Label<'a> = Option<&'a str>;

/// Pair of load and store operations for an attachment aspect.
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// separate `loadOp` and `storeOp` fields are used instead.
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct Operations<V> {
    /// How data should be read through this attachment.
    pub load: LoadOp<V>,
    /// Whether data will be written to through this attachment.
    pub store: bool,
}

impl<V: Default> Default for Operations<V> {
    fn default() -> Self {
        Self {
            load: Default::default(),
            store: true,
        }
    }
}

/// Operation to perform to the output attachment at the start of a render pass.
///
/// The render target must be cleared at least once before its content is loaded.
///
/// Corresponds to [WebGPU `GPULoadOp`](https://gpuweb.github.io/gpuweb/#enumdef-gpuloadop).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub enum LoadOp<V> {
    /// Clear with a specified value.
    Clear(V),
    /// Load from memory.
    Load,
}

impl<V: Default> Default for LoadOp<V> {
    fn default() -> Self {
        Self::Clear(Default::default())
    }
}

/// Error type
#[derive(Debug)]
pub enum Error {
    /// Out of memory error
    OutOfMemory {
        /// Lower level source of the error.
        source: Box<dyn std::error::Error + Send + 'static>,
    },
    /// Validation error, signifying a bug in code or data
    Validation {
        /// Lower level source of the error.
        source: Box<dyn std::error::Error + Send + 'static>,
        /// Description of the validation error.
        description: String,
    },
}
static_assertions::assert_impl_all!(Error: Send);

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::OutOfMemory { source } => Some(source.as_ref()),
            Error::Validation { source, .. } => Some(source.as_ref()),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OutOfMemory { .. } => f.write_str("Out of Memory"),
            Error::Validation { description, .. } => f.write_str(description),
        }
    }
}

/// Filter for error scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub enum ErrorFilter {
    /// Catch only out-of-memory errors.
    OutOfMemory,
    /// Catch only validation errors.
    Validation,
}
static_assertions::assert_impl_all!(ErrorFilter: Send, Sync);

/// Type for the callback of uncaptured error handler
pub trait UncapturedErrorHandler: Fn(Error) + Send + 'static {}

impl<T> UncapturedErrorHandler for T where T: Fn(Error) + Send + 'static {}

pub(crate) type Data = dyn Any + Send + Sync;
