use pi_share::Share;

use super::{super::wgt, AdapterContext};

#[derive(Debug)]
pub(crate) struct Device {
    pub(crate) adapter: AdapterContext,
    pub(crate) state: super::GLState,

    pub(crate) features: wgt::Features,
    pub(crate) limits: wgt::Limits,
    pub(crate) downlevel: wgt::DownlevelCapabilities,
}

impl Device {
    #[inline]
    pub(crate) fn create_buffer(
        &self,
        desc: &super::super::BufferDescriptor,
    ) -> Result<super::Buffer, super::super::DeviceError> {
        super::Buffer::new(self.state.clone(), &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_texture(
        &self,
        desc: &super::super::TextureDescriptor,
    ) -> Result<super::Texture, super::super::DeviceError> {
        super::Texture::new(self.state.clone(), &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_sampler(
        &self,
        desc: &super::super::SamplerDescriptor,
    ) -> Result<super::Sampler, super::super::DeviceError> {
        super::Sampler::new(self.state.clone(), &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_command_encoder(
        &self,
        desc: &super::super::CommandEncoderDescriptor,
    ) -> Result<super::CommandEncoder, super::super::DeviceError> {
        super::CommandEncoder::new(self.state.clone(), &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_bind_group_layout(
        &self,
        desc: &super::super::BindGroupLayoutDescriptor,
    ) -> Result<super::BindGroupLayout, super::super::DeviceError> {
        super::BindGroupLayout::new(desc)
    }

    #[inline]
    pub(crate) fn create_pipeline_layout(
        &self,
        desc: &super::super::PipelineLayoutDescriptor,
    ) -> Result<super::PipelineLayout, super::super::DeviceError> {
        super::PipelineLayout::new(&self.features, &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_bind_group(
        &self,
        desc: &super::super::BindGroupDescriptor,
    ) -> Result<super::BindGroup, super::super::DeviceError> {
        log::info!("=========== + device.create_bind_group");
        super::BindGroup::new(desc)
    }

    #[inline]
    pub(crate) fn create_shader_module(
        &self,
        desc: &super::super::ShaderModuleDescriptor,
    ) -> Result<super::ShaderModule, super::ShaderError> {
        log::info!("=========== + device.create_shader_module");
        super::ShaderModule::new(self.state.clone(), &self.adapter, desc)
    }

    #[inline]
    pub(crate) fn create_render_pipeline(
        &self,
        desc: &super::super::RenderPipelineDescriptor,
    ) -> Result<super::RenderPipeline, super::PipelineError> {
        log::info!("=========== + device.create_pipeline");
        let imp = super::RenderPipelineImpl::new(&self.state, &self.adapter, &self.features, desc)?;
        Ok(super::RenderPipeline(Share::new(imp)))
    }
}
