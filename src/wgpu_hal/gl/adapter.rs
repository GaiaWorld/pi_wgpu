use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Adapter {}

unsafe impl Send for Adapter {}
unsafe impl Sync for Adapter {}


impl api::Adapter<super::Api> for Adapter {
    unsafe fn open(
        &self,
        features: wgt::Features,
        _limits: &wgt::Limits,
    ) -> super::DeviceResult<OpenDevice<super::Api>> {
        Err(DeviceError::Lost)
    }
    unsafe fn texture_format_capabilities(
        &self,
        format: wgt::TextureFormat,
    ) -> TextureFormatCapabilities {
        TextureFormatCapabilities::empty()
    }

    unsafe fn surface_capabilities(&self, surface: &super::Surface) -> Option<SurfaceCapabilities> {
        None
    }

    unsafe fn get_presentation_timestamp(&self) -> wgt::PresentationTimestamp {
        wgt::PresentationTimestamp::INVALID_TIMESTAMP
    }
}
