use crate::{BindGroupLayout, Label, PushConstantRange, wgpu_hal::{self as hal, Device}};

/// Handle to a pipeline layout.
///
/// A `PipelineLayout` object describes the available binding groups of a pipeline.
/// It can be created with [`Device::create_pipeline_layout`].
///
/// Corresponds to [WebGPU `GPUPipelineLayout`](https://gpuweb.github.io/gpuweb/#gpupipelinelayout).
#[derive(Debug)]
pub struct PipelineLayout {
    pub(crate) inner: Option<<hal::GL as hal::Api>::PipelineLayout>,

    pub(crate) device: crate::Device,
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        profiling::scope!("wgpu_core::PipelineLayout::drop");

        if let Some(pipeline_layout) = self.inner.take() {
            unsafe { self.device.inner.destroy_pipeline_layout(pipeline_layout) }
        }
    }
}

/// Describes a [`PipelineLayout`].
///
/// For use with [`Device::create_pipeline_layout`].
///
/// Corresponds to [WebGPU `GPUPipelineLayoutDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpupipelinelayoutdescriptor).
#[derive(Clone, Debug, Default)]
pub struct PipelineLayoutDescriptor<'a> {
    /// Debug label of the pipeline layout. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// Bind groups that this pipeline uses. The first entry will provide all the bindings for
    /// "set = 0", second entry will provide all the bindings for "set = 1" etc.
    pub bind_group_layouts: &'a [&'a BindGroupLayout],
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    ///
    /// If this array is non-empty, the [`Features::PUSH_CONSTANTS`] must be enabled.
    pub push_constant_ranges: &'a [PushConstantRange],
}
