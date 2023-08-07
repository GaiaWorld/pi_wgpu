use std::future::Future;
use super::api::HalApi;
use crate::{
    BindGroup, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, CommandEncoder, ComputePipeline,
    ComputePipelineDescriptor, Error, ErrorFilter, Label, PipelineLayout, PipelineLayoutDescriptor,
    QuerySet, RenderBundleEncoder, RenderBundleEncoderDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModule, ShaderModuleDescriptor,
    ShaderModuleDescriptorSpirV, SubmissionIndex, Texture, UncapturedErrorHandler,
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
pub struct Device {}

static_assertions::assert_impl_all!(Device: Send, Sync);

impl Drop for Device {
    fn drop(&mut self) {
        unimplemented!("Device::drop is not implemented")
    }
}

impl Device {
    /// Check for resource cleanups and mapping callbacks.
    ///
    /// Return `true` if the queue is empty, or `false` if there are more queue
    /// submissions still in flight. (Note that, unless access to the [`Queue`] is
    /// coordinated somehow, this information could be out of date by the time
    /// the caller receives it. `Queue`s can be shared between threads, so
    /// other threads could submit new work at any time.)
    ///
    /// On the web, this is a no-op. `Device`s are automatically polled.
    pub fn poll(&self, maintain: Maintain) -> bool {
        unimplemented!("Device::poll is not implemented")
    }

    /// List all features that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    pub fn features(&self) -> Features {
        unimplemented!("Device::features is not implemented")
    }

    /// List all limits that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    pub fn limits(&self) -> Limits {
        unimplemented!("Device::limits is not implemented")
    }

    /// Creates a shader module from either SPIR-V or WGSL source code.
    pub fn create_shader_module(&self, desc: ShaderModuleDescriptor) -> ShaderModule {
        unimplemented!("Device::create_shader_module is not implemented")
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
        desc: ShaderModuleDescriptor,
    ) -> ShaderModule {
        unimplemented!("Device::create_shader_module_unchecked is not implemented")
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
        desc: &ShaderModuleDescriptorSpirV,
    ) -> ShaderModule {
        unimplemented!("Device::create_shader_module_spirv is not implemented")
    }

    /// Creates an empty [`CommandEncoder`].
    pub fn create_command_encoder(&self, desc: &crate::CommandEncoderDescriptor) -> CommandEncoder {
        unimplemented!("Device::create_command_encoder is not implemented")
    }

    /// Creates an empty [`RenderBundleEncoder`].
    pub fn create_render_bundle_encoder(
        &self,
        desc: &RenderBundleEncoderDescriptor,
    ) -> RenderBundleEncoder {
        unimplemented!("Device::create_render_bundle_encoder is not implemented")
    }

    /// Creates a new [`BindGroup`].
    pub fn create_bind_group(&self, desc: &crate::BindGroupDescriptor) -> BindGroup {
        unimplemented!("Device::create_bind_group is not implemented")
    }

    /// Creates a [`BindGroupLayout`].
    pub fn create_bind_group_layout(&self, desc: &BindGroupLayoutDescriptor) -> BindGroupLayout {
        unimplemented!("Device::create_bind_group_layout is not implemented")
    }

    /// Creates a [`PipelineLayout`].
    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor) -> PipelineLayout {
        unimplemented!("Device::create_pipeline_layout is not implemented")
    }

    /// Creates a [`RenderPipeline`].
    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor) -> RenderPipeline {
        unimplemented!("Device::create_render_pipeline is not implemented")
    }

    /// Creates a [`ComputePipeline`].
    pub fn create_compute_pipeline(&self, desc: &ComputePipelineDescriptor) -> ComputePipeline {
        unimplemented!("Device::create_compute_pipeline is not implemented")
    }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &crate::BufferDescriptor) -> Buffer {
        unimplemented!("Device::create_buffer is not implemented")
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    pub fn create_texture(&self, desc: &crate::TextureDescriptor) -> Texture {
        unimplemented!("Device::create_texture is not implemented")
    }

    /// Creates a [`Texture`] from a wgpu-hal Texture.
    ///
    /// # Safety
    ///
    /// - `hal_texture` must be created from this device internal handle
    /// - `hal_texture` must be created respecting `desc`
    /// - `hal_texture` must be initialized
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten", feature = "webgl"))]
    pub unsafe fn create_texture_from_hal<A: HalApi>(
        &self,
        hal_texture: A::Texture,
        desc: &crate::TextureDescriptor,
    ) -> Texture {
        unimplemented!("Device::create_texture_from_hal is not implemented")
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    pub fn create_sampler(&self, desc: &SamplerDescriptor) -> Sampler {
        unimplemented!("Device::create_sampler is not implemented")
    }

    /// Creates a new [`QuerySet`].
    pub fn create_query_set(&self, desc: &crate::QuerySetDescriptor) -> QuerySet {
        unimplemented!("Device::create_query_set is not implemented")
    }

    /// Set a callback for errors that are not handled in error scopes.
    pub fn on_uncaptured_error(&self, handler: Box<dyn UncapturedErrorHandler>) {
        unimplemented!("Device::on_uncaptured_error is not implemented")
    }

    /// Push an error scope.
    pub fn push_error_scope(&self, filter: ErrorFilter) {
        unimplemented!("Device::push_error_scope is not implemented")
    }

    /// Pop an error scope.
    pub fn pop_error_scope(&self) -> impl Future<Output = Option<Error>> + Send {
        
        unimplemented!("Device::pop_error_scope is not implemented");
        
        use futures::future::FutureExt;
        async { None }.boxed()
    }

    /// Starts frame capture.
    pub fn start_capture(&self) {
        unimplemented!("Device::start_capture is not implemented")
    }

    /// Stops frame capture.
    pub fn stop_capture(&self) {
        unimplemented!("Device::stop_capture is not implemented")
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
    pub unsafe fn as_hal<A: HalApi, F: FnOnce(Option<&A::Device>) -> R, R>(
        &self,
        hal_device_callback: F,
    ) -> R {
        unimplemented!("Device::as_hal is not implemented")
    }
}

/// Describes a [`Device`].
///
/// For use with [`Adapter::request_device`].
///
/// Corresponds to [WebGPU `GPUDeviceDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpudevicedescriptor).
pub type DeviceDescriptor<'a> = wgt::DeviceDescriptor<Label<'a>>;

static_assertions::assert_impl_all!(DeviceDescriptor: Send, Sync);

/// Requesting a device failed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RequestDeviceError;
static_assertions::assert_impl_all!(RequestDeviceError: Send, Sync);

impl std::fmt::Display for RequestDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Requesting a device failed")
    }
}

impl std::error::Error for RequestDeviceError {}

use wgt::{Limits, Features};
pub use wgt::Maintain as MaintainBase;
/// Passed to [`Device::poll`] to control how and if it should block.
pub type Maintain = wgt::Maintain<SubmissionIndex>;
