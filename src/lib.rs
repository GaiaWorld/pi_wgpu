// #![feature(hash_drain_filter)]
#![feature(hash_extract_if)]

use std::future::Future;

#[cfg(feature = "use_wgpu")]
pub use wgpu::*;

#[cfg(not(feature = "use_wgpu"))]
mod pi_wgpu;
#[cfg(not(feature = "use_wgpu"))]
pub use pi_wgpu::*;
#[macro_use]
extern crate lazy_static;

pub trait PiWgpuAdapter {
    fn request_device(
        &self,
        desc: &DeviceDescriptor,
        _trace_path: Option<&std::path::Path>,
    ) -> impl Future<Output = Result<(Device, Queue), RequestDeviceError>> + Send;
}

#[cfg(feature = "use_wgpu")]
impl PiWgpuAdapter for Adapter {
    fn request_device(
        &self,
        desc: &DeviceDescriptor,
        _trace_path: Option<&std::path::Path>,
    ) -> impl Future<Output = Result<(Device, Queue), RequestDeviceError>> + Send {
        self.request_device(desc, _trace_path)
    }
}