use glow::HasContext;
use pi_share::Share;

use super::GLState;
use crate::wgt;

#[derive(Debug)]
pub(crate) struct PipelineLayout {
    pub group_infos: Box<[super::BindGroupLayoutInfo]>,
}

impl PipelineLayout {
    pub fn new(
        state: GLState,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> Result<Self, crate::DeviceError> {
    }
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

impl RenderPipeline {
    pub fn new(
        state: GLState,
        desc: &crate::RenderPipelineDescriptor,
    ) -> Result<Self, super::PipelineError> {
    }
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) raw: glow::Program,
    pub(crate) state: GLState,
}

impl Drop for Program {
    fn drop(&mut self) {
        let gl = self.state.get_gl();

        unsafe {
            gl.delete_program(self.raw);
        }
    }
}

impl Program {
    fn new(
        state: GLState,
        vs: super::ShaderModule,
        fs: super::ShaderModule,
    ) -> Result<Self, super::ShaderError> {
        assert!(vs.shader_type == glow::VERTEX_SHADER);
        assert!(fs.shader_type == glow::FRAGMENT_SHADER);

        let gl = state.get_gl();

        let raw = unsafe {
            let raw = gl.create_program().unwrap();

            gl.attach_shader(raw, vs.raw);
            gl.attach_shader(raw, fs.raw);

            gl.link_program(raw);

            if !gl.get_program_link_status(raw) {
                let info = gl.get_program_info_log(raw);

                log::error!("program link error, info = {:?}", info);

                gl.delete_program(raw);

                return Err(super::ShaderError::LinkProgram(format!(
                    "program link error, info = {:?}",
                    info
                )));
            }

            raw
        };

        Ok(Self { raw, state })
    }
}
