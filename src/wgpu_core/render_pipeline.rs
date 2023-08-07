use crate::{
    BindGroupLayout, BufferAddress, ColorTargetState, DepthStencilState, Label, MultisampleState,
    PipelineLayout, PrimitiveState, ShaderModule, VertexAttribute, VertexStepMode,
};
use std::num::NonZeroU32;

/// Handle to a rendering (graphics) pipeline.
///
/// A `RenderPipeline` object represents a graphics pipeline and its stages, bindings, vertex
/// buffers and targets. It can be created with [`Device::create_render_pipeline`].
///
/// Corresponds to [WebGPU `GPURenderPipeline`](https://gpuweb.github.io/gpuweb/#render-pipeline).
#[derive(Debug)]
pub struct RenderPipeline {}
static_assertions::assert_impl_all!(RenderPipeline: Send, Sync);

impl Drop for RenderPipeline {
    fn drop(&mut self) {
        unimplemented!("RenderPipeline::drop is not implemented")
    }
}

impl RenderPipeline {
    /// Get an object representing the bind group layout at a given index.
    pub fn get_bind_group_layout(&self, index: u32) -> BindGroupLayout {
        unimplemented!("RenderPipeline::get_bind_group_layout")
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
static_assertions::assert_impl_all!(RenderPipelineDescriptor: Send, Sync);

/// Describes the vertex processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUVertexState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuvertexstate).
#[derive(Clone, Debug)]
pub struct VertexState<'a> {
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    /// in the shader.
    pub entry_point: &'a str,
    /// The format of any vertex buffers used with this pipeline.
    pub buffers: &'a [VertexBufferLayout<'a>],
}
static_assertions::assert_impl_all!(VertexState: Send, Sync);

/// Describes how the vertex buffer is interpreted.
///
/// For use in [`VertexState`].
///
/// Corresponds to [WebGPU `GPUVertexBufferLayout`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpassdescriptor).
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VertexBufferLayout<'a> {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: &'a [VertexAttribute],
}
static_assertions::assert_impl_all!(VertexBufferLayout: Send, Sync);

/// Describes the fragment processing in a render pipeline.
///
/// For use in [`RenderPipelineDescriptor`].
///
/// Corresponds to [WebGPU `GPUFragmentState`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpufragmentstate).
#[derive(Clone, Debug)]
pub struct FragmentState<'a> {
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    /// in the shader.
    pub entry_point: &'a str,
    /// The color state of the render targets.
    pub targets: &'a [Option<ColorTargetState>],
}
static_assertions::assert_impl_all!(FragmentState: Send, Sync);
