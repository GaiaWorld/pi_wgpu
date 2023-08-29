use pi_share::Share;

use crate::wgt;

#[derive(Debug)]
pub(crate) struct ComputePipeline {}

#[derive(Debug)]
pub(crate) struct PipelineLayout {
    pub group_infos: Box<[super::BindGroupLayoutInfo]>,
    pub naga_options: naga::back::glsl::Options,
}

pub(crate) type PipelineID = u64;

#[derive(Debug)]
pub(crate) struct RenderPipeline {
    pub(crate) pipeline_id: PipelineID,

    pub(crate) topology: u32,
    pub(crate) alpha_to_coverage_enabled: bool,

    pub(crate) program: Share<super::Program>,
    
    pub(crate) color_writes: wgt::ColorWrites,
    pub(crate) vertex_attributes: [Option<super::AttributeDesc>; super::MAX_VERTEX_ATTRIBUTES],

    pub(crate) rs: Share<super::RasterState>,
    pub(crate) ds: Share<super::DepthState>,
    pub(crate) bs: Share<super::BlendState>,
    pub(crate) ss: Share<super::StencilState>,
}
