use crate::{BindGroupLayout, Label, PipelineLayout, ShaderModule};

/// Handle to a compute pipeline.
///
/// A `ComputePipeline` object represents a compute pipeline and its single shader stage.
/// It can be created with [`Device::create_compute_pipeline`].
///
/// Corresponds to [WebGPU `GPUComputePipeline`](https://gpuweb.github.io/gpuweb/#compute-pipeline).
#[derive(Debug)]
pub struct ComputePipeline;

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unimplemented!("wgc::ComputePipeline::drop is not implemented")
    }
}

impl ComputePipeline {
    /// Get an object representing the bind group layout at a given index.
    pub fn get_bind_group_layout(&self, _index: u32) -> BindGroupLayout {
        unimplemented!("wgc::ComputePipeline::get_bind_group_layout")
    }
}

/// Describes a compute pipeline.
///
/// For use with [`Device::create_compute_pipeline`].
///
/// Corresponds to [WebGPU `GPUComputePipelineDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpucomputepipelinedescriptor).
#[derive(Clone, Debug)]
pub struct ComputePipelineDescriptor<'a> {
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<&'a PipelineLayout>,
    /// The compiled shader module for this stage.
    pub module: &'a ShaderModule,
    /// The name of the entry point in the compiled shader. There must be a function with this name
    /// and no return value in the shader.
    pub entry_point: &'a str,
}

