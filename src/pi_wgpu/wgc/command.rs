use std::ops::Range;

use crate::pi_wgpu::hal::AdapterContextLock;

use super::super::{
    hal, BindGroup, Buffer, BufferSlice, Color, DynamicOffset, IndexFormat, Label, Operations,
    RenderPipeline, TextureView,
};

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
    pub fn finish(mut self) -> CommandBuffer {
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
        let gl = self.inner.begin_render_pass(desc);

        log::trace!("==================== begin_render_pass");

        RenderPass {
            gl,
            encoder: &self.inner,
        }
    }
}

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
    pub color_attachments: &'desc [Option<RenderPassColorAttachment<'tex>>],
    /// The depth and stencil attachment of the render pass, if any.
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'tex>>,
}

/// In-progress recording of a render pass.
///
/// It can be created with [`CommandEncoder::begin_render_pass`].
///
/// Corresponds to [WebGPU `GPURenderPassEncoder`](
/// https://gpuweb.github.io/gpuweb/#render-pass-encoder).
#[derive(Debug)]
pub struct RenderPass<'a> {
    gl: AdapterContextLock<'a>,
    encoder: &'a hal::CommandEncoder,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        log::trace!("++++++++++++ RenderPass Drop");
        self.encoder.end_render_pass()
    }
}

impl<'a> RenderPass<'a> {
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

        log::trace!("============ wgc::RenderPass::set_bind_group()");

        self.encoder
            .set_bind_group(index, &bind_group.inner, offsets)
    }

    /// Sets the active render pipeline.
    ///
    /// Subsequent draw calls will exhibit the behavior defined by `pipeline`.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        log::trace!("============ wgc::RenderPass::set_pipeline()");
        self.encoder.set_render_pipeline(&self.gl, &pipeline.inner)
    }

    /// Sets the blend color as used by some of the blending modes.
    ///
    /// Subsequent blending tests will test against this value.
    pub fn set_blend_constant(&mut self, color: Color) {
        log::trace!("============ wgc::RenderPass::set_blend_constant()");
        let arr = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        self.encoder.set_blend_constants(&self.gl, &arr)
    }

    /// Sets the active index buffer.
    ///
    /// Subsequent calls to [`draw_indexed`](RenderPass::draw_indexed) on this [`RenderPass`] will
    /// use `buffer` as the source index buffer.
    pub fn set_index_buffer(&mut self, buffer_slice: BufferSlice<'a>, index_format: IndexFormat) {
        log::trace!("============ wgc::RenderPass::set_index_buffer()");
        let binding = super::super::BufferBinding {
            buffer: buffer_slice.buffer,
            offset: buffer_slice.offset,
            size: buffer_slice.size,
        };

        self.encoder
            .set_index_buffer(&self.gl, binding, index_format)
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
        log::trace!("============ wgc::RenderPass::set_vertex_buffer()");
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
        log::trace!("============ wgc::RenderPass::set_scissor_rect()");
        self.encoder
            .set_scissor_rect(&self.gl, x as i32, y as i32, width as i32, height as i32)
    }

    /// Sets the viewport region.
    ///
    /// Subsequent draw calls will draw any fragments in this region.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        log::trace!("============ wgc::RenderPass::set_viewport()");
        self.encoder.set_viewport(
            &self.gl, x as i32, y as i32, w as i32, h as i32, min_depth, max_depth,
        )
    }

    /// Sets the stencil reference.
    ///
    /// Subsequent stencil tests will test against this value.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        log::trace!("============ wgc::RenderPass::set_stencil_reference()");
        self.encoder.set_stencil_reference(&self.gl, reference)
    }

    /// Draws primitives from the active vertex buffer(s).
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        log::trace!("============ wgc::RenderPass::draw()");
        self.encoder.draw(
            &self.gl,
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
        log::trace!("============ wgc::RenderPass::draw_indexed()");
        self.encoder.draw_indexed(
            &self.gl,
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
    pub view: &'tex TextureView,
    /// The view that will receive the resolved output if multisampling is used.
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
