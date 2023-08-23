use std::sync::Arc;

use arrayvec::ArrayVec;
use glow::HasContext;

use crate::wgt;
use super::{AdapterShared, MAX_COLOR_ATTACHMENTS};

#[derive(Debug)]
pub(crate) struct Queue {
    shared: Arc<AdapterShared>,
    features: wgt::Features,
    draw_fbo: glow::Framebuffer,
    copy_fbo: glow::Framebuffer,
    /// Shader program used to clear the screen for [`Workarounds::MESA_I915_SRGB_SHADER_CLEAR`]
    /// devices.
    shader_clear_program: glow::Program,
    /// The uniform location of the color uniform in the shader clear program
    shader_clear_program_color_uniform_location: glow::UniformLocation,
    /// Keep a reasonably large buffer filled with zeroes, so that we can implement `ClearBuffer` of
    /// zeroes by copying from it.
    zero_buffer: glow::Buffer,
    temp_query_results: Vec<u64>,
    draw_buffer_count: u8,
    current_index_buffer: Option<glow::Buffer>,
}

impl Queue {
    pub(crate) unsafe fn submit(
        &mut self,
        command_buffers: &[&super::CommandBuffer],
        signal_fence: Option<(&mut super::Fence, super::FenceValue)>,
    ) -> Result<(), crate::DeviceError> {
        let shared = Arc::clone(&self.shared);
        let gl = &shared.context.lock();
        unsafe { self.reset_state(gl) };
        for cmd_buf in command_buffers.iter() {
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(ref label) = cmd_buf.label {
                unsafe { gl.push_debug_group(glow::DEBUG_SOURCE_APPLICATION, DEBUG_ID, label) };
            }

            for command in cmd_buf.commands.iter() {
                unsafe { self.process(gl, command, &cmd_buf.data_bytes, &cmd_buf.queries) };
            }

            #[cfg(not(target_arch = "wasm32"))]
            if cmd_buf.label.is_some() {
                unsafe { gl.pop_debug_group() };
            }
        }

        if let Some((fence, value)) = signal_fence {
            fence.maintain(gl);
            let sync = unsafe { gl.fence_sync(glow::SYNC_GPU_COMMANDS_COMPLETE, 0) }
                .map_err(|_| crate::DeviceError::OutOfMemory)?;
            fence.pending.push((value, sync));
        }

        Ok(())
    }

    pub(crate) unsafe fn present(
        &mut self,
        surface: &mut super::Surface,
        texture: super::Texture,
    ) -> Result<(), crate::SurfaceError> {
        #[cfg(any(not(target_arch = "wasm32"), feature = "emscripten"))]
        let gl = unsafe { &self.shared.context.get_without_egl_lock() };

        #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
        let gl = &self.shared.context.glow_context;

        unsafe { surface.present(texture, gl) }
    }

    pub(crate) unsafe fn get_timestamp_period(&self) -> f32 {
        1.0
    }
}

impl super::Queue {
    /// Performs a manual shader clear, used as a workaround for a clearing bug on mesa
    unsafe fn perform_shader_clear(&self, gl: &glow::Context, draw_buffer: u32, color: [f32; 4]) {
        unsafe { gl.use_program(Some(self.shader_clear_program)) };
        unsafe {
            gl.uniform_4_f32(
                Some(&self.shader_clear_program_color_uniform_location),
                color[0],
                color[1],
                color[2],
                color[3],
            )
        };
        unsafe { gl.disable(glow::DEPTH_TEST) };
        unsafe { gl.disable(glow::STENCIL_TEST) };
        unsafe { gl.disable(glow::SCISSOR_TEST) };
        unsafe { gl.disable(glow::BLEND) };
        unsafe { gl.disable(glow::CULL_FACE) };
        unsafe { gl.draw_buffers(&[glow::COLOR_ATTACHMENT0 + draw_buffer]) };
        unsafe { gl.draw_arrays(glow::TRIANGLES, 0, 3) };

        if self.draw_buffer_count != 0 {
            // Reset the draw buffers to what they were before the clear
            let indices = (0..self.draw_buffer_count as u32)
                .map(|i| glow::COLOR_ATTACHMENT0 + i)
                .collect::<ArrayVec<_, { MAX_COLOR_ATTACHMENTS }>>();
            unsafe { gl.draw_buffers(&indices) };
        }
        #[cfg(not(target_arch = "wasm32"))]
        for draw_buffer in 0..self.draw_buffer_count as u32 {
            unsafe { gl.disable_draw_buffer(glow::BLEND, draw_buffer) };
        }
    }

    unsafe fn reset_state(&mut self, gl: &glow::Context) {
        unsafe { gl.use_program(None) };
        unsafe { gl.bind_framebuffer(glow::FRAMEBUFFER, None) };
        unsafe { gl.disable(glow::DEPTH_TEST) };
        unsafe { gl.disable(glow::STENCIL_TEST) };
        unsafe { gl.disable(glow::SCISSOR_TEST) };
        unsafe { gl.disable(glow::BLEND) };
        unsafe { gl.disable(glow::CULL_FACE) };
        unsafe { gl.disable(glow::POLYGON_OFFSET_FILL) };
        if self.features.contains(wgt::Features::DEPTH_CLIP_CONTROL) {
            unsafe { gl.disable(glow::DEPTH_CLAMP) };
        }

        unsafe { gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None) };
        self.current_index_buffer = None;
    }

    unsafe fn set_attachment(
        &self,
        gl: &glow::Context,
        fbo_target: u32,
        attachment: u32,
        view: &super::TextureView,
    ) {
        match view.inner {
            super::TextureInner::Renderbuffer { raw } => {
                unsafe {
                    gl.framebuffer_renderbuffer(
                        fbo_target,
                        attachment,
                        glow::RENDERBUFFER,
                        Some(raw),
                    )
                };
            }
            super::TextureInner::DefaultRenderbuffer => panic!("Unexpected default RBO"),
            super::TextureInner::Texture { raw, target } => {
                let num_layers = view.array_layers.end - view.array_layers.start;
                if num_layers > 1 {
                    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
                    unsafe {
                        gl.framebuffer_texture_multiview_ovr(
                            fbo_target,
                            attachment,
                            Some(raw),
                            view.mip_levels.start as i32,
                            view.array_layers.start as i32,
                            num_layers as i32,
                        )
                    };
                } else if is_layered_target(target) {
                    unsafe {
                        gl.framebuffer_texture_layer(
                            fbo_target,
                            attachment,
                            Some(raw),
                            view.mip_levels.start as i32,
                            view.array_layers.start as i32,
                        )
                    };
                } else if target == glow::TEXTURE_CUBE_MAP {
                    unsafe {
                        gl.framebuffer_texture_2d(
                            fbo_target,
                            attachment,
                            CUBEMAP_FACES[view.array_layers.start as usize],
                            Some(raw),
                            view.mip_levels.start as i32,
                        )
                    };
                } else {
                    unsafe {
                        gl.framebuffer_texture_2d(
                            fbo_target,
                            attachment,
                            target,
                            Some(raw),
                            view.mip_levels.start as i32,
                        )
                    };
                }
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            super::TextureInner::ExternalFramebuffer { ref inner } => unsafe {
                gl.bind_external_framebuffer(glow::FRAMEBUFFER, inner);
            },
        }
    }
}

pub(crate) fn is_layered_target(target: super::BindTarget) -> bool {
    match target {
        glow::TEXTURE_2D_ARRAY | glow::TEXTURE_3D | glow::TEXTURE_CUBE_MAP_ARRAY => true,
        _ => false,
    }
}