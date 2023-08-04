use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Instance;

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

impl api::Instance<super::Api> for Instance {
    unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        Ok(Instance)
    }

    unsafe fn create_surface(
        &self,
        _display_handle: raw_window_handle::RawDisplayHandle,
        _window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<super::Surface, InstanceError> {
        Ok(super::Surface)
    }
    
    unsafe fn destroy_surface(&self, surface: super::Surface) {}

    unsafe fn enumerate_adapters(&self) -> Vec<ExposedAdapter<super::Api>> {
        Vec::new()
    }
}
