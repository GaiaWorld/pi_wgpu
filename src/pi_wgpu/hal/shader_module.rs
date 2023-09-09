use std::borrow::Borrow;

use glow::HasContext;
use naga::{
    back::glsl::{self, ReflectionInfo},
    proc::BoundsCheckPolicy,
    valid::{Capabilities as Caps, ModuleInfo},
    Module, ShaderStage,
};
use pi_share::{cell::Ref, Share, ShareCell};

use super::{super::ShaderBindGroupInfo, AdapterContext, GLState};
use crate::{pi_wgpu::wgt, ShaderModuleDescriptor};

pub(crate) type ShaderID = u64;

#[derive(Debug)]
pub(crate) struct ShaderModule {
    pub(crate) imp: Share<ShareCell<ShaderModuleImpl>>,
}

impl ShaderModule {
    #[inline]
    pub fn new(
        state: GLState,
        adapter: &AdapterContext,
        desc: &ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
        let module = ShaderModuleImpl::new(state, adapter, desc)?;
        let imp = Share::new(ShareCell::new(module));
        Ok(Self { imp })
    }

    #[inline]
    pub fn get_inner<'a>(&'a self) -> Ref<'a, ShaderModuleImpl> {
        self.imp.as_ref().borrow()
    }

    #[inline]
    pub fn compile(
        &self,
        shader_stage: naga::ShaderStage,
        version: &glow::Version,
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        entry_point: String,
        multiview: Option<std::num::NonZeroU32>,
        naga_options: &naga::back::glsl::Options,
    ) -> Result<(), super::ShaderError> {
        self.imp.borrow_mut().compile(
            shader_stage,
            version,
            features,
            downlevel,
            entry_point,
            multiview,
            naga_options,
        )
    }
}

#[derive(Debug)]
pub(crate) struct ShaderModuleImpl {
    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,

    pub(crate) input: Option<ShaderInput>,
    pub(crate) inner: Option<ShaderInner>,
}

impl Drop for ShaderModuleImpl {
    #[inline]
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_ref() {
            let gl = self.adapter.imp.as_ref().borrow();
            let gl = gl.lock();
            unsafe {
                gl.delete_shader(inner.raw);
            }
        }
    }
}

impl ShaderModuleImpl {
    #[inline]
    pub fn new(
        state: GLState,
        adapter: &AdapterContext,
        desc: &ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
        Ok(Self {
            state,
            adapter: adapter.clone(),
            input: Some(ShaderInput::from(desc)),
            inner: None,
        })
    }

    // 在 crate_render_pipeline 中调用
    pub fn compile(
        &mut self,
        shader_stage: naga::ShaderStage,
        version: &glow::Version,
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        entry_point: String,
        multiview: Option<std::num::NonZeroU32>,
        naga_options: &naga::back::glsl::Options,
    ) -> Result<(), super::ShaderError> {
        // 如果编译过了，直接返回
        if self.inner.is_some() {
            return Ok(());
        }

        let mut module: Option<Module> = None;

        let input = self
            .input
            .as_ref()
            .ok_or_else(|| super::ShaderError::Compilation("no input parameter".to_string()))?;

        let module_ref: &Module = match input {
            ShaderInput::Naga(module) => module,
            ShaderInput::Glsl {
                shader,
                stage,
                defines,
            } => {
                assert!(*stage == shader_stage);

                let options = naga::front::glsl::Options {
                    stage: *stage,
                    defines: defines.clone(),
                };
                let mut parser = naga::front::glsl::Frontend::default();
                let m = parser.parse(&options, shader).map_err(|e| {
                    super::ShaderError::Compilation(format!("naga compile shader err = {:?}", e))
                })?;

                module = Some(m);
                module.as_ref().unwrap()
            }
        };

        let entry_point_index = module_ref
            .entry_points
            .iter()
            .position(|ep| ep.name.as_str() == entry_point)
            .ok_or(super::ShaderError::Compilation(
                "Shader not find entry point".to_string(),
            ))?;

        let info = get_shader_info(module_ref, features, downlevel)?;

        let (gl_str, reflection_info) = compile_naga_shader(
            module_ref,
            version,
            &info,
            shader_stage,
            entry_point,
            naga_options,
            multiview,
        )?;

        let shader_type = match shader_stage {
            naga::ShaderStage::Vertex => glow::VERTEX_SHADER,
            naga::ShaderStage::Fragment => glow::FRAGMENT_SHADER,
            naga::ShaderStage::Compute => unreachable!(),
        };

        let raw = {
            let gl = self.adapter.imp.as_ref().borrow();
            let gl = gl.lock();
            compile_gl_shader(&gl, gl_str.as_ref(), shader_type)?
        };

        let bind_group_layout = consume_naga_reflection(
            module_ref,
            &info.get_entry_point(entry_point_index),
            reflection_info,
        )?;

        self.inner = Some(ShaderInner {
            id: self.state.next_shader_id(),
            raw,
            shader_type,
            bind_group_layout,
        });
        self.input = None;

        Ok(())
    }
}

fn get_shader_info(
    module: &Module,
    features: &wgt::Features,
    downlevel: &wgt::DownlevelCapabilities,
) -> Result<ModuleInfo, super::ShaderError> {
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

    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), caps)
        .validate(&module)
        .map_err(|e| super::ShaderError::Compilation(e.to_string()))
}

fn compile_naga_shader(
    module: &Module,
    version: &glow::Version,
    module_info: &ModuleInfo,
    shader_stage: ShaderStage,
    entry_point: String,
    naga_options: &naga::back::glsl::Options,
    multiview: Option<std::num::NonZeroU32>,
) -> Result<(String, ReflectionInfo), super::ShaderError> {
    let image_check = if !version.is_embedded && (version.major, version.minor) >= (1, 3) {
        BoundsCheckPolicy::ReadZeroSkipWrite
    } else {
        BoundsCheckPolicy::Unchecked
    };

    // Other bounds check are either provided by glsl or not implemented yet.
    let policies = naga::proc::BoundsCheckPolicies {
        index: BoundsCheckPolicy::Unchecked,
        buffer: BoundsCheckPolicy::Unchecked,
        image: image_check,
        binding_array: BoundsCheckPolicy::Unchecked,
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage,
        entry_point,
        multiview,
    };

    let mut output = String::new();
    let mut writer = glsl::Writer::new(
        &mut output,
        &module,
        &module_info,
        naga_options,
        &pipeline_options,
        policies,
    )
    .map_err(|e| super::ShaderError::Compilation(format!("glsl::Writer::new() error = {:?}", e)))?;

    let reflection_info = writer.write().map_err(|e| {
        super::ShaderError::Compilation(format!("glsl::Writer::write() error = {:?}", e))
    })?;

    Ok((output, reflection_info))
}

fn compile_gl_shader(
    gl: &glow::Context,
    source: &str,
    shader_type: u32,
) -> Result<glow::Shader, super::ShaderError> {
    let raw = unsafe {
        gl.create_shader(shader_type)
            .map_err(|e| super::ShaderError::Compilation("gl.create_shader error".to_string()))
    }?;

    unsafe { gl.shader_source(raw, source.as_ref()) };

    unsafe { gl.compile_shader(raw) };

    if unsafe { gl.get_shader_completion_status(raw) } {
        Ok(raw)
    } else {
        let info = unsafe { gl.get_shader_info_log(raw) };

        log::error!(
            "shader compile error, type = {:?}, info = {:?}, source = {:?}",
            shader_type,
            info,
            source
        );

        unsafe { gl.delete_shader(raw) };

        Err(super::ShaderError::Compilation(format!(
            "shader compile error, info = {:?}",
            info
        )))
    }
}

fn consume_naga_reflection(
    module: &naga::Module,
    ep_info: &naga::valid::FunctionInfo,
    reflection_info: naga::back::glsl::ReflectionInfo,
) -> Result<Vec<ShaderBindGroupInfo>, super::ShaderError> {
    let mut r = vec![];

    for (handle, name) in reflection_info.uniforms {
        let var = &module.global_variables[handle];
        let br = var.binding.as_ref().unwrap();
        r.push(ShaderBindGroupInfo {
            name: name.clone(),
            set: br.group as usize,
            binding: br.binding as usize,
            ty: crate::PiBindingType::Buffer,
        });
    }

    for (name, mapping) in reflection_info.texture_mapping {
        assert!(mapping.sampler.is_some());
        let sampler_handle = mapping.sampler.unwrap();
        let sampler_var = &module.global_variables[sampler_handle];
        let sampler_br = sampler_var.binding.as_ref().unwrap();
        r.push(ShaderBindGroupInfo {
            name: name.clone(),
            set: sampler_br.group as usize,
            binding: sampler_br.binding as usize,
            ty: crate::PiBindingType::Sampler,
        });

        let tex_var = &module.global_variables[mapping.texture];
        let tex_br = tex_var.binding.as_ref().unwrap();
        r.push(ShaderBindGroupInfo {
            name: name,
            set: tex_br.group as usize,
            binding: tex_br.binding as usize,
            ty: crate::PiBindingType::Texture,
        });
    }

    Ok(r)
}

#[derive(Debug)]
pub(crate) enum ShaderInput {
    Naga(naga::Module),
    Glsl {
        shader: String,
        stage: naga::ShaderStage,
        defines: naga::FastHashMap<String, String>,
    },
}

impl From<&ShaderModuleDescriptor<'_>> for ShaderInput {
    #[inline]
    fn from(value: &ShaderModuleDescriptor) -> Self {
        match &value.source {
            crate::ShaderSource::Naga(module) => {
                let module = match module {
                    std::borrow::Cow::Borrowed(m) => (**m).clone(),
                    std::borrow::Cow::Owned(m) => (*m).clone(),
                };
                Self::Naga(module)
            }
            crate::ShaderSource::Glsl {
                shader,
                stage,
                defines,
            } => Self::Glsl {
                shader: shader.to_string(),
                stage: *stage,
                defines: defines.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct ShaderInner {
    pub(crate) id: ShaderID,
    pub(crate) raw: glow::Shader,
    pub(crate) shader_type: u32, // glow::VERTEX_SHADER,
    pub(crate) bind_group_layout: Vec<ShaderBindGroupInfo>,
}
