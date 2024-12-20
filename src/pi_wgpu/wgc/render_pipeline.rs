use derive_more::derive::Debug;

use super::super::{
    hal, BufferAddress, ColorTargetState, DepthStencilState, Label,
    MultisampleState, PipelineLayout, PrimitiveState, ShaderModule, VertexAttribute,
    VertexStepMode,
};
use std::num::NonZeroU32;

/// Handle to a rendering (graphics) pipeline.
///
/// A `RenderPipeline` object represents a graphics pipeline and its stages, bindings, vertex
/// buffers and targets. It can be created with [`Device::create_render_pipeline`].
///
/// Corresponds to [WebGPU `GPURenderPipeline`](https://gpuweb.github.io/gpuweb/#render-pipeline).
#[derive(Debug)]
pub struct RenderPipeline {
    pub(crate) inner: hal::RenderPipeline,
}

impl RenderPipeline {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::RenderPipeline) -> Self {
        Self { inner }
    }
}

/// Describes a render (graphics) pipeline.
///
/// For use with [`Device::create_render_pipeline`].
///
/// Corresponds to [WebGPU `GPURenderPipelineDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpipelinedescriptor).
#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor<'a> {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    #[debug("{}", match layout {Some(r) => format!("Some(&pipeline_layout{:?})", r.inner.id), None => "None".to_string()})]
    pub layout: Option<&'a PipelineLayout>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexState<'a>,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<DepthStencilState>,
    /// The multi-sampling properties of the pip.
    pub multisample: MultisampleState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: Option<FragmentState<'a>>,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
}

/// Describes the vertex processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUVertexState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuvertexstate).
#[derive(Clone, Debug)]
pub struct VertexState<'a> {
    #[debug("&shader_module{:?}", module.inner.id)]
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    /// in the shader.
    pub entry_point: &'a str,
    /// The format of any vertex buffers used with this pipeline
    #[debug("&{buffers:?}")]
    pub buffers: &'a [VertexBufferLayout<'a>],
}


/// Describes how the vertex buffer is interpreted.
///
/// For use in [`VertexState`].
///
/// Corresponds to [WebGPU `GPUVertexBufferLayout`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpassdescriptor).
#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct VertexBufferLayout<'a> {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: BufferAddress,
    /// How often this vertex buffer is "stepped" forward
    pub step_mode: VertexStepMode,
    #[debug("&{attributes:?}")]
    /// The list of attributes which comprise a single vertex.
    pub attributes: &'a [VertexAttribute],
}

/// Describes the fragment processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUFragmentState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpufragmentstate).
#[derive(Clone, Debug)]
pub struct FragmentState<'a> {
    /// The compiled shader module for this stage.
    #[debug("&shader_module{:?}", module.inner.id)]
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    /// in the shader.
    pub entry_point: &'a str,
    /// The color state of the render targets.
    #[debug("&{targets:?}")]
    pub targets: &'a [Option<ColorTargetState>],
}
