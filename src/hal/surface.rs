use std::{os::raw, time::Duration};

use glow::HasContext;
use thiserror::Error;

use super::{egl_impl, AcquiredSurfaceTexture, EglContext, SrgbFrameBufferKind, TextureFormatDesc};
use crate::{hal, wgt, DeviceError, MissingDownlevelFlags};

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
    pub(crate) unsafe fn configure(
        &mut self,
        device: &super::Device,
        config: &super::SurfaceConfiguration,
    ) -> Result<(), hal::SurfaceError> {
        todo!()
    }

    // pub(crate) unsafe fn configure(
    //     &mut self,
    //     device: &super::Device,
    //     config: &super::SurfaceConfiguration,
    // ) -> Result<(), hal::SurfaceError> {
    //     use raw_window_handle::RawWindowHandle as Rwh;

    //     let (surface, wl_window) = match unsafe { self.unconfigure_impl(device) } {
    //         Some(pair) => pair,
    //         None => {
    //             let mut wl_window = None;
    //             let (mut temp_xlib_handle, mut temp_xcb_handle);
    //             #[allow(trivial_casts)]
    //             let native_window_ptr = match (self.wsi.kind, self.raw_window_handle) {
    //                 (WindowKind::Unknown | WindowKind::X11, Rwh::Xlib(handle)) => {
    //                     temp_xlib_handle = handle.window;
    //                     &mut temp_xlib_handle as *mut _ as *mut std::ffi::c_void
    //                 }
    //                 (WindowKind::AngleX11, Rwh::Xlib(handle)) => {
    //                     handle.window as *mut std::ffi::c_void
    //                 }
    //                 (WindowKind::Unknown | WindowKind::X11, Rwh::Xcb(handle)) => {
    //                     temp_xcb_handle = handle.window;
    //                     &mut temp_xcb_handle as *mut _ as *mut std::ffi::c_void
    //                 }
    //                 (WindowKind::AngleX11, Rwh::Xcb(handle)) => {
    //                     handle.window as *mut std::ffi::c_void
    //                 }
    //                 (WindowKind::Unknown, Rwh::AndroidNdk(handle)) => handle.a_native_window,
    //                 (WindowKind::Wayland, Rwh::Wayland(handle)) => {
    //                     let library = self.wsi.library.as_ref().unwrap();
    //                     let wl_egl_window_create: libloading::Symbol<
    //                         egl_impl::WlEglWindowCreateFun,
    //                     > = unsafe { library.get(b"wl_egl_window_create") }.unwrap();
    //                     let window = unsafe { wl_egl_window_create(handle.surface, 640, 480) }
    //                         as *mut _ as *mut std::ffi::c_void;
    //                     wl_window = Some(window);
    //                     window
    //                 }
    //                 #[cfg(feature = "emscripten")]
    //                 (WindowKind::Unknown, Rwh::Web(handle)) => handle.id as *mut std::ffi::c_void,
    //                 (WindowKind::Unknown, Rwh::Win32(handle)) => handle.hwnd,
    //                 (WindowKind::Unknown, Rwh::AppKit(handle)) => {
    //                     #[cfg(not(target_os = "macos"))]
    //                     let window_ptr = handle.ns_view;
    //                     #[cfg(target_os = "macos")]
    //                     let window_ptr = {
    //                         use objc::{msg_send, runtime::Object, sel, sel_impl};
    //                         // ns_view always have a layer and don't need to verify that it exists.
    //                         let layer: *mut Object =
    //                             msg_send![handle.ns_view as *mut Object, layer];
    //                         layer as *mut ffi::c_void
    //                     };
    //                     window_ptr
    //                 }
    //                 _ => {
    //                     log::warn!(
    //                         "Initialized platform {:?} doesn't work with window {:?}",
    //                         self.wsi.kind,
    //                         self.raw_window_handle
    //                     );
    //                     return Err(hal::SurfaceError::Other("incompatible window kind"));
    //                 }
    //             };

    //             let mut attributes = vec![
    //                 egl::RENDER_BUFFER,
    //                 // We don't want any of the buffering done by the driver, because we
    //                 // manage a swapchain on our side.
    //                 // Some drivers just fail on surface creation seeing `EGL_SINGLE_BUFFER`.
    //                 if cfg!(any(target_os = "android", target_os = "macos"))
    //                     || cfg!(windows)
    //                     || self.wsi.kind == WindowKind::AngleX11
    //                 {
    //                     egl::BACK_BUFFER
    //                 } else {
    //                     egl::SINGLE_BUFFER
    //                 },
    //             ];
    //             match self.srgb_kind {
    //                 SrgbFrameBufferKind::None => {}
    //                 SrgbFrameBufferKind::Core => {
    //                     attributes.push(egl::GL_COLORSPACE);
    //                     attributes.push(egl::GL_COLORSPACE_SRGB);
    //                 }
    //                 SrgbFrameBufferKind::Khr => {
    //                     attributes.push(egl_impl::EGL_GL_COLORSPACE_KHR as i32);
    //                     attributes.push(egl_impl::EGL_GL_COLORSPACE_SRGB_KHR as i32);
    //                 }
    //             }
    //             attributes.push(egl::ATTRIB_NONE as i32);

    //             #[cfg(not(feature = "emscripten"))]
    //             let egl1_5 = self.egl.instance.upcast::<egl::EGL1_5>();

    //             #[cfg(feature = "emscripten")]
    //             let egl1_5: Option<&Arc<EglInstance>> = Some(&self.egl.instance);

    //             // Careful, we can still be in 1.4 version even if `upcast` succeeds
    //             let raw_result = match egl1_5 {
    //                 Some(egl) if self.wsi.kind != WindowKind::Unknown => {
    //                     let attributes_usize = attributes
    //                         .into_iter()
    //                         .map(|v| v as usize)
    //                         .collect::<Vec<_>>();
    //                     egl.create_platform_window_surface(
    //                         self.egl.display,
    //                         self.config,
    //                         native_window_ptr,
    //                         &attributes_usize,
    //                     )
    //                 }
    //                 _ => unsafe {
    //                     self.egl.instance.create_window_surface(
    //                         self.egl.display,
    //                         self.config,
    //                         native_window_ptr,
    //                         Some(&attributes),
    //                     )
    //                 },
    //             };

    //             match raw_result {
    //                 Ok(raw) => (raw, wl_window),
    //                 Err(e) => {
    //                     log::warn!("Error in create_window_surface: {:?}", e);
    //                     return Err(hal::SurfaceError::Lost);
    //                 }
    //             }
    //         }
    //     };

    //     if let Some(window) = wl_window {
    //         let library = self.wsi.library.as_ref().unwrap();
    //         let wl_egl_window_resize: libloading::Symbol<egl_impl::WlEglWindowResizeFun> =
    //             unsafe { library.get(b"wl_egl_window_resize") }.unwrap();
    //         unsafe {
    //             wl_egl_window_resize(
    //                 window,
    //                 config.extent.width as i32,
    //                 config.extent.height as i32,
    //                 0,
    //                 0,
    //             )
    //         };
    //     }

    //     let format_desc = device.shared.describe_texture_format(config.format);
    //     let gl = &device.shared.context.lock();
    //     let renderbuffer = unsafe { gl.create_renderbuffer() }.unwrap();
    //     unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer)) };
    //     unsafe {
    //         gl.renderbuffer_storage(
    //             glow::RENDERBUFFER,
    //             format_desc.internal,
    //             config.extent.width as _,
    //             config.extent.height as _,
    //         )
    //     };
    //     let framebuffer = unsafe { gl.create_framebuffer() }.unwrap();
    //     unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(framebuffer)) };
    //     unsafe {
    //         gl.framebuffer_renderbuffer(
    //             glow::READ_FRAMEBUFFER,
    //             glow::COLOR_ATTACHMENT0,
    //             glow::RENDERBUFFER,
    //             Some(renderbuffer),
    //         )
    //     };
    //     unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, None) };
    //     unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None) };

    //     self.swapchain = Some(Swapchain {
    //         surface,
    //         wl_window,
    //         renderbuffer,
    //         framebuffer,
    //         extent: config.extent,
    //         format: config.format,
    //         format_desc,
    //         sample_type: wgt::TextureSampleType::Float { filterable: false },
    //     });

    //     Ok(())
    // }

    pub(crate) unsafe fn unconfigure(&mut self, device: &super::Device) {
        if let Some((surface, wl_window)) = unsafe { self.unconfigure_impl(device) } {
            let egl = self.egl.0.as_ref();

            egl.egl.destroy_surface(egl.display, surface).unwrap();
            if let Some(window) = wl_window {
                todo!()

                // let library = self.wsi.library.as_ref().expect("unsupported window");
                // let wl_egl_window_destroy: libloading::Symbol<egl_impl::WlEglWindowDestroyFun> =
                //     unsafe { library.get(b"wl_egl_window_destroy") }.unwrap();
                // unsafe { wl_egl_window_destroy(window) };
            }
        }
    }

    pub(crate) unsafe fn acquire_texture(
        &mut self,
        _timeout_ms: Option<Duration>, //TODO
    ) -> Result<Option<AcquiredSurfaceTexture<hal::GL>>, hal::SurfaceError> {
        todo!()
    }

    // pub(crate) unsafe fn acquire_texture(
    //     &mut self,
    //     _timeout_ms: Option<Duration>, //TODO
    // ) -> Result<Option<AcquiredSurfaceTexture<hal::GL>>, hal::SurfaceError> {
    //     let sc = self.swapchain.as_ref().unwrap();
    //     let texture = super::Texture {
    //         inner: super::TextureInner::Renderbuffer {
    //             raw: sc.renderbuffer,
    //         },
    //         drop_guard: None,
    //         array_layer_count: 1,
    //         mip_level_count: 1,
    //         format: sc.format,
    //         format_desc: sc.format_desc.clone(),
    //         copy_size: CopyExtent {
    //             width: sc.extent.width,
    //             height: sc.extent.height,
    //             depth: 1,
    //         },
    //         is_cubemap: false,
    //     };
    //     Ok(Some(AcquiredSurfaceTexture {
    //         texture,
    //         suboptimal: false,
    //     }))
    // }

    pub(crate) unsafe fn discard_texture(&mut self, _texture: super::Texture) {}
}

impl Surface {
    pub(crate) unsafe fn present(
        &mut self,
        _suf_texture: hal::Texture,
        gl: &glow::Context,
    ) -> Result<(), hal::SurfaceError> {
        let sc = self.swapchain.as_ref().unwrap();

        let egl = self.egl.0.as_ref();

        egl.egl
            .make_current(
                egl.display,
                Some(sc.surface),
                Some(sc.surface),
                Some(egl.raw),
            )
            .map_err(|e| {
                log::error!("make_current(surface) failed: {}", e);
                hal::SurfaceError::Lost
            })?;

        unsafe { gl.disable(glow::SCISSOR_TEST) };
        unsafe { gl.color_mask(true, true, true, true) };

        unsafe { gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None) };
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(sc.framebuffer)) };
        // Note the Y-flipping here. GL's presentation is not flipped,
        // but main rendering is. Therefore, we Y-flip the output positions
        // in the shader, and also this blit.
        unsafe {
            gl.blit_framebuffer(
                0,
                sc.extent.height as i32,
                sc.extent.width as i32,
                0,
                0,
                0,
                sc.extent.width as i32,
                sc.extent.height as i32,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            )
        };
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None) };

        egl.egl.swap_buffers(egl.display, sc.surface).map_err(|e| {
            log::error!("swap_buffers failed: {}", e);
            hal::SurfaceError::Lost
        })?;

        egl.egl
            .make_current(egl.display, None, None, None)
            .map_err(|e| {
                log::error!("make_current(null) failed: {}", e);
                hal::SurfaceError::Lost
            })?;

        Ok(())
    }

    unsafe fn unconfigure_impl(
        &mut self,
        device: &super::Device,
    ) -> Option<(egl::Surface, Option<*mut raw::c_void>)> {
        todo!();
        // let gl = &device.0.context.lock();
        // match self.swapchain.take() {
        //     Some(sc) => {
        //         unsafe { gl.delete_renderbuffer(sc.renderbuffer) };
        //         unsafe { gl.delete_framebuffer(sc.framebuffer) };
        //         Some((sc.surface, sc.wl_window))
        //     }
        //     None => None,
        // }
    }

    pub fn supports_srgb(&self) -> bool {
        match self.srgb_kind {
            SrgbFrameBufferKind::None => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Swapchain {
    surface: egl::Surface,
    wl_window: Option<*mut raw::c_void>,
    framebuffer: glow::Framebuffer,
    renderbuffer: glow::Renderbuffer,
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
