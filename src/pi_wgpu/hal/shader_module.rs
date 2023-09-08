use glow::HasContext;
use naga::{
    valid::{Capabilities as Caps, ModuleInfo},
    Module,
};
use pi_share::Share;

use crate::pi_wgpu::wgt;

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
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        desc: &super::super::ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
        let gl = adapter.lock();

        match &desc.source {
            super::super::ShaderSource::Naga(module) => {
                todo!()
            }
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

                    todo!()
                    // (raw, bind_group_layout.clone())
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

// fn compile_to_glsl3(features: &wgt::Features) {
//     let mut parser = naga::front::glsl::Frontend::default();
//     let module = parser
//         .parse(
//             &naga::front::glsl::Options::from(naga::ShaderStage::Fragment),
//             s,
//         )
//         .unwrap();
//     // println!("Naga module:\n{:?}", &module);
//     let module_info = info(&module, features);

//     // let version = gl.version(); // glow::gl.version()
//     let version = glow::Version {
//         major: 3,
//         minor: 0,
//         is_embedded: false,
//         revision: None,
//         vendor_info: "sun".to_string(),
//     };

//     let image_check = if !version.is_embedded && (version.major, version.minor) >= (1, 3) {
//         BoundsCheckPolicy::ReadZeroSkipWrite
//     } else {
//         BoundsCheckPolicy::Unchecked
//     };

//     // Other bounds check are either provided by glsl or not implemented yet.
//     let policies = naga::proc::BoundsCheckPolicies {
//         index: BoundsCheckPolicy::Unchecked,
//         buffer: BoundsCheckPolicy::Unchecked,
//         image: image_check,
//         binding_array: BoundsCheckPolicy::Unchecked,
//     };

//     let pipeline_options = naga::back::glsl::PipelineOptions {
//         shader_stage: naga::ShaderStage::Fragment,
//         entry_point: "main".to_string(), // create_render_pipeline函数会传入该参数,
//         multiview: None,                 // create_render_pipeline函数会传入该参数,
//     };

//     let naga_options = naga::back::glsl::Options::default();
//     let mut output = String::new();
//     let mut writer = naga::back::glsl::Writer::new(
//         &mut output,
//         &module,
//         &module_info,
//         &naga_options, // &context.layout.naga_options, // 在create_pipeline_layout中创建该字段
//         &pipeline_options,
//         policies,
//     )
//     .unwrap(); // 处理错误， TODO
//                // .map_err(|e| {
//                // 	let msg = format!("{e}");
//                // 	crate::PipelineError::Linkage(map_naga_stage(naga_stage), msg)
//                // })?;

//     let reflection_info = writer.write().unwrap(); // 处理错误， TODO
//     println!("Naga generated shader:\n{}", &output);
// }

fn get_shader_info(
    module: &Module,
    features: &wgt::Features,
    downlevel: &wgt::DownlevelCapabilities,
) -> ModuleInfo {
    let mut caps = Caps::empty();
    caps.set(
        Caps::PUSH_CONSTANT,
        features.contains(wgt::Features::PUSH_CONSTANTS),
    );
    caps.set(Caps::FLOAT64, features.contains(wgt::Features::SHADER_F64));
    caps.set(
        Caps::PRIMITIVE_INDEX,
        features.contains(wgt::Features::SHADER_PRIMITIVE_INDEX),
    );
    caps.set(
        Caps::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING),
    );
    caps.set(
        Caps::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING),
    );
    // TODO: This needs a proper wgpu feature
    caps.set(
        Caps::SAMPLER_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING),
    );
    caps.set(
        Caps::STORAGE_TEXTURE_16BIT_NORM_FORMATS,
        features.contains(wgt::Features::TEXTURE_FORMAT_16BIT_NORM),
    );
    caps.set(Caps::MULTIVIEW, features.contains(wgt::Features::MULTIVIEW));
    caps.set(
        Caps::EARLY_DEPTH_TEST,
        features.contains(wgt::Features::SHADER_EARLY_DEPTH_TEST),
    );
    caps.set(
        Caps::MULTISAMPLED_SHADING,
        downlevel
            .flags
            .contains(wgt::DownlevelFlags::MULTISAMPLED_SHADING),
    );

    let info = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), caps)
        .validate(&module)
        .unwrap(); // 处理错误， TODO

    info
}
