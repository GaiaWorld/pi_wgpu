mod gl_impl;

mod adapter;
mod device;
mod instance;
mod surface;

mod command;
mod queue;

mod buffer;
mod sampler;
mod texture;

mod fence;
mod query_set;
mod shader_module;

mod bind_group;
mod bind_group_layout;
mod pipeline_layout;

mod compute_pipeline;
mod render_pipeline;

use adapter::Adapter;
use device::Device;
use instance::Instance;
use surface::Surface;

use command::{CommandBuffer, CommandEncoder};
use queue::Queue;

use buffer::Buffer;
use sampler::Sampler;
use texture::{Texture, TextureView};

use fence::Fence;
use query_set::QuerySet;
use shader_module::ShaderModule;

use bind_group::BindGroup;
use bind_group_layout::BindGroupLayout;
use pipeline_layout::PipelineLayout;

use compute_pipeline::ComputePipeline;
use render_pipeline::RenderPipeline;

pub(self) type DeviceResult<T> = Result<T, super::DeviceError>;

#[derive(Clone)]
pub struct Api;

impl super::Api for Api {
    type Instance = Instance;
    type Surface = Surface;
    type Adapter = Adapter;
    type Device = Device;

    type Queue = Queue;
    type CommandEncoder = CommandEncoder;
    type CommandBuffer = CommandBuffer;

    type Buffer = Buffer;
    type Texture = Texture;
    type SurfaceTexture = Texture;
    type TextureView = TextureView;
    type Sampler = Sampler;
    type QuerySet = QuerySet;
    type Fence = Fence;

    type BindGroupLayout = BindGroupLayout;
    type BindGroup = BindGroup;
    type PipelineLayout = PipelineLayout;
    type ShaderModule = ShaderModule;
    type RenderPipeline = RenderPipeline;
    type ComputePipeline = ComputePipeline;
}
