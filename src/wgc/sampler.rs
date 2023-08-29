use crate::{AddressMode, CompareFunction, FilterMode, Label, SamplerBorderColor};
use std::num::NonZeroU8;

/// Handle to a sampler.
///
/// A `Sampler` object defines how a pipeline will sample from a [`TextureView`]. Samplers define
/// image filters (including anisotropy) and address (wrapping) modes, among other things. See
/// the documentation for [`SamplerDescriptor`] for more information.
///
/// It can be created with [`Device::create_sampler`].
///
/// Corresponds to [WebGPU `GPUSampler`](https://gpuweb.github.io/gpuweb/#sampler-interface).
#[derive(Debug)]
pub struct Sampler {
    pub(crate) inner: crate::hal::Sampler,
}

impl Sampler {
    #[inline]
    pub(crate) fn from_hal(inner: crate::hal::Sampler) -> Self {
        Self { inner }
    }
}

/// Describes a [`Sampler`].
///
/// For use with [`Device::create_sampler`].
///
/// Corresponds to [WebGPU `GPUSamplerDescriptor`](
/// https://gpuweb.github.io/gpuweb/#dictdef-gpusamplerdescriptor).
#[derive(Clone, Debug, PartialEq)]
pub struct SamplerDescriptor<'a> {
    /// Debug label of the sampler. This will show up in graphics debuggers for easy identification.
    pub label: Label<'a>,
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    pub address_mode_u: AddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    pub address_mode_v: AddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    pub address_mode_w: AddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: FilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: FilterMode,
    /// How to filter between mip map levels
    pub mipmap_filter: FilterMode,
    /// Minimum level of detail (i.e. mip level) to use
    pub lod_min_clamp: f32,
    /// Maximum level of detail (i.e. mip level) to use
    pub lod_max_clamp: f32,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    pub compare: Option<CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    pub anisotropy_clamp: Option<NonZeroU8>,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    pub border_color: Option<SamplerBorderColor>,
}

impl Default for SamplerDescriptor<'_> {
    fn default() -> Self {
        Self {
            label: None,
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: std::f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        }
    }
}
