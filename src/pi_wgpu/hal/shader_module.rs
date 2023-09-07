use glow::HasContext;
use pi_share::Share;

use super::super::ShaderBindGroupInfo;

use super::{AdapterContext, GLState};

pub(crate) type ShaderID = u64;

#[derive(Debug)]
pub(crate) struct ShaderModule {
    pub(crate) state: GLState,
    pub(crate) adapter: Share<AdapterContext>,

    pub(crate) id: ShaderID,
    pub(crate) raw: glow::Shader,
    pub(crate) shader_type: u32, // glow::VERTEX_SHADER,

    pub(crate) bind_group_layout: Vec<ShaderBindGroupInfo>,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let gl = self.adapter.lock();
        unsafe {
            gl.delete_shader(self.raw);
        }
    }
}

impl ShaderModule {
    pub fn new(
        state: GLState,
        adapter: &Share<AdapterContext>,
        desc: &super::super::ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
        let gl = adapter.lock();

        match &desc.source {
            super::super::ShaderSource::Naga(module) => {}
            super::super::ShaderSource::Glsl {
                shader,
                stage,
                defines,
            } => {
                assert!(defines.len() == 0);

                let shader_type = match stage {
                    naga::ShaderStage::Vertex => glow::VERTEX_SHADER,
                    naga::ShaderStage::Fragment => glow::FRAGMENT_SHADER,
                    naga::ShaderStage::Compute => unreachable!(),
                };

                let (raw, bind_group_layout) = unsafe {
                    let raw = gl.create_shader(shader_type).unwrap();

                    gl.shader_source(raw, shader.as_ref());

                    gl.compile_shader(raw);

                    if !gl.get_shader_completion_status(raw) {
                        let info = gl.get_shader_info_log(raw);

                        log::error!(
                            "shader compile error, type = {:?}, info = {:?}, source = {:?}",
                            shader_type,
                            info,
                            shader
                        );

                        gl.delete_shader(raw);

                        return Err(super::ShaderError::Compilation(format!(
                            "shader compile error, info = {:?}",
                            info
                        )));
                    }

                    (raw, bind_group_layout.clone())
                };

                let id = state.next_shader_id();
                Ok(Self {
                    state,
                    adapter: adapter.clone(),

                    id,
                    raw,
                    shader_type,
                    bind_group_layout,
                })
            }
        }
    }
}
