use glow::HasContext;
use pi_share::Share;

use super::{
    super::{wgt, AstcChannel},
    AdapterContext, GLState,
};

#[derive(Debug)]
pub(crate) struct Adapter {
    pub(crate) context: Share<AdapterContext>,
}

impl Adapter {
    // 枚举 gl 环境的 特性
    pub(crate) fn expose(
        context: Share<AdapterContext>,
    ) -> Option<super::super::ExposedAdapter<super::GL>> {
        let info = context.info.clone();
        let features = context.features.clone();
        let limits = context.limits.clone();
        let downlevel = context.downlevel.clone();

        let adapter = super::Adapter { context };

        Some(super::ExposedAdapter {
            adapter,
            info,
            features,
            limits,
            downlevel,
        })
    }

    pub(crate) fn open(
        &self,
        features: wgt::Features,
        _limits: &wgt::Limits,
    ) -> Result<super::OpenDevice<super::GL>, super::super::DeviceError> {
        // Verify all features were exposed by the adapter
        if !self.context.features.contains(features) {
            return Err(super::super::DeviceError::UnsupportedFeature(
                features - self.context.features,
            ));
        }

        let gl = &self.context.lock();

        unsafe { gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1) };
        unsafe { gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1) };

        let state = GLState::new(&gl);

        Ok(super::OpenDevice {
            device: super::Device {
                state: state.clone(),
                adapter: self.context.clone(),

                features,
                limits: self.context.limits.clone(),
                downlevel: self.context.downlevel.clone(),
            },
            queue: super::Queue {
                state,
                adapter: self.context.clone(),
            },
        })
    }

    /// Return the set of supported capabilities for a texture format.
    pub(crate) fn texture_format_capabilities(
        &self,
        format: wgt::TextureFormat,
    ) -> super::TextureFormatCapabilities {
        use super::TextureFormatCapabilities as Tfc;
        use wgt::TextureFormat as Tf;

        let sample_count = {
            let max_samples = unsafe { self.context.lock().get_parameter_i32(glow::MAX_SAMPLES) };
            if max_samples >= 8 {
                Tfc::MULTISAMPLE_X2 | Tfc::MULTISAMPLE_X4 | Tfc::MULTISAMPLE_X8
            } else if max_samples >= 4 {
                Tfc::MULTISAMPLE_X2 | Tfc::MULTISAMPLE_X4
            } else {
                Tfc::MULTISAMPLE_X2
            }
        };

        // Base types are pulled from the table in the OpenGLES 3.0 spec in section 3.8.
        //
        // The storage types are based on table 8.26, in section
        // "TEXTURE IMAGE LOADS AND STORES" of OpenGLES-3.2 spec.
        let empty = Tfc::empty();
        let base = Tfc::COPY_SRC | Tfc::COPY_DST;
        let unfilterable = base | Tfc::SAMPLED;
        let depth = base | Tfc::SAMPLED | sample_count | Tfc::DEPTH_STENCIL_ATTACHMENT;
        let filterable = unfilterable | Tfc::SAMPLED_LINEAR;
        let renderable =
            unfilterable | Tfc::COLOR_ATTACHMENT | sample_count | Tfc::MULTISAMPLE_RESOLVE;
        let filterable_renderable = filterable | renderable | Tfc::COLOR_ATTACHMENT_BLEND;
        let storage = base | Tfc::STORAGE | Tfc::STORAGE_READ_WRITE;

        let feature_fn = |f, caps| {
            if self.context.features.contains(f) {
                caps
            } else {
                empty
            }
        };

        let bcn_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_BC, filterable);
        let etc2_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ETC2, filterable);
        let astc_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ASTC, filterable);
        let astc_hdr_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ASTC_HDR, filterable);

        let private_caps_fn = |f, caps| {
            if self.context.private_caps.contains(f) {
                caps
            } else {
                empty
            }
        };

        let half_float_renderable = private_caps_fn(
            super::PrivateCapabilities::COLOR_BUFFER_HALF_FLOAT,
            Tfc::COLOR_ATTACHMENT
                | Tfc::COLOR_ATTACHMENT_BLEND
                | sample_count
                | Tfc::MULTISAMPLE_RESOLVE,
        );

        let float_renderable = private_caps_fn(
            super::PrivateCapabilities::COLOR_BUFFER_FLOAT,
            Tfc::COLOR_ATTACHMENT
                | Tfc::COLOR_ATTACHMENT_BLEND
                | sample_count
                | Tfc::MULTISAMPLE_RESOLVE,
        );

        let texture_float_linear =
            private_caps_fn(super::PrivateCapabilities::TEXTURE_FLOAT_LINEAR, filterable);

        match format {
            Tf::R8Unorm => filterable_renderable,
            Tf::R8Snorm => filterable,
            Tf::R8Uint => renderable,
            Tf::R8Sint => renderable,
            Tf::R16Uint => renderable,
            Tf::R16Sint => renderable,
            Tf::R16Unorm => empty,
            Tf::R16Snorm => empty,
            Tf::R16Float => filterable | half_float_renderable,
            Tf::Rg8Unorm => filterable_renderable,
            Tf::Rg8Snorm => filterable,
            Tf::Rg8Uint => renderable,
            Tf::Rg8Sint => renderable,
            Tf::R32Uint => renderable | storage,
            Tf::R32Sint => renderable | storage,
            Tf::R32Float => unfilterable | storage | float_renderable | texture_float_linear,
            Tf::Rg16Uint => renderable,
            Tf::Rg16Sint => renderable,
            Tf::Rg16Unorm => empty,
            Tf::Rg16Snorm => empty,
            Tf::Rg16Float => filterable | half_float_renderable,
            Tf::Rgba8Unorm | Tf::Rgba8UnormSrgb => filterable_renderable | storage,
            Tf::Bgra8Unorm | Tf::Bgra8UnormSrgb => filterable_renderable,
            Tf::Rgba8Snorm => filterable,
            Tf::Rgba8Uint => renderable | storage,
            Tf::Rgba8Sint => renderable | storage,
            Tf::Rgb10a2Unorm => filterable_renderable,
            Tf::Rg11b10Float => filterable | float_renderable,
            Tf::Rg32Uint => renderable,
            Tf::Rg32Sint => renderable,
            Tf::Rg32Float => unfilterable | float_renderable | texture_float_linear,
            Tf::Rgba16Uint => renderable | storage,
            Tf::Rgba16Sint => renderable | storage,
            Tf::Rgba16Unorm => empty,
            Tf::Rgba16Snorm => empty,
            Tf::Rgba16Float => filterable | storage | half_float_renderable,
            Tf::Rgba32Uint => renderable | storage,
            Tf::Rgba32Sint => renderable | storage,
            Tf::Rgba32Float => unfilterable | storage | float_renderable | texture_float_linear,
            Tf::Stencil8
            | Tf::Depth16Unorm
            | Tf::Depth32Float
            | Tf::Depth32FloatStencil8
            | Tf::Depth24Plus
            | Tf::Depth24PlusStencil8 => depth,
            Tf::Rgb9e5Ufloat => filterable,
            Tf::Bc1RgbaUnorm
            | Tf::Bc1RgbaUnormSrgb
            | Tf::Bc2RgbaUnorm
            | Tf::Bc2RgbaUnormSrgb
            | Tf::Bc3RgbaUnorm
            | Tf::Bc3RgbaUnormSrgb
            | Tf::Bc4RUnorm
            | Tf::Bc4RSnorm
            | Tf::Bc5RgUnorm
            | Tf::Bc5RgSnorm
            | Tf::Bc6hRgbFloat
            | Tf::Bc6hRgbUfloat
            | Tf::Bc7RgbaUnorm
            | Tf::Bc7RgbaUnormSrgb => bcn_features,
            Tf::Etc2Rgb8Unorm
            | Tf::Etc2Rgb8UnormSrgb
            | Tf::Etc2Rgb8A1Unorm
            | Tf::Etc2Rgb8A1UnormSrgb
            | Tf::Etc2Rgba8Unorm
            | Tf::Etc2Rgba8UnormSrgb
            | Tf::EacR11Unorm
            | Tf::EacR11Snorm
            | Tf::EacRg11Unorm
            | Tf::EacRg11Snorm => etc2_features,
            Tf::Astc {
                block: _,
                channel: AstcChannel::Unorm | AstcChannel::UnormSrgb,
            } => astc_features,
            Tf::Astc {
                block: _,
                channel: AstcChannel::Hdr,
            } => astc_hdr_features,
        }
    }

    /// Returns the capabilities of working with a specified surface.
    ///
    /// `None` means presentation is not supported for it.
    pub(crate) fn surface_capabilities(
        &self,
        surface: &super::Surface,
    ) -> Option<super::SurfaceCapabilities> {
        if surface.get_presentable() {
            let mut formats = vec![
                wgt::TextureFormat::Rgba8Unorm,
                #[cfg(not(target_arch = "wasm32"))]
                wgt::TextureFormat::Bgra8Unorm,
            ];
            if surface.supports_srgb() {
                formats.extend([
                    wgt::TextureFormat::Rgba8UnormSrgb,
                    #[cfg(not(target_arch = "wasm32"))]
                    wgt::TextureFormat::Bgra8UnormSrgb,
                ])
            }
            if self
                .context
                .private_caps
                .contains(super::PrivateCapabilities::COLOR_BUFFER_HALF_FLOAT)
            {
                formats.push(wgt::TextureFormat::Rgba16Float)
            }

            Some(super::SurfaceCapabilities {
                formats,
                present_modes: vec![wgt::PresentMode::Fifo], //TODO
                composite_alpha_modes: vec![wgt::CompositeAlphaMode::Opaque], //TODO
                swap_chain_sizes: 2..=2,
                current_extent: None,
                extents: wgt::Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                }..=wgt::Extent3d {
                    width: self.context.max_texture_size,
                    height: self.context.max_texture_size,
                    depth_or_array_layers: 1,
                },
                usage: super::TextureUses::COLOR_TARGET,
            })
        } else {
            None
        }
    }
}
