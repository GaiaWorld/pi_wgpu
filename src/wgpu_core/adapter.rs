use super::api::HalApi;
use crate::{
    wgpu_hal as hal, AdapterInfo, Device, DownlevelCapabilities, Features, Limits,
    PresentationTimestamp, Queue, Surface, TextureFormat, TextureFormatFeatures,
};
use std::future::Future;

/// Handle to a physical graphics and/or compute device.
///
/// Adapters can be used to open a connection to the corresponding [`Device`]
/// on the host system by using [`Adapter::request_device`].
///
/// Does not have to be kept alive.
///
/// Corresponds to [WebGPU `GPUAdapter`](https://gpuweb.github.io/gpuweb/#gpu-adapter).
#[derive(Debug)]
pub struct Adapter {
    inner: <hal::GL as hal::Api>::Adapter,
}

static_assertions::assert_impl_all!(Adapter: Send, Sync);

impl Drop for Adapter {
    fn drop(&mut self) {
        unimplemented!("Adapter::drop is not implemented")
    }
}

impl Adapter {
    /// Requests a connection to a physical device, creating a logical device.
    ///
    /// Returns the [`Device`] together with a [`Queue`] that executes command buffers.
    ///
    /// # Arguments
    ///
    /// - `desc` - Description of the features and limits requested from the given device.
    /// - `trace_path` - Can be used for API call tracing, if that feature is
    ///   enabled in `wgpu-core`.
    ///
    /// # Panics
    ///
    /// - Features specified by `desc` are not supported by this adapter.
    /// - Unsafe features were requested but not enabled when requesting the adapter.
    /// - Limits requested exceed the values provided by the adapter.
    /// - Adapter does not support all features wgpu requires to safely operate.
    pub fn request_device(
        &self,
        desc: &crate::DeviceDescriptor,
        trace_path: Option<&std::path::Path>,
    ) -> impl Future<Output = Result<(Device, Queue), crate::RequestDeviceError>> + Send {
        unimplemented!("Adapter::request_device is not implemented");

        use futures::future::FutureExt;
        async { Err(crate::RequestDeviceError) }.boxed()
    }

    /// Create a wgpu [`Device`] and [`Queue`] from a wgpu-hal `OpenDevice`
    ///
    /// # Safety
    ///
    /// - `hal_device` must be created from this adapter internal handle.
    /// - `desc.features` must be a subset of `hal_device` features.
    #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
    pub unsafe fn create_device_from_hal<A: HalApi>(
        &self,
        hal_device: crate::wgpu_hal::OpenDevice<A>,
        desc: &crate::DeviceDescriptor,
        trace_path: Option<&std::path::Path>,
    ) -> Result<(Device, Queue), crate::RequestDeviceError> {
        unimplemented!("Adapter::create_device_from_hal is not implemented")
    }

    /// Apply a callback to this `Adapter`'s underlying backend adapter.
    ///
    /// If this `Adapter` is implemented by the backend API given by `A` (Vulkan,
    /// Dx12, etc.), then apply `hal_adapter_callback` to `Some(&adapter)`, where
    /// `adapter` is the underlying backend adapter type, [`A::Adapter`].
    ///
    /// If this `Adapter` uses a different backend, apply `hal_adapter_callback`
    /// to `None`.
    ///
    /// The adapter is locked for reading while `hal_adapter_callback` runs. If
    /// the callback attempts to perform any `wgpu` operations that require
    /// write access to the adapter, deadlock will occur. The locks are
    /// automatically released when the callback returns.
    ///
    /// # Safety
    ///
    /// - The raw handle passed to the callback must not be manually destroyed.
    ///
    /// [`A::Adapter`]: crate::wgpu_hal::Api::Adapter
    #[cfg(any(not(target_arch = "wasm32"), feature = "webgl"))]
    pub unsafe fn as_hal<A: HalApi, F: FnOnce(Option<&A::Adapter>) -> R, R>(
        &self,
        hal_adapter_callback: F,
    ) -> R {
        unimplemented!("Adapter::as_hal is not implemented")
    }

    /// Returns whether this adapter may present to the passed surface.
    pub fn is_surface_supported(&self, surface: &Surface) -> bool {
        unimplemented!("Adapter::is_surface_supported is not implemented")
    }

    /// List all features that are supported with this adapter.
    ///
    /// Features must be explicitly requested in [`Adapter::request_device`] in order
    /// to use them.
    pub fn features(&self) -> Features {
        unimplemented!("Adapter::features is not implemented")
    }

    /// List the "best" limits that are supported by this adapter.
    ///
    /// Limits must be explicitly requested in [`Adapter::request_device`] to set
    /// the values that you are allowed to use.
    pub fn limits(&self) -> Limits {
        unimplemented!("Adapter::limits is not implemented")
    }

    /// Get info about the adapter itself.
    pub fn get_info(&self) -> AdapterInfo {
        unimplemented!("Adapter::get_info is not implemented")
    }

    /// Get info about the adapter itself.
    pub fn get_downlevel_capabilities(&self) -> DownlevelCapabilities {
        unimplemented!("Adapter::get_downlevel_capabilities is not implemented")
    }

    /// Returns the features supported for a given texture format by this adapter.
    ///
    /// Note that the WebGPU spec further restricts the available usages/features.
    /// To disable these restrictions on a device, request the [`Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`] feature.
    pub fn get_texture_format_features(&self, format: TextureFormat) -> TextureFormatFeatures {
        unimplemented!("Adapter::get_texture_format_features is not implemented")
    }

    /// Generates a timestamp using the clock used by the presentation engine.
    ///
    /// When comparing completely opaque timestamp systems, we need a way of generating timestamps that signal
    /// the exact same time. You can do this by calling your own timestamp function immediately after a call to
    /// this function. This should result in timestamps that are 0.5 to 5 microseconds apart. There are locks
    /// that must be taken during the call, so don't call your function before.
    ///
    /// ```no_run
    /// # let adapter: wgpu::Adapter = panic!();
    /// # let some_code = || wgpu::PresentationTimestamp::INVALID_TIMESTAMP;
    /// use std::time::{Duration, Instant};
    /// let presentation = adapter.get_presentation_timestamp();
    /// let instant = Instant::now();
    ///
    /// // We can now turn a new presentation timestamp into an Instant.
    /// let some_pres_timestamp = some_code();
    /// let duration = Duration::from_nanos((some_pres_timestamp.0 - presentation.0) as u64);
    /// let new_instant: Instant = instant + duration;
    /// ```
    //
    /// [Instant]: std::time::Instant
    pub fn get_presentation_timestamp(&self) -> PresentationTimestamp {
        unimplemented!("Adapter::get_presentation_timestamp is not implemented")
    }
}

pub use wgt::RequestAdapterOptions as RequestAdapterOptionsBase;

/// Additional information required when requesting an adapter.
///
/// For use with [`Instance::request_adapter`].
///
/// Corresponds to [WebGPU `GPURequestAdapterOptions`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions).
pub type RequestAdapterOptions<'a> = RequestAdapterOptionsBase<&'a Surface>;
static_assertions::assert_impl_all!(RequestAdapterOptions: Send, Sync);
