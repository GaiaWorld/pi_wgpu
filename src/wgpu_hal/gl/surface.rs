use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Surface;

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl api::Surface<super::Api> for Surface {
    unsafe fn configure(
        &mut self,
        device: &super::Device,
        config: &SurfaceConfiguration,
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    unsafe fn unconfigure(&mut self, device: &super::Device) {}

    unsafe fn acquire_texture(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<Option<AcquiredSurfaceTexture<super::Api>>, SurfaceError> {
        Ok(None)
    }

    unsafe fn discard_texture(&mut self, texture: super::Texture) {}
}
