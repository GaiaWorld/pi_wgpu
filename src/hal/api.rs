use crate::hal;

pub trait Api: Clone + Sized {
    type Instance;
    type Surface;
    type Adapter;
    type Device;
    type Queue;
    type CommandEncoder;

    type CommandBuffer;
    type Buffer;
    type Texture;
    type SurfaceTexture;
    type TextureView;
    type Sampler;
    type BindGroupLayout;
    type BindGroup;
    type PipelineLayout;
    type ShaderModule;
    type RenderPipeline;
}

pub trait HalApi: Api {}

#[derive(Debug, Clone)]
pub(crate) struct GL;

impl Api for GL {
    type Instance = hal::Instance;
    type Surface = hal::Surface;
    type Adapter = hal::Adapter;
    type Device = hal::Device;
    type Queue = hal::Queue;
    type CommandEncoder = hal::CommandEncoder;
    
    type CommandBuffer = hal::CommandBuffer;
    type Buffer = hal::Buffer;
    type Texture = hal::Texture;
    type SurfaceTexture = hal::Texture;
    type TextureView = hal::TextureView;
    type Sampler = hal::Sampler;
    type BindGroupLayout = hal::BindGroupLayout;
    type BindGroup = hal::BindGroup;
    type PipelineLayout = hal::PipelineLayout;
    type ShaderModule = hal::ShaderModule;
    type RenderPipeline = hal::RenderPipeline;
}

impl HalApi for GL {}
