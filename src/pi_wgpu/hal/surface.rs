use glow::HasContext;
use pi_share::{Share, ShareCell, cell::Ref};
use thiserror::Error;

use super::{
    super::{wgt, DeviceError, MissingDownlevelFlags},
    gl_conv as conv, AdapterContext, SrgbFrameBufferKind,
};

#[derive(Debug, Clone)]
pub(crate) struct Surface {
    imp: Share<ShareCell<SurfaceImpl>>,
}

impl Surface {
    #[inline]
    pub(crate) fn new(
        adapter: AdapterContext,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Self, super::InstanceError> {
        SurfaceImpl::new(adapter, window_handle).map(|imp| Self {
            imp: Share::new(ShareCell::new(imp)),
        })
    }

    #[inline]
    pub(crate) fn update_swapchain(&self) {
        self.imp.as_ref().borrow_mut().update_swapchain()
    }

    #[inline]
    pub(crate) fn get_presentable(&self) -> bool {
        self.imp.as_ref().borrow().swapchain.is_none()
    }

    #[inline]
    pub(crate) fn configure(
        &self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        self.imp.as_ref().borrow_mut().configure(device, config)
    }

    #[inline]
    pub(crate) fn adapter(&self) -> Ref<'_, AdapterContext> {
        self.imp.as_ref().borrow().map(|s| &s.adapter)
    }

    #[inline]
    pub(crate) fn acquire_texture(&self) -> Option<super::Texture> {
        self.imp.as_ref().borrow_mut().acquire_texture()
    }

    #[inline]
    pub(crate) fn supports_srgb(&self) -> bool {
        self.imp.as_ref().borrow().supports_srgb()
    }
}

#[derive(Debug)]
struct SurfaceImpl {
    raw: egl::Surface,
    adapter: AdapterContext,

    // 永远握住这个
    swapchain_impl: super::TextureImpl,

    // 当 present 之后，这里就会有新的值，供acquire_texture取
    swapchain: Option<super::Texture>,
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

        Ok(Self {
            raw,
            adapter,
            swapchain: None,
            swapchain_impl: Self::default_swapchain(),
        })
    }

    fn configure(
        &mut self,
        device: &super::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        device.adapter.set_surface(Some(self.raw));

        let sc = &mut self.swapchain_impl;
        sc.format = config.format;
        sc.format_desc = conv::describe_texture_format(config.format);

        let size = &mut sc.copy_size;
        size.width = config.width;
        size.height = config.height;

        Ok(())
    }

    #[inline]
    fn acquire_texture(&mut self) -> Option<super::Texture> {
        self.swapchain.take()
    }
}

impl SurfaceImpl {
    #[inline]
    fn update_swapchain(&mut self) {
        if self.swapchain.is_none() {
            let imp = self.swapchain_impl.clone();
            self.swapchain = Some(super::Texture(Share::new(imp)));
        }
    }

    #[inline]
    fn default_swapchain() -> super::TextureImpl {
        let format = wgt::TextureFormat::Rgba8Unorm;
        let format_desc = conv::describe_texture_format(format);

        super::TextureImpl {
            inner: super::TextureInner::DefaultRenderbuffer,
            array_layer_count: 1,
            mip_level_count: 1,
            format,
            format_desc,
            copy_size: super::CopyExtent {
                width: 1,
                height: 1,
                depth: 1,
            },
            is_cubemap: false,
        }
    }

    #[inline]
    fn supports_srgb(&self) -> bool {
        match self.adapter.egl_srgb_support() {
            SrgbFrameBufferKind::None => false,
            _ => true,
        }
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
