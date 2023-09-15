use std::thread;

use glow::HasContext;
use parking_lot::Mutex;
use pi_share::Share;
use thiserror::Error;

use super::{
    super::{wgt, DeviceError, MissingDownlevelFlags},
    AdapterContext, GLState, SrgbFrameBufferKind,
};

#[derive(Debug, Clone)]
pub(crate) struct Surface {
    imp: Share<Mutex<SurfaceImpl>>,
}

impl Surface {
    #[inline]
    pub(crate) fn new(
        adapter: AdapterContext,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Self, super::InstanceError> {
        SurfaceImpl::new(adapter, window_handle).map(|imp| Self {
            imp: Share::new(Mutex::new(imp)),
        })
    }

    #[inline]
    pub(crate) fn present(&self) -> Result<(), egl::Error> {
        log::trace!(
            "========== Surface::present lock, thread_id = {:?}",
            thread::current().id()
        );

        {
            let mut imp = self.imp.as_ref().lock();

            let is_ready = unsafe { imp.flip_surface() };
            if is_ready {
                let r = imp.adapter.swap_buffers();
                imp.update_current_texture();
            }
        }

        log::trace!(
            "========== Surface::present unlock, thread_id = {:?}",
            thread::current().id()
        );

        Ok(())
    }

    #[inline]
    pub(crate) fn configure(
        &self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        log::trace!(
            "========== Surface::configure lock, thread_id = {:?}",
            thread::current().id()
        );

        if config.width == 0 || config.height == 0 {
            log::warn!(
                "hal::Surface::configure() has 0 dimensions, size = ({}, {})",
                config.width,
                config.height
            );

            return Ok(());
        }

        let r = { self.imp.as_ref().lock().configure(device, config) };

        log::trace!(
            "========== Surface::configure unlock, thread_id = {:?}",
            thread::current().id()
        );

        r
    }

    #[inline]
    pub(crate) fn acquire_texture(&self) -> Option<super::Texture> {
        log::trace!(
            "========== Surface::acquire_texture lock, thread_id = {:?}",
            thread::current().id()
        );

        let r = { self.imp.as_ref().lock().acquire_texture() };

        log::trace!(
            "========== Surface::acquire_texture unlock, thread_id = {:?}",
            thread::current().id()
        );

        r
    }
}

#[derive(Debug)]
struct SurfaceImpl {
    raw: egl::Surface,
    gl_fbo: glow::Framebuffer,

    state: Option<GLState>,
    adapter: AdapterContext,

    // 用于 Screen Coordinate: Flip-Y
    // configure 时候会 创建 / 销毁
    texture_size: (i32, i32),
    texture: Option<super::Texture>,

    // 初始化 有值
    // 每次 acquire_texture 就为 None
    // present 后 会重新 有值
    current_texture: Option<super::Texture>,
}

impl Drop for SurfaceImpl {
    fn drop(&mut self) {
        self.adapter.remove_surface(self.raw);

        let egl = self.adapter.egl_ref();
        egl.instance.destroy_surface(egl.display, self.raw);
    }
}

unsafe impl Sync for SurfaceImpl {}
unsafe impl Send for SurfaceImpl {}

impl SurfaceImpl {
    fn new(
        adapter: AdapterContext,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Self, super::InstanceError> {
        use raw_window_handle::RawWindowHandle as Rwh;

        #[allow(trivial_casts)]
        let native_window_ptr = match window_handle {
            Rwh::Win32(handle) => handle.hwnd,
            Rwh::AndroidNdk(handle) => handle.a_native_window,
            #[cfg(feature = "emscripten")]
            Rwh::Web(handle) => handle.id as *mut std::ffi::c_void,
            _ => {
                log::error!(
                    "Initialized platform doesn't work with window {:?}",
                    window_handle
                );
                return Err(super::InstanceError);
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

        match adapter.egl_srgb_support() {
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

        log::trace!(
            "============== create_window_surface attributes = {:?}, srgb = {:?}",
            attributes,
            adapter.egl_srgb_support()
        );

        let raw = {
            let inner = adapter.egl_ref();

            #[cfg(not(feature = "emscripten"))]
            let egl1_5 = inner.instance.upcast::<egl::EGL1_5>();

            #[cfg(feature = "emscripten")]
            let egl1_5: Option<&Arc<EglInstance>> = Some(&inner.instance);

            unsafe {
                inner
                    .instance
                    .create_window_surface(
                        inner.display,
                        inner.config.clone(),
                        native_window_ptr,
                        Some(&attributes),
                    )
                    .map_err(|_| super::InstanceError)?
            }
        };

        let gl_fbo = unsafe {
            let gl = adapter.lock();
            gl.create_framebuffer().unwrap()
        };

        Ok(Self {
            raw,
            state: None,
            adapter,

            gl_fbo,

            texture_size: (0, 0),
            texture: None,

            current_texture: None,
        })
    }

    unsafe fn flip_surface(&self) -> bool {
        if self.state.is_none() {
            return false;
        }

        let gl = self.adapter.lock();
        self.state.as_ref().unwrap().flip_surface(
            &gl,
            self.gl_fbo,
            self.texture_size.0,
            self.texture_size.1,
        );

        true
    }

    fn configure(
        &mut self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        // TODO 处理 wgt::TextureFormat::Bgra8UnormSrgb
        let format = match config.format {
            wgt::TextureFormat::Rgba8Unorm => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8Unorm => wgt::TextureFormat::Bgra8Unorm,
            wgt::TextureFormat::Rgba8UnormSrgb => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8UnormSrgb => wgt::TextureFormat::Bgra8Unorm,
            _ => unreachable!(),
        };

        self.state = Some(device.state.clone());
        device.adapter.set_surface(Some(self.raw));

        let need_update_texture = match self.texture.as_ref() {
            None => true,
            Some(sc) => {
                let size = sc.0.as_ref().copy_size;

                size.width != config.width || size.height != config.height
            }
        };

        if need_update_texture {
            log::info!("============ hal::Surface::configure() create_surface_texture, w = {}, h={}, format = {:?}", config.width, config.height, format);

            self.acquire_texture();

            let texture =
                Self::create_surface_texture(device, config.width, config.height, format)?;

            let rb_raw = match &texture.0.as_ref().inner {
                super::TextureInner::Renderbuffer { raw, .. } => *raw,
                _ => {
                    log::error!(
                        "texture inner is not RenderBuffer, inner = {:#?}",
                        &texture.0.as_ref().inner
                    );
                    unreachable!()
                }
            };

            self.texture = Some(texture);
            self.texture_size = (config.width as i32, config.height as i32);

            // 绑定 fbo 和 texture
            unsafe {
                let gl = device.adapter.lock();

                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.gl_fbo));

                gl.framebuffer_renderbuffer(
                    glow::DRAW_FRAMEBUFFER,
                    glow::COLOR_ATTACHMENT0,
                    glow::RENDERBUFFER,
                    Some(rb_raw),
                );

                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            }

            self.update_current_texture();
        }

        Ok(())
    }

    #[inline]
    fn update_current_texture(&mut self) {
        assert!(self.current_texture.is_none());

        self.current_texture = Some(self.texture.as_ref().unwrap().clone());
    }

    #[inline]
    fn acquire_texture(&mut self) -> Option<super::Texture> {
        let r = self.current_texture.take();

        log::trace!("======== hal::Surface acquire_texture = {:#?}", r);

        r
    }

    fn create_surface_texture(
        device: &super::Device,
        width: u32,
        height: u32,
        format: wgt::TextureFormat,
    ) -> Result<super::Texture, super::SurfaceError> {
        let desc = super::super::TextureDescriptor {
            label: None,
            size: super::super::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgt::TextureDimension::D2,
            format,
            usage: wgt::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        device
            .create_texture(&desc)
            .map_err(|e| super::SurfaceError::Other("create_texture error!"))
    }
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
