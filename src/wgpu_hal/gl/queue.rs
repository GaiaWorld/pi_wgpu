use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Queue;

unsafe impl Send for Queue {}
unsafe impl Sync for Queue {}

impl api::Queue<super::Api> for Queue {
    unsafe fn submit(
        &mut self,
        command_buffers: &[&super::CommandBuffer],
        signal_fence: Option<(&mut super::Fence, FenceValue)>,
    ) -> super::DeviceResult<()> {
        Ok(())
    }
    unsafe fn present(
        &mut self,
        surface: &mut super::Surface,
        texture: super::Texture,
    ) -> Result<(), SurfaceError> {
        Ok(())
    }

    unsafe fn get_timestamp_period(&self) -> f32 {
        1.0
    }
}
