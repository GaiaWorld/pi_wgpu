use crate::{wgpu_hal as hal, Label};

/// Handle to a query set.
///
/// It can be created with [`Device::create_query_set`].
///
/// Corresponds to [WebGPU `GPUQuerySet`](https://gpuweb.github.io/gpuweb/#queryset).
pub struct QuerySet {
    inner: <hal::GL as hal::Api>::QuerySet,
}

impl Drop for QuerySet {
    fn drop(&mut self) {
        unimplemented!("QuerySet::drop")
    }
}

/// Describes a [`QuerySet`].
///
/// For use with [`Device::create_query_set`].
///
/// Corresponds to [WebGPU `GPUQuerySetDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpuquerysetdescriptor).
pub type QuerySetDescriptor<'a> = crate::wgpu_types::QuerySetDescriptor<Label<'a>>;


