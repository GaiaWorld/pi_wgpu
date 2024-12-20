


mod hal;
mod wgc;
mod wgt;
pub mod util;

pub use hal::{api::*, ExposedAdapter};

pub use wgc::*;
use derive_more::Debug;

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
    InstanceFlags, Gles3MinorVersion,
    
};

#[cfg(target_arch = "wasm32")]
pub use wgt::{ImageCopyExternalImage, ExternalImageSource};
pub use util::TextureDataOrder;

use std::any::Any;

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
    ///
    /// Note that resolve textures (if specified) are always written to,
    /// regardless of this setting.
    pub store: StoreOp,
}

/// Operation to perform to the output attachment at the end of a render pass.
///
/// Corresponds to [WebGPU `GPUStoreOp`](https://gpuweb.github.io/gpuweb/#enumdef-gpustoreop).
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub enum StoreOp {
    /// Stores the resulting value of the render pass for this attachment.
    #[default]
    #[debug("StoreOp::Store")]
    Store,
    /// Discards the resulting value of the render pass for this attachment.
    ///
    /// The attachment will be treated as uninitialized afterwards.
    /// (If only either Depth or Stencil texture-aspects is set to `Discard`,
    /// the respective other texture-aspect will be preserved.)
    ///
    /// This can be significantly faster on tile-based render hardware.
    ///
    /// Prefer this if the attachment is not read by subsequent passes.
    #[debug("StoreOp::Discard")]
    Discard,
}

impl<V: Default> Default for Operations<V> {
    #[inline]
    fn default() -> Self {
        Self {
            load: LoadOp::<V>::default(),
            store: StoreOp::default(),
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
    #[debug("LoadOp::Clear({:?})", _0)]
    Clear(V),
    /// Load from memory.
    #[debug("LoadOp::Load")]
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

pub(crate) type Data = dyn Any + Send + Sync;
