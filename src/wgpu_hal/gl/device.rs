use crate::wgpu_hal::{api, types::*};

#[derive(Debug)]
pub struct Device {}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl api::Device<super::Api> for Device {
    unsafe fn exit(self, queue: super::Queue) {}

    unsafe fn create_buffer(&self, desc: &BufferDescriptor) -> super::DeviceResult<super::Buffer> {
        Ok(super::Buffer)
    }

    unsafe fn destroy_buffer(&self, buffer: super::Buffer) {}

    unsafe fn map_buffer(
        &self,
        buffer: &super::Buffer,
        range: MemoryRange,
    ) -> super::DeviceResult<BufferMapping> {
        Err(DeviceError::Lost)
    }

    unsafe fn unmap_buffer(&self, buffer: &super::Buffer) -> super::DeviceResult<()> {
        Ok(())
    }

    unsafe fn flush_mapped_ranges<I>(&self, buffer: &super::Buffer, ranges: I) {}

    unsafe fn invalidate_mapped_ranges<I>(&self, buffer: &super::Buffer, ranges: I) {}

    unsafe fn create_texture(
        &self,
        desc: &TextureDescriptor,
    ) -> super::DeviceResult<super::Texture> {
        Ok(super::Texture)
    }

    unsafe fn destroy_texture(&self, texture: super::Texture) {}

    unsafe fn create_texture_view(
        &self,
        texture: &super::Texture,
        desc: &TextureViewDescriptor,
    ) -> super::DeviceResult<super::TextureView> {
        Ok(super::TextureView)
    }

    unsafe fn destroy_texture_view(&self, view: super::TextureView) {}

    unsafe fn create_sampler(
        &self,
        desc: &SamplerDescriptor,
    ) -> super::DeviceResult<super::Sampler> {
        Ok(super::Sampler)
    }

    unsafe fn destroy_sampler(&self, sampler: super::Sampler) {}

    unsafe fn create_command_encoder(
        &self,
        desc: &CommandEncoderDescriptor<super::Api>,
    ) -> super::DeviceResult<super::CommandEncoder> {
        Ok(super::CommandEncoder)
    }

    unsafe fn destroy_command_encoder(&self, encoder: super::CommandEncoder) {}

    unsafe fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> super::DeviceResult<super::BindGroupLayout> {
        Ok(super::BindGroupLayout)
    }

    unsafe fn destroy_bind_group_layout(&self, bg_layout: super::BindGroupLayout) {}

    unsafe fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDescriptor<super::Api>,
    ) -> super::DeviceResult<super::PipelineLayout> {
        Ok(super::PipelineLayout)
    }

    unsafe fn destroy_pipeline_layout(&self, pipeline_layout: super::PipelineLayout) {}

    unsafe fn create_bind_group(
        &self,
        desc: &BindGroupDescriptor<super::Api>,
    ) -> super::DeviceResult<super::BindGroup> {
        Ok(super::BindGroup)
    }

    unsafe fn destroy_bind_group(&self, group: super::BindGroup) {}

    unsafe fn create_shader_module(
        &self,
        desc: &ShaderModuleDescriptor,
        shader: ShaderInput,
    ) -> Result<super::ShaderModule, ShaderError> {
        Ok(super::ShaderModule)
    }

    unsafe fn destroy_shader_module(&self, module: super::ShaderModule) {}

    unsafe fn create_render_pipeline(
        &self,
        desc: &RenderPipelineDescriptor<super::Api>,
    ) -> Result<super::RenderPipeline, PipelineError> {
        Ok(super::RenderPipeline)
    }

    unsafe fn destroy_render_pipeline(&self, pipeline: super::RenderPipeline) {}

    unsafe fn create_compute_pipeline(
        &self,
        desc: &ComputePipelineDescriptor<super::Api>,
    ) -> Result<super::ComputePipeline, PipelineError> {
        Ok(super::ComputePipeline)
    }

    unsafe fn destroy_compute_pipeline(&self, pipeline: super::ComputePipeline) {}

    unsafe fn create_query_set(
        &self,
        desc: &crate::wgpu_types::QuerySetDescriptor<Label>,
    ) -> super::DeviceResult<super::QuerySet> {
        Ok(super::QuerySet)
    }

    unsafe fn destroy_query_set(&self, set: super::QuerySet) {}

    unsafe fn create_fence(&self) -> super::DeviceResult<super::Fence> {
        Ok(super::Fence)
    }

    unsafe fn destroy_fence(&self, fence: super::Fence) {}

    unsafe fn get_fence_value(&self, fence: &super::Fence) -> super::DeviceResult<FenceValue> {
        Ok(0)
    }

    unsafe fn wait(
        &self,
        fence: &super::Fence,
        value: FenceValue,
        timeout_ms: u32,
    ) -> super::DeviceResult<bool> {
        Ok(true)
    }

    unsafe fn start_capture(&self) -> bool {
        false
    }

    unsafe fn stop_capture(&self) {}
}
