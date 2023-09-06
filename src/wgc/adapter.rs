use std::future::Future;

use futures::future::FutureExt;
use thiserror::Error;

use crate::{
    hal, wgt::RequestAdapterOptions as RequestAdapterOptionsBase, AdapterInfo, Device,
    DeviceDescriptor, DeviceError, DownlevelCapabilities, Features, Limits, Queue,
    RequestDeviceError, Surface, TextureFormat, TextureFormatFeatures,
};

/// Additional information required when requesting an adapter.
///
/// For use with [`Instance::request_adapter`].
///
/// Corresponds to [WebGPU `GPURequestAdapterOptions`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions).
pub type RequestAdapterOptions<'a> = RequestAdapterOptionsBase<&'a Surface>;

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
    pub(crate) inner: hal::Adapter,
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
        desc: &DeviceDescriptor,
        _trace_path: Option<&std::path::Path>,
    ) -> impl Future<Output = Result<(Device, Queue), RequestDeviceError>> + Send {
        let open =
            unsafe { self.inner.open(desc.features, &desc.limits) }.map_err(|err| match err {
                DeviceError::Lost => RequestDeviceError,
                DeviceError::OutOfMemory => RequestDeviceError,
                _ => RequestDeviceError,
            });

        let r = match open {
            Ok(open) => {
                let device = Device { inner: open.device };

                let queue = Queue { inner: open.queue };

                Ok((device, queue))
            }
            Err(e) => Err(e),
        };

        async { r }.boxed()
    }

    /// Returns whether this adapter may present to the passed surface.
    pub fn is_surface_supported(&self, surface: &Surface) -> bool {
        todo!()
    }

    /// List all features that are supported with this adapter.
    ///
    /// Features must be explicitly requested in [`Adapter::request_device`] in order
    /// to use them.
    pub fn features(&self) -> Features {
        todo!()
    }

    /// List the "best" limits that are supported by this adapter.
    ///
    /// Limits must be explicitly requested in [`Adapter::request_device`] to set
    /// the values that you are allowed to use.
    pub fn limits(&self) -> Limits {
        todo!()
    }

    /// Get info about the adapter itself.
    pub fn get_info(&self) -> AdapterInfo {
        todo!()
    }

    /// Get info about the adapter itself.
    pub fn get_downlevel_capabilities(&self) -> DownlevelCapabilities {
        todo!()
    }

    /// Returns the features supported for a given texture format by this adapter.
    ///
    /// Note that the WebGPU spec further restricts the available usages/features.
    /// To disable these restrictions on a device, request the [`Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`] feature.
    pub fn get_texture_format_features(&self, format: TextureFormat) -> TextureFormatFeatures {
        todo!()
    }
}

#[derive(Debug, Error)]
#[error("adapter is invalid")]
pub struct InvalidAdapter;

#[derive(Debug, Error)]
pub enum RequestAdapterError {
    #[error("no suitable adapter found")]
    NotFound,
    #[error("surface {0:?} is invalid")]
    InvalidSurface(Surface),
}
