use pi_hash::XHashMap;

use super::{
    super::ShaderModuleDescriptor,
    AdapterContext, GLState,
};

pub(crate) type ShaderID = u64;

#[derive(Debug)]
pub(crate) struct ShaderModule {
    state: GLState,
    adapter: AdapterContext,

    pub(crate) id: ShaderID,
    pub(crate) input: ShaderInput,
}

impl Drop for ShaderModule {
    #[inline]
    fn drop(&mut self) {
		log::trace!("Dropping ShaderModule {:?}", self.id);
        self.state.remove_shader(self.id);
    }
}

impl ShaderModule {
    #[inline]
    pub(crate) fn new(
        state: GLState,
        adapter: &AdapterContext,
        desc: &ShaderModuleDescriptor,
    ) -> Result<Self, super::ShaderError> {
        let id = state.next_shader_id();

        Ok(Self {
            state,
            adapter: adapter.clone(),
            id,
            input: ShaderInput::from(desc),
        })
    }
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
    next_ubo_id: usize,
    max_uniform_buffer_bindings: usize,
    ubo_map: XHashMap<PiResourceBinding, usize>,

    next_sampler_id: usize,
    max_textures_slots: usize,
    sampler_map: XHashMap<PiResourceBinding, usize>,
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
    pub(crate) fn get_or_insert_ubo(&mut self, binding: PiResourceBinding) -> u32 {
        let r = self.ubo_map.entry(binding).or_insert_with(|| {
            let r = self.next_ubo_id;

            self.next_ubo_id += 1;
            self.next_ubo_id %= self.max_uniform_buffer_bindings;

            r
        });

        *r as u32
    }

    #[inline]
    pub(crate) fn get_or_insert_sampler(&mut self, binding: PiResourceBinding) -> u32 {
        let r = self.sampler_map.entry(binding).or_insert_with(|| {
            let r = self.next_sampler_id;

            self.next_sampler_id += 1;
            self.next_sampler_id %= self.max_textures_slots;

            r
        });

        *r as u32
    }
}

/// Pipeline binding information for global resources.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct PiResourceBinding {
    /// The bind group index.
    pub group: u32,
    /// Binding number within the group.
    pub binding: u32,
}
