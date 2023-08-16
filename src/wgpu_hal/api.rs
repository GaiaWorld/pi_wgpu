use std::{borrow::Borrow, fmt, ops::Range};

use super::types::*;

pub use super::empty::Api as Empty;

#[cfg(feature = "gl")]
pub use super::gl::Api as GL;

pub trait Api: Clone + Sized {
    type Instance: Instance<Self>;
    type Surface: Surface<Self>;
    type Adapter: Adapter<Self>;
    type Device: Device<Self>;

    type Queue: Queue<Self>;
    type CommandEncoder: CommandEncoder<Self>;
    type CommandBuffer: Send + Sync + fmt::Debug;

    type Buffer: fmt::Debug + Send + Sync + 'static;
    type Texture: fmt::Debug + Send + Sync + 'static;
    type SurfaceTexture: fmt::Debug + Send + Sync + Borrow<Self::Texture>;
    type TextureView: fmt::Debug + Send + Sync;
    type Sampler: fmt::Debug + Send + Sync;
    type QuerySet: fmt::Debug + Send + Sync;
    type Fence: fmt::Debug + Send + Sync;

    type BindGroupLayout: Send + Sync;
    type BindGroup: fmt::Debug + Send + Sync;
    type PipelineLayout: Send + Sync;
    type ShaderModule: fmt::Debug + Send + Sync;
    type RenderPipeline: Send + Sync;
    type ComputePipeline: Send + Sync;
}

pub trait Instance<A: Api>: Sized + Send + Sync {
    unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError>;

    unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<A::Surface, InstanceError>;

    unsafe fn destroy_surface(&self, surface: A::Surface);

    unsafe fn enumerate_adapters(&self) -> Vec<ExposedAdapter<A>>;
}

pub trait Surface<A: Api>: Send + Sync {
    unsafe fn configure(
        &mut self,
        device: &A::Device,
        config: &SurfaceConfiguration,
    ) -> Result<(), SurfaceError>;

    unsafe fn unconfigure(&mut self, device: &A::Device);

    /// Returns the next texture to be presented by the swapchain for drawing
    ///
    /// A `timeout` of `None` means to wait indefinitely, with no timeout.
    ///
    /// # Portability
    ///
    /// Some backends can't support a timeout when acquiring a texture and
    /// the timeout will be ignored.
    ///
    /// Returns `None` on timing out.
    unsafe fn acquire_texture(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<Option<AcquiredSurfaceTexture<A>>, SurfaceError>;

    unsafe fn discard_texture(&mut self, texture: A::SurfaceTexture);
}

pub trait Adapter<A: Api>: Send + Sync {
    unsafe fn open(
        &self,
        features: crate::wgpu_types::Features,
        limits: &crate::wgpu_types::Limits,
    ) -> Result<OpenDevice<A>, DeviceError>;

    /// Return the set of supported capabilities for a texture format.
    unsafe fn texture_format_capabilities(
        &self,
        format: crate::wgpu_types::TextureFormat,
    ) -> TextureFormatCapabilities;

    /// Returns the capabilities of working with a specified surface.
    ///
    /// `None` means presentation is not supported for it.
    unsafe fn surface_capabilities(&self, surface: &A::Surface) -> Option<SurfaceCapabilities>;

    /// Creates a [`PresentationTimestamp`] using the adapter's WSI.
    ///
    /// [`PresentationTimestamp`]: crate::wgpu_types::PresentationTimestamp
    unsafe fn get_presentation_timestamp(&self) -> crate::wgpu_types::PresentationTimestamp;
}

pub trait Device<A: Api>: Send + Sync {
    /// Exit connection to this logical device.
    unsafe fn exit(self, queue: A::Queue);

    /// Creates a new buffer.
    ///
    /// The initial usage is `BufferUses::empty()`.
    unsafe fn create_buffer(&self, desc: &BufferDescriptor) -> Result<A::Buffer, DeviceError>;

    unsafe fn destroy_buffer(&self, buffer: A::Buffer);

    //TODO: clarify if zero-sized mapping is allowed
    unsafe fn map_buffer(
        &self,
        buffer: &A::Buffer,
        range: MemoryRange,
    ) -> Result<BufferMapping, DeviceError>;

    unsafe fn unmap_buffer(&self, buffer: &A::Buffer) -> Result<(), DeviceError>;

    unsafe fn flush_mapped_ranges<I>(&self, buffer: &A::Buffer, ranges: I)
    where
        I: Iterator<Item = MemoryRange>;

    unsafe fn invalidate_mapped_ranges<I>(&self, buffer: &A::Buffer, ranges: I)
    where
        I: Iterator<Item = MemoryRange>;

    /// Creates a new texture.
    ///
    /// The initial usage for all subresources is `TextureUses::UNINITIALIZED`.
    unsafe fn create_texture(&self, desc: &TextureDescriptor) -> Result<A::Texture, DeviceError>;

    unsafe fn destroy_texture(&self, texture: A::Texture);

    unsafe fn create_texture_view(
        &self,
        texture: &A::Texture,
        desc: &TextureViewDescriptor,
    ) -> Result<A::TextureView, DeviceError>;

    unsafe fn destroy_texture_view(&self, view: A::TextureView);

    unsafe fn create_sampler(&self, desc: &SamplerDescriptor) -> Result<A::Sampler, DeviceError>;

    unsafe fn destroy_sampler(&self, sampler: A::Sampler);

    unsafe fn create_command_encoder(
        &self,
        desc: &CommandEncoderDescriptor<A>,
    ) -> Result<A::CommandEncoder, DeviceError>;

    unsafe fn destroy_command_encoder(&self, pool: A::CommandEncoder);

    /// Creates a bind group layout.
    unsafe fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> Result<A::BindGroupLayout, DeviceError>;

    unsafe fn destroy_bind_group_layout(&self, bg_layout: A::BindGroupLayout);

    unsafe fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDescriptor<A>,
    ) -> Result<A::PipelineLayout, DeviceError>;

    unsafe fn destroy_pipeline_layout(&self, pipeline_layout: A::PipelineLayout);

    unsafe fn create_bind_group(
        &self,
        desc: &BindGroupDescriptor<A>,
    ) -> Result<A::BindGroup, DeviceError>;

    unsafe fn destroy_bind_group(&self, group: A::BindGroup);

    unsafe fn create_shader_module(
        &self,
        desc: &ShaderModuleDescriptor,
        shader: ShaderInput,
    ) -> Result<A::ShaderModule, ShaderError>;

    unsafe fn destroy_shader_module(&self, module: A::ShaderModule);

    unsafe fn create_render_pipeline(
        &self,
        desc: &RenderPipelineDescriptor<A>,
    ) -> Result<A::RenderPipeline, PipelineError>;

    unsafe fn destroy_render_pipeline(&self, pipeline: A::RenderPipeline);

    unsafe fn create_compute_pipeline(
        &self,
        desc: &ComputePipelineDescriptor<A>,
    ) -> Result<A::ComputePipeline, PipelineError>;

    unsafe fn destroy_compute_pipeline(&self, pipeline: A::ComputePipeline);

    unsafe fn create_query_set(
        &self,
        desc: &crate::wgpu_types::QuerySetDescriptor<Label>,
    ) -> Result<A::QuerySet, DeviceError>;

    unsafe fn destroy_query_set(&self, set: A::QuerySet);

    unsafe fn create_fence(&self) -> Result<A::Fence, DeviceError>;

    unsafe fn destroy_fence(&self, fence: A::Fence);

    unsafe fn get_fence_value(&self, fence: &A::Fence) -> Result<FenceValue, DeviceError>;

    /// Calling wait with a lower value than the current fence value will immediately return.
    unsafe fn wait(
        &self,
        fence: &A::Fence,
        value: FenceValue,
        timeout_ms: u32,
    ) -> Result<bool, DeviceError>;

    unsafe fn start_capture(&self) -> bool;

    unsafe fn stop_capture(&self);
}

pub trait Queue<A: Api>: Send + Sync {
    /// Submits the command buffers for execution on GPU.
    ///
    /// Valid usage:
    /// - all of the command buffers were created from command pools
    ///   that are associated with this queue.
    /// - all of the command buffers had `CommadBuffer::finish()` called.
    unsafe fn submit(
        &mut self,
        command_buffers: &[&A::CommandBuffer],
        signal_fence: Option<(&mut A::Fence, FenceValue)>,
    ) -> Result<(), DeviceError>;

    unsafe fn present(
        &mut self,
        surface: &mut A::Surface,
        texture: A::SurfaceTexture,
    ) -> Result<(), SurfaceError>;

    unsafe fn get_timestamp_period(&self) -> f32;
}

/// Encoder for commands in command buffers.
/// Serves as a parent for all the encoded command buffers.
/// Works in bursts of action: one or more command buffers are recorded,
/// then submitted to a queue, and then it needs to be `reset_all()`.
pub trait CommandEncoder<A: Api>: Send + Sync + fmt::Debug {
    /// Begin encoding a new command buffer.
    unsafe fn begin_encoding(&mut self, label: Label) -> Result<(), DeviceError>;

    /// Discard currently recorded list, if any.
    unsafe fn discard_encoding(&mut self);

    unsafe fn end_encoding(&mut self) -> Result<A::CommandBuffer, DeviceError>;

    /// Reclaims all resources that are allocated for this encoder.
    /// Must get all of the produced command buffers back,
    /// and they must not be used by GPU at this moment.
    unsafe fn reset_all<I>(&mut self, command_buffers: I)
    where
        I: Iterator<Item = A::CommandBuffer>;

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = BufferBarrier<'a, A>>;

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = TextureBarrier<'a, A>>;

    // copy operations

    unsafe fn clear_buffer(&mut self, buffer: &A::Buffer, range: MemoryRange);

    unsafe fn copy_buffer_to_buffer<T>(&mut self, src: &A::Buffer, dst: &A::Buffer, regions: T)
    where
        T: Iterator<Item = BufferCopy>;

    /// Copy from an external image to an internal texture.
    /// Works with a single array layer.
    /// Note: `dst` current usage has to be `TextureUses::COPY_DST`.
    /// Note: the copy extent is in physical size (rounded to the block size)
    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    unsafe fn copy_external_image_to_texture<T>(
        &mut self,
        src: &crate::wgpu_types::ImageCopyExternalImage,
        dst: &A::Texture,
        dst_premultiplication: bool,
        regions: T,
    ) where
        T: Iterator<Item = TextureCopy>;

    /// Copy from one texture to another.
    /// Works with a single array layer.
    /// Note: `dst` current usage has to be `TextureUses::COPY_DST`.
    /// Note: the copy extent is in physical size (rounded to the block size)
    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &A::Texture,
        src_usage: TextureUses,
        dst: &A::Texture,
        regions: T,
    ) where
        T: Iterator<Item = TextureCopy>;

    /// Copy from buffer to texture.
    /// Works with a single array layer.
    /// Note: `dst` current usage has to be `TextureUses::COPY_DST`.
    /// Note: the copy extent is in physical size (rounded to the block size)
    unsafe fn copy_buffer_to_texture<T>(&mut self, src: &A::Buffer, dst: &A::Texture, regions: T)
    where
        T: Iterator<Item = BufferTextureCopy>;

    /// Copy from texture to buffer.
    /// Works with a single array layer.
    /// Note: the copy extent is in physical size (rounded to the block size)
    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &A::Texture,
        src_usage: TextureUses,
        dst: &A::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = BufferTextureCopy>;

    // pass common

    /// Sets the bind group at `index` to `group`, assuming the layout
    /// of all the preceeding groups to be taken from `layout`.
    unsafe fn set_bind_group(
        &mut self,
        layout: &A::PipelineLayout,
        index: u32,
        group: &A::BindGroup,
        dynamic_offsets: &[crate::wgpu_types::DynamicOffset],
    );

    unsafe fn set_push_constants(
        &mut self,
        layout: &A::PipelineLayout,
        stages: crate::wgpu_types::ShaderStages,
        offset: u32,
        data: &[u32],
    );

    unsafe fn insert_debug_marker(&mut self, label: &str);

    unsafe fn begin_debug_marker(&mut self, group_label: &str);

    unsafe fn end_debug_marker(&mut self);

    // queries

    unsafe fn begin_query(&mut self, set: &A::QuerySet, index: u32);

    unsafe fn end_query(&mut self, set: &A::QuerySet, index: u32);

    unsafe fn write_timestamp(&mut self, set: &A::QuerySet, index: u32);

    unsafe fn reset_queries(&mut self, set: &A::QuerySet, range: Range<u32>);

    unsafe fn copy_query_results(
        &mut self,
        set: &A::QuerySet,
        range: Range<u32>,
        buffer: &A::Buffer,
        offset: crate::wgpu_types::BufferAddress,
        stride: crate::wgpu_types::BufferSize,
    );

    // render passes

    // Begins a render pass, clears all active bindings.
    unsafe fn begin_render_pass(&mut self, desc: &RenderPassDescriptor<A>);

    unsafe fn end_render_pass(&mut self);

    unsafe fn set_render_pipeline(&mut self, pipeline: &A::RenderPipeline);

    unsafe fn set_index_buffer<'a>(
        &mut self,
        binding: BufferBinding<'a, A>,
        format: crate::wgpu_types::IndexFormat,
    );

    unsafe fn set_vertex_buffer<'a>(&mut self, index: u32, binding: BufferBinding<'a, A>);

    unsafe fn set_viewport(&mut self, rect: &Rect<f32>, depth_range: Range<f32>);

    unsafe fn set_scissor_rect(&mut self, rect: &Rect<u32>);

    unsafe fn set_stencil_reference(&mut self, value: u32);

    unsafe fn set_blend_constants(&mut self, color: &[f32; 4]);

    unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    );

    unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    );

    unsafe fn draw_indirect(
        &mut self,
        buffer: &A::Buffer,
        offset: crate::wgpu_types::BufferAddress,
        draw_count: u32,
    );

    unsafe fn draw_indexed_indirect(
        &mut self,
        buffer: &A::Buffer,
        offset: crate::wgpu_types::BufferAddress,
        draw_count: u32,
    );

    unsafe fn draw_indirect_count(
        &mut self,
        buffer: &A::Buffer,
        offset: crate::wgpu_types::BufferAddress,
        count_buffer: &A::Buffer,
        count_offset: crate::wgpu_types::BufferAddress,
        max_count: u32,
    );

    unsafe fn draw_indexed_indirect_count(
        &mut self,
        buffer: &A::Buffer,
        offset: crate::wgpu_types::BufferAddress,
        count_buffer: &A::Buffer,
        count_offset: crate::wgpu_types::BufferAddress,
        max_count: u32,
    );

    // compute passes

    // Begins a compute pass, clears all active bindings.
    unsafe fn begin_compute_pass(&mut self, desc: &ComputePassDescriptor);

    unsafe fn end_compute_pass(&mut self);

    unsafe fn set_compute_pipeline(&mut self, pipeline: &A::ComputePipeline);

    unsafe fn dispatch(&mut self, count: [u32; 3]);

    unsafe fn dispatch_indirect(&mut self, buffer: &A::Buffer, offset: crate::wgpu_types::BufferAddress);
}
