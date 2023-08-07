use std::{borrow::Cow, marker::PhantomData};

use crate::Label;

/// Handle to a compiled shader module.
///
/// A `ShaderModule` represents a compiled shader module on the GPU. It can be created by passing
/// source code to [`Device::create_shader_module`] or valid SPIR-V binary to
/// [`Device::create_shader_module_spirv`]. Shader modules are used to define programmable stages
/// of a pipeline.
///
/// Corresponds to [WebGPU `GPUShaderModule`](https://gpuweb.github.io/gpuweb/#shader-module).
#[derive(Debug)]
pub struct ShaderModule {}
static_assertions::assert_impl_all!(ShaderModule: Send, Sync);

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unimplemented!("ShaderModule::drop is not implemented")
    }
}

/// Source of a shader module.
///
/// The source will be parsed and validated.
///
/// Any necessary shader translation (e.g. from WGSL to SPIR-V or vice versa)
/// will be done internally by wgpu.
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// only WGSL source code strings are accepted.
#[cfg_attr(feature = "naga", allow(clippy::large_enum_variant))]
#[derive(Clone)]
#[non_exhaustive]
pub enum ShaderSource<'a> {
    /// SPIR-V module represented as a slice of words.
    ///
    /// See also: [`util::make_spirv`], [`include_spirv`]
    #[cfg(feature = "spirv")]
    SpirV(Cow<'a, [u32]>),
    /// GLSL module as a string slice.
    ///
    /// Note: GLSL is not yet fully supported and must be a specific ShaderStage.
    #[cfg(feature = "glsl")]
    Glsl {
        /// The source code of the shader.
        shader: Cow<'a, str>,
        /// The shader stage that the shader targets. For example, `naga::ShaderStage::Vertex`
        stage: naga::ShaderStage,
        /// Defines to unlock configured shader features.
        defines: naga::FastHashMap<String, String>,
    },
    /// WGSL module as a string slice.
    #[cfg(feature = "wgsl")]
    Wgsl(Cow<'a, str>),
    /// Naga module.
    #[cfg(feature = "naga")]
    Naga(Cow<'static, naga::Module>),
    /// Dummy variant because `Naga` doesn't have a lifetime and without enough active features it
    /// could be the last one active.
    #[doc(hidden)]
    Dummy(PhantomData<&'a ()>),
}
static_assertions::assert_impl_all!(ShaderSource: Send, Sync);

/// Descriptor for use with [`Device::create_shader_module`].
///
/// Corresponds to [WebGPU `GPUShaderModuleDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpushadermoduledescriptor).
#[derive(Clone)]
pub struct ShaderModuleDescriptor<'a> {
    /// Debug label of the shader module. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// Source code for the shader.
    pub source: ShaderSource<'a>,
}
static_assertions::assert_impl_all!(ShaderModuleDescriptor: Send, Sync);

/// Descriptor for a shader module given by SPIR-V binary, for use with
/// [`Device::create_shader_module_spirv`].
///
/// This type is unique to the Rust API of `wgpu`. In the WebGPU specification,
/// only WGSL source code strings are accepted.
pub struct ShaderModuleDescriptorSpirV<'a> {
    /// Debug label of the shader module. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// Binary SPIR-V data, in 4-byte words.
    pub source: Cow<'a, [u32]>,
}
static_assertions::assert_impl_all!(ShaderModuleDescriptorSpirV: Send, Sync);
