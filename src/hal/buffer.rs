use glow::HasContext;

use crate::BufferUsages;

use super::{BindTarget, GLState};

#[derive(Debug)]
pub(crate) struct Buffer {
    pub(crate) state: GLState,

    pub(crate) raw: glow::Buffer,
    pub(crate) gl_target: BindTarget,
    pub(crate) gl_usage: u32, // glow::STATIC_DRAW, glow::STREAM_DRAW

    pub(crate) size: i32,

    pub(crate) stride: i32,
    pub(crate) divisor_step: u32,
}

impl Drop for Buffer {
    #[inline]
    fn drop(&mut self) {
        let gl = self.state.get_gl();

        unsafe {
            gl.delete_buffer(self.raw);
        }

        self.state.remove_buffer(self.raw);
    }
}

impl Buffer {
    pub fn new(state: GLState, desc: &crate::BufferDescriptor) -> Result<Self, crate::DeviceError> {
        profiling::scope!("hal::Buffer::new");
        let gl = state.get_gl();

        let (gl_target, gl_usage) = if desc.usage.contains(BufferUsages::VERTEX) {
            (glow::ARRAY_BUFFER, glow::STATIC_DRAW)
        } else if desc.usage.contains(BufferUsages::INDEX) {
            (glow::ELEMENT_ARRAY_BUFFER, glow::STATIC_DRAW)
        } else if desc.usage.contains(BufferUsages::UNIFORM) {
            (glow::UNIFORM_BUFFER, glow::DYNAMIC_DRAW)
        } else {
            unreachable!();
        };

        let size = desc.size as i32;

        let raw = unsafe {
            let r = gl.create_buffer().ok();

            gl.bind_buffer(gl_target, r);

            gl.buffer_data_size(gl_target, size, gl_usage);

            r.unwrap()
        };

        Ok(Self {
            state,
            raw,
            gl_target,
            gl_usage,
            size,

            stride: 0,
            divisor_step: 1,
        })
    }

    pub fn write_buffer(&self, offset: i32, data: &[u8]) {
        profiling::scope!("hal::Buffer::write_buffer");
        let gl = self.state.get_gl();

        unsafe {
            gl.bind_buffer(self.gl_target, Some(self.raw));

            gl.buffer_sub_data_u8_slice(self.gl_target, offset, data);
        }
    }
}
