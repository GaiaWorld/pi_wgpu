use std::ops::Range;

use crate::pi_wgpu::hal::AdapterContextLock;

use super::super::{
    hal, BindGroup, Buffer, BufferSlice, Color, DynamicOffset, IndexFormat, Label, Operations,
    RenderPipeline, TextureView,
};
use derive_more::Debug;
use glow::HasContext;

/// Handle to a command buffer on the GPU.
///
/// A `CommandBuffer` represents a complete sequence of commands that may be submitted to a command
/// queue with [`Queue::submit`]. A `CommandBuffer` is obtained by recording a series of commands to
/// a [`CommandEncoder`] and then calling [`CommandEncoder::finish`].
///
/// Corresponds to [WebGPU `GPUCommandBuffer`](https://gpuweb.github.io/gpuweb/#command-buffer).
#[derive(Debug)]
pub struct CommandBuffer {
    pub(crate) inner: hal::CommandBuffer,
}

/// Encodes a series of GPU operations.
///
/// A command encoder can record [`RenderPass`]es, [`ComputePass`]es,
/// and transfer operations between driver-managed resources like [`Buffer`]s and [`Texture`]s.
///
/// When finished recording, call [`CommandEncoder::finish`] to obtain a [`CommandBuffer`] which may
/// be submitted for execution.
///
/// Corresponds to [WebGPU `GPUCommandEncoder`](https://gpuweb.github.io/gpuweb/#command-encoder).
#[derive(Debug)]
pub struct CommandEncoder {
    pub(crate) inner: hal::CommandEncoder,
}

impl CommandEncoder {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::CommandEncoder) -> Self {
        Self { inner }
    }
}

impl CommandEncoder {
    /// Finishes recording and returns a [`CommandBuffer`] that can be submitted for execution.
    pub fn finish(self) -> CommandBuffer {
        log::trace!("command_encoder.finish();
        }}");

        CommandBuffer {
            inner: hal::CommandBuffer,
        }
    }
    /// Begins recording of a render pass.
    ///
    /// This function returns a [`RenderPass`] object which records a single render pass.
    pub fn begin_render_pass<'pass>(
        &'pass mut self,
        desc: &RenderPassDescriptor<'pass, '_>,
    ) -> RenderPass<'pass> {
        log::trace!(
            "let mut render_pass = command_encoder.begin_render_pass(&{:?});",
            desc,
        );

        let gl = self.inner.begin_render_pass(desc);

        RenderPass {
            lock: gl,
            encoder: &self.inner,
        }
    }
}

/// Handle to a query set.
///
/// It can be created with [`Device::create_query_set`].
///
/// Corresponds to [WebGPU `GPUQuerySet`](https://gpuweb.github.io/gpuweb/#queryset).
#[derive(Debug)]
pub struct QuerySet {

}


/// Describes the timestamp writes of a render pass.
///
/// For use with [`RenderPassDescriptor`].
/// At least one of `beginning_of_pass_write_index` and `end_of_pass_write_index` must be `Some`.
///
/// Corresponds to [WebGPU `GPURenderPassTimestampWrite`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpasstimestampwrites).
#[derive(Clone, Debug)]
pub struct RenderPassTimestampWrites<'a> {
    /// The query set to write to.
    pub query_set: &'a QuerySet,
    /// The index of the query set at which a start timestamp of this pass is written, if any.
    pub beginning_of_pass_write_index: Option<u32>,
    /// The index of the query set at which an end timestamp of this pass is written, if any.
    pub end_of_pass_write_index: Option<u32>,
}
#[cfg(send_sync)]
static_assertions::assert_impl_all!(RenderPassTimestampWrites<'_>: Send, Sync);

/// Describes the attachments of a render pass.
///
/// For use with [`CommandEncoder::begin_render_pass`].
///
/// Note: separate lifetimes are needed because the texture views (`'tex`)
/// have to live as long as the pass is recorded, while everything else (`'desc`) doesn't.
///
/// Corresponds to [WebGPU `GPURenderPassDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderpassdescriptor).
#[derive(Clone, Debug, Default)]
pub struct RenderPassDescriptor<'tex, 'desc> {
    /// Debug label of the render pass. This will show up in graphics debuggers for easy identification.
    pub label: Label<'desc>,
    /// The color attachments of the render pass.
    #[debug("&{color_attachments:?}")]
    pub color_attachments: &'desc [Option<RenderPassColorAttachment<'tex>>],
    /// The depth and stencil attachment of the render pass, if any.
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'tex>>,
    /// Defines which timestamp values will be written for this pass, and where to write them to.
    ///
    /// Requires [`Features::TIMESTAMP_QUERY`] to be enabled.
    pub timestamp_writes: Option<RenderPassTimestampWrites<'desc>>,
    /// Defines where the occlusion query results will be stored for this pass.
    pub occlusion_query_set: Option<&'tex QuerySet>,
}

/// In-progress recording of a render pass.
///
/// It can be created with [`CommandEncoder::begin_render_pass`].
///
/// Corresponds to [WebGPU `GPURenderPassEncoder`](
/// https://gpuweb.github.io/gpuweb/#render-pass-encoder).
#[derive(Debug)]
pub struct RenderPass<'a> {
    lock: AdapterContextLock<'a>,
    encoder: &'a hal::CommandEncoder,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        // log::trace!("Dropping RenderPass");
        self.encoder.end_render_pass();
    }
}

impl<'a> RenderPass<'a> {
    pub fn flush(&self) {
        unsafe { self.lock.get_glow().flush();}
    }
    /// Sets the active bind group for a given bind group index. The bind group layout
    /// in the active pipeline when any `draw()` function is called must match the layout of this bind group.
    ///
    /// If the bind group have dynamic offsets, provide them in binding order.
    /// These offsets have to be aligned to [`Limits::min_uniform_buffer_offset_alignment`]
    /// or [`Limits::min_storage_buffer_offset_alignment`] appropriately.
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a BindGroup,
        offsets: &[DynamicOffset],
    ) {
        log::trace!(
            "render_pass.set_bind_group({:?}, &bind_group{:?}, &{:?});",
            index,
            bind_group.inner.id,
            offsets
        );

        self.encoder
            .set_bind_group(index, &bind_group.inner, offsets)
    }

    /// Sets the active render pipeline.
    ///
    /// Subsequent draw calls will exhibit the behavior defined by `pipeline`.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        log::trace!(
            "render_pass.set_pipeline(&render_pipeline{:?});",
            pipeline.inner.0.id
        );
        self.encoder
            .set_render_pipeline(&self.lock.get_glow(), &pipeline.inner)
    }

    /// Sets the blend color as used by some of the blending modes.
    ///
    /// Subsequent blending tests will test against this value.
    pub fn set_blend_constant(&mut self, color: Color) {
        log::trace!("render_pass.set_blend_constant({:?});", color);
        let arr = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        self.encoder
            .set_blend_constants(&self.lock.get_glow(), &arr)
    }

    /// Sets the active index buffer.
    ///
    /// Subsequent calls to [`draw_indexed`](RenderPass::draw_indexed) on this [`RenderPass`] will
    /// use `buffer` as the source index buffer.
    pub fn set_index_buffer(&mut self, buffer_slice: BufferSlice<'a>, index_format: IndexFormat) {
		#[cfg(not(target_arch = "wasm32"))]
        match buffer_slice.size {
            Some(r) => log::trace!(
                "render_pass.set_index_buffer(buffer{}.slice({}..{}), IndexFormat::{:?});",
                buffer_slice.buffer.inner.0.raw.0.get(),
                buffer_slice.offset,
                buffer_slice.offset + r.get(),
                index_format
            ),
            None => log::trace!(
                "render_pass.set_index_buffer(buffer{}.slice({}..), IndexFormat::{:?});",
                buffer_slice.buffer.inner.0.raw.0.get(),
                buffer_slice.offset,
                index_format
            ),
        };

        let binding = super::super::BufferBinding {
            buffer: buffer_slice.buffer,
            offset: buffer_slice.offset,
            size: buffer_slice.size,
        };

        self.encoder
            .set_index_buffer(&self.lock.get_glow(), binding, index_format)
    }

    /// Assign a vertex buffer to a slot.
    ///
    /// Subsequent calls to [`draw`] and [`draw_indexed`] on this
    /// [`RenderPass`] will use `buffer` as one of the source vertex buffers.
    ///
    /// The `slot` refers to the index of the matching descriptor in
    /// [`VertexState::buffers`].
    ///
    /// [`draw`]: RenderPass::draw
    /// [`draw_indexed`]: RenderPass::draw_indexed
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: BufferSlice<'a>) {
		#[cfg(not(target_arch = "wasm32"))]
        match buffer_slice.size {
            Some(r) => log::trace!(
                "render_pass.set_vertex_buffer({}, buffer{}.slice({}..{}));",
                slot,
                buffer_slice.buffer.inner.0.raw.0.get(),
                buffer_slice.offset,
                buffer_slice.offset + r.get()
            ),
            None => log::trace!(
                "render_pass.set_vertex_buffer({}, buffer{}.slice({}..));",
                slot,
                buffer_slice.buffer.inner.0.raw.0.get(),
                buffer_slice.offset
            ),
        };

        let binding = super::super::BufferBinding {
            buffer: buffer_slice.buffer,
            offset: buffer_slice.offset,
            size: buffer_slice.size,
        };
        self.encoder.set_vertex_buffer(slot, binding)
    }

    /// Sets the scissor region.
    ///
    /// Subsequent draw calls will discard any fragments that fall outside this region.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        log::trace!("render_pass.set_scissor_rect({x}, {y}, {width}, {height});");
        self.encoder.set_scissor_rect(
            &self.lock.get_glow(),
            x as i32,
            y as i32,
            width as i32,
            height as i32,
        )
    }

    /// Sets the viewport region.
    ///
    /// Subsequent draw calls will draw any fragments in this region.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        log::trace!("render_pass.set_viewport({x:?}, {y:?}, {w:?}, {h:?}, {min_depth:?}, {max_depth:?});");
        self.encoder.set_viewport(
            &self.lock.get_glow(),
            x as i32,
            y as i32,
            w as i32,
            h as i32,
            min_depth,
            max_depth,
        )
    }

    /// Sets the stencil reference.
    ///
    /// Subsequent stencil tests will test against this value.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        log::trace!("render_pass.set_stencil_reference({reference});");
        self.encoder
            .set_stencil_reference(&self.lock.get_glow(), reference)
    }

    /// Draws primitives from the active vertex buffer(s).
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        log::trace!("render_pass.draw({:?}, {:?});", vertices, instances);
        self.encoder.draw(
            &self.lock.get_glow(),
            vertices.start,
            vertices.len() as u32,
            instances.start,
            instances.len() as u32,
        )
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers.
    ///
    /// The active index buffer can be set with [`RenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        log::trace!("render_pass.draw_indexed({indices:?}, {base_vertex:?}, {instances:?});");
        self.encoder.draw_indexed(
            &self.lock.get_glow(),
            indices.start,
            indices.len() as u32,
            base_vertex,
            instances.start,
            instances.len() as u32,
        )
    }
}

/// Describes a color attachment to a [`RenderPass`].
///
/// For use with [`RenderPassDescriptor`].
///
/// Corresponds to [WebGPU `GPURenderPassColorAttachment`](
/// https://gpuweb.github.io/gpuweb/#color-attachments).
#[derive(Clone, Debug)]
pub struct RenderPassColorAttachment<'tex> {
    /// The view to use as an attachment.
    #[debug("&texture_view{:?}", view.inner.id)]
    pub view: &'tex TextureView,
    /// The view that will receive the resolved output if multisampling is used.
    #[debug("{}", match resolve_target {
        Some(r) => format!("Some(&texture_view{})", r.inner.id),
        None => "None".to_string()
    })]
    pub resolve_target: Option<&'tex TextureView>,
    /// What operations will be performed on this color attachment.
    pub ops: Operations<Color>,
}

/// Describes a depth/stencil attachment to a [`RenderPass`].
///
/// For use with [`RenderPassDescriptor`].
///
/// Corresponds to [WebGPU `GPURenderPassDepthStencilAttachment`](
/// https://gpuweb.github.io/gpuweb/#depth-stencil-attachments).
#[derive(Clone, Debug)]
pub struct RenderPassDepthStencilAttachment<'tex> {
    /// The view to use as an attachment.
    #[debug("&texture_view{:?}", view.inner.id)]
    pub view: &'tex TextureView,
    /// What operations will be performed on the depth part of the attachment.
    pub depth_ops: Option<Operations<f32>>,
    /// What operations will be performed on the stencil part of the attachment.
    pub stencil_ops: Option<Operations<u32>>,
}

/// Describes a [`CommandEncoder`].
///
/// For use with [`Device::create_command_encoder`].
///
/// Corresponds to [WebGPU `GPUCommandEncoderDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpucommandencoderdescriptor).
pub type CommandEncoderDescriptor<'a> = super::super::wgt::CommandEncoderDescriptor<Label<'a>>;

pub use super::super::wgt::ImageCopyBuffer as ImageCopyBufferBase;

/// View of a buffer which can be used to copy to/from a texture.
///
/// Corresponds to [WebGPU `GPUImageCopyBuffer`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuimagecopybuffer).
pub type ImageCopyBuffer<'a> = ImageCopyBufferBase<&'a Buffer>;
