use thiserror::Error;

use super::super::{
    wgt, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferUsages,
    CommandEncoder, Label, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor,
    SubmissionIndex, Texture, TextureUsages,
};

/// Open connection to a graphics and/or compute device.
///
/// Responsible for the creation of most rendering and compute resources.
/// These are then used in commands, which are submitted to a [`Queue`].
///
/// A device may be requested from an adapter with [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUDevice`](https://gpuweb.github.io/gpuweb/#gpu-device).
#[derive(Debug)]
pub struct Device {
    pub(crate) inner: super::super::hal::Device,
}

impl Device {
    /// List all features that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    #[inline]
    pub fn features(&self) -> Features {
        self.inner.features.clone()
    }

    /// List all limits that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    #[inline]
    pub fn limits(&self) -> Limits {
        self.inner.limits.clone()
    }

    /// Creates a shader module from either SPIR-V or WGSL source code.
    #[inline]
    pub fn create_shader_module(&self, desc: ShaderModuleDescriptor) -> ShaderModule {
        let r = self.inner.create_shader_module(&desc);
        let r = r.unwrap();
        ShaderModule::from_hal(r)
    }

    /// Creates an empty [`CommandEncoder`].
    #[inline]
    pub fn create_command_encoder(
        &self,
        desc: &super::super::CommandEncoderDescriptor,
    ) -> CommandEncoder {
        let r = self.inner.create_command_encoder(&desc);
        let r = r.unwrap();
        CommandEncoder::from_hal(r)
    }

    /// Creates a [`BindGroupLayout`].
    #[inline]
    pub fn create_bind_group_layout(&self, desc: &BindGroupLayoutDescriptor) -> BindGroupLayout {
        let r = self.inner.create_bind_group_layout(&desc);
        let r = r.unwrap();
        BindGroupLayout::from_hal(r)
    }

    /// Creates a new [`BindGroup`].
    #[inline]
    pub fn create_bind_group(&self, desc: &super::super::BindGroupDescriptor) -> BindGroup {
        let r = self.inner.create_bind_group(&desc);
        let r = r.unwrap();
        BindGroup::from_hal(r)
    }

    /// Creates a [`PipelineLayout`].
    #[inline]
    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor) -> PipelineLayout {
        let r = self.inner.create_pipeline_layout(&desc);
        let r = r.unwrap();
        PipelineLayout::from_hal(r)
    }

    /// Creates a [`RenderPipeline`].
    #[inline]
    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor) -> RenderPipeline {
        let r = self.inner.create_render_pipeline(&desc);
        let r = r.unwrap();
        RenderPipeline::from_hal(r)
    }

    /// Creates a [`Buffer`].
    #[inline]
    pub fn create_buffer(&self, desc: &super::super::BufferDescriptor) -> Buffer {
        #[cfg(debug_assertions)]
        {
            // 判断 Buffer 的 用途 只能有一个
            fn is_usage_valid(usage: &BufferUsages) -> bool {
                let has_uniform = if usage.contains(BufferUsages::UNIFORM) {
                    1
                } else {
                    0
                };
                let has_vertex = if usage.contains(BufferUsages::VERTEX) {
                    1
                } else {
                    0
                };
                let has_index = if usage.contains(BufferUsages::INDEX) {
                    1
                } else {
                    0
                };

                let count = has_uniform + has_vertex + has_index;

                count == 1
            }

            debug_assert!(is_usage_valid(&desc.usage));
            debug_assert!(!desc.usage.contains(BufferUsages::STORAGE));
            debug_assert!(!desc.usage.contains(BufferUsages::INDIRECT));
        }

        let r = self.inner.create_buffer(&desc);
        let r = r.unwrap();
        Buffer::from_hal(r, desc.usage, desc.size)
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    #[inline]
    pub fn create_texture(&self, desc: &super::super::TextureDescriptor) -> Texture {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!desc.usage.contains(TextureUsages::STORAGE_BINDING));
        }

        let r = self.inner.create_texture(&desc);
        let r = r.unwrap();
        Texture::from_hal(r, desc)
    }

    // 从窗口表面创建
    #[inline]
    pub(crate) fn create_texture_from_surface(
        &self,
        width: u32,
        height: u32,
        format: crate::TextureFormat,
    ) -> super::Texture {
        let desc = super::super::TextureDescriptor {
            label: None,
            size: crate::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: crate::TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[format],
        };

        let r = self
            .inner
            .create_texture_from_surface(width, height, format);

        Texture::from_hal(r, &desc)
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    #[inline]
    pub fn create_sampler(&self, desc: &SamplerDescriptor) -> Sampler {
        let r = self.inner.create_sampler(&desc);
        let r = r.unwrap();
        Sampler::from_hal(r)
    }
}

/// Describes a [`Device`].
///
/// For use with [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUDeviceDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpudevicedescriptor).
pub type DeviceDescriptor<'a> = wgt::DeviceDescriptor<Label<'a>>;

/// Requesting a device failed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RequestDeviceError;

impl std::fmt::Display for RequestDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Requesting a device failed")
    }
}

impl std::error::Error for RequestDeviceError {}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum DeviceError {
    #[error("parent device is invalid")]
    Invalid,
    #[error("parent device is lost")]
    Lost,
    #[error("not enough memory left")]
    OutOfMemory,
    #[error("unsupported features were requested: {0:?}")]
    UnsupportedFeature(wgt::Features),
}

pub use wgt::Maintain as MaintainBase;
use wgt::{Features, Limits};
/// Passed to [`Device::poll`] to control how and if it should block.
pub type Maintain = wgt::Maintain<SubmissionIndex>;

#[derive(Clone, Debug, Error)]
#[error("Features {0:?} are required but not enabled on the device")]
pub(crate) struct MissingFeatures(pub wgt::Features);

#[derive(Clone, Debug)]
#[cfg_attr(feature = "trace", derive(serde::Serialize))]
#[cfg_attr(feature = "replay", derive(serde::Deserialize))]
pub struct ShaderModuleDescriptorImpl<'a> {
    pub label: Label<'a>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub shader_bound_checks: wgt::ShaderBoundChecks,
}

#[derive(Clone, Debug, Error)]
#[error(
    "Downlevel flags {0:?} are required but not supported on the device.\n{}",
    DOWNLEVEL_ERROR_MESSAGE
)]
pub(crate) struct MissingDownlevelFlags(pub wgt::DownlevelFlags);

const DOWNLEVEL_ERROR_MESSAGE: &str = "This is not an invalid use of WebGPU: the underlying API or device does not \
support enough features to be a fully compliant implementation. A subset of the features can still be used. \
If you are running this program on native and not in a browser and wish to work around this issue, call \
Adapter::downlevel_properties or Device::downlevel_properties to get a listing of the features the current \
platform supports.";
