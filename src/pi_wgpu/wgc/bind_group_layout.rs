use super::super::{
    BindGroupLayoutEntry, BufferBinding, Label, Sampler, TextureView,
};
use derive_more::Debug;

/// Resource that can be bound to a pipeline.
///
/// Corresponds to [WebGPU `GPUBindingResource`](
/// https://gpuweb.github.io/gpuweb/#typedefdef-gpubindingresource).
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum BindingResource<'a> {
    /// Binding is backed by a buffer.
    ///
    /// Corresponds to [`super::super::wgt::BufferBindingType::Uniform`] and [`super::super::wgt::BufferBindingType::Storage`]
    /// with [`BindGroupLayoutEntry::count`] set to None.
    #[debug("BindingResource::Buffer({:?})", _0)]
    Buffer(BufferBinding<'a>),
    /// Binding is backed by an array of buffers.
    ///
    /// [`Features::BUFFER_BINDING_ARRAY`] must be supported to use this feature.
    ///
    /// Corresponds to [`super::super::wgt::BufferBindingType::Uniform`] and [`super::super::wgt::BufferBindingType::Storage`]
    /// with [`BindGroupLayoutEntry::count`] set to Some.
    BufferArray(&'a [BufferBinding<'a>]),
    /// Binding is a sampler.
    ///
    /// Corresponds to [`super::super::wgt::BindingType::Sampler`] with [`BindGroupLayoutEntry::count`] set to None.
    #[debug("BindingResource::Sampler(&sampler{:?})", _0.inner.0.raw)]
    Sampler(&'a Sampler),
    /// Binding is backed by an array of samplers.
    ///
    /// [`Features::TEXTURE_BINDING_ARRAY`] must be supported to use this feature.
    ///
    /// Corresponds to [`super::super::wgt::BindingType::Sampler`] with [`BindGroupLayoutEntry::count`] set
    /// to Some.
    SamplerArray(&'a [&'a Sampler]),
    /// Binding is backed by a texture.
    ///
    /// Corresponds to [`super::super::wgt::BindingType::Texture`] and [`super::super::wgt::BindingType::StorageTexture`] with
    /// [`BindGroupLayoutEntry::count`] set to None.
    #[debug("BindingResource::TextureView(&texture_view{:?})", _0.inner.id)]
    TextureView(&'a TextureView),
    /// Binding is backed by an array of textures.
    ///
    /// [`Features::TEXTURE_BINDING_ARRAY`] must be supported to use this feature.
    ///
    /// Corresponds to [`super::super::wgt::BindingType::Texture`] and [`super::super::wgt::BindingType::StorageTexture`] with
    /// [`BindGroupLayoutEntry::count`] set to Some.
    TextureViewArray(&'a [&'a TextureView]),
}

/// Handle to a binding group layout.
///
/// A `BindGroupLayout` is a handle to the GPU-side layout of a binding group. It can be used to
/// create a [`BindGroupDescriptor`] object, which in turn can be used to create a [`BindGroup`]
/// object with [`Device::create_bind_group`]. A series of `BindGroupLayout`s can also be used to
/// create a [`PipelineLayoutDescriptor`], which can be used to create a [`PipelineLayout`].
///
/// It can be created with [`Device::create_bind_group_layout`].
///
/// Corresponds to [WebGPU `GPUBindGroupLayout`](
/// https://gpuweb.github.io/gpuweb/#gpubindgrouplayout).
#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) inner: super::super::hal::BindGroupLayout,
}

impl BindGroupLayout {
    #[inline]
    pub(crate) fn from_hal(inner: super::super::hal::BindGroupLayout) -> Self {
        Self { inner }
    }
}

/// Describes a [`BindGroupLayout`].
///
/// For use with [`Device::create_bind_group_layout`].
///
/// Corresponds to [WebGPU `GPUBindGroupLayoutDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpubindgrouplayoutdescriptor).
#[derive(Clone, Debug)]
pub struct BindGroupLayoutDescriptor<'a> {
    /// Debug label of the bind group layout. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,

    /// Array of entries in this BindGroupLayout
    #[debug("&{entries:?}")]
    pub entries: &'a [BindGroupLayoutEntry],
}
