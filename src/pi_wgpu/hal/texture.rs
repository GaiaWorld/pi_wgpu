use std::{ops::Range, sync::atomic::AtomicU32};

use glow::HasContext;
use pi_share::Share;

use crate::TextureFormat;

use super::super::{hal::gl_conv as conv, wgt};
use super::{AdapterContext, GLState, TextureFormatDesc};

#[derive(Debug, Clone)]
pub(crate) struct Texture(pub(crate) Share<TextureImpl>);

impl Texture {
    pub fn new(
        state: GLState,
        adapter: &AdapterContext,
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

        let lock = adapter.lock(None);
        let gl = lock.get_glow();

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

            unsafe {
                gl.active_texture(glow::TEXTURE0);
                gl.bind_texture(target, Some(raw))
            };

            //Note: this has to be done before defining the storage!
            match desc.format.sample_type(None, Some(adapter.imp.borrow().as_ref().unwrap().features)) {
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

            unsafe {
                // gl.tex_parameter_i32(target, glow::TEXTURE_BASE_LEVEL, 0);

                gl.tex_parameter_i32(target, glow::TEXTURE_MAX_LEVEL, 0);
            }

            let block_dims = desc.format.block_dimensions();
            if block_dims != (1, 1) {
                if is_3d {
                    // TODO 目前并不清楚 3D 压缩纹理的 深度格式，不实现
                    unimplemented!();
                } else if desc.sample_count > 1 {
                    unimplemented!()
                } else {
                    let block_size = get_texture_bytes_per_block(desc.format);

                    let block_width = (desc.size.width + block_dims.0 - 1) / block_dims.0;

                    let block_height = (desc.size.height + block_dims.1 - 1) / block_dims.1;

                    let image_size = block_width * block_height * block_size; // ASTC 块总是使用 128 位，即 16 字节。

                    let data = vec![0u8; image_size as usize];
                    unsafe {
                        gl.compressed_tex_image_2d(
                            target,
                            0,
                            format_desc.internal as i32,
                            desc.size.width as i32,
                            desc.size.height as i32,
                            0,
                            image_size as i32,
                            data.as_slice(),
                        );
                    }
                }
            } else {
                if is_3d {
                    unsafe {
                        gl.tex_image_3d(
                            target,
                            0,
                            format_desc.internal as i32,
                            desc.size.width as i32,
                            desc.size.height as i32,
                            desc.size.depth_or_array_layers as i32,
                            0,
                            format_desc.external,
                            format_desc.data_type,
                            None,
                        );
                    };
                } else if desc.sample_count > 1 {
                    unimplemented!()
                } else {
                    unsafe {
                        gl.tex_image_2d(
                            target,
                            0,
                            format_desc.internal as i32,
                            desc.size.width as i32,
                            desc.size.height as i32,
                            0,
                            format_desc.external,
                            format_desc.data_type,
                            None,
                        );
                    }
                }
            }

            state.restore_current_texture(&gl, 0, target);

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

    // 从窗口表面创建
    pub(crate) fn with_window_surface(width: u32, height: u32, format: TextureFormat) -> Self {
        let format_desc = conv::map_texture_format(format);

        let imp = TextureImpl {
            inner: TextureInner::NativeRenderBuffer,
            mip_level_count: 1,
            array_layer_count: 1,
            format,
            copy_size: super::CopyExtent {
                width,
                height,
                depth: 1,
            },
            format_desc,
            is_cubemap: false,
        };

        Self(Share::new(imp))
    }
}

impl Texture {
    pub fn write_data(
        state: &GLState,
        copy: super::super::ImageCopyTexture,
        data1: &[u8],
        _data_layout: super::super::ImageDataLayout,
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

        let lock = adapter.lock(None);
        let  gl = lock.get_glow();

        unsafe {
            // gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(dst_target, Some(raw));
        }

        if !inner.format.is_compressed() {
            let data = glow::PixelUnpackData::Slice(data1);

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
					unsafe { gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1) };
					unsafe { gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1) };
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
            let data = glow::CompressedPixelUnpackData::Slice(data1);
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

        state.restore_current_texture(&gl, 0, dst_target);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write_compress_jsdata(
        state: &GLState,
        copy: super::super::ImageCopyTexture,
        data: &js_sys::Object,
        _data_layout: super::super::ImageDataLayout,
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

        let lock = adapter.lock(None);
        let  gl = lock.get_glow();

        unsafe {
            // gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(dst_target, Some(raw));
        }

        if !inner.format.is_compressed() {
            unreachable!()
        } else {
            match dst_target {
                glow::TEXTURE_3D | glow::TEXTURE_CUBE_MAP_ARRAY | glow::TEXTURE_2D_ARRAY => {
                    unsafe {
                        gl.compressed_tex_sub_image_3d_jsobj(
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
                        gl.compressed_tex_sub_image_2d_jsobj(
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
                        gl.compressed_tex_sub_image_2d_jsobj(
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

        state.restore_current_texture(&gl, 0, dst_target);
    }

    #[cfg(target_arch = "wasm32")]
    pub fn write_external_image(
        state: &GLState,
        src: &crate::ImageCopyExternalImage,
        copy: crate::ImageCopyTexture,
        size: super::super::Extent3d,
        dst_premultiplication: bool,
    ) {
        profiling::scope!("hal::Texture::write_external_image");

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

        let lock = adapter.lock(None);
        let  gl = lock.get_glow();

        unsafe {
            const UNPACK_FLIP_Y_WEBGL: u32 =
                    web_sys::WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL;
            const UNPACK_PREMULTIPLY_ALPHA_WEBGL: u32 =
                web_sys::WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL;
            if src.flip_y {
                gl.pixel_store_bool(UNPACK_FLIP_Y_WEBGL, false);
            }
            if dst_premultiplication {
                gl.pixel_store_bool(UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
            }
        }


        unsafe {
            // gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(dst_target, Some(raw));
        }

        if is_layered_target(dst_target) {
            match src.source {
                wgt::ExternalImageSource::HTMLImageElement(ref b) => unsafe {
                    unreachable!()
                },
                wgt::ExternalImageSource::ImageBitmap(ref b) => unsafe {
                    gl.tex_sub_image_3d_with_image_bitmap(
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
                        b,
                    );
                },
                wgt::ExternalImageSource::HTMLVideoElement(ref v) => unsafe {
                    gl.tex_sub_image_3d_with_html_video_element(
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
                        v,
                    );
                },
                wgt::ExternalImageSource::HTMLCanvasElement(ref c) => unsafe {
                    gl.tex_sub_image_3d_with_html_canvas_element(
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
                        c,
                    );
                },
                wgt::ExternalImageSource::OffscreenCanvas(_) => unreachable!(),
            }
        } else {
            let dst_target = match dst_target {
                glow::TEXTURE_2D => {
                    unsafe { gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1) };
                    unsafe { gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1) };
                    dst_target
                },
                glow::TEXTURE_2D => super::CUBEMAP_FACES[size.depth_or_array_layers as usize],
                _ => unreachable!(),
            };
            match src.source {
                wgt::ExternalImageSource::HTMLImageElement(ref b) => unsafe {
                    // HtmlImageElmement
                    gl.tex_sub_image_2d_with_html_image_and_width_and_height(
                        dst_target,
                        copy.mip_level as i32,
                        copy.origin.x as i32,
                        copy.origin.y as i32,
                        size.width as i32,
                        size.height as i32,
                        format_desc.external,
                        format_desc.data_type,
                        &b,
                    );
                },
                wgt::ExternalImageSource::ImageBitmap(ref b) => unsafe {
                    // 当前实现将 ImageBitmap 视为 HtmlImageElmement
                    gl.tex_sub_image_2d_with_image_bitmap_and_width_and_height(
                        dst_target,
                        copy.mip_level as i32,
                        copy.origin.x as i32,
                        copy.origin.y as i32,
                        size.width as i32,
                        size.height as i32,
                        format_desc.external,
                        format_desc.data_type,
                        &b,
                    );
                },
                wgt::ExternalImageSource::HTMLVideoElement(ref v) => unsafe {
                    gl.tex_sub_image_2d_with_html_video_and_width_and_height(
                        dst_target,
                        copy.mip_level as i32,
                        copy.origin.x as i32,
                        copy.origin.y as i32,
                        size.width as i32,
                        size.height as i32,
                        format_desc.external,
                        format_desc.data_type,
                        v,
                    )
                },
                wgt::ExternalImageSource::HTMLCanvasElement(ref c) => unsafe {
                    gl.tex_sub_image_2d_with_html_canvas_and_width_and_height(
                        dst_target,
                        copy.mip_level as i32,
                        copy.origin.x as i32,
                        copy.origin.y as i32,
                        size.width as i32,
                        size.height as i32,
                        format_desc.external,
                        format_desc.data_type,
                        c,
                    )
                },
                wgt::ExternalImageSource::OffscreenCanvas(_) => unreachable!(),
            }
        }

        state.restore_current_texture(&gl, 0, dst_target);
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
    // pub(crate) sample_type: wgt::TextureSampleType,
    pub(crate) aspects: super::FormatAspects,
    pub(crate) mip_levels: Range<u32>,
    pub(crate) array_layers: Range<u32>,
    pub(crate) format: wgt::TextureFormat,
    pub(crate) id: u32,
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
            // sample_type: imp.format.sample_type(None, None).unwrap(),

            format_desc: imp.format_desc.clone(),

            aspects: super::FormatAspects::new(imp.format, desc.aspect),
            id: TEXTURE_VIEW_AROM.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        })
    }
}
lazy_static! {
    static ref TEXTURE_VIEW_AROM: AtomicU32 = AtomicU32::new(1);
}

#[derive(Debug, Clone)]
pub(crate) struct TextureImpl {
    pub inner: TextureInner,

    pub mip_level_count: u32,
    pub array_layer_count: u32,
    pub format: wgt::TextureFormat,

    pub copy_size: super::CopyExtent,

    pub format_desc: TextureFormatDesc,

    pub is_cubemap: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum TextureInner {
    NativeRenderBuffer,

    Renderbuffer {
        state: GLState,
        adapter: AdapterContext,
        raw: glow::Renderbuffer,
    },

    Texture {
        state: GLState,
        adapter: AdapterContext,
        raw: glow::Texture,
        target: super::BindTarget,
    },
}

impl TextureInner {
    pub(crate) fn debug_str(&self) -> String {
        #[cfg(not(target_arch = "wasm32"))]
        let r = match self {
            crate::pi_wgpu::hal::TextureInner::NativeRenderBuffer => "native".to_string(),
            crate::pi_wgpu::hal::TextureInner::Renderbuffer { raw, .. } => {
                "render".to_string() + raw.0.get().to_string().as_str()
            }
            crate::pi_wgpu::hal::TextureInner::Texture { raw, .. } => {
                "".to_string() + raw.0.get().to_string().as_str()
            }
        };
        #[cfg(target_arch = "wasm32")]
        let r = "".to_string();
        r
    }
}

impl Drop for TextureInner {
    fn drop(&mut self) {
        profiling::scope!("hal::TextureInner::drop");
        log::trace!("Dropping TextureInner {:?}", self);
        match &self {
            &TextureInner::NativeRenderBuffer => {}
            &TextureInner::Renderbuffer {
                ref adapter,
                ref state,
                ref raw,
            } => {
                let lock = adapter.lock(None);
                let gl = lock.get_glow();

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
                let lock = adapter.lock(None);
                let gl = lock.get_glow();

                unsafe {
                    gl.delete_texture(*raw);
                }
                state.remove_texture(&gl, *raw);
            }
        }
    }
}

fn get_texture_bytes_per_block(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::Bc1RgbaUnorm => 8,
        TextureFormat::Bc1RgbaUnormSrgb => 8,
        TextureFormat::Bc2RgbaUnorm => 16,
        TextureFormat::Bc2RgbaUnormSrgb => 16,

        TextureFormat::Bc3RgbaUnorm => 16,
        TextureFormat::Bc3RgbaUnormSrgb => 16,
        TextureFormat::Bc4RUnorm => 8,
        TextureFormat::Bc4RSnorm => 8,
        TextureFormat::Bc5RgUnorm => 16,
        TextureFormat::Bc5RgSnorm => 16,
        TextureFormat::Bc6hRgbUfloat => 16,
        TextureFormat::Bc6hRgbFloat => 16,
        TextureFormat::Bc7RgbaUnorm => 16,
        TextureFormat::Bc7RgbaUnormSrgb => 16,

        TextureFormat::Astc { .. } => 16, // ASTC 块总是使用 128 位，即 16 字节。

        _ => unimplemented!(),
    }
}

fn is_layered_target(target: u32) -> bool {
    match target {
        glow::TEXTURE_2D | glow::TEXTURE_CUBE_MAP => false,
        glow::TEXTURE_2D_ARRAY | glow::TEXTURE_3D => true,
        _ => unreachable!(),
    }
}
