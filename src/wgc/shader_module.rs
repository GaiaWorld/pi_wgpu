use std::{borrow::Cow, marker::PhantomData};

use thiserror::Error;

use crate::{DeviceError, Label, MissingFeatures};

/// Handle to a compiled shader module.
///
/// A `ShaderModule` represents a compiled shader module on the GPU. It can be created by passing
/// source code to [`Device::create_shader_module`] or valid SPIR-V binary to
/// [`Device::create_shader_module_spirv`]. Shader modules are used to define programmable stages
/// of a pipeline.
///
/// Corresponds to [WebGPU `GPUShaderModule`](https://gpuweb.github.io/gpuweb/#shader-module).
#[derive(Debug)]
pub struct ShaderModule {
    pub(crate) inner: crate::hal::ShaderModule,
}

impl ShaderModule {
    #[inline]
    pub(crate) fn from_hal(inner: crate::hal::ShaderModule) -> Self {
        Self { inner }
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
#[derive(Clone)]
#[non_exhaustive]
pub enum ShaderSource<'a> {
    Glsl {
        /// The source code of the shader.
        shader: Cow<'a, str>,
        /// The shader stage that the shader targets. For example, `naga::ShaderStage::Vertex`
        stage: naga::ShaderStage,
        /// Defines to unlock configured shader features.
        defines: naga::FastHashMap<String, String>,

        // pi_wgpu 特有 字段: 每个索引 都是 set 的 索引
        bind_group_layout: Vec<ShaderBindGroupInfo>,
    },
}

#[derive(Clone, Debug)]
pub struct ShaderBindGroupInfo {
    set: u32,
    binding: u32,
    name: String,
    ty: BindingType,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub enum BindingType {
    Buffer,
    Texture,
    Sampler,
}

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

#[allow(clippy::large_enum_variant)]
pub(crate) enum ShaderModuleSource<'a> {
    #[cfg(feature = "wgsl")]
    Wgsl(Cow<'a, str>),
    Naga(Cow<'static, naga::Module>),
    /// Dummy variant because `Naga` doesn't have a lifetime and without enough active features it
    /// could be the last one active.
    #[doc(hidden)]
    Dummy(PhantomData<&'a ()>),
}

//Note: `Clone` would require `WithSpan: Clone`.
#[derive(Debug, Error)]
pub(crate) enum CreateShaderModuleError {
    #[cfg(feature = "wgsl")]
    #[error(transparent)]
    Parsing(#[from] ShaderError<naga::front::wgsl::ParseError>),
    #[error("Failed to generate the backend-specific code")]
    Generation,
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error(transparent)]
    Validation(#[from] ShaderError<naga::WithSpan<naga::valid::ValidationError>>),
    #[error(transparent)]
    MissingFeatures(#[from] MissingFeatures),
    #[error(
        "shader global {bind:?} uses a group index {group} that exceeds the max_bind_groups limit of {limit}."
    )]
    InvalidGroupIndex {
        bind: naga::ResourceBinding,
        group: u32,
        limit: u32,
    },
}

impl CreateShaderModuleError {
    pub fn location(&self, source: &str) -> Option<naga::SourceLocation> {
        match *self {
            #[cfg(feature = "wgsl")]
            CreateShaderModuleError::Parsing(ref err) => err.inner.location(source),
            CreateShaderModuleError::Validation(ref err) => err.inner.location(source),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShaderError<E> {
    pub source: String,
    pub label: Option<String>,
    pub inner: Box<E>,
}

#[cfg(feature = "wgsl")]
impl std::fmt::Display for ShaderError<naga::front::wgsl::ParseError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = self.label.as_deref().unwrap_or_default();
        let string = self.inner.emit_to_string(&self.source);
        write!(f, "\nShader '{}' parsing {}", label, string)
    }
}
impl std::fmt::Display for ShaderError<naga::WithSpan<naga::valid::ValidationError>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use codespan_reporting::{
            diagnostic::{Diagnostic, Label},
            files::SimpleFile,
            term,
        };

        let label = self.label.as_deref().unwrap_or_default();
        let files = SimpleFile::new(label, &self.source);
        let config = term::Config::default();
        let mut writer = term::termcolor::Ansi::new(Vec::new());

        let diagnostic = Diagnostic::error().with_labels(
            self.inner
                .spans()
                .map(|&(span, ref desc)| {
                    Label::primary((), span.to_range().unwrap()).with_message(desc.to_owned())
                })
                .collect(),
        );

        term::emit(&mut writer, &config, &files, &diagnostic).expect("cannot write error");

        write!(
            f,
            "\nShader validation {}",
            String::from_utf8_lossy(&writer.into_inner())
        )
    }
}

impl<E> std::error::Error for ShaderError<E>
where
    ShaderError<E>: std::fmt::Display,
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}
