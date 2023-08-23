use std::{ffi, os::raw, sync::Arc, ptr, time::Duration};

use parking_lot::{Mutex, MutexGuard};

use super::{SrgbFrameBufferKind, InstanceError};

/// The amount of time to wait while trying to obtain a lock to the adapter context
pub(crate) const CONTEXT_LOCK_TIMEOUT_SECS: u64 = 1;

pub(crate) const EGL_CONTEXT_FLAGS_KHR: i32 = 0x30FC;
pub(crate) const EGL_CONTEXT_OPENGL_DEBUG_BIT_KHR: i32 = 0x0001;
pub(crate) const EGL_CONTEXT_OPENGL_ROBUST_ACCESS_EXT: i32 = 0x30BF;
pub(crate) const EGL_PLATFORM_WAYLAND_KHR: u32 = 0x31D8;
pub(crate) const EGL_PLATFORM_X11_KHR: u32 = 0x31D5;
pub(crate) const EGL_PLATFORM_ANGLE_ANGLE: u32 = 0x3202;
pub(crate) const EGL_PLATFORM_ANGLE_NATIVE_PLATFORM_TYPE_ANGLE: u32 = 0x348F;
pub(crate) const EGL_PLATFORM_ANGLE_DEBUG_LAYERS_ENABLED: u32 = 0x3451;
pub(crate) const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;
pub(crate) const EGL_GL_COLORSPACE_KHR: u32 = 0x309D;
pub(crate) const EGL_GL_COLORSPACE_SRGB_KHR: u32 = 0x3089;

pub(crate) const EGL_DEBUG_MSG_CRITICAL_KHR: u32 = 0x33B9;
pub(crate) const EGL_DEBUG_MSG_ERROR_KHR: u32 = 0x33BA;
pub(crate) const EGL_DEBUG_MSG_WARN_KHR: u32 = 0x33BB;
pub(crate) const EGL_DEBUG_MSG_INFO_KHR: u32 = 0x33BC;

pub(crate) type XOpenDisplayFun =
    unsafe extern "system" fn(display_name: *const raw::c_char) -> *mut raw::c_void;

pub(crate) type WlDisplayConnectFun =
    unsafe extern "system" fn(display_name: *const raw::c_char) -> *mut raw::c_void;

pub(crate) type WlDisplayDisconnectFun = unsafe extern "system" fn(display: *const raw::c_void);

pub(crate) type EglDebugMessageControlFun =
    unsafe extern "system" fn(proc: EGLDEBUGPROCKHR, attrib_list: *const egl::Attrib) -> raw::c_int;

pub(crate) type EglLabel = *const raw::c_void;

pub(crate) type WlEglWindowResizeFun = unsafe extern "system" fn(
    window: *const raw::c_void,
    width: raw::c_int,
    height: raw::c_int,
    dx: raw::c_int,
    dy: raw::c_int,
);

pub(crate) type WlEglWindowCreateFun = unsafe extern "system" fn(
    surface: *const raw::c_void,
    width: raw::c_int,
    height: raw::c_int,
) -> *mut raw::c_void;

pub(crate) type WlEglWindowDestroyFun = unsafe extern "system" fn(window: *const raw::c_void);

#[allow(clippy::upper_case_acronyms)]
pub(crate) type EGLDEBUGPROCKHR = Option<
    unsafe extern "system" fn(
        error: egl::Enum,
        command: *const raw::c_char,
        message_type: u32,
        thread_label: EglLabel,
        object_label: EglLabel,
        message: *const raw::c_char,
    ),
>;

/// A wrapper around a [`glow::Context`] and the required EGL context that uses locking to guarantee
/// exclusive access when shared with multiple threads.
#[derive(Debug)]
pub(crate) struct AdapterContext {
    glow: Mutex<glow::Context>,
    egl: Option<EglContext>,

    pub(crate) copy_fbo: glow::Framebuffer,
    pub(crate) draw_fbo: glow::Framebuffer,
}

unsafe impl Sync for AdapterContext {}
unsafe impl Send for AdapterContext {}

impl AdapterContext {
    pub fn is_owned(&self) -> bool {
        self.egl.is_some()
    }

    /// Returns the EGL instance.
    ///
    /// This provides access to EGL functions and the ability to load GL and EGL extension functions.
    pub fn egl_instance(&self) -> Option<&EglInstance> {
        self.egl.as_ref().map(|egl| &*egl.instance)
    }

    /// Returns the EGLDisplay corresponding to the adapter context.
    ///
    /// Returns [`None`] if the adapter was externally created.
    pub fn raw_display(&self) -> Option<&egl::Display> {
        self.egl.as_ref().map(|egl| &egl.display)
    }

    /// Returns the EGL version the adapter context was created with.
    ///
    /// Returns [`None`] if the adapter was externally created.
    pub fn egl_version(&self) -> Option<(i32, i32)> {
        self.egl.as_ref().map(|egl| egl.version)
    }

    pub fn raw_context(&self) -> *mut raw::c_void {
        match self.egl {
            Some(ref egl) => egl.raw.as_ptr(),
            None => ptr::null_mut(),
        }
    }
}

impl AdapterContext {
    /// Get's the [`glow::Context`] without waiting for a lock
    ///
    /// # Safety
    ///
    /// This should only be called when you have manually made sure that the current thread has made
    /// the EGL context current and that no other thread also has the EGL context current.
    /// Additionally, you must manually make the EGL context **not** current after you are done with
    /// it, so that future calls to `lock()` will not fail.
    ///
    /// > **Note:** Calling this function **will** still lock the [`glow::Context`] which adds an
    /// > extra safe-guard against accidental concurrent access to the context.
    pub(crate) unsafe fn get_without_egl_lock(&self) -> MutexGuard<glow::Context> {
        self.glow
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.")
    }

    /// Obtain a lock to the EGL context and get handle to the [`glow::Context`] that can be used to
    /// do rendering.
    #[track_caller]
    pub(crate) fn lock<'a>(&'a self) -> AdapterContextLock<'a> {
        let glow = self
            .glow
            // Don't lock forever. If it takes longer than 1 second to get the lock we've got a
            // deadlock and should panic to show where we got stuck
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.");

        let egl = self.egl.as_ref().map(|egl| {
            egl.make_current();
            EglContextLock {
                instance: &egl.instance,
                display: egl.display,
            }
        });

        AdapterContextLock { glow, egl }
    }
}

struct EglContextLock<'a> {
    instance: &'a Arc<EglInstance>,
    display: egl::Display,
}

/// A guard containing a lock to an [`AdapterContext`]
pub struct AdapterContextLock<'a> {
    glow: MutexGuard<'a, glow::Context>,
    egl: Option<EglContextLock<'a>>,
}

impl<'a> std::ops::Deref for AdapterContextLock<'a> {
    type Target = glow::Context;

    fn deref(&self) -> &Self::Target {
        &self.glow
    }
}

impl<'a> Drop for AdapterContextLock<'a> {
    fn drop(&mut self) {
        if let Some(egl) = self.egl.take() {
            egl.instance
                .make_current(egl.display, None, None, None)
                .unwrap();
        }
    }
}


#[derive(Clone, Debug)]
pub(crate) struct EglContext {
    pub(crate) instance: Arc<EglInstance>,
    pub(crate) version: (i32, i32),
    pub(crate) display: egl::Display,
    pub(crate) raw: egl::Context,
    pub(crate) pbuffer: Option<egl::Surface>,
}

#[cfg(not(feature = "emscripten"))]
pub(crate) type EglInstance = egl::DynamicInstance<egl::EGL1_4>;

#[cfg(feature = "emscripten")]
type EglInstance = egl::Instance<egl::Static>;

impl EglContext {
    pub(crate) fn make_current(&self) {
        self.instance
            .make_current(self.display, self.pbuffer, self.pbuffer, Some(self.raw))
            .unwrap();
    }
    pub(crate) fn unmake_current(&self) {
        self.instance
            .make_current(self.display, None, None, None)
            .unwrap();
    }
}

pub(crate) unsafe extern "system" fn egl_debug_proc(
    error: egl::Enum,
    command_raw: *const raw::c_char,
    message_type: u32,
    _thread_label: super::egl_impl::EglLabel,
    _object_label: super::egl_impl::EglLabel,
    message_raw: *const raw::c_char,
) {
    let log_severity = match message_type {
        super::egl_impl::EGL_DEBUG_MSG_CRITICAL_KHR | super::egl_impl::EGL_DEBUG_MSG_ERROR_KHR => {
            log::Level::Error
        }
        EGL_DEBUG_MSG_WARN_KHR => log::Level::Warn,
        EGL_DEBUG_MSG_INFO_KHR => log::Level::Info,
        _ => log::Level::Debug,
    };
    let command = unsafe { ffi::CStr::from_ptr(command_raw) }.to_string_lossy();
    let message = if message_raw.is_null() {
        "".into()
    } else {
        unsafe { ffi::CStr::from_ptr(message_raw) }.to_string_lossy()
    };

    log::log!(
        log_severity,
        "EGL '{}' code 0x{:x}: {}",
        command,
        error,
        message,
    );
}

/// Choose GLES framebuffer configuration.
pub(super) fn choose_config(
    egl: &EglInstance,
    display: egl::Display,
    srgb_kind: SrgbFrameBufferKind,
) -> Result<(egl::Config, bool), InstanceError> {
    //TODO: EGL_SLOW_CONFIG
    let tiers = [
        (
            "off-screen",
            &[
                egl::SURFACE_TYPE,
                egl::PBUFFER_BIT,
                egl::RENDERABLE_TYPE,
                egl::OPENGL_ES2_BIT,
            ][..],
        ),
        ("presentation", &[egl::SURFACE_TYPE, egl::WINDOW_BIT][..]),
        #[cfg(not(target_os = "android"))]
        (
            "native-render",
            &[egl::NATIVE_RENDERABLE, egl::TRUE as _][..],
        ),
    ];

    let mut attributes = Vec::with_capacity(9);
    for tier_max in (0..tiers.len()).rev() {
        let name = tiers[tier_max].0;
        log::info!("\tTrying {}", name);

        attributes.clear();
        for &(_, tier_attr) in tiers[..=tier_max].iter() {
            attributes.extend_from_slice(tier_attr);
        }
        // make sure the Alpha is enough to support sRGB
        match srgb_kind {
            SrgbFrameBufferKind::None => {}
            _ => {
                attributes.push(egl::ALPHA_SIZE);
                attributes.push(8);
            }
        }
        attributes.push(egl::NONE);

        match egl.choose_first_config(display, &attributes) {
            Ok(Some(config)) => {
                if tier_max == 1 {
                    //Note: this has been confirmed to malfunction on Intel+NV laptops,
                    // but also on Angle.
                    log::warn!("EGL says it can present to the window but not natively",);
                }
                // Android emulator can't natively present either.
                let tier_threshold = if cfg!(target_os = "android") || cfg!(windows) {
                    1
                } else {
                    2
                };
                return Ok((config, tier_max >= tier_threshold));
            }
            Ok(None) => {
                log::warn!("No config found!");
            }
            Err(e) => {
                log::error!("error in choose_first_config: {:?}", e);
            }
        }
    }

    Err(InstanceError)
}

