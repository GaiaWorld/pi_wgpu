//!
//! + 仅 单线程
//! + 录制 即为 调用 GL 指令
//!
//! `CommandEncoder` 目前 仅支持 如下接口：
//!
//! + begin_render_pass / end_render_pass
//! + set_render_pipeline
//! + set_bind_group
//! + set_vertex_buffer
//! + set_index_buffer
//! + set_viewport
//! + set_scissor_rect
//! + set_stencil_reference
//! + set_blend_constants
//! + draw / draw_indexed
//!

use pi_share::Share;

use super::super::wgt;
use super::{AdapterContext, AdapterContextLock, GLState};

#[derive(Debug, Clone)]
pub(crate) struct CommandEncoder {
    state: GLState,
    adapter: Share<AdapterContext>,
}

impl CommandEncoder {
    pub fn new(
        state: GLState,
        adapter: &Share<AdapterContext>,
        desc: &super::super::CommandEncoderDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        Ok(Self {
            state,
            adapter: adapter.clone(),
        })
    }
}

impl CommandEncoder {
    #[inline]
    pub(crate) unsafe fn begin_render_pass<'a>(
        &'a self,
        desc: &super::super::RenderPassDescriptor,
    ) -> AdapterContextLock<'a> {
        let gl = self.adapter.lock();
        self.state.set_render_target(&gl, desc);
        gl
    }

    #[inline]
    pub(crate) unsafe fn end_render_pass(&self) {}

    #[inline]
    pub(crate) unsafe fn set_bind_group(
        &self,
        gl: &glow::Context,
        index: u32,
        group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        self.state.set_bind_group(gl, index, group, dynamic_offsets);
    }

    #[inline]
    pub(crate) unsafe fn set_render_pipeline(
        &self,
        gl: &glow::Context,
        pipeline: &super::RenderPipeline,
    ) {
        self.state.set_render_pipeline(gl, pipeline);
    }

    #[inline]
    pub(crate) unsafe fn set_vertex_buffer<'a>(
        &self,
        gl: &glow::Context,
        index: u32,
        binding: super::super::BufferBinding<'a>,
    ) {
        self.state.set_vertex_buffer(
            gl,
            index as usize,
            &binding.buffer.inner,
            binding.offset as i32,
            binding.size,
        );
    }

    #[inline]
    pub(crate) unsafe fn set_index_buffer<'a>(
        &self,
        gl: &glow::Context,
        binding: super::super::BufferBinding<'a>,
        format: wgt::IndexFormat,
    ) {
        self.state.set_index_buffer(
            gl,
            &binding.buffer.inner,
            format,
            binding.offset as i32,
            binding.size,
        )
    }

    #[inline]
    pub(crate) unsafe fn set_viewport(
        &self,
        gl: &glow::Context,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.state.set_viewport(gl, x, y, w, h);

        self.state.set_depth_range(gl, min_depth, max_depth);
    }

    #[inline]
    pub(crate) unsafe fn set_scissor_rect(
        &self,
        gl: &glow::Context,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.state.set_scissor(gl, x, y, w, h);
    }

    #[inline]
    pub(crate) unsafe fn set_stencil_reference(&self, gl: &glow::Context, value: u32) {
        self.state.set_stencil_reference(gl, value as i32);
    }

    #[inline]
    pub(crate) unsafe fn set_blend_constants(&self, gl: &glow::Context, color: &[f32; 4]) {
        self.state.set_blend_color(gl, color);
    }

    #[inline]
    pub(crate) unsafe fn draw(
        &self,
        gl: &glow::Context,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
        debug_assert!(start_instance == 0);

        self.state
            .draw(gl, start_vertex, vertex_count, instance_count);
    }

    #[inline]
    pub(crate) unsafe fn draw_indexed(
        &self,
        gl: &glow::Context,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
        debug_assert!(start_instance == 0);
        debug_assert!(base_vertex == 0);

        self.state.draw_indexed(
            gl,
            start_index as i32,
            index_count as i32,
            instance_count as i32,
        );
    }
}
