use glow::HasContext;

use super::super::ShaderBindGroupInfo;

use super::GLState;

pub(crate) type ShaderID = u64;

#[derive(Debug)]
pub(crate) struct ShaderModule {
    pub(crate) id: ShaderID,

    pub(crate) raw: glow::Shader,
    pub(crate) shader_type: u32, // glow::VERTEX_SHADER,
    pub(crate) state: GLState,
    pub(crate) bind_group_layout: Vec<ShaderBindGroupInfo>,
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        let gl = &self.state.0.borrow().gl;
        unsafe {
            gl.delete_shader(self.raw);
        }
    }
}

impl ShaderModule {
    pub fn new(
        state: GLState,
        desc: &super::super::ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
		todo!()
        // let gl = &state.0.borrow().gl;
        // match &desc.source {
        //     super::super::ShaderSource::Glsl {
        //         shader,
        //         stage,
        //         defines,
        //         bind_group_layout,
        //     } => {
        //         assert!(defines.len() == 0);

        //         let shader_type = match stage {
        //             naga::ShaderStage::Vertex => glow::VERTEX_SHADER,
        //             naga::ShaderStage::Fragment => glow::FRAGMENT_SHADER,
        //             naga::ShaderStage::Compute => unreachable!(),
        //         };

        //         let (raw, bind_group_layout) = unsafe {
        //             let raw = gl.create_shader(shader_type).unwrap();

        //             gl.shader_source(raw, shader.as_ref());

        //             gl.compile_shader(raw);

        //             if !gl.get_shader_completion_status(raw) {
        //                 let info = gl.get_shader_info_log(raw);

        //                 log::error!(
        //                     "shader compile error, type = {:?}, info = {:?}, source = {:?}",
        //                     shader_type,
        //                     info,
        //                     shader
        //                 );

        //                 gl.delete_shader(raw);

        //                 return Err(super::ShaderError::Compilation(format!(
        //                     "shader compile error, info = {:?}",
        //                     info
        //                 )));
        //             }

        //             (raw, bind_group_layout.clone())
        //         };

        //         Ok(Self {
        //             id: state.next_shader_id(),
        //             raw,
        //             shader_type,
        //             state: state.clone(),
        //             bind_group_layout,
        //         })
        //     }
        // }
    }
}
