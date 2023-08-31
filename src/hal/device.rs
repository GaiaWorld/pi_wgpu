#[derive(Debug)]
pub(crate) struct Device {
    pub(crate) state: super::GLState,
}

impl Device {
    #[inline]
    pub(crate) unsafe fn create_buffer(
        &self,
        desc: &crate::BufferDescriptor,
    ) -> Result<super::Buffer, crate::DeviceError> {
        super::Buffer::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_texture(
        &self,
        desc: &crate::TextureDescriptor,
    ) -> Result<super::Texture, crate::DeviceError> {
        super::Texture::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_texture_view(
        &self,
        texture: &super::Texture,
        desc: &crate::TextureViewDescriptor,
    ) -> Result<super::TextureView, crate::DeviceError> {
        super::TextureView::new(texture, desc)
    }

    #[inline]
    pub(crate) unsafe fn create_sampler(
        &self,
        desc: &crate::SamplerDescriptor,
    ) -> Result<super::Sampler, crate::DeviceError> {
        super::Sampler::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_command_encoder(
        &self,
        desc: &crate::CommandEncoderDescriptor,
    ) -> Result<super::CommandEncoder, crate::DeviceError> {
        super::CommandEncoder::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_bind_group_layout(
        &self,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> Result<super::BindGroupLayout, crate::DeviceError> {
        super::BindGroupLayout::new(desc)
    }

    #[inline]
    pub(crate) unsafe fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> Result<super::PipelineLayout, crate::DeviceError> {
        super::PipelineLayout::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor,
    ) -> Result<super::BindGroup, crate::DeviceError> {
        super::BindGroup::new(desc)
    }

    #[inline]
    pub(crate) unsafe fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDescriptor,
    ) -> Result<super::ShaderModule, super::ShaderError> {
        super::ShaderModule::new(self.state.clone(), desc)
    }

    #[inline]
    pub(crate) unsafe fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> Result<super::RenderPipeline, super::PipelineError> {
        super::RenderPipeline::new(self.state.clone(), desc)
    }
}
