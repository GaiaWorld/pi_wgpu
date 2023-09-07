use glow::HasContext;
use thiserror::Error;

use super::super::{wgt, DeviceError, MissingDownlevelFlags};
use super::{gl_conv as conv, EglContext, SrgbFrameBufferKind, TextureFormatDesc};

#[derive(Debug)]
pub(crate) struct Surface {
    egl: EglContext,
    config: egl::Config,
    pub(crate) presentable: bool,
    raw_window_handle: raw_window_handle::RawWindowHandle,
    swapchain: Option<Swapchain>,
    srgb_kind: SrgbFrameBufferKind,
}

unsafe impl Sync for Surface {}
unsafe impl Send for Surface {}

impl Surface {
    // config = &SurfaceConfiguration {
    //     usage: TextureUsages::RENDER_ATTACHMENT,
    //     format: TextureFormat::Bgra8Unorm 或  TextureFormat::Bgra8UnormSrgb,
    //     width: 交换链宽, 和 surface 宽 一样,
    //     height: 交换链高, 和 surface 高 一样,
    //     present_mode: PresentMode::Fifo,
    //     alpha_mode: CompositeAlphaMode,
    //     view_formats: Vec<TextureFormat>,
    // };
    pub(crate) unsafe fn configure(
        &mut self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        use raw_window_handle::RawWindowHandle as Rwh;

        let surface = match unsafe { self.unconfigure_impl(device) } {
            Some(surface) => surface,
            None => {
                #[allow(trivial_casts)]
                let native_window_ptr = match self.raw_window_handle {
                    Rwh::Win32(handle) => handle.hwnd,
                    Rwh::AndroidNdk(handle) => handle.a_native_window,
                    #[cfg(feature = "emscripten")]
                    Rwh::Web(handle) => handle.id as *mut std::ffi::c_void,
                    _ => {
                        log::error!(
                            "Initialized platform doesn't work with window {:?}",
                            self.raw_window_handle
                        );
                        return Err(super::SurfaceError::Other("incompatible window kind"));
                    }
                };

                let mut attributes = vec![
                    egl::RENDER_BUFFER,
                    // We don't want any of the buffering done by the driver, because we
                    // manage a swapchain on our side.
                    // Some drivers just fail on surface creation seeing `EGL_SINGLE_BUFFER`.
                    if cfg!(target_os = "android") || cfg!(windows) {
                        egl::BACK_BUFFER
                    } else {
                        egl::SINGLE_BUFFER
                    },
                ];

                match self.srgb_kind {
                    SrgbFrameBufferKind::None => {}
                    SrgbFrameBufferKind::Core => {
                        attributes.push(egl::GL_COLORSPACE);
                        attributes.push(egl::GL_COLORSPACE_SRGB);
                    }
                    SrgbFrameBufferKind::Khr => {
                        attributes.push(super::EGL_GL_COLORSPACE_KHR as i32);
                        attributes.push(super::EGL_GL_COLORSPACE_SRGB_KHR as i32);
                    }
                }
                attributes.push(egl::ATTRIB_NONE as i32);

                #[cfg(not(feature = "emscripten"))]
                let egl1_5 = self.egl.0.egl.upcast::<egl::EGL1_5>();

                #[cfg(feature = "emscripten")]
                let egl1_5: Option<&Arc<EglInstance>> = Some(&self.egl.instance);

                // Careful, we can still be in 1.4 version even if `upcast` succeeds
                let raw_result = match egl1_5 {
                    _ => unsafe {
                        todo!()
                        // self.egl.0.egl.create_window_surface(
                        //     self.egl.0.display,
                        //     self.config,
                        //     native_window_ptr,
                        //     Some(&attributes),
                        // )
                    },
                };

                match raw_result {
                    Ok(raw) => raw,
                    Err(e) => {
                        log::warn!("Error in create_window_surface: {:?}", e);
                        return Err(super::SurfaceError::Lost);
                    }
                }
            }
        };

        let format_desc = conv::describe_texture_format(config.format);
        let gl = &device.state.0.borrow().gl;
        let renderbuffer = unsafe { gl.create_renderbuffer() }.unwrap();
        unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer)) };
        unsafe {
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                format_desc.internal,
                config.width as _,
                config.height as _,
            )
        };

        let framebuffer = unsafe { gl.create_framebuffer() }.unwrap();
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(framebuffer)) };
        unsafe {
            gl.framebuffer_renderbuffer(
                glow::READ_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::RENDERBUFFER,
                Some(renderbuffer),
            )
        };
        unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, None) };
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None) };

        self.swapchain = Some(Swapchain {
            surface,
            extent: wgt::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            format: config.format,
            format_desc,
            sample_type: wgt::TextureSampleType::Float { filterable: false },
        });

        Ok(())
    }

    pub(crate) unsafe fn unconfigure(&mut self, device: &super::Device) {
        todo!()
        // if let Some(surface) = unsafe { self.unconfigure_impl(device) } {
        //     self.egl
        //         .0
        //         .egl
        //         .destroy_surface(self.egl.0.display, surface)
        //         .unwrap();
        // }
    }

    pub(crate) unsafe fn acquire_texture(
        &mut self,
    ) -> Result<Option<super::AcquiredSurfaceTexture<super::GL>>, crate::SurfaceError> {
        let sc = self.swapchain.as_ref().unwrap();

        
        Ok(Some(super::AcquiredSurfaceTexture {
            texture: todo!(),
            suboptimal: false,
        }))
    }
}

impl Surface {
    pub(crate) fn supports_srgb(&self) -> bool {
        match self.srgb_kind {
            SrgbFrameBufferKind::None => false,
            _ => true,
        }
    }

    #[inline]
    unsafe fn unconfigure_impl(&mut self, device: &super::Device) -> Option<egl::Surface> {
        self.swapchain.take().map(|sc| sc.surface)
    }
}

#[derive(Debug)]
pub(crate) struct Swapchain {
    surface: egl::Surface,

    /// Extent because the window lies
    extent: wgt::Extent3d,
    format: wgt::TextureFormat,

    format_desc: TextureFormatDesc,

    #[allow(unused)]
    sample_type: wgt::TextureSampleType,
}

#[derive(Clone, Debug, Error)]
pub(crate) enum ConfigureSurfaceError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("invalid surface")]
    InvalidSurface,
    #[error("The view format {0:?} is not compatible with texture format {1:?}, only changing srgb-ness is allowed.")]
    InvalidViewFormat(wgt::TextureFormat, wgt::TextureFormat),
    #[error(transparent)]
    MissingDownlevelFlags(#[from] MissingDownlevelFlags),
    #[error("`SurfaceOutput` must be dropped before a new `Surface` is made")]
    PreviousOutputExists,
    #[error("Both `Surface` width and height must be non-zero. Wait to recreate the `Surface` until the window has non-zero area.")]
    ZeroArea,
    #[error("surface does not support the adapter's queue family")]
    UnsupportedQueueFamily,
    #[error("requested format {requested:?} is not in list of supported formats: {available:?}")]
    UnsupportedFormat {
        requested: wgt::TextureFormat,
        available: Vec<wgt::TextureFormat>,
    },
    #[error("requested present mode {requested:?} is not in the list of supported present modes: {available:?}")]
    UnsupportedPresentMode {
        requested: wgt::PresentMode,
        available: Vec<wgt::PresentMode>,
    },
    #[error("requested alpha mode {requested:?} is not in the list of supported alpha modes: {available:?}")]
    UnsupportedAlphaMode {
        requested: wgt::CompositeAlphaMode,
        available: Vec<wgt::CompositeAlphaMode>,
    },
    #[error("requested usage is not supported")]
    UnsupportedUsage,
}
