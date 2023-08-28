use std::sync::Arc;

use crate::wgt;

#[derive(Debug)]
pub(crate) struct ComputePipeline {}

#[derive(Debug)]
pub(crate) struct PipelineLayout {
    pub group_infos: Box<[super::BindGroupLayoutInfo]>,
    pub naga_options: naga::back::glsl::Options,
}

#[derive(Debug)]
pub(crate) struct RenderPipeline {
    inner: Arc<PipelineInner>,
    primitive: wgt::PrimitiveState,
    vertex_buffers: Box<[VertexBufferDesc]>,
    vertex_attributes: Box<[AttributeDesc]>,
    color_targets: Box<[ColorTargetDesc]>,
    depth: Option<DepthState>,
    depth_bias: wgt::DepthBiasState,
    stencil: Option<StencilState>,
    alpha_to_coverage_enabled: bool,
}

#[derive(Clone, Debug, Default)]
struct UniformDesc {
    location: Option<glow::UniformLocation>,
    size: u32,
    utype: u32,
}

/// For each texture in the pipeline layout, store the index of the only
/// sampler (in this layout) that the texture is used with.
type SamplerBindMap = [Option<u8>; super::MAX_TEXTURE_SLOTS];

#[derive(Debug)]
struct PipelineInner {
    program: glow::Program,
    sampler_map: SamplerBindMap,
    uniforms: [UniformDesc; super::MAX_PUSH_CONSTANTS],
}

#[derive(Clone, Debug, Default, PartialEq)]
struct VertexBufferDesc {
    step: wgt::VertexStepMode,
    stride: u32,
}

#[derive(Debug, Clone, Copy)]
enum VertexAttribKind {
    Float, // glVertexAttribPointer
    Integer, // glVertexAttribIPointer
           //Double,  // glVertexAttribLPointer
}

impl Default for VertexAttribKind {
    fn default() -> Self {
        Self::Float
    }
}

#[derive(Clone, Debug, Default)]
struct VertexFormatDesc {
    element_count: i32,
    element_format: u32,
    attrib_kind: VertexAttribKind,
}

#[derive(Clone, Debug, Default)]
struct AttributeDesc {
    location: u32,
    offset: u32,
    buffer_index: u32,
    format_desc: VertexFormatDesc,
}

#[derive(Clone, Debug, PartialEq)]
struct BlendComponent {
    src: u32,
    dst: u32,
    equation: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct BlendDesc {
    alpha: BlendComponent,
    color: BlendComponent,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ColorTargetDesc {
    mask: wgt::ColorWrites,
    blend: Option<BlendDesc>,
}

#[derive(Clone, Debug)]
struct DepthState {
    function: u32,
    mask: bool,
}

#[derive(Clone, Debug, Default)]
struct StencilState {
    front: StencilSide,
    back: StencilSide,
}

#[derive(Clone, Debug, PartialEq)]
struct StencilOps {
    pass: u32,
    fail: u32,
    depth_fail: u32,
}

impl Default for StencilOps {
    fn default() -> Self {
        Self {
            pass: glow::KEEP,
            fail: glow::KEEP,
            depth_fail: glow::KEEP,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct StencilSide {
    function: u32,
    mask_read: u32,
    mask_write: u32,
    reference: u32,
    ops: StencilOps,
}

impl Default for StencilSide {
    fn default() -> Self {
        Self {
            function: glow::ALWAYS,
            mask_read: 0xFF,
            mask_write: 0xFF,
            reference: 0,
            ops: StencilOps::default(),
        }
    }
}
