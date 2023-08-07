use std::{marker::PhantomData, num::NonZeroU32, ops::Range};

use crate::{
    BindGroup, Buffer, BufferAddress, BufferSize, BufferSlice, Color, ComputePipeline,
    DynamicOffset, Extent3d, ImageSubresourceRange, IndexFormat, Label, Operations, QuerySet,
    RenderBundleDepthStencil, RenderPipeline, ShaderStages, Texture, TextureFormat, TextureView,
};

/// Handle to a command buffer on the GPU.
///
/// A `CommandBuffer` represents a complete sequence of commands that may be submitted to a command
/// queue with [`Queue::submit`]. A `CommandBuffer` is obtained by recording a series of commands to
/// a [`CommandEncoder`] and then calling [`CommandEncoder::finish`].
///
/// Corresponds to [WebGPU `GPUCommandBuffer`](https://gpuweb.github.io/gpuweb/#command-buffer).
#[derive(Debug)]
pub struct CommandBuffer {}

static_assertions::assert_impl_all!(CommandBuffer: Send, Sync);

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unimplemented!("CommandBuffer::drop is not implemented")
    }
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
pub struct CommandEncoder {}

static_assertions::assert_impl_all!(CommandEncoder: Send, Sync);

impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unimplemented!("CommandEncoder::drop is not implemented")
    }
}

impl CommandEncoder {
    /// Finishes recording and returns a [`CommandBuffer`] that can be submitted for execution.
    pub fn finish(mut self) -> CommandBuffer {
        unimplemented!("CommandEncoder::finish is not implemented")
    }

    /// Begins recording of a render pass.
    ///
    /// This function returns a [`RenderPass`] object which records a single render pass.
    pub fn begin_render_pass<'pass>(
        &'pass mut self,
        desc: &RenderPassDescriptor<'pass, '_>,
    ) -> RenderPass<'pass> {
        unimplemented!("CommandEncoder::begin_render_pass is not implemented")
    }

    /// Begins recording of a compute pass.
    ///
    /// This function returns a [`ComputePass`] object which records a single compute pass.
    pub fn begin_compute_pass(&mut self, desc: &ComputePassDescriptor) -> ComputePass {
        unimplemented!("CommandEncoder::begin_compute_pass is not implemented")
    }

    /// Copy data from one buffer to another.
    ///
    /// # Panics
    ///
    /// - Buffer offsets or copy size not a multiple of [`COPY_BUFFER_ALIGNMENT`].
    /// - Copy would overrun buffer.
    /// - Copy within the same buffer.
    pub fn copy_buffer_to_buffer(
        &mut self,
        source: &Buffer,
        source_offset: BufferAddress,
        destination: &Buffer,
        destination_offset: BufferAddress,
        copy_size: BufferAddress,
    ) {
        unimplemented!("CommandEncoder::copy_buffer_to_buffer is not implemented")
    }

    /// Copy data from a buffer to a texture.
    ///
    /// # Panics
    ///
    /// - Copy would overrun buffer.
    /// - Copy would overrun texture.
    /// - `source.layout.bytes_per_row` isn't divisible by [`COPY_BYTES_PER_ROW_ALIGNMENT`].
    pub fn copy_buffer_to_texture(
        &mut self,
        source: crate::ImageCopyBuffer,
        destination: crate::ImageCopyTexture,
        copy_size: Extent3d,
    ) {
        unimplemented!("CommandEncoder::copy_buffer_to_texture is not implemented")
    }

    /// Copy data from a texture to a buffer.
    ///
    /// # Panics
    ///
    /// - Copy would overrun buffer.
    /// - Copy would overrun texture.
    /// - `source.layout.bytes_per_row` isn't divisible by [`COPY_BYTES_PER_ROW_ALIGNMENT`].
    pub fn copy_texture_to_buffer(
        &mut self,
        source: crate::ImageCopyTexture,
        destination: crate::ImageCopyBuffer,
        copy_size: Extent3d,
    ) {
        unimplemented!("CommandEncoder::copy_texture_to_buffer is not implemented")
    }

    /// Copy data from one texture to another.
    ///
    /// # Panics
    ///
    /// - Textures are not the same type
    /// - If a depth texture, or a multisampled texture, the entire texture must be copied
    /// - Copy would overrun either texture
    pub fn copy_texture_to_texture(
        &mut self,
        source: crate::ImageCopyTexture,
        destination: crate::ImageCopyTexture,
        copy_size: Extent3d,
    ) {
        unimplemented!("CommandEncoder::copy_texture_to_texture is not implemented")
    }

    /// Clears texture to zero.
    ///
    /// Note that unlike with clear_buffer, `COPY_DST` usage is not required.
    ///
    /// # Implementation notes
    ///
    /// - implemented either via buffer copies and render/depth target clear, path depends on texture usages
    /// - behaves like texture zero init, but is performed immediately (clearing is *not* delayed via marking it as uninitialized)
    ///
    /// # Panics
    ///
    /// - `CLEAR_TEXTURE` extension not enabled
    /// - Range is out of bounds
    pub fn clear_texture(&mut self, texture: &Texture, subresource_range: &ImageSubresourceRange) {
        unimplemented!("CommandEncoder::clear_texture is not implemented")
    }

    /// Clears buffer to zero.
    ///
    /// # Panics
    ///
    /// - Buffer does not have `COPY_DST` usage.
    /// - Range it out of bounds
    pub fn clear_buffer(
        &mut self,
        buffer: &Buffer,
        offset: BufferAddress,
        size: Option<BufferSize>,
    ) {
        unimplemented!("CommandEncoder::clear_buffer is not implemented")
    }

    /// Inserts debug marker.
    pub fn insert_debug_marker(&mut self, label: &str) {
        unimplemented!("CommandEncoder::insert_debug_marker is not implemented")
    }

    /// Start record commands and group it into debug marker group.
    pub fn push_debug_group(&mut self, label: &str) {
        unimplemented!("CommandEncoder::push_debug_group is not implemented")
    }

    /// Stops command recording and creates debug group.
    pub fn pop_debug_group(&mut self) {
        unimplemented!("CommandEncoder::pop_debug_group is not implemented")
    }
}

/// [`Features::TIMESTAMP_QUERY`] must be enabled on the device in order to call these functions.
impl CommandEncoder {
    /// Issue a timestamp command at this point in the queue.
    /// The timestamp will be written to the specified query set, at the specified index.
    ///
    /// Must be multiplied by [`Queue::get_timestamp_period`] to get
    /// the value in nanoseconds. Absolute values have no meaning,
    /// but timestamps can be subtracted to get the time it takes
    /// for a string of operations to complete.
    pub fn write_timestamp(&mut self, query_set: &QuerySet, query_index: u32) {
        unimplemented!("CommandEncoder::write_timestamp is not implemented")
    }
}

/// [`Features::TIMESTAMP_QUERY`] or [`Features::PIPELINE_STATISTICS_QUERY`] must be enabled on the device in order to call these functions.
impl CommandEncoder {
    /// Resolve a query set, writing the results into the supplied destination buffer.
    ///
    /// Queries may be between 8 and 40 bytes each. See [`PipelineStatisticsTypes`] for more information.
    pub fn resolve_query_set(
        &mut self,
        query_set: &QuerySet,
        query_range: Range<u32>,
        destination: &Buffer,
        destination_offset: BufferAddress,
    ) {
        unimplemented!("CommandEncoder::resolve_query_set is not implemented")
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

static_assertions::assert_impl_all!(RenderPassDescriptor: Send, Sync);

/// In-progress recording of a render pass.
///
/// It can be created with [`CommandEncoder::begin_render_pass`].
///
/// Corresponds to [WebGPU `GPURenderPassEncoder`](
/// https://gpuweb.github.io/gpuweb/#render-pass-encoder).
#[derive(Debug)]
pub struct RenderPass<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {}
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
        unimplemented!("RenderPass::set_bind_group is not implemented")
    }

    /// Sets the active render pipeline.
    ///
    /// Subsequent draw calls will exhibit the behavior defined by `pipeline`.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        unimplemented!("RenderPass::set_pipeline is not implemented")
    }

    /// Sets the blend color as used by some of the blending modes.
    ///
    /// Subsequent blending tests will test against this value.
    pub fn set_blend_constant(&mut self, color: Color) {
        unimplemented!("RenderPass::set_blend_constant is not implemented")
    }

    /// Sets the active index buffer.
    ///
    /// Subsequent calls to [`draw_indexed`](RenderPass::draw_indexed) on this [`RenderPass`] will
    /// use `buffer` as the source index buffer.
    pub fn set_index_buffer(&mut self, buffer_slice: BufferSlice<'a>, index_format: IndexFormat) {
        unimplemented!("RenderPass::set_index_buffer is not implemented")
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
        unimplemented!("RenderPass::set_vertex_buffer is not implemented")
    }

    /// Sets the scissor region.
    ///
    /// Subsequent draw calls will discard any fragments that fall outside this region.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        unimplemented!("RenderPass::set_scissor_rect is not implemented")
    }

    /// Sets the viewport region.
    ///
    /// Subsequent draw calls will draw any fragments in this region.
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, min_depth: f32, max_depth: f32) {
        unimplemented!("RenderPass::set_viewport is not implemented")
    }

    /// Sets the stencil reference.
    ///
    /// Subsequent stencil tests will test against this value.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        unimplemented!("RenderPass::set_stencil_reference is not implemented")
    }

    /// Draws primitives from the active vertex buffer(s).
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        unimplemented!("RenderPass::draw is not implemented")
    }

    /// Inserts debug marker.
    pub fn insert_debug_marker(&mut self, label: &str) {
        unimplemented!("RenderPass::insert_debug_marker is not implemented")
    }

    /// Start record commands and group it into debug marker group.
    pub fn push_debug_group(&mut self, label: &str) {
        unimplemented!("RenderPass::push_debug_group is not implemented")
    }

    /// Stops command recording and creates debug group.
    pub fn pop_debug_group(&mut self) {
        unimplemented!("RenderPass::pop_debug_group is not implemented")
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers.
    ///
    /// The active index buffer can be set with [`RenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        unimplemented!("RenderPass::draw_indexed is not implemented")
    }

    /// Draws primitives from the active vertex buffer(s) based on the contents of the `indirect_buffer`.
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndirect`](crate::util::DrawIndirect).
    pub fn draw_indirect(&mut self, indirect_buffer: &'a Buffer, indirect_offset: BufferAddress) {
        unimplemented!("RenderPass::draw_indirect is not implemented")
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers,
    /// based on the contents of the `indirect_buffer`.
    ///
    /// The active index buffer can be set with [`RenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndexedIndirect`](crate::util::DrawIndexedIndirect).
    pub fn draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
    ) {
        unimplemented!("RenderPass::draw_indexed_indirect is not implemented")
    }

    /// Execute a [render bundle][RenderBundle], which is a set of pre-recorded commands
    /// that can be run together.
    pub fn execute_bundles<I: IntoIterator<Item = &'a RenderBundle> + 'a>(
        &mut self,
        render_bundles: I,
    ) {
        unimplemented!("RenderPass::execute_bundles is not implemented")
    }
}

/// [`Features::MULTI_DRAW_INDIRECT`] must be enabled on the device in order to call these functions.
impl<'a> RenderPass<'a> {
    /// Dispatches multiple draw calls from the active vertex buffer(s) based on the contents of the `indirect_buffer`.
    /// `count` draw calls are issued.
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndirect`](crate::util::DrawIndirect).
    ///
    /// These draw structures are expected to be tightly packed.
    pub fn multi_draw_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
        count: u32,
    ) {
        unimplemented!("RenderPass::multi_draw_indirect is not implemented")
    }

    /// Dispatches multiple draw calls from the active index buffer and the active vertex buffers,
    /// based on the contents of the `indirect_buffer`. `count` draw calls are issued.
    ///
    /// The active index buffer can be set with [`RenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndexedIndirect`](crate::util::DrawIndexedIndirect).
    ///
    /// These draw structures are expected to be tightly packed.
    pub fn multi_draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
        count: u32,
    ) {
        unimplemented!("RenderPass::multi_draw_indexed_indirect is not implemented")
    }
}

/// [`Features::MULTI_DRAW_INDIRECT_COUNT`] must be enabled on the device in order to call these functions.
impl<'a> RenderPass<'a> {
    /// Dispatches multiple draw calls from the active vertex buffer(s) based on the contents of the `indirect_buffer`.
    /// The count buffer is read to determine how many draws to issue.
    ///
    /// The indirect buffer must be long enough to account for `max_count` draws, however only `count` will
    /// draws will be read. If `count` is greater than `max_count`, `max_count` will be used.
    ///
    /// The active vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndirect`](crate::util::DrawIndirect).
    ///
    /// These draw structures are expected to be tightly packed.
    ///
    /// The structure expected in `count_buffer` is the following:
    ///
    /// ```rust
    /// #[repr(C)]
    /// struct DrawIndirectCount {
    ///     count: u32, // Number of draw calls to issue.
    /// }
    /// ```
    pub fn multi_draw_indirect_count(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
        count_buffer: &'a Buffer,
        count_offset: BufferAddress,
        max_count: u32,
    ) {
        unimplemented!("RenderPass::multi_draw_indirect_count is not implemented")
    }

    /// Dispatches multiple draw calls from the active index buffer and the active vertex buffers,
    /// based on the contents of the `indirect_buffer`. The count buffer is read to determine how many draws to issue.
    ///
    /// The indirect buffer must be long enough to account for `max_count` draws, however only `count` will
    /// draws will be read. If `count` is greater than `max_count`, `max_count` will be used.
    ///
    /// The active index buffer can be set with [`RenderPass::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderPass::set_vertex_buffer`].
    ///
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndexedIndirect`](crate::util::DrawIndexedIndirect).
    ///
    /// These draw structures are expected to be tightly packed.
    ///
    /// The structure expected in `count_buffer` is the following:
    ///
    /// ```rust
    /// #[repr(C)]
    /// struct DrawIndexedIndirectCount {
    ///     count: u32, // Number of draw calls to issue.
    /// }
    /// ```
    pub fn multi_draw_indexed_indirect_count(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
        count_buffer: &'a Buffer,
        count_offset: BufferAddress,
        max_count: u32,
    ) {
        unimplemented!("RenderPass::multi_draw_indexed_indirect_count is not implemented")
    }
}

/// [`Features::PUSH_CONSTANTS`] must be enabled on the device in order to call these functions.
impl<'a> RenderPass<'a> {
    /// Set push constant data for subsequent draw calls.
    ///
    /// Write the bytes in `data` at offset `offset` within push constant
    /// storage, all of which are accessible by all the pipeline stages in
    /// `stages`, and no others.  Both `offset` and the length of `data` must be
    /// multiples of [`PUSH_CONSTANT_ALIGNMENT`], which is always 4.
    ///
    /// For example, if `offset` is `4` and `data` is eight bytes long, this
    /// call will write `data` to bytes `4..12` of push constant storage.
    ///
    /// # Stage matching
    ///
    /// Every byte in the affected range of push constant storage must be
    /// accessible to exactly the same set of pipeline stages, which must match
    /// `stages`. If there are two bytes of storage that are accessible by
    /// different sets of pipeline stages - say, one is accessible by fragment
    /// shaders, and the other is accessible by both fragment shaders and vertex
    /// shaders - then no single `set_push_constants` call may affect both of
    /// them; to write both, you must make multiple calls, each with the
    /// appropriate `stages` value.
    ///
    /// Which pipeline stages may access a given byte is determined by the
    /// pipeline's [`PushConstant`] global variable and (if it is a struct) its
    /// members' offsets.
    ///
    /// For example, suppose you have twelve bytes of push constant storage,
    /// where bytes `0..8` are accessed by the vertex shader, and bytes `4..12`
    /// are accessed by the fragment shader. This means there are three byte
    /// ranges each accessed by a different set of stages:
    ///
    /// - Bytes `0..4` are accessed only by the fragment shader.
    ///
    /// - Bytes `4..8` are accessed by both the fragment shader and the vertex shader.
    ///
    /// - Bytes `8..12` are accessed only by the vertex shader.
    ///
    /// To write all twelve bytes requires three `set_push_constants` calls, one
    /// for each range, each passing the matching `stages` mask.
    ///
    /// [`PushConstant`]: https://docs.rs/naga/latest/naga/enum.StorageClass.html#variant.PushConstant
    pub fn set_push_constants(&mut self, stages: ShaderStages, offset: u32, data: &[u8]) {
        unimplemented!("RenderPass::set_push_constants is not implemented")
    }
}

/// [`Features::WRITE_TIMESTAMP_INSIDE_PASSES`] must be enabled on the device in order to call these functions.
impl<'a> RenderPass<'a> {
    /// Issue a timestamp command at this point in the queue. The
    /// timestamp will be written to the specified query set, at the specified index.
    ///
    /// Must be multiplied by [`Queue::get_timestamp_period`] to get
    /// the value in nanoseconds. Absolute values have no meaning,
    /// but timestamps can be subtracted to get the time it takes
    /// for a string of operations to complete.
    pub fn write_timestamp(&mut self, query_set: &QuerySet, query_index: u32) {
        unimplemented!("RenderPass::write_timestamp is not implemented")
    }
}

/// [`Features::PIPELINE_STATISTICS_QUERY`] must be enabled on the device in order to call these functions.
impl<'a> RenderPass<'a> {
    /// Start a pipeline statistics query on this render pass. It can be ended with
    /// `end_pipeline_statistics_query`. Pipeline statistics queries may not be nested.
    pub fn begin_pipeline_statistics_query(&mut self, query_set: &QuerySet, query_index: u32) {
        unimplemented!("RenderPass::begin_pipeline_statistics_query is not implemented")
    }

    /// End the pipeline statistics query on this render pass. It can be started with
    /// `begin_pipeline_statistics_query`. Pipeline statistics queries may not be nested.
    pub fn end_pipeline_statistics_query(&mut self) {
        unimplemented!("RenderPass::end_pipeline_statistics_query is not implemented")
    }
}

/// Pre-prepared reusable bundle of GPU operations.
///
/// It only supports a handful of render commands, but it makes them reusable. Executing a
/// [`RenderBundle`] is often more efficient than issuing the underlying commands manually.
///
/// It can be created by use of a [`RenderBundleEncoder`], and executed onto a [`CommandEncoder`]
/// using [`RenderPass::execute_bundles`].
///
/// Corresponds to [WebGPU `GPURenderBundle`](https://gpuweb.github.io/gpuweb/#render-bundle).
#[derive(Debug)]
pub struct RenderBundle {}

static_assertions::assert_impl_all!(RenderBundle: Send, Sync);

impl Drop for RenderBundle {
    fn drop(&mut self) {
        unimplemented!("RenderBundle::drop is not implemented")
    }
}

/// Describes a [`RenderBundle`].
///
/// For use with [`RenderBundleEncoder::finish`].
///
/// Corresponds to [WebGPU `GPURenderBundleDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderbundledescriptor).
pub type RenderBundleDescriptor<'a> = wgt::RenderBundleDescriptor<Label<'a>>;
static_assertions::assert_impl_all!(RenderBundleDescriptor: Send, Sync);

/// Describes the attachments of a compute pass.
///
/// For use with [`CommandEncoder::begin_compute_pass`].
///
/// Corresponds to [WebGPU `GPUComputePassDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpucomputepassdescriptor).
#[derive(Clone, Debug, Default)]
pub struct ComputePassDescriptor<'a> {
    /// Debug label of the compute pass. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
}

static_assertions::assert_impl_all!(ComputePassDescriptor: Send, Sync);

/// In-progress recording of a compute pass.
///
/// It can be created with [`CommandEncoder::begin_compute_pass`].
///
/// Corresponds to [WebGPU `GPUComputePassEncoder`](
/// https://gpuweb.github.io/gpuweb/#compute-pass-encoder).
#[derive(Debug)]
pub struct ComputePass<'a> {
    _data: PhantomData<&'a ()>,
}

impl<'a> Drop for ComputePass<'a> {
    fn drop(&mut self) {
        unimplemented!("ComputePass::drop is not implemented")
    }
}

impl<'a> ComputePass<'a> {
    /// Sets the active bind group for a given bind group index. The bind group layout
    /// in the active pipeline when the `dispatch()` function is called must match the layout of this bind group.
    ///
    /// If the bind group have dynamic offsets, provide them in the binding order.
    /// These offsets have to be aligned to [`Limits::min_uniform_buffer_offset_alignment`]
    /// or [`Limits::min_storage_buffer_offset_alignment`] appropriately.
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a BindGroup,
        offsets: &[DynamicOffset],
    ) {
        unimplemented!("ComputePass::set_bind_group is not implemented")
    }

    /// Sets the active compute pipeline.
    pub fn set_pipeline(&mut self, pipeline: &'a ComputePipeline) {
        unimplemented!("ComputePass::set_pipeline is not implemented")
    }

    /// Inserts debug marker.
    pub fn insert_debug_marker(&mut self, label: &str) {
        unimplemented!("ComputePass::insert_debug_marker is not implemented")
    }

    /// Start record commands and group it into debug marker group.
    pub fn push_debug_group(&mut self, label: &str) {
        unimplemented!("ComputePass::push_debug_group is not implemented")
    }

    /// Stops command recording and creates debug group.
    pub fn pop_debug_group(&mut self) {
        unimplemented!("ComputePass::pop_debug_group is not implemented")
    }

    /// Dispatches compute work operations.
    ///
    /// `x`, `y` and `z` denote the number of work groups to dispatch in each dimension.
    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        unimplemented!("ComputePass::dispatch_workgroups is not implemented")
    }

    /// Dispatches compute work operations, based on the contents of the `indirect_buffer`.
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DispatchIndirect`](crate::util::DispatchIndirect).
    pub fn dispatch_workgroups_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
    ) {
        unimplemented!("ComputePass::dispatch_workgroups_indirect is not implemented")
    }
}

/// [`Features::PUSH_CONSTANTS`] must be enabled on the device in order to call these functions.
impl<'a> ComputePass<'a> {
    /// Set push constant data for subsequent dispatch calls.
    ///
    /// Write the bytes in `data` at offset `offset` within push constant
    /// storage.  Both `offset` and the length of `data` must be
    /// multiples of [`PUSH_CONSTANT_ALIGNMENT`], which is always 4.
    ///
    /// For example, if `offset` is `4` and `data` is eight bytes long, this
    /// call will write `data` to bytes `4..12` of push constant storage.
    pub fn set_push_constants(&mut self, offset: u32, data: &[u8]) {
        unimplemented!("ComputePass::set_push_constants is not implemented")
    }
}

/// [`Features::WRITE_TIMESTAMP_INSIDE_PASSES`] must be enabled on the device in order to call these functions.
impl<'a> ComputePass<'a> {
    /// Issue a timestamp command at this point in the queue. The timestamp will be written to the specified query set, at the specified index.
    ///
    /// Must be multiplied by [`Queue::get_timestamp_period`] to get
    /// the value in nanoseconds. Absolute values have no meaning,
    /// but timestamps can be subtracted to get the time it takes
    /// for a string of operations to complete.
    pub fn write_timestamp(&mut self, query_set: &QuerySet, query_index: u32) {
        unimplemented!("ComputePass::write_timestamp is not implemented")
    }
}

/// [`Features::PIPELINE_STATISTICS_QUERY`] must be enabled on the device in order to call these functions.
impl<'a> ComputePass<'a> {
    /// Start a pipeline statistics query on this render pass. It can be ended with
    /// `end_pipeline_statistics_query`. Pipeline statistics queries may not be nested.
    pub fn begin_pipeline_statistics_query(&mut self, query_set: &QuerySet, query_index: u32) {
        unimplemented!("ComputePass::begin_pipeline_statistics_query is not implemented")
    }

    /// End the pipeline statistics query on this render pass. It can be started with
    /// `begin_pipeline_statistics_query`. Pipeline statistics queries may not be nested.
    pub fn end_pipeline_statistics_query(&mut self) {
        unimplemented!("ComputePass::end_pipeline_statistics_query is not implemented")
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
static_assertions::assert_impl_all!(RenderPassColorAttachment: Send, Sync);

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
static_assertions::assert_impl_all!(RenderPassDepthStencilAttachment: Send, Sync);

/// Encodes a series of GPU operations into a reusable "render bundle".
///
/// It only supports a handful of render commands, but it makes them reusable.
/// It can be created with [`Device::create_render_bundle_encoder`].
/// It can be executed onto a [`CommandEncoder`] using [`RenderPass::execute_bundles`].
///
/// Executing a [`RenderBundle`] is often more efficient than issuing the underlying commands
/// manually.
///
/// Corresponds to [WebGPU `GPURenderBundleEncoder`](
/// https://gpuweb.github.io/gpuweb/#gpurenderbundleencoder).
#[derive(Debug)]
pub struct RenderBundleEncoder<'a> {
    _data: PhantomData<&'a ()>,
}
// TODO static_assertions::assert_not_impl_any!(RenderBundleEncoder<'_>: Send, Sync);

impl<'a> RenderBundleEncoder<'a> {
    /// Finishes recording and returns a [`RenderBundle`] that can be executed in other render passes.
    pub fn finish(self, desc: &RenderBundleDescriptor) -> RenderBundle {
        unimplemented!("RenderBundleEncoder::finish is not implemented")
    }

    /// Sets the active bind group for a given bind group index. The bind group layout
    /// in the active pipeline when any `draw()` function is called must match the layout of this bind group.
    ///
    /// If the bind group have dynamic offsets, provide them in the binding order.
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a BindGroup,
        offsets: &[DynamicOffset],
    ) {
        unimplemented!("RenderBundleEncoder::set_bind_group is not implemented")
    }

    /// Sets the active render pipeline.
    ///
    /// Subsequent draw calls will exhibit the behavior defined by `pipeline`.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) {
        unimplemented!("RenderBundleEncoder::set_pipeline is not implemented")
    }

    /// Sets the active index buffer.
    ///
    /// Subsequent calls to [`draw_indexed`](RenderBundleEncoder::draw_indexed) on this [`RenderBundleEncoder`] will
    /// use `buffer` as the source index buffer.
    pub fn set_index_buffer(&mut self, buffer_slice: BufferSlice<'a>, index_format: IndexFormat) {
        unimplemented!("RenderBundleEncoder::set_index_buffer is not implemented")
    }

    /// Assign a vertex buffer to a slot.
    ///
    /// Subsequent calls to [`draw`] and [`draw_indexed`] on this
    /// [`RenderBundleEncoder`] will use `buffer` as one of the source vertex buffers.
    ///
    /// The `slot` refers to the index of the matching descriptor in
    /// [`VertexState::buffers`].
    ///
    /// [`draw`]: RenderBundleEncoder::draw
    /// [`draw_indexed`]: RenderBundleEncoder::draw_indexed
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: BufferSlice<'a>) {
        unimplemented!("RenderBundleEncoder::set_vertex_buffer is not implemented")
    }

    /// Draws primitives from the active vertex buffer(s).
    ///
    /// The active vertex buffers can be set with [`RenderBundleEncoder::set_vertex_buffer`].
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        unimplemented!("RenderBundleEncoder::draw is not implemented")
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers.
    ///
    /// The active index buffer can be set with [`RenderBundleEncoder::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderBundleEncoder::set_vertex_buffer`].
    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        unimplemented!("RenderBundleEncoder::draw_indexed is not implemented")
    }

    /// Draws primitives from the active vertex buffer(s) based on the contents of the `indirect_buffer`.
    ///
    /// The active vertex buffers can be set with [`RenderBundleEncoder::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndirect`](crate::util::DrawIndirect).
    pub fn draw_indirect(&mut self, indirect_buffer: &'a Buffer, indirect_offset: BufferAddress) {
        unimplemented!("RenderBundleEncoder::draw_indirect is not implemented")
    }

    /// Draws indexed primitives using the active index buffer and the active vertex buffers,
    /// based on the contents of the `indirect_buffer`.
    ///
    /// The active index buffer can be set with [`RenderBundleEncoder::set_index_buffer`], while the active
    /// vertex buffers can be set with [`RenderBundleEncoder::set_vertex_buffer`].
    ///
    /// The structure expected in `indirect_buffer` must conform to [`DrawIndexedIndirect`](crate::util::DrawIndexedIndirect).
    pub fn draw_indexed_indirect(
        &mut self,
        indirect_buffer: &'a Buffer,
        indirect_offset: BufferAddress,
    ) {
        unimplemented!("RenderBundleEncoder::draw_indexed_indirect is not implemented")
    }
}

/// [`Features::PUSH_CONSTANTS`] must be enabled on the device in order to call these functions.
impl<'a> RenderBundleEncoder<'a> {
    /// Set push constant data.
    ///
    /// Offset is measured in bytes, but must be a multiple of [`PUSH_CONSTANT_ALIGNMENT`].
    ///
    /// Data size must be a multiple of 4 and must have an alignment of 4.
    /// For example, with an offset of 4 and an array of `[u8; 8]`, that will write to the range
    /// of 4..12.
    ///
    /// For each byte in the range of push constant data written, the union of the stages of all push constant
    /// ranges that covers that byte must be exactly `stages`. There's no good way of explaining this simply,
    /// so here are some examples:
    ///
    /// ```text
    /// For the given ranges:
    /// - 0..4 Vertex
    /// - 4..8 Fragment
    /// ```
    ///
    /// You would need to upload this in two set_push_constants calls. First for the `Vertex` range, second for the `Fragment` range.
    ///
    /// ```text
    /// For the given ranges:
    /// - 0..8  Vertex
    /// - 4..12 Fragment
    /// ```
    ///
    /// You would need to upload this in three set_push_constants calls. First for the `Vertex` only range 0..4, second
    /// for the `Vertex | Fragment` range 4..8, third for the `Fragment` range 8..12.
    pub fn set_push_constants(&mut self, stages: ShaderStages, offset: u32, data: &[u8]) {
        unimplemented!("RenderBundleEncoder::set_push_constants is not implemented")
    }
}

/// Describes a [`RenderBundleEncoder`].
///
/// For use with [`Device::create_render_bundle_encoder`].
///
/// Corresponds to [WebGPU `GPURenderBundleEncoderDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpurenderbundleencoderdescriptor).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct RenderBundleEncoderDescriptor<'a> {
    /// Debug label of the render bundle encoder. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The formats of the color attachments that this render bundle is capable to rendering to. This
    /// must match the formats of the color attachments in the render pass this render bundle is executed in.
    pub color_formats: &'a [Option<TextureFormat>],
    /// Information about the depth attachment that this render bundle is capable to rendering to. This
    /// must match the format of the depth attachments in the render pass this render bundle is executed in.
    pub depth_stencil: Option<RenderBundleDepthStencil>,
    /// Sample count this render bundle is capable of rendering to. This must match the pipelines and
    /// the render passes it is used in.
    pub sample_count: u32,
    /// If this render bundle will rendering to multiple array layers in the attachments at the same time.
    pub multiview: Option<NonZeroU32>,
}
static_assertions::assert_impl_all!(RenderBundleEncoderDescriptor: Send, Sync);

/// Describes a [`CommandEncoder`].
///
/// For use with [`Device::create_command_encoder`].
///
/// Corresponds to [WebGPU `GPUCommandEncoderDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpucommandencoderdescriptor).
pub type CommandEncoderDescriptor<'a> = wgt::CommandEncoderDescriptor<Label<'a>>;
static_assertions::assert_impl_all!(CommandEncoderDescriptor: Send, Sync);

pub use wgt::ImageCopyBuffer as ImageCopyBufferBase;

/// View of a buffer which can be used to copy to/from a texture.
///
/// Corresponds to [WebGPU `GPUImageCopyBuffer`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuimagecopybuffer).
pub type ImageCopyBuffer<'a> = ImageCopyBufferBase<&'a Buffer>;
static_assertions::assert_impl_all!(ImageCopyBuffer: Send, Sync);
