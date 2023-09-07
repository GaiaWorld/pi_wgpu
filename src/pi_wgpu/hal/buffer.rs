use glow::HasContext;
use pi_share::Share;

use super::super::BufferUsages;

use super::{AdapterContext, BindTarget, GLState};

#[derive(Debug, Clone)]
pub(crate) struct Buffer(pub(crate) Share<BufferImpl>);

impl Buffer {
    pub fn new(
        state: GLState,
        adapter: &Share<AdapterContext>,
        desc: &super::super::BufferDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        profiling::scope!("hal::Buffer::new");

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

        let raw = {
            let gl = adapter.lock();
            unsafe { gl.create_buffer().unwrap() }
        };

        let imp = BufferImpl {
            state,
            adapter: adapter.clone(),
            raw,
            gl_target,
            gl_usage,
            size,
        };

        let gl = adapter.lock();
        imp.state.set_buffer_size(&gl, &imp, size);

        Ok(Self(Share::new(imp)))
    }

    #[inline]
    pub fn write_buffer(&self, gl: &glow::Context, offset: i32, data: &[u8]) {
        profiling::scope!("hal::Buffer::write_buffer");

        let imp = self.0.as_ref();
        imp.state.set_buffer_sub_data(gl, imp, offset, data);
    }
}

#[derive(Debug)]
pub(crate) struct BufferImpl {
    pub(crate) state: GLState,
    pub(crate) adapter: Share<AdapterContext>,

    pub(crate) raw: glow::Buffer,
    pub(crate) gl_target: BindTarget, // glow::ARRAY_BUFFER, glow::ELEMENT_ARRAY_BUFFER
    pub(crate) gl_usage: u32,         // glow::STATIC_DRAW, glow::STREAM_DRAW

    pub(crate) size: i32,
}

impl Drop for BufferImpl {
    #[inline]
    fn drop(&mut self) {
        let gl = self.adapter.lock();
        unsafe {
            gl.delete_buffer(self.raw);
        }

        self.state.remove_buffer(&gl, self.gl_target, self.raw);
    }
}
