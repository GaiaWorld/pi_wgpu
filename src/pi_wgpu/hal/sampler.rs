use glow::HasContext;
use pi_share::Share;

use super::{gl_conv as conv, AdapterContext, GLState};

#[derive(Debug, Clone)]
pub(crate) struct Sampler(pub(crate) Share<SamplerImpl>);

impl Sampler {
    pub fn new(
        state: GLState,
        adapter: &AdapterContext,
        desc: &super::super::SamplerDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        let gl = adapter.imp.as_ref().borrow();
        let gl = gl.lock();

        let raw = unsafe { gl.create_sampler().unwrap() };

        let (min, mag) =
            conv::map_filter_modes(desc.min_filter, desc.mag_filter, desc.mipmap_filter);

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
        };
        Ok(Self(Share::new(imp)))
    }
}

#[derive(Debug)]
pub(crate) struct SamplerImpl {
    pub(crate) raw: glow::Sampler,

    pub(crate) state: GLState,
    pub(crate) adapter: AdapterContext,
}

impl Drop for SamplerImpl {
    #[inline]
    fn drop(&mut self) {
        let gl = self.adapter.imp.as_ref().borrow();
        let gl = gl.lock();

        unsafe {
            gl.delete_sampler(self.raw);
        }
        self.state.remove_sampler(&gl, self.raw);
    }
}
