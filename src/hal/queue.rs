use super::GLState;
use crate::wgt;

#[derive(Debug)]
pub(crate) struct Queue {
    pub(crate) state: GLState,
}

impl Queue {
    pub(crate) unsafe fn present(
        &mut self,
        surface: &mut super::Surface,
        texture: super::Texture,
    ) -> Result<(), crate::SurfaceError> {
        unimplemented!()
    }
}