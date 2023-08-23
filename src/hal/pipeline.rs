#[derive(Debug)]
pub(crate) struct PipelineLayout {
    pub group_infos: Box<[super::BindGroupLayoutInfo]>,
    pub naga_options: naga::back::glsl::Options,
}

#[derive(Debug)]
pub(crate) struct RenderPipeline {}

#[derive(Debug)]
pub(crate) struct ComputePipeline {}
