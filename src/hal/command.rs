//!
//! + 仅 单线程
//! + 录制 即为 调用 GL 指令
//!
//! `CommandEncoder` 目前 仅支持 如下接口：
//!
//! + begin_encoding / end_encoding
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

use super::GLState;
use crate::wgt;

#[derive(Debug)]
pub(crate) struct CommandBuffer;

#[derive(Debug, Clone)]
pub(crate) struct CommandEncoder {
    state: GLState,
}

impl CommandEncoder {
    pub fn new(
        state: GLState,
        desc: &crate::CommandEncoderDescriptor,
    ) -> Result<Self, crate::DeviceError> {
        Ok(Self { state })
    }
}

impl CommandEncoder {
    #[inline]
    pub(crate) unsafe fn begin_encoding(
        &mut self,
        label: super::Label,
    ) -> Result<(), crate::DeviceError> {
        Ok(())
    }

    #[inline]
    pub(crate) unsafe fn end_encoding(
        &mut self,
    ) -> Result<super::CommandBuffer, crate::DeviceError> {
        Ok(CommandBuffer)
    }

    #[inline]
    pub(crate) unsafe fn begin_render_pass(&mut self, desc: &crate::RenderPassDescriptor) {
        self.state.0.borrow_mut().set_render_target(desc);
    }

    #[inline]
    pub(crate) unsafe fn end_render_pass(&mut self) {}

    #[inline]
    pub(crate) unsafe fn set_bind_group(
        &mut self,
        index: u32,
        group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        self.state
            .0
            .borrow_mut()
            .set_bind_group(index, group, dynamic_offsets);
    }

    #[inline]
    pub(crate) unsafe fn set_render_pipeline(&mut self, pipeline: &super::RenderPipeline) {
        self.state.0.borrow_mut().set_render_pipeline(pipeline);
    }

    #[inline]
    pub(crate) unsafe fn set_vertex_buffer<'a>(
        &mut self,
        index: u32,
        binding: crate::BufferBinding<'a>,
    ) {
        self.state.0.borrow_mut().set_vertex_buffer(
            index as usize,
            &binding.buffer.inner,
            binding.offset as i32,
            binding.size,
        );
    }

    #[inline]
    pub(crate) unsafe fn set_index_buffer<'a>(
        &mut self,
        binding: crate::BufferBinding<'a>,
        format: wgt::IndexFormat,
    ) {
        self.state.0.borrow_mut().set_index_buffer(
            &binding.buffer.inner,
            format,
            binding.offset as i32,
            binding.size,
        )
    }

    #[inline]
    pub(crate) unsafe fn set_viewport(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.state.0.borrow_mut().set_viewport(x, y, w, h);

        self.state
            .0
            .borrow_mut()
            .set_depth_range(min_depth, max_depth);
    }

    #[inline]
    pub(crate) unsafe fn set_scissor_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.state.0.borrow_mut().set_scissor(x, y, w, h);
    }

    #[inline]
    pub(crate) unsafe fn set_stencil_reference(&mut self, value: u32) {
        self.state
            .0
            .borrow_mut()
            .set_stencil_reference(value as i32);
    }

    #[inline]
    pub(crate) unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {
        self.state.0.borrow_mut().set_blend_color(color);
    }

    #[inline]
    pub(crate) unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
        debug_assert!(start_instance == 0);

        self.state
            .0
            .borrow_mut()
            .draw(start_vertex, vertex_count, instance_count);
    }

    #[inline]
    pub(crate) unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
        debug_assert!(start_instance == 0);
        debug_assert!(base_vertex == 0);

        self.state.0.borrow_mut().draw_indexed(
            start_index as i32,
            index_count as i32,
            instance_count as i32,
        );
    }
}
