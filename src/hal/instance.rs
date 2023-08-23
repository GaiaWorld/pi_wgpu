use std::{
    os::raw,
    ptr,
    sync::{Arc, Mutex},
};

use bitflags::bitflags;
use thiserror::Error;

use super::{
    egl_impl::{EglContext, EglInstance},
    SrgbFrameBufferKind, VALIDATION_CANARY,
};
use crate::{
    hal,
    hal::{egl_debug_proc, egl_impl::choose_config},
    wgt,
};

#[derive(Debug)]
pub(crate) struct Instance {
    wsi: WindowSystemInterface,
    flags: InstanceFlags,
    inner: Mutex<Inner>,
}

impl Instance {
    pub(crate) unsafe fn init(desc: &InstanceDescriptor) -> Result<Instance, InstanceError> {
        #[cfg(feature = "emscripten")]
        let egl_result: Result<EglInstance, egl::Error> = Ok(egl::Instance::new(egl::Static));

        #[cfg(not(feature = "emscripten"))]
        let egl_result = if cfg!(windows) {
            unsafe {
                egl::DynamicInstance::<egl::EGL1_4>::load_required_from_filename("libEGL.dll")
            }
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            unsafe {
                egl::DynamicInstance::<egl::EGL1_4>::load_required_from_filename("libEGL.dylib")
            }
        } else {
            unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
        };
        let egl = match egl_result {
            Ok(egl) => Arc::new(egl),
            Err(e) => {
                log::info!("Unable to open libEGL: {:?}", e);
                return Err(InstanceError);
            }
        };

        let client_extensions = egl.query_string(None, egl::EXTENSIONS);

        let client_ext_str = match client_extensions {
            Ok(ext) => ext.to_string_lossy().into_owned(),
            Err(_) => String::new(),
        };
        log::debug!(
            "Client extensions: {:#?}",
            client_ext_str.split_whitespace().collect::<Vec<_>>()
        );

        let wayland_library = if client_ext_str.contains("EGL_EXT_platform_wayland") {
            test_wayland_display()
        } else {
            None
        };
        let x11_display_library = if client_ext_str.contains("EGL_EXT_platform_x11") {
            open_x_display()
        } else {
            None
        };
        let angle_x11_display_library = if client_ext_str.contains("EGL_ANGLE_platform_angle") {
            open_x_display()
        } else {
            None
        };

        #[cfg(not(feature = "emscripten"))]
        let egl1_5 = egl.upcast::<egl::EGL1_5>();

        #[cfg(feature = "emscripten")]
        let egl1_5: Option<&Arc<EglInstance>> = Some(&egl);

        let (display, wsi_library, wsi_kind) = if let (Some(library), Some(egl)) =
            (wayland_library, egl1_5)
        {
            log::info!("Using Wayland platform");
            let display_attributes = [egl::ATTRIB_NONE];
            let display = egl
                .get_platform_display(
                    super::egl_impl::EGL_PLATFORM_WAYLAND_KHR,
                    egl::DEFAULT_DISPLAY,
                    &display_attributes,
                )
                .unwrap();
            (display, Some(Arc::new(library)), WindowKind::Wayland)
        } else if let (Some((display, library)), Some(egl)) = (x11_display_library, egl1_5) {
            log::info!("Using X11 platform");
            let display_attributes = [egl::ATTRIB_NONE];
            let display = egl
                .get_platform_display(
                    super::egl_impl::EGL_PLATFORM_X11_KHR,
                    display.as_ptr(),
                    &display_attributes,
                )
                .unwrap();
            (display, Some(Arc::new(library)), WindowKind::X11)
        } else if let (Some((display, library)), Some(egl)) = (angle_x11_display_library, egl1_5) {
            log::info!("Using Angle platform with X11");
            let display_attributes = [
                super::egl_impl::EGL_PLATFORM_ANGLE_NATIVE_PLATFORM_TYPE_ANGLE as egl::Attrib,
                super::egl_impl::EGL_PLATFORM_X11_KHR as egl::Attrib,
                super::egl_impl::EGL_PLATFORM_ANGLE_DEBUG_LAYERS_ENABLED as egl::Attrib,
                usize::from(desc.flags.contains(InstanceFlags::VALIDATION)),
                egl::ATTRIB_NONE,
            ];
            let display = egl
                .get_platform_display(
                    super::egl_impl::EGL_PLATFORM_ANGLE_ANGLE,
                    display.as_ptr(),
                    &display_attributes,
                )
                .unwrap();
            (display, Some(Arc::new(library)), WindowKind::AngleX11)
        } else if client_ext_str.contains("EGL_MESA_platform_surfaceless") {
            log::info!("No windowing system present. Using surfaceless platform");
            let egl = egl1_5.expect("Failed to get EGL 1.5 for surfaceless");
            let display = egl
                .get_platform_display(
                    super::egl_impl::EGL_PLATFORM_SURFACELESS_MESA,
                    std::ptr::null_mut(),
                    &[egl::ATTRIB_NONE],
                )
                .unwrap();
            (display, None, WindowKind::Unknown)
        } else {
            log::info!("EGL_MESA_platform_surfaceless not available. Using default platform");
            let display = egl.get_display(egl::DEFAULT_DISPLAY).unwrap();
            (display, None, WindowKind::Unknown)
        };

        if desc.flags.contains(InstanceFlags::VALIDATION)
            && client_ext_str.contains("EGL_KHR_debug")
        {
            log::info!("Enabling EGL debug output");
            let function: super::egl_impl::EglDebugMessageControlFun = {
                let addr = egl.get_proc_address("eglDebugMessageControlKHR").unwrap();
                unsafe { std::mem::transmute(addr) }
            };
            let attributes = [
                super::egl_impl::EGL_DEBUG_MSG_CRITICAL_KHR as egl::Attrib,
                1,
                super::egl_impl::EGL_DEBUG_MSG_ERROR_KHR as egl::Attrib,
                1,
                super::egl_impl::EGL_DEBUG_MSG_WARN_KHR as egl::Attrib,
                1,
                super::egl_impl::EGL_DEBUG_MSG_INFO_KHR as egl::Attrib,
                1,
                egl::ATTRIB_NONE,
            ];
            unsafe { (function)(Some(egl_debug_proc), attributes.as_ptr()) };
        }

        let inner = Inner::create(desc.flags, egl, display)?;

        Ok(Instance {
            wsi: WindowSystemInterface {
                library: wsi_library,
                kind: wsi_kind,
            },
            flags: desc.flags,
            inner: Mutex::new(inner),
        })
    }

    pub(crate) unsafe fn enumerate_adapters(&self) -> Vec<crate::ExposedAdapter<crate::GL>> {
        let inner = self.inner.lock();
        inner.egl.make_current();

        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                inner
                    .egl
                    .instance
                    .get_proc_address(name)
                    .map_or(ptr::null(), |p| p as *const _)
            })
        };

        if self.flags.contains(InstanceFlags::DEBUG) && gl.supports_debug() {
            log::info!("Max label length: {}", unsafe {
                gl.get_parameter_i32(glow::MAX_LABEL_LENGTH)
            });
        }

        if self.flags.contains(InstanceFlags::VALIDATION) && gl.supports_debug() {
            log::info!("Enabling GLES debug output");
            unsafe { gl.enable(glow::DEBUG_OUTPUT) };
            unsafe { gl.debug_message_callback(gl_debug_message_callback) };
        }

        inner.egl.unmake_current();

        unsafe {
            super::Adapter::expose(AdapterContext {
                glow: Mutex::new(gl),
                egl: Some(inner.egl.clone()),
            })
        }
        .into_iter()
        .collect()
    }

    #[cfg_attr(target_os = "macos", allow(unused, unused_mut, unreachable_code))]
    pub(crate) unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<hal::Surface, InstanceError> {
        use raw_window_handle::RawWindowHandle as Rwh;

        #[cfg_attr(any(target_os = "android", feature = "emscripten"), allow(unused_mut))]
        let mut inner = self.inner.lock();

        match (window_handle, display_handle) {
            (Rwh::Xlib(_), _) => {}
            (Rwh::Xcb(_), _) => {}
            (Rwh::Win32(_), _) => {}
            (Rwh::AppKit(_), _) => {}
            #[cfg(target_os = "android")]
            (Rwh::AndroidNdk(handle), _) => {
                let format = inner
                    .egl
                    .instance
                    .get_config_attrib(inner.egl.display, inner.config, egl::NATIVE_VISUAL_ID)
                    .unwrap();

                let ret = unsafe {
                    ANativeWindow_setBuffersGeometry(handle.a_native_window, 0, 0, format)
                };

                if ret != 0 {
                    log::error!("Error returned from ANativeWindow_setBuffersGeometry");
                    return Err(crate::InstanceError);
                }
            }
            #[cfg(not(feature = "emscripten"))]
            (Rwh::Wayland(_), raw_window_handle::RawDisplayHandle::Wayland(display_handle)) => {
                /* Wayland displays are not sharable between surfaces so if the
                 * surface we receive from this handle is from a different
                 * display, we must re-initialize the context.
                 *
                 * See gfx-rs/gfx#3545
                 */
                log::warn!("Re-initializing Gles context due to Wayland window");
                if inner
                    .wl_display
                    .map(|ptr| ptr != display_handle.display)
                    .unwrap_or(true)
                {
                    use std::ops::DerefMut;
                    let display_attributes = [egl::ATTRIB_NONE];

                    let display = inner
                        .egl
                        .instance
                        .upcast::<egl::EGL1_5>()
                        .unwrap()
                        .get_platform_display(
                            super::egl_impl::EGL_PLATFORM_WAYLAND_KHR,
                            display_handle.display,
                            &display_attributes,
                        )
                        .unwrap();

                    let new_inner =
                        Inner::create(self.flags, Arc::clone(&inner.egl.instance), display)
                            .map_err(|_| InstanceError)?;

                    let old_inner = std::mem::replace(inner.deref_mut(), new_inner);
                    inner.wl_display = Some(display_handle.display);

                    drop(old_inner);
                }
            }
            #[cfg(feature = "emscripten")]
            (Rwh::Web(_), _) => {}
            other => {
                log::error!("Unsupported window: {:?}", other);
                return Err(InstanceError);
            }
        };

        inner.egl.unmake_current();

        Ok(hal::Surface {
            egl: inner.egl.clone(),
            wsi: self.wsi.clone(),
            config: inner.config,
            presentable: inner.supports_native_window,
            raw_window_handle: window_handle,
            swapchain: None,
            srgb_kind: inner.srgb_kind,
        })
    }

    pub(crate) unsafe fn destroy_surface(&self, _surface: hal::Surface) {}
}

#[derive(Clone, Debug)]
pub(crate) struct WindowSystemInterface {
    pub(crate) library: Option<Arc<libloading::Library>>,
    pub(crate) kind: WindowKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum WindowKind {
    Wayland,
    X11,
    AngleX11,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("Not supported")]
pub(crate) struct InstanceError;

bitflags!(
    /// Instance initialization flags.
    pub(crate) struct InstanceFlags: u32 {
        /// Generate debug information in shaders and objects.
        const DEBUG = 1 << 0;
        /// Enable validation, if possible.
        const VALIDATION = 1 << 1;
    }
);

#[derive(Clone, Debug)]
pub(crate) struct InstanceDescriptor<'a> {
    pub name: &'a str,
    pub flags: InstanceFlags,
    pub dx12_shader_compiler: wgt::Dx12Compiler,
}

#[derive(Debug)]
struct Inner {
    /// Note: the context contains a dummy pbuffer (1x1).
    /// Required for `eglMakeCurrent` on platforms that doesn't supports `EGL_KHR_surfaceless_context`.
    egl: EglContext,
    #[allow(unused)]
    version: (i32, i32),
    supports_native_window: bool,
    config: egl::Config,
    #[cfg_attr(feature = "emscripten", allow(dead_code))]
    wl_display: Option<*mut raw::c_void>,
    /// Method by which the framebuffer should support srgb
    srgb_kind: SrgbFrameBufferKind,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Err(e) = self
            .egl
            .instance
            .destroy_context(self.egl.display, self.egl.raw)
        {
            log::warn!("Error in destroy_context: {:?}", e);
        }
        if let Err(e) = self.egl.instance.terminate(self.egl.display) {
            log::warn!("Error in terminate: {:?}", e);
        }
    }
}

impl Inner {
    fn create(
        flags: InstanceFlags,
        egl: Arc<EglInstance>,
        display: egl::Display,
    ) -> Result<Self, InstanceError> {
        let version = egl.initialize(display).map_err(|_| InstanceError)?;
        let vendor = egl.query_string(Some(display), egl::VENDOR).unwrap();
        let display_extensions = egl
            .query_string(Some(display), egl::EXTENSIONS)
            .unwrap()
            .to_string_lossy();
        log::info!("Display vendor {:?}, version {:?}", vendor, version,);
        log::debug!(
            "Display extensions: {:#?}",
            display_extensions.split_whitespace().collect::<Vec<_>>()
        );

        let srgb_kind = if version >= (1, 5) {
            log::info!("\tEGL surface: +srgb");
            SrgbFrameBufferKind::Core
        } else if display_extensions.contains("EGL_KHR_gl_colorspace") {
            log::info!("\tEGL surface: +srgb khr");
            SrgbFrameBufferKind::Khr
        } else {
            log::warn!("\tEGL surface: -srgb");
            SrgbFrameBufferKind::None
        };

        if log::max_level() >= log::LevelFilter::Trace {
            log::trace!("Configurations:");
            let config_count = egl.get_config_count(display).unwrap();
            let mut configurations = Vec::with_capacity(config_count);
            egl.get_configs(display, &mut configurations).unwrap();
            for &config in configurations.iter() {
                log::trace!("\tCONFORMANT=0x{:X}, RENDERABLE=0x{:X}, NATIVE_RENDERABLE=0x{:X}, SURFACE_TYPE=0x{:X}, ALPHA_SIZE={}",
                    egl.get_config_attrib(display, config, egl::CONFORMANT).unwrap(),
                    egl.get_config_attrib(display, config, egl::RENDERABLE_TYPE).unwrap(),
                    egl.get_config_attrib(display, config, egl::NATIVE_RENDERABLE).unwrap(),
                    egl.get_config_attrib(display, config, egl::SURFACE_TYPE).unwrap(),
                    egl.get_config_attrib(display, config, egl::ALPHA_SIZE).unwrap(),
                );
            }
        }

        let (config, supports_native_window) = choose_config(&egl, display, srgb_kind)?;
        egl.bind_api(egl::OPENGL_ES_API).unwrap();

        let needs_robustness = true;
        let mut khr_context_flags = 0;
        let supports_khr_context = display_extensions.contains("EGL_KHR_create_context");

        //TODO: make it so `Device` == EGL Context
        let mut context_attributes = vec![
            egl::CONTEXT_CLIENT_VERSION,
            3, // Request GLES 3.0 or higher
        ];
        if flags.contains(InstanceFlags::DEBUG) {
            if version >= (1, 5) {
                log::info!("\tEGL context: +debug");
                context_attributes.push(egl::CONTEXT_OPENGL_DEBUG);
                context_attributes.push(egl::TRUE as _);
            } else if supports_khr_context {
                log::info!("\tEGL context: +debug KHR");
                khr_context_flags |= super::egl_impl::EGL_CONTEXT_OPENGL_DEBUG_BIT_KHR;
            } else {
                log::info!("\tEGL context: -debug");
            }
        }
        if needs_robustness {
            //Note: the core version can fail if robustness is not supported
            // (regardless of whether the extension is supported!).
            // In fact, Angle does precisely that awful behavior, so we don't try it there.
            if version >= (1, 5) && !display_extensions.contains("EGL_ANGLE_") {
                log::info!("\tEGL context: +robust access");
                context_attributes.push(egl::CONTEXT_OPENGL_ROBUST_ACCESS);
                context_attributes.push(egl::TRUE as _);
            } else if display_extensions.contains("EGL_EXT_create_context_robustness") {
                log::info!("\tEGL context: +robust access EXT");
                context_attributes.push(super::egl_impl::EGL_CONTEXT_OPENGL_ROBUST_ACCESS_EXT);
                context_attributes.push(egl::TRUE as _);
            } else {
                //Note: we aren't trying `EGL_CONTEXT_OPENGL_ROBUST_ACCESS_BIT_KHR`
                // because it's for desktop GL only, not GLES.
                log::warn!("\tEGL context: -robust access");
            }

            //TODO do we need `egl::CONTEXT_OPENGL_NOTIFICATION_STRATEGY_EXT`?
        }
        if khr_context_flags != 0 {
            context_attributes.push(super::egl_impl::EGL_CONTEXT_FLAGS_KHR);
            context_attributes.push(khr_context_flags);
        }
        context_attributes.push(egl::NONE);
        let context = match egl.create_context(display, config, None, &context_attributes) {
            Ok(context) => context,
            Err(e) => {
                log::warn!("unable to create GLES 3.x context: {:?}", e);
                return Err(InstanceError);
            }
        };

        // Testing if context can be binded without surface
        // and creating dummy pbuffer surface if not.
        let pbuffer = if version >= (1, 5)
            || display_extensions.contains("EGL_KHR_surfaceless_context")
            || cfg!(feature = "emscripten")
        {
            log::info!("\tEGL context: +surfaceless");
            None
        } else {
            let attributes = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
            egl.create_pbuffer_surface(display, config, &attributes)
                .map(Some)
                .map_err(|e| {
                    log::warn!("Error in create_pbuffer_surface: {:?}", e);
                    InstanceError
                })?
        };

        Ok(Self {
            egl: EglContext {
                instance: egl,
                display,
                raw: context,
                pbuffer,
                version,
            },
            version,
            supports_native_window,
            config,
            wl_display: None,
            srgb_kind,
        })
    }
}

fn open_x_display() -> Option<(ptr::NonNull<raw::c_void>, libloading::Library)> {
    log::info!("Loading X11 library to get the current display");
    unsafe {
        let library = libloading::Library::new("libX11.so").ok()?;
        let func: libloading::Symbol<super::egl_impl::XOpenDisplayFun> =
            library.get(b"XOpenDisplay").unwrap();
        let result = func(ptr::null());
        ptr::NonNull::new(result).map(|ptr| (ptr, library))
    }
}

fn test_wayland_display() -> Option<libloading::Library> {
    /* We try to connect and disconnect here to simply ensure there
     * is an active wayland display available.
     */
    log::info!("Loading Wayland library to get the current display");
    let library = unsafe {
        let client_library = find_library(&["libwayland-client.so.0", "libwayland-client.so"])?;
        let wl_display_connect: libloading::Symbol<super::egl_impl::WlDisplayConnectFun> =
            client_library.get(b"wl_display_connect").unwrap();
        let wl_display_disconnect: libloading::Symbol<super::egl_impl::WlDisplayDisconnectFun> =
            client_library.get(b"wl_display_disconnect").unwrap();
        let display = ptr::NonNull::new(wl_display_connect(ptr::null()))?;
        wl_display_disconnect(display.as_ptr());
        find_library(&["libwayland-egl.so.1", "libwayland-egl.so"])?
    };
    Some(library)
}

unsafe fn find_library(paths: &[&str]) -> Option<libloading::Library> {
    for path in paths {
        match unsafe { libloading::Library::new(path) } {
            Ok(lib) => return Some(lib),
            _ => continue,
        };
    }
    None
}

fn gl_debug_message_callback(source: u32, gltype: u32, id: u32, severity: u32, message: &str) {
    let source_str = match source {
        glow::DEBUG_SOURCE_API => "API",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "ShaderCompiler",
        glow::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        glow::DEBUG_SOURCE_APPLICATION => "Application",
        glow::DEBUG_SOURCE_OTHER => "Other",
        _ => unreachable!(),
    };

    let log_severity = match severity {
        glow::DEBUG_SEVERITY_HIGH => log::Level::Error,
        glow::DEBUG_SEVERITY_MEDIUM => log::Level::Warn,
        glow::DEBUG_SEVERITY_LOW => log::Level::Info,
        glow::DEBUG_SEVERITY_NOTIFICATION => log::Level::Trace,
        _ => unreachable!(),
    };

    let type_str = match gltype {
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
        glow::DEBUG_TYPE_ERROR => "Error",
        glow::DEBUG_TYPE_MARKER => "Marker",
        glow::DEBUG_TYPE_OTHER => "Other",
        glow::DEBUG_TYPE_PERFORMANCE => "Performance",
        glow::DEBUG_TYPE_POP_GROUP => "Pop Group",
        glow::DEBUG_TYPE_PORTABILITY => "Portability",
        glow::DEBUG_TYPE_PUSH_GROUP => "Push Group",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
        _ => unreachable!(),
    };

    let _ = std::panic::catch_unwind(|| {
        log::log!(
            log_severity,
            "GLES: [{}/{}] ID {} : {}",
            source_str,
            type_str,
            id,
            message
        );
    });

    if cfg!(debug_assertions) && log_severity == log::Level::Error {
        // Set canary and continue
        VALIDATION_CANARY.set();
    }
}
