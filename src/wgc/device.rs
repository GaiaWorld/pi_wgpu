use std::{future::Future, sync::Arc};

use thiserror::Error;

use crate::{
    hal, wgt, BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, CommandEncoder,
    ComputePipeline, ComputePipelineDescriptor, Error, ErrorFilter, Label, PipelineLayout,
    PipelineLayoutDescriptor, QuerySet, Queue, RenderBundleEncoder, RenderBundleEncoderDescriptor,
    RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModule,
    ShaderModuleDescriptor, ShaderModuleDescriptorSpirV, SubmissionIndex, Texture,
    UncapturedErrorHandler,
};

/// Open connection to a graphics and/or compute device.
///
/// Responsible for the creation of most rendering and compute resources.
/// These are then used in commands, which are submitted to a [`Queue`].
///
/// A device may be requested from an adapter with [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUDevice`](https://gpuweb.github.io/gpuweb/#gpu-device).
#[derive(Debug, Clone)]
pub struct Device(pub(crate) Arc<DeviceInner>);

#[derive(Debug)]
pub(crate) struct DeviceInner {
    pub(crate) raw: hal::Adapter,

    pub(crate) queue: Queue,

    pub(crate) limits: wgt::Limits,
    pub(crate) features: wgt::Features,
    pub(crate) downlevel: wgt::DownlevelCapabilities,
}

impl Drop for Device {
    fn drop(&mut self) {}
}

impl Device {
    /// List all features that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    pub fn features(&self) -> Features {
        self.0.features.clone()
    }

    /// List all limits that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> Limits {
        self.0.limits.clone()
    }

    /// Creates a shader module from either SPIR-V or WGSL source code.
    pub fn create_shader_module(&self, desc: ShaderModuleDescriptor) -> ShaderModule {
        unimplemented!("wgc::Device::create_shader_module");
    }

    /// Creates an empty [`CommandEncoder`].
    pub fn create_command_encoder(&self, desc: &crate::CommandEncoderDescriptor) -> CommandEncoder {
        unimplemented!("wgc::Device::create_command_encoder");
    }

    /// Creates a [`BindGroupLayout`].
    pub fn create_bind_group_layout(&self, desc: &BindGroupLayoutDescriptor) -> BindGroupLayout {
        unimplemented!("wgc::Device::create_bind_group_layout");
    }

    /// Creates a new [`BindGroup`].
    pub fn create_bind_group(&self, desc: &crate::BindGroupDescriptor) -> BindGroup {
        unimplemented!("wgc::Device::create_bind_group");
    }

    /// Creates a [`PipelineLayout`].
    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor) -> PipelineLayout {
        unimplemented!("wgc::Device::create_pipeline_layout");
    }

    /// Creates a [`RenderPipeline`].
    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor) -> RenderPipeline {
        unimplemented!("wgc::Device::create_render_pipeline");
    }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &crate::BufferDescriptor) -> Buffer {
        unimplemented!("wgc::Device::create_buffer");
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    pub fn create_texture(&self, desc: &crate::TextureDescriptor) -> Texture {
        unimplemented!("wgc::Device::create_texture");
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    pub fn create_sampler(&self, desc: &SamplerDescriptor) -> Sampler {
        unimplemented!("wgc::Device::create_sampler");
    }

    /// Check for resource cleanups and mapping callbacks.
    ///
    /// Return `true` if the queue is empty, or `false` if there are more queue
    /// submissions still in flight. (Note that, unless access to the [`Queue`] is
    /// coordinated somehow, this information could be out of date by the time
    /// the caller receives it. `Queue`s can be shared between threads, so
    /// other threads could submit new work at any time.)
    ///
    /// On the web, this is a no-op. `Device`s are automatically polled.
    pub fn poll(&self, _maintain: Maintain) -> bool {
        unimplemented!("wgc::Device::poll is not implemented")
    }

    /// Creates a shader module from either SPIR-V or WGSL source code without runtime checks.
    ///
    /// # Safety
    /// In contrast with [`create_shader_module`](Self::create_shader_module) this function
    /// creates a shader module without runtime checks which allows shaders to perform
    /// operations which can lead to undefined behavior like indexing out of bounds, thus it's
    /// the caller responsibility to pass a shader which doesn't perform any of this
    /// operations.
    ///
    /// This has no effect on web.
    pub unsafe fn create_shader_module_unchecked(
        &self,
        _desc: ShaderModuleDescriptor,
    ) -> ShaderModule {
        unimplemented!("wgc::Device::create_shader_module_unchecked is not implemented")
    }

    /// Creates a shader module from SPIR-V binary directly.
    ///
    /// # Safety
    ///
    /// This function passes binary data to the backend as-is and can potentially result in a
    /// driver crash or bogus behaviour. No attempt is made to ensure that data is valid SPIR-V.
    ///
    /// See also [`include_spirv_raw!`] and [`util::make_spirv_raw`].
    pub unsafe fn create_shader_module_spirv(
        &self,
        _desc: &ShaderModuleDescriptorSpirV,
    ) -> ShaderModule {
        unimplemented!("wgc::Device::create_shader_module_spirv is not implemented")
    }

    /// Creates an empty [`RenderBundleEncoder`].
    pub fn create_render_bundle_encoder(
        &self,
        _desc: &RenderBundleEncoderDescriptor,
    ) -> RenderBundleEncoder {
        unimplemented!("wgc::Device::create_render_bundle_encoder is not implemented")
    }

    /// Creates a [`ComputePipeline`].
    pub fn create_compute_pipeline(&self, _desc: &ComputePipelineDescriptor) -> ComputePipeline {
        unimplemented!("wgc::Device::create_compute_pipeline is not implemented")
    }

    /// Creates a [`Texture`] from a wgpu-hal Texture.
    ///
    /// # Safety
    ///
    /// ++
    ///
    /// - `hal_texture` must be created from this device internal handle
    /// - `hal_texture` must be created respecting `desc`
    /// - `hal_texture` must be initialized
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten", feature = "webgl"))]
    pub unsafe fn create_texture_from_hal<A: hal::HalApi>(
        &self,
        _hal_texture: A::Texture,
        _desc: &crate::TextureDescriptor,
    ) -> Texture {
        unimplemented!("wgc::Device::create_texture_from_hal is not implemented")
    }

    /// Creates a new [`QuerySet`].
    pub fn create_query_set(&self, _desc: &crate::QuerySetDescriptor) -> QuerySet {
        unimplemented!("wgc::Device::create_query_set is not implemented")
    }

    /// Set a callback for errors that are not handled in error scopes.
    pub fn on_uncaptured_error(&self, _handler: Box<dyn UncapturedErrorHandler>) {
        unimplemented!("wgc::Device::on_uncaptured_error is not implemented")
    }

    /// Push an error scope.
    pub fn push_error_scope(&self, _filter: ErrorFilter) {
        unimplemented!("wgc::Device::push_error_scope is not implemented")
    }

    /// Pop an error scope.
    pub fn pop_error_scope(&self) -> impl Future<Output = Option<Error>> + Send {
        unimplemented!("wgc::Device::pop_error_scope is not implemented");

        use futures::future::FutureExt;
        #[allow(unreachable_code)]
        async { None }.boxed()
    }

    /// Starts frame capture.
    pub fn start_capture(&self) {
        unimplemented!("wgc::Device::start_capture is not implemented")
    }

    /// Stops frame capture.
    pub fn stop_capture(&self) {
        unimplemented!("wgc::Device::stop_capture is not implemented")
    }

    /// Apply a callback to this `Device`'s underlying backend device.
    ///
    /// If this `Device` is implemented by the backend API given by `A` (Vulkan,
    /// Dx12, etc.), then apply `hal_device_callback` to `Some(&device)`, where
    /// `device` is the underlying backend device type, [`A::Device`].
    ///
    /// If this `Device` uses a different backend, apply `hal_device_callback`
    /// to `None`.
    ///
    /// The device is locked for reading while `hal_device_callback` runs. If
    /// the callback attempts to perform any `wgpu` operations that require
    /// write access to the device (destroying a buffer, say), deadlock will
    /// occur. The locks are automatically released when the callback returns.
    ///
    /// # Safety
    ///
    /// - The raw handle passed to the callback must not be manually destroyed.
    ///
    /// [`A::Device`]: crate::wgpu_hal::Api::Device
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub unsafe fn as_hal<A: hal::HalApi, F: FnOnce(Option<&A::Device>) -> R, R>(
        &self,
        _hal_device_callback: F,
    ) -> R {
        unimplemented!("wgc::Device::as_hal is not implemented")
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

#[derive(Clone, Debug, Error)]
pub enum DeviceError {
    #[error("parent device is invalid")]
    Invalid,
    #[error("parent device is lost")]
    Lost,
    #[error("not enough memory left")]
    OutOfMemory,
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
