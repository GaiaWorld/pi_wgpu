use super::super::{BindGroupLayout, BindingResource, Label};

/// Handle to a binding group.
///
/// A `BindGroup` represents the set of resources bound to the bindings described by a
/// [`BindGroupLayout`]. It can be created with [`Device::create_bind_group`]. A `BindGroup` can
/// be bound to a particular [`RenderPass`] with [`RenderPass::set_bind_group`]
///
/// Corresponds to [WebGPU `GPUBindGroup`](https://gpuweb.github.io/gpuweb/#gpubindgroup).
#[derive(Debug)]
pub struct BindGroup {
    pub(crate) inner: super::super::hal::BindGroup,
}

impl BindGroup {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::BindGroup) -> Self {
        Self { inner }
    }
}

/// Describes a group of bindings and the resources to be bound.
///
/// For use with [`Device::create_bind_group`].
///
/// Corresponds to [WebGPU `GPUBindGroupDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubindgroupdescriptor).
#[derive(Clone, Debug)]
pub struct BindGroupDescriptor<'a> {
    /// Debug label of the bind group. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// The [`BindGroupLayout`] that corresponds to this bind group.
    pub layout: &'a BindGroupLayout,
    /// The resources to bind to this bind group.
    pub entries: &'a [BindGroupEntry<'a>],
}

/// An element of a [`BindGroupDescriptor`], consisting of a bindable resource
/// and the slot to bind it to.
///
/// Corresponds to [WebGPU `GPUBindGroupEntry`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubindgroupentry).
#[derive(Clone, Debug)]
pub struct BindGroupEntry<'a> {
    /// Slot for which binding provides resource. Corresponds to an entry of the same
    /// binding index in the [`BindGroupLayoutDescriptor`].
    pub binding: u32,
    /// Resource to attach to the binding
    pub resource: BindingResource<'a>,
}
