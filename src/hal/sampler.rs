use glow::HasContext;

use super::{GLState, gl_conv as conv};

#[derive(Debug)]
pub(crate) struct Sampler {
    raw: glow::Sampler,
    state: GLState,
}

impl Drop for Sampler {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let gl = self.state.get_gl();
            gl.delete_sampler(self.raw);
        }

        self.state.remove_sampler(self.raw);
    }
}

impl Sampler {
    pub fn new(
        state: GLState,
        desc: &crate::SamplerDescriptor,
    ) -> Result<Self, crate::DeviceError> {
        let gl = state.get_gl();

        let raw = unsafe { gl.create_sampler().unwrap() };

        let (min, mag) = conv::map_filter_modes(desc.min_filter, desc.mag_filter, desc.mipmap_filter);

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

        Ok(Self { raw, state })
    }
}

