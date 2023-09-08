use std::ops::Range;

use glow::HasContext;
use pi_share::Share;

use super::super::{hal::gl_conv as conv, wgt};
use super::{adapter, AdapterContext, GLState, TextureFormatDesc};

pub(crate) type TextureID = u64;

#[derive(Debug)]
pub(crate) struct Texture(pub(crate) Share<TextureImpl>);

impl Texture {
    pub fn new(
        state: GLState,
        adapter: &Share<AdapterContext>,
        desc: &super::super::TextureDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        profiling::scope!("hal::Texture::new");

        let usage = conv::map_texture_usage(desc.usage, desc.format.into());

        let render_usage = super::TextureUses::COLOR_TARGET
            | super::TextureUses::DEPTH_STENCIL_WRITE
            | super::TextureUses::DEPTH_STENCIL_READ;

        let format_desc = conv::map_texture_format(desc.format);

        let mut copy_size = super::CopyExtent {
            width: desc.size.width,
            height: desc.size.height,
            depth: 1,
        };

        let gl = adapter.lock();
        let (inner, is_cubemap) = if render_usage.contains(usage)
            && desc.dimension == wgt::TextureDimension::D2
            && desc.size.depth_or_array_layers == 1
        {
            // 纹理 仅作为 渲染目标，不作为 Sampler 或 Storage 或 Copy，则直接创建 RenderBuffer
            let raw = unsafe { gl.create_renderbuffer().unwrap() };

            unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, Some(raw)) };

            if desc.sample_count > 1 {
                unsafe {
                    gl.renderbuffer_storage_multisample(
                        glow::RENDERBUFFER,
                        desc.sample_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            } else {
                unsafe {
                    gl.renderbuffer_storage(
                        glow::RENDERBUFFER,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            }
            unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, None) };

            (
                TextureInner::Renderbuffer {
                    raw,
                    adapter: adapter.clone(),
                    state,
                },
                false,
            )
        } else {
            let raw = unsafe { gl.create_texture().unwrap() };
            let (target, is_3d, is_cubemap) = Texture::get_info_from_desc(&mut copy_size, desc);

            unsafe { gl.bind_texture(target, Some(raw)) };

            //Note: this has to be done before defining the storage!
            match desc.format.sample_type(None) {
                Some(
                    wgt::TextureSampleType::Float { filterable: false }
                    | wgt::TextureSampleType::Uint
                    | wgt::TextureSampleType::Sint,
                ) => {
                    // reset default filtering mode
                    unsafe {
                        gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32)
                    };
                    unsafe {
                        gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32)
                    };
                }
                _ => {}
            }

            if is_3d {
                unsafe {
                    gl.tex_storage_3d(
                        target,
                        desc.mip_level_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                        desc.size.depth_or_array_layers as i32,
                    )
                };
            } else if desc.sample_count > 1 {
                unimplemented!()
            } else {
                unsafe {
                    gl.tex_storage_2d(
                        target,
                        desc.mip_level_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            }

            unsafe { gl.bind_texture(target, None) };

            (
                TextureInner::Texture {
                    raw,
                    target,
                    state,
                    adapter: adapter.clone(),
                },
                is_cubemap,
            )
        };

        let imp = TextureImpl {
            inner,
            mip_level_count: desc.mip_level_count,
            array_layer_count: if desc.dimension == wgt::TextureDimension::D2 {
                desc.size.depth_or_array_layers
            } else {
                1
            },
            format: desc.format,
            copy_size,
            format_desc,
            is_cubemap,
        };

        Ok(Self(Share::new(imp)))
    }
}

impl Texture {
    pub fn write_data(
        copy: super::super::ImageCopyTexture,
        data: &[u8],
        data_layout: super::super::ImageDataLayout,
        size: super::super::Extent3d,
    ) {
        profiling::scope!("hal::Texture::write_data");

        let inner = copy.texture.inner.0.as_ref();

        let (raw, dst_target, adapter) = match &inner.inner {
            TextureInner::Texture {
                raw,
                target,
                adapter,
                ..
            } => (*raw, *target, adapter),
            _ => unreachable!(),
        };

        let format_desc = &inner.format_desc;

        let gl = adapter.lock();
        unsafe {
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(dst_target, Some(raw));
        }

        if !inner.format.is_compressed() {
            let data = glow::PixelUnpackData::Slice(data);

            match dst_target {
                glow::TEXTURE_3D => {
                    unsafe {
                        gl.tex_sub_image_3d(
                            dst_target,
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            copy.origin.z as i32,
                            size.width as i32,
                            size.height as i32,
                            size.depth_or_array_layers as i32,
                            format_desc.external,
                            format_desc.data_type,
                            data,
                        )
                    };
                }
                glow::TEXTURE_2D_ARRAY => {
                    unsafe {
                        gl.tex_sub_image_3d(
                            dst_target,
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            copy.origin.z as i32,
                            size.width as i32,
                            size.height as i32,
                            size.depth_or_array_layers as i32,
                            format_desc.external,
                            format_desc.data_type,
                            data,
                        )
                    };
                }
                glow::TEXTURE_2D => {
                    unsafe {
                        gl.tex_sub_image_2d(
                            dst_target,
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            size.width as i32,
                            size.height as i32,
                            format_desc.external,
                            format_desc.data_type,
                            data,
                        )
                    };
                }
                glow::TEXTURE_CUBE_MAP => {
                    unsafe {
                        gl.tex_sub_image_2d(
                            super::CUBEMAP_FACES[size.depth_or_array_layers as usize],
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            size.width as i32,
                            size.height as i32,
                            format_desc.external,
                            format_desc.data_type,
                            data,
                        )
                    };
                }
                _ => unreachable!(),
            }
        } else {
            let data = glow::CompressedPixelUnpackData::Slice(data);
            match dst_target {
                glow::TEXTURE_3D | glow::TEXTURE_CUBE_MAP_ARRAY | glow::TEXTURE_2D_ARRAY => {
                    unsafe {
                        gl.compressed_tex_sub_image_3d(
                            dst_target,
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            copy.origin.z as i32,
                            size.width as i32,
                            size.height as i32,
                            size.depth_or_array_layers as i32,
                            format_desc.internal,
                            data,
                        )
                    };
                }
                glow::TEXTURE_2D => {
                    unsafe {
                        gl.compressed_tex_sub_image_2d(
                            dst_target,
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            size.width as i32,
                            size.height as i32,
                            format_desc.internal,
                            data,
                        )
                    };
                }
                glow::TEXTURE_CUBE_MAP => {
                    unsafe {
                        gl.compressed_tex_sub_image_2d(
                            super::CUBEMAP_FACES[size.depth_or_array_layers as usize],
                            copy.mip_level as i32,
                            copy.origin.x as i32,
                            copy.origin.y as i32,
                            size.width as i32,
                            size.height as i32,
                            format_desc.internal,
                            data,
                        )
                    };
                }
                _ => unreachable!(),
            }
        }

        unsafe {
            gl.bind_texture(dst_target, None);
        }
    }
}

impl Texture {
    /// Returns the `target`, whether the image is 3d and whether the image is a cubemap.
    #[inline]
    fn get_info_from_desc(
        copy_size: &mut super::CopyExtent,
        desc: &super::super::TextureDescriptor,
    ) -> (u32, bool, bool) {
        match desc.dimension {
            wgt::TextureDimension::D1 | wgt::TextureDimension::D2 => {
                if desc.size.depth_or_array_layers > 1 {
                    //HACK: detect a cube map
                    let cube_count = if desc.size.width == desc.size.height
                        && desc.size.depth_or_array_layers % 6 == 0
                        && desc.sample_count == 1
                    {
                        Some(desc.size.depth_or_array_layers / 6)
                    } else {
                        None
                    };
                    match cube_count {
                        None => (glow::TEXTURE_2D_ARRAY, true, false),
                        Some(1) => (glow::TEXTURE_CUBE_MAP, false, true),
                        Some(_) => unimplemented!(),
                    }
                } else {
                    (glow::TEXTURE_2D, false, false)
                }
            }
            wgt::TextureDimension::D3 => {
                copy_size.depth = desc.size.depth_or_array_layers;
                (glow::TEXTURE_3D, true, false)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TextureView {
    pub(crate) inner: Share<TextureImpl>,

    pub(crate) format_desc: TextureFormatDesc,
    pub(crate) sample_type: wgt::TextureSampleType,
    pub(crate) aspects: super::FormatAspects,
    pub(crate) mip_levels: Range<u32>,
    pub(crate) array_layers: Range<u32>,
    pub(crate) format: wgt::TextureFormat,
}

impl TextureView {
    pub fn new(
        texture: &Texture,
        desc: &super::super::TextureViewDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        profiling::scope!("hal::TextureView::new");

        let imp = texture.0.as_ref();

        let mip_count = match desc.mip_level_count {
            Some(count) => count.into(),
            None => imp.mip_level_count,
        };
        let mip_levels = desc.base_mip_level..(mip_count - desc.base_mip_level);

        let layer_count = match desc.array_layer_count {
            Some(count) => count.into(),
            None => imp.array_layer_count,
        };
        let array_layers = desc.base_array_layer..(layer_count - desc.base_array_layer);

        Ok(TextureView {
            inner: texture.0.clone(),

            mip_levels,
            array_layers,

            format: imp.format,
            sample_type: imp.format.sample_type(None).unwrap(),

            format_desc: imp.format_desc.clone(),

            aspects: super::FormatAspects::from(imp.format)
                & super::FormatAspects::from(desc.aspect),
        })
    }
}

#[derive(Debug)]
pub(crate) struct TextureImpl {
    pub inner: TextureInner,

    pub mip_level_count: u32,
    pub array_layer_count: u32,
    pub format: wgt::TextureFormat,

    pub copy_size: super::CopyExtent,

    pub format_desc: TextureFormatDesc,

    pub is_cubemap: bool,
}

#[derive(Debug)]
pub(crate) enum TextureInner {
    DefaultRenderbuffer,

    Renderbuffer {
        state: GLState,
        adapter: Share<AdapterContext>,
        raw: glow::Renderbuffer,
    },

    Texture {
        state: GLState,
        adapter: Share<AdapterContext>,

        raw: glow::Texture,
        target: super::BindTarget,
    },
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        profiling::scope!("hal::TextureInner::drop");

        match &self {
            &TextureInner::Renderbuffer {
                ref adapter,
                ref state,
                ref raw,
            } => {
                let gl = adapter.lock();
                unsafe {
                    gl.delete_renderbuffer(*raw);
                }
                state.remove_render_buffer(&gl, *raw);
            }
            &TextureInner::Texture {
                ref adapter,
                ref state,
                ref raw,
                ..
            } => {
                let gl = adapter.lock();
                unsafe {
                    gl.delete_texture(*raw);
                }
                state.remove_texture(&gl, *raw);
            }
            &TextureInner::DefaultRenderbuffer => {}
        }
    }
}
