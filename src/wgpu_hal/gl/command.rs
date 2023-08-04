use std::ops::Range;

use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct CommandBuffer;

unsafe impl Send for CommandBuffer {}
unsafe impl Sync for CommandBuffer {}

#[derive(Debug)]
pub struct CommandEncoder;

unsafe impl Send for CommandEncoder {}
unsafe impl Sync for CommandEncoder {}

impl api::CommandEncoder<super::Api> for CommandEncoder {
    unsafe fn begin_encoding(&mut self, label: Label) -> super::DeviceResult<()> {
        Ok(())
    }
    unsafe fn discard_encoding(&mut self) {}

    unsafe fn end_encoding(&mut self) -> super::DeviceResult<CommandBuffer> {
        Ok(CommandBuffer)
    }

    unsafe fn reset_all<I>(&mut self, command_buffers: I) {}

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = BufferBarrier<'a, super::Api>>,
    {
    }

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = TextureBarrier<'a, super::Api>>,
    {
    }

    unsafe fn clear_buffer(&mut self, buffer: &super::Buffer, range: MemoryRange) {}

    unsafe fn copy_buffer_to_buffer<T>(
        &mut self,
        src: &super::Buffer,
        dst: &super::Buffer,
        regions: T,
    ) {
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    unsafe fn copy_external_image_to_texture<T>(
        &mut self,
        src: &wgt::ImageCopyExternalImage,
        dst: &super::Texture,
        dst_premultiplication: bool,
        regions: T,
    ) where
        T: Iterator<Item = TextureCopy>,
    {
    }

    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &super::Texture,
        src_usage: TextureUses,
        dst: &super::Texture,
        regions: T,
    ) {
    }

    unsafe fn copy_buffer_to_texture<T>(
        &mut self,
        src: &super::Buffer,
        dst: &super::Texture,
        regions: T,
    ) {
    }

    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &super::Texture,
        src_usage: TextureUses,
        dst: &super::Buffer,
        regions: T,
    ) {
    }

    unsafe fn begin_query(&mut self, set: &super::QuerySet, index: u32) {}

    unsafe fn end_query(&mut self, set: &super::QuerySet, index: u32) {}

    unsafe fn write_timestamp(&mut self, set: &super::QuerySet, index: u32) {}

    unsafe fn reset_queries(&mut self, set: &super::QuerySet, range: Range<u32>) {}

    unsafe fn copy_query_results(
        &mut self,
        set: &super::QuerySet,
        range: Range<u32>,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        stride: wgt::BufferSize,
    ) {
    }

    // render

    unsafe fn begin_render_pass(&mut self, desc: &RenderPassDescriptor<super::Api>) {}

    unsafe fn end_render_pass(&mut self) {}

    unsafe fn set_bind_group(
        &mut self,
        layout: &super::PipelineLayout,
        index: u32,
        group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
    }

    unsafe fn set_push_constants(
        &mut self,
        layout: &super::PipelineLayout,
        stages: wgt::ShaderStages,
        offset: u32,
        data: &[u32],
    ) {
    }

    unsafe fn insert_debug_marker(&mut self, label: &str) {}

    unsafe fn begin_debug_marker(&mut self, group_label: &str) {}

    unsafe fn end_debug_marker(&mut self) {}

    unsafe fn set_render_pipeline(&mut self, pipeline: &super::RenderPipeline) {}

    unsafe fn set_index_buffer<'a>(
        &mut self,
        binding: BufferBinding<'a, super::Api>,
        format: wgt::IndexFormat,
    ) {
    }

    unsafe fn set_vertex_buffer<'a>(&mut self, index: u32, binding: BufferBinding<'a, super::Api>) {
    }

    unsafe fn set_viewport(&mut self, rect: &Rect<f32>, depth_range: Range<f32>) {}

    unsafe fn set_scissor_rect(&mut self, rect: &Rect<u32>) {}

    unsafe fn set_stencil_reference(&mut self, value: u32) {}

    unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {}

    unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
    }

    unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
    }

    unsafe fn draw_indirect(
        &mut self,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        draw_count: u32,
    ) {
    }

    unsafe fn draw_indexed_indirect(
        &mut self,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        draw_count: u32,
    ) {
    }

    unsafe fn draw_indirect_count(
        &mut self,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        count_buffer: &super::Buffer,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }

    unsafe fn draw_indexed_indirect_count(
        &mut self,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        count_buffer: &super::Buffer,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }

    // compute

    unsafe fn begin_compute_pass(&mut self, desc: &ComputePassDescriptor) {}

    unsafe fn end_compute_pass(&mut self) {}

    unsafe fn set_compute_pipeline(&mut self, pipeline: &super::ComputePipeline) {}

    unsafe fn dispatch(&mut self, count: [u32; 3]) {}

    unsafe fn dispatch_indirect(&mut self, buffer: &super::Buffer, offset: wgt::BufferAddress) {}
}
