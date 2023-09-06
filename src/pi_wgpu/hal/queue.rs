use super::GLState;

#[derive(Debug)]
pub(crate) struct Queue {
    pub(crate) state: GLState,
}

impl Queue {
    pub(crate) unsafe fn present(
        &mut self,
        surface: &mut super::Surface,
        texture: super::Texture,
    ) -> Result<(), super::super::SurfaceError> {
        todo!()
    }
}