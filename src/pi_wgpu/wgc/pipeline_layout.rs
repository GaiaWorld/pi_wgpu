use super::super::{BindGroupLayout, Label, PushConstantRange};
use derive_more::Debug;

/// Handle to a pipeline layout.
///
/// A `PipelineLayout` object describes the available binding groups of a pipeline.
/// It can be created with [`Device::create_pipeline_layout`].
///
/// Corresponds to [WebGPU `GPUPipelineLayout`](https://gpuweb.github.io/gpuweb/#gpupipelinelayout).
#[derive(Debug)]
pub struct PipelineLayout {
    pub(crate) inner: super::super::hal::PipelineLayout,
}

impl PipelineLayout {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::PipelineLayout) -> Self {
        Self { inner }
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
    #[debug("&[{}]", bind_group_layouts.iter().map(|r| {format!("&bind_group_layout{:?}, lable: {:?}", r.inner.id, r.inner.lable)}).collect::<Vec<String>>().join(", "))]  
    pub bind_group_layouts: &'a [&'a BindGroupLayout],
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    ///
    /// If this array is non-empty, the [`Features::PUSH_CONSTANTS`] must be enabled.
    #[debug("&{push_constant_ranges:?}")]
    pub push_constant_ranges: &'a [PushConstantRange],
}
