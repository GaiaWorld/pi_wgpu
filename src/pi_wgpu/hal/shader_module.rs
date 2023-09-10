use std::collections::HashMap;

use glow::HasContext;
use naga::{
    back::glsl::{self, ReflectionInfo},
    proc::BoundsCheckPolicy,
    valid::{Capabilities as Caps, ModuleInfo},
    Module, ShaderStage,
};
use pi_share::{Share, ShareCell};

use super::{
    super::{wgt, ShaderModuleDescriptor},
    AdapterContext, GLState,
};

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
    pub fn compile(
        &self,
        shader_stage: naga::ShaderStage,
        version: &glow::Version,
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        entry_point: String,
        multiview: Option<std::num::NonZeroU32>,
        naga_options: &naga::back::glsl::Options,
        shader_binding_map: &mut ShaderBindingMap,
    ) -> Result<(), super::ShaderError> {
        self.imp.borrow_mut().compile(
            shader_stage,
            version,
            features,
            downlevel,
            entry_point,
            multiview,
            naga_options,
            shader_binding_map,
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
            let gl = self.adapter.lock();
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
        shader_binding_map: &mut ShaderBindingMap,
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
            let gl = self.adapter.lock();
            compile_gl_shader(&gl, gl_str.as_ref(), shader_type)?
        };

        let bg_set_info = consume_naga_reflection(
            module_ref,
            &info.get_entry_point(entry_point_index),
            reflection_info,
            shader_binding_map,
        )?;

        self.inner = Some(ShaderInner {
            id: self.state.next_shader_id(),
            raw,
            shader_type,
            bg_set_info,
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
    shader_binding_map: &mut ShaderBindingMap,
) -> Result<[Box<[PiBindEntry]>; super::MAX_BIND_GROUPS], super::ShaderError> {
    let mut r = [vec![], vec![], vec![], vec![]];

    // UBO
    for (handle, name) in reflection_info.uniforms {
        let var = &module.global_variables[handle];
        let br = var.binding.as_ref().unwrap();

        let glow_binding = shader_binding_map.get_or_insert_ubo(br.clone());

        let set = &mut r[br.group as usize];
        set.push(PiBindEntry {
            binding: br.binding as usize,
            ty: PiBindingType::Buffer,

            glsl_name: name,
            glow_binding,
        });
    }

    // Sampler / Texture
    for (name, mapping) in reflection_info.texture_mapping {
        assert!(mapping.sampler.is_some());

        let sampler_handle = mapping.sampler.unwrap();
        let sampler_var = &module.global_variables[sampler_handle];
        let sampler_br = sampler_var.binding.as_ref().unwrap();

        let glow_binding = shader_binding_map.get_or_insert_ubo(sampler_br.clone());

        let set = &mut r[sampler_br.group as usize];
        set.push(PiBindEntry {
            binding: sampler_br.binding as usize,
            ty: PiBindingType::Sampler,

            glsl_name: name.clone(),
            glow_binding,
        });

        let tex_var = &module.global_variables[mapping.texture];
        let tex_br = tex_var.binding.as_ref().unwrap();
        let set = &mut r[tex_br.group as usize];
        set.push(PiBindEntry {
            binding: tex_br.binding as usize,
            ty: PiBindingType::Texture,

            glsl_name: name,
            glow_binding,
        });
    }

    let mut us: [Box<[PiBindEntry]>; super::MAX_BIND_GROUPS] = Default::default();
    for (boxed_slice, vec) in us.iter_mut().zip(r.iter_mut()) {
        *boxed_slice = vec.drain(..).collect::<Vec<_>>().into_boxed_slice();
    }
    Ok(us)
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
    pub(crate) bg_set_info: [Box<[PiBindEntry]>; super::MAX_BIND_GROUPS],
}

//
// create_program: 确定 binding 和 Type
//      对 UBO:
//          var blockIndex = gl.getUniformBlockIndex(program, glsl_name);
//          gl.uniformBlockBinding(program, blockIndex, glow_binding);
//      对 Texture / Sampler
//          var mySampler = gl.getUniformLocation(program, glsl_name);
//          gl.uniform1i(mySampler, glow_binding);
//
// set_bind_group: 比较 和 设置 gl-函数
//      对 UBO:
//          gl.bindBufferRange(gl.UNIFORM_BUFFER, glow_binding, ubuffer, offset, size);
//      对 Texture:
//          gl.activeTexture(gl.TEXTURE0 + glow_binding);
//          gl.bindTexture(gl.TEXTURE_2D, texture);
//      对 Sampler:
//          gl.bindSampler(glow_binding, sampler);
//
#[derive(Clone, Debug)]
pub(crate) struct PiBindEntry {
    pub(crate) binding: usize, // glsl 450 写的 lyaout(set=yyy, bind=yyy) 的 yyy
    pub(crate) ty: PiBindingType,

    // glsl 300 中：Sampler 和 Texture 对应的是同一个名字和绑定
    pub(crate) glsl_name: String, // 编译后的名字
    pub(crate) glow_binding: u32, // 编译后 由 程序分配的绑定，Buffer 和 Sampler/Texture 分两组编码
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub(crate) enum PiBindingType {
    Buffer,
    Texture,
    Sampler,
}

#[derive(Debug)]
pub(crate) struct ShaderBindingMap {
    pub(crate) next_ubo_id: usize,
    pub(crate) max_uniform_buffer_bindings: usize,
    pub(crate) ubo_map: HashMap<naga::ResourceBinding, usize>,

    pub(crate) next_sampler_id: usize,
    pub(crate) max_textures_slots: usize,
    pub(crate) sampler_map: HashMap<naga::ResourceBinding, usize>,
}

impl ShaderBindingMap {
    #[inline]
    pub(crate) fn new(max_uniform_buffer_bindings: usize, max_textures_slots: usize) -> Self {
        Self {
            next_ubo_id: 0,
            max_uniform_buffer_bindings,
            ubo_map: Default::default(),

            next_sampler_id: 0,
            max_textures_slots,
            sampler_map: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn get_or_insert_ubo(&mut self, binding: naga::ResourceBinding) -> u32 {
        let r = self.ubo_map.entry(binding).or_insert_with(|| {
            let r = self.next_ubo_id;
            self.next_ubo_id += 1;
            self.next_ubo_id %= self.max_uniform_buffer_bindings;
            r
        });

        *r as u32
    }

    #[inline]
    pub(crate) fn get_or_insert_sampler(&mut self, binding: naga::ResourceBinding) -> u32 {
        let r = self.sampler_map.entry(binding).or_insert_with(|| {
            let r = self.next_sampler_id;
            self.next_sampler_id += 1;
            self.next_sampler_id %= self.max_textures_slots;
            r
        });

        *r as u32
    }
}
