use glow::HasContext;
use pi_share::Share;

use crate::{AddressMode, CompareFunction, FilterMode, SamplerDescriptor};

use super::{gl_conv as conv, AdapterContext, GLState};

#[derive(Debug, Clone)]
pub(crate) struct Sampler(pub(crate) Share<SamplerImpl>);

impl Sampler {
    pub fn new(
        state: GLState,
        adapter: &AdapterContext,
        desc: &super::super::SamplerDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        let lock = adapter.lock(None);
        let gl = lock.get_glow();

        let raw = unsafe { gl.create_sampler().unwrap() };

        let (min, mag) =
            conv::map_filter_modes(desc.min_filter, desc.mag_filter, desc.mipmap_filter, true);

        unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MIN_FILTER, min as i32) };
        unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MAG_FILTER, mag as i32) };

        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_S,
                conv::map_address_mode(desc.address_mode_u) as i32,
            )
        };
        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_T,
                conv::map_address_mode(desc.address_mode_v) as i32,
            )
        };
        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_R,
                conv::map_address_mode(desc.address_mode_w) as i32,
            )
        };

        unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MIN_LOD, desc.lod_min_clamp) };
        unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MAX_LOD, desc.lod_max_clamp) };

        if let Some(compare) = desc.compare {
            unsafe {
                gl.sampler_parameter_i32(
                    raw,
                    glow::TEXTURE_COMPARE_MODE,
                    glow::COMPARE_REF_TO_TEXTURE as i32,
                )
            };
            unsafe {
                gl.sampler_parameter_i32(
                    raw,
                    glow::TEXTURE_COMPARE_FUNC,
                    conv::map_compare_func(compare) as i32,
                )
            };
        }

        let imp = SamplerImpl {
            raw,
            state: state,
            adapter: adapter.clone(),
            desc: Box::new(SamplerDescriptorInner::from(desc)),
        };
        Ok(Self(Share::new(imp)))
    }

    // pub fn new(
    //     state: GLState,
    //     adapter: &AdapterContext,
    //     desc: &super::super::SamplerDescriptor,
    // ) -> Result<Self, super::super::DeviceError> {
    //     let lock = adapter.lock(None);
    //     let gl = lock.get_glow();

    //     let raw = unsafe { gl.create_sampler().unwrap() };

    //     let (min, mag) =
    //         conv::map_filter_modes(desc.min_filter, desc.mag_filter, desc.mipmap_filter);

    //     unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MIN_FILTER, min as i32) };
    //     unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MAG_FILTER, mag as i32) };

    //     unsafe {
    //         gl.sampler_parameter_i32(
    //             raw,
    //             glow::TEXTURE_WRAP_S,
    //             conv::map_address_mode(desc.address_mode_u) as i32,
    //         )
    //     };
    //     unsafe {
    //         gl.sampler_parameter_i32(
    //             raw,
    //             glow::TEXTURE_WRAP_T,
    //             conv::map_address_mode(desc.address_mode_v) as i32,
    //         )
    //     };
    //     unsafe {
    //         gl.sampler_parameter_i32(
    //             raw,
    //             glow::TEXTURE_WRAP_R,
    //             conv::map_address_mode(desc.address_mode_w) as i32,
    //         )
    //     };

    //     unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MIN_LOD, desc.lod_min_clamp) };
    //     unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MAX_LOD, desc.lod_max_clamp) };

    //     if let Some(compare) = desc.compare {
    //         unsafe {
    //             gl.sampler_parameter_i32(
    //                 raw,
    //                 glow::TEXTURE_COMPARE_MODE,
    //                 glow::COMPARE_REF_TO_TEXTURE as i32,
    //             )
    //         };
    //         unsafe {
    //             gl.sampler_parameter_i32(
    //                 raw,
    //                 glow::TEXTURE_COMPARE_FUNC,
    //                 conv::map_compare_func(compare) as i32,
    //             )
    //         };
    //     }

    //     let imp = SamplerImpl {
    //         raw,
    //         state: state,
    //         adapter: adapter.clone(),
    //     };
    //     Ok(Self(Share::new(imp)))
    // }
}

#[derive(Debug)]
pub(crate) struct SamplerImpl {
    pub(crate) raw: glow::Sampler,
    pub(crate) desc: Box<SamplerDescriptorInner>,

    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,
}

impl Drop for SamplerImpl {
    #[inline]
    fn drop(&mut self) {
        // log::trace!("Dropping SamplerImpl {:?}", self.raw);
        let lock = self.adapter.lock(None);
        let gl = lock.get_glow();

        unsafe {
            gl.delete_sampler(self.raw);
        }
        self.state.remove_sampler(&gl, self.raw);
    }
}

#[derive(Debug)]
pub struct SamplerDescriptorInner {
    /// Debug label of the sampler. This will show up in graphics debuggers for easy identification.
    // pub label: Label<'a>,

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
    /// Must be at least 1. If this is not 1, all filter modes must be linear.
    pub anisotropy_clamp: u16,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    pub border_color: Option<super::super::wgt::SamplerBorderColor>,
}

impl SamplerDescriptorInner {
    fn from(value: &SamplerDescriptor) -> Self {
        Self {
            address_mode_u: value.address_mode_u,
            address_mode_v: value.address_mode_v,
            address_mode_w: value.address_mode_w,
            mag_filter: value.mag_filter,
            min_filter: value.min_filter,
            mipmap_filter: value.mipmap_filter,
            lod_min_clamp: value.lod_min_clamp,
            lod_max_clamp: value.lod_max_clamp,
            compare: value.compare,
            anisotropy_clamp: value.anisotropy_clamp,
            border_color: value.border_color,
        }
    }
}
