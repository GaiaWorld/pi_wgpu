use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Adapter {}

unsafe impl Send for Adapter {}
unsafe impl Sync for Adapter {}


impl api::Adapter<super::Api> for Adapter {
    unsafe fn open(
        &self,
        features: crate::wgpu_types::Features,
        _limits: &crate::wgpu_types::Limits,
    ) -> super::DeviceResult<OpenDevice<super::Api>> {
        Err(DeviceError::Lost)
    }
    unsafe fn texture_format_capabilities(
        &self,
        format: crate::wgpu_types::TextureFormat,
    ) -> TextureFormatCapabilities {
        TextureFormatCapabilities::empty()
    }

    unsafe fn surface_capabilities(&self, surface: &super::Surface) -> Option<SurfaceCapabilities> {
        None
    }

    unsafe fn get_presentation_timestamp(&self) -> crate::wgpu_types::PresentationTimestamp {
        crate::wgpu_types::PresentationTimestamp::INVALID_TIMESTAMP
    }
}
