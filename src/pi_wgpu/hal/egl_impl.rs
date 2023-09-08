use std::{
    ffi,
    os::raw,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use glow::HasContext;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use pi_share::Share;

use super::{InstanceError, SrgbFrameBufferKind, VALIDATION_CANARY};

/// The amount of time to wait while trying to obtain a lock to the adapter context
pub(crate) const CONTEXT_LOCK_TIMEOUT_SECS: u64 = 1;

pub(crate) const EGL_CONTEXT_FLAGS_KHR: i32 = 0x30FC;
pub(crate) const EGL_CONTEXT_OPENGL_DEBUG_BIT_KHR: i32 = 0x0001;
#[allow(unused)]
pub(crate) const EGL_CONTEXT_OPENGL_ROBUST_ACCESS_EXT: i32 = 0x30BF;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_WAYLAND_KHR: u32 = 0x31D8;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_X11_KHR: u32 = 0x31D5;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_ANGLE_ANGLE: u32 = 0x3202;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_ANGLE_NATIVE_PLATFORM_TYPE_ANGLE: u32 = 0x348F;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_ANGLE_DEBUG_LAYERS_ENABLED: u32 = 0x3451;
#[allow(unused)]
pub(crate) const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;
pub(crate) const EGL_GL_COLORSPACE_KHR: u32 = 0x309D;
pub(crate) const EGL_GL_COLORSPACE_SRGB_KHR: u32 = 0x3089;

pub(crate) const EGL_DEBUG_MSG_CRITICAL_KHR: u32 = 0x33B9;
pub(crate) const EGL_DEBUG_MSG_ERROR_KHR: u32 = 0x33BA;
pub(crate) const EGL_DEBUG_MSG_WARN_KHR: u32 = 0x33BB;
pub(crate) const EGL_DEBUG_MSG_INFO_KHR: u32 = 0x33BC;

#[allow(unused)]
pub(crate) type XOpenDisplayFun =
    unsafe extern "system" fn(display_name: *const raw::c_char) -> *mut raw::c_void;

#[allow(unused)]
pub(crate) type WlDisplayConnectFun =
    unsafe extern "system" fn(display_name: *const raw::c_char) -> *mut raw::c_void;

#[allow(unused)]
pub(crate) type WlDisplayDisconnectFun = unsafe extern "system" fn(display: *const raw::c_void);

pub(crate) type EglDebugMessageControlFun =
    unsafe extern "system" fn(proc: EGLDEBUGPROCKHR, attrib_list: *const egl::Attrib) -> raw::c_int;

pub(crate) type EglLabel = *const raw::c_void;

#[allow(unused)]
pub(crate) type WlEglWindowResizeFun = unsafe extern "system" fn(
    window: *const raw::c_void,
    width: raw::c_int,
    height: raw::c_int,
    dx: raw::c_int,
    dy: raw::c_int,
);

#[allow(unused)]
pub(crate) type WlEglWindowCreateFun = unsafe extern "system" fn(
    surface: *const raw::c_void,
    width: raw::c_int,
    height: raw::c_int,
) -> *mut raw::c_void;

#[allow(unused)]
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

#[derive(Debug)]
pub(crate) struct EglContext {
    pub(crate) instance: EglInstance,

    pub(crate) raw: egl::Context, // gl-上下文，用来 运行 gl-函数

    #[allow(unused)]
    pub(crate) config: egl::Config,

    pub(crate) display: egl::Display,

    pub(crate) version: (i32, i32), // EGL 版本, (1, 5) 或者 (1, 4)
    #[allow(unused)]
    pub(crate) srgb_kind: SrgbFrameBufferKind,
}

unsafe impl Send for EglContext {}

unsafe impl Sync for EglContext {}

impl Drop for EglContext {
    fn drop(&mut self) {
        if let Err(e) = self.instance.destroy_context(self.display, self.raw) {
            log::error!("Error in destroy_context: {:?}", e);
        }
        if let Err(e) = self.instance.terminate(self.display) {
            log::error!("Error in terminate: {:?}", e);
        }
    }
}

impl EglContext {
    pub(crate) fn new(
        flags: super::InstanceFlags,
        egl: EglInstance,
        display: egl::Display,
    ) -> Result<Self, InstanceError> {
        // ========== 1. 初始化 EGL，得到 EGL版本
        let version = egl.initialize(display).map_err(|_| InstanceError)?;

        // ========== 2. 取 厂商 信息
        let vendor = egl.query_string(Some(display), egl::VENDOR).unwrap();

        log::info!("EGL Display vendor {:?}, version {:?}", vendor, version,);

        // ========== 3. 取 EGL 扩展
        let display_extensions = egl
            .query_string(Some(display), egl::EXTENSIONS)
            .unwrap()
            .to_string_lossy();
        log::info!(
            "EGL Display extensions: {:#?}",
            display_extensions.split_whitespace().collect::<Vec<_>>()
        );

        // ========== 4. 查询 表面的 Srgb 支持情况

        let srgb_kind = if version >= (1, 5) {
            log::info!("\tEGL surface: support srgb core");
            SrgbFrameBufferKind::Core
        } else if display_extensions.contains("EGL_KHR_gl_colorspace") {
            log::info!("\tEGL surface: support srgb khr extension");
            SrgbFrameBufferKind::Khr
        } else {
            log::warn!("\tEGL surface: no srgb support !!!");
            SrgbFrameBufferKind::None
        };

        // ========== 5. 如果 log 过滤等级是 Debug 或者 Trace，就会打印

        if log::max_level() >= log::LevelFilter::Debug {
            log::debug!("EGL All Configurations:");

            let config_count = egl.get_config_count(display).unwrap();
            let mut configurations = Vec::with_capacity(config_count);

            egl.get_configs(display, &mut configurations).unwrap();

            for &config in configurations.iter() {
                log::debug!("\t EGL Conformant = 0x{:X}, Renderable = 0x{:X}, Native_Renderable = 0x{:X}, Surface_Type = 0x{:X}, Alpha_Size = {}",
                        // EGL_OPENGL_BIT: 表示该配置支持OpenGL
                        // EGL_OPENGL_ES_BIT: 表示该配置支持OpenGL ES 1.x
                        // EGL_OPENGL_ES2_BIT: 表示该配置支持OpenGL ES 2.x
                        // EGL_OPENGL_ES3_BIT_KHR: 表示该配置支持OpenGL ES 3.x
                        egl.get_config_attrib(display, config, egl::CONFORMANT).unwrap(),
                        // EGL_OPENGL_BIT: 支持OpenGL
                        // EGL_OPENGL_ES_BIT: 支持OpenGL ES 1.x
                        // EGL_OPENGL_ES2_BIT: 支持OpenGL ES 2.x
                        // EGL_OPENGL_ES3_BIT_KHR: 支持OpenGL ES 3.x
                        egl.get_config_attrib(display, config, egl::RENDERABLE_TYPE).unwrap(),
                        // EGL_TRUE: 可用硬件渲染
                        // EGL_FALSE: 不能用硬件渲染
                        egl.get_config_attrib(display, config, egl::NATIVE_RENDERABLE).unwrap(),
                        // EGL_WINDOW_BIT 支持 窗口表面
                        // EGL_PBUFFER_BIT: 支持 像素缓冲区 表面（pbuffer surfaces）。
                        // EGL_PIXMAP_BIT: 支持 Pixmap 表面。
                        // EGL_MULTISAMPLE_RESOLVE_BOX_BIT: 支持多重采样解析。
                        // EGL_SWAP_BEHAVIOR_PRESERVED_BIT: 支持保存交换行为。
                        egl.get_config_attrib(display, config, egl::SURFACE_TYPE).unwrap(),
                        // 返回 0 表示 无 alpha 通道，返回 8 表示 使用 1B 保存 alpha 通道
                        egl.get_config_attrib(display, config, egl::ALPHA_SIZE).unwrap(),
                    );
            }
        }

        // ========== 6. 根据平台，选择 config

        let config = choose_config(&egl, display, srgb_kind)?;

        egl.bind_api(egl::OPENGL_ES_API).unwrap();

        // ========== 7. 选择 Context 属性

        let mut khr_context_flags = 0;
        let supports_khr_context = display_extensions.contains("EGL_KHR_create_context");

        let mut context_attributes = vec![
            egl::CONTEXT_CLIENT_VERSION,
            3, // 必须 GLES 3+
        ];

        // TODO 小米9，加上 egl::CONTEXT_OPENGL_DEBUG 之后，会崩溃；wgc层屏蔽掉了 DEBUG
        if flags.contains(super::InstanceFlags::DEBUG) {
            if version >= (1, 5) {
                log::info!("\t EGL context: Support Debug Core");
                context_attributes.push(egl::CONTEXT_OPENGL_DEBUG);
                context_attributes.push(egl::TRUE as _);
            } else if supports_khr_context {
                log::info!("\tEGL context: Support Debug KHR");
                khr_context_flags |= super::egl_impl::EGL_CONTEXT_OPENGL_DEBUG_BIT_KHR;
            } else {
                log::info!("\tEGL context: No Support debug");
            }
        }

        if khr_context_flags != 0 {
            context_attributes.push(super::egl_impl::EGL_CONTEXT_FLAGS_KHR);
            context_attributes.push(khr_context_flags);
        }

        context_attributes.push(egl::NONE);

        // ========== 8. 创建 Context，这里是 GLES-3

        let raw = match egl.create_context(display, config, None, &context_attributes) {
            Ok(context) => context,
            Err(e) => {
                log::warn!("unable to create GLES 3.x context: {:?}", e);
                return Err(InstanceError);
            }
        };

        Ok(Self {
            config,
            instance: egl,
            display,
            raw,
            version,
            srgb_kind,
        })
    }

    pub(crate) fn create_glow_context(&self, flags: super::InstanceFlags) -> glow::Context {
        // =========== 1. 让当前的 Surface 起作用

        self.instance
            .make_current(self.display, None, None, Some(self.raw))
            .unwrap();

        // =========== 2. 取 glow 环境

        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                self.instance
                    .get_proc_address(name)
                    .map_or(ptr::null(), |p| p as *const _)
            })
        };

        if flags.contains(super::InstanceFlags::DEBUG) && gl.supports_debug() {
            log::info!("Max label length: {}", unsafe {
                gl.get_parameter_i32(glow::MAX_LABEL_LENGTH)
            });
        }

        if flags.contains(super::InstanceFlags::VALIDATION) && gl.supports_debug() {
            log::info!("Enabling GLES debug output");
            unsafe { gl.enable(glow::DEBUG_OUTPUT) };
            unsafe { gl.debug_message_callback(gl_debug_message_callback) };
        }

        // =========== 3. 解绑表面

        self.instance
            .make_current(self.display, None, None, None)
            .unwrap();

        gl
    }
}

impl EglContext {
    pub(crate) fn make_current(&self) {
        self.instance
            .make_current(self.display, None, None, Some(self.raw))
            .unwrap();
    }

    pub(crate) fn unmake_current(&self) {
        self.instance
            .make_current(self.display, None, None, None)
            .unwrap();
    }
}

#[cfg(not(feature = "emscripten"))]
pub(crate) type EglInstance = egl::DynamicInstance<egl::EGL1_4>;

#[cfg(feature = "emscripten")]
type EglInstance = egl::Instance<egl::Static>;

/// A wrapper around a [`glow::Context`] and the required EGL context that uses locking to guarantee
/// exclusive access when shared with multiple threads.
#[derive(Debug)]
pub(crate) struct AdapterContext {
    pub(crate) egl: EglContext,
    pub(crate) reentrant_count: Share<AtomicUsize>,
    pub(crate) glow: ReentrantMutex<glow::Context>, // 可重入锁，为了性能
}

unsafe impl Sync for AdapterContext {}
unsafe impl Send for AdapterContext {}

impl AdapterContext {
    /// Returns the EGL instance.
    ///
    /// This provides access to EGL functions and the ability to load GL and EGL extension functions.
    #[inline]
    pub(crate) fn egl_ref(&self) -> &EglContext {
        &self.egl
    }

    /// Returns the EGL instance.
    ///
    /// This provides access to EGL functions and the ability to load GL and EGL extension functions.
    #[inline]
    #[allow(unused)]
    pub(crate) fn egl_instance(&self) -> &EglInstance {
        &self.egl.instance
    }

    /// Returns the EGLDisplay corresponding to the adapter context.
    ///
    /// Returns [`None`] if the adapter was externally created.
    #[inline]
    #[allow(unused)]
    pub(crate) fn raw_display(&self) -> &egl::Display {
        &self.egl.display
    }

    /// Returns the EGL version the adapter context was created with.
    ///
    /// Returns [`None`] if the adapter was externally created.
    #[inline]
    #[allow(unused)]
    pub(crate) fn egl_version(&self) -> (i32, i32) {
        self.egl.version
    }
}

#[derive(Debug)]
struct EglContextLock<'a> {
    instance: &'a EglInstance,
    display: egl::Display,
}

/// A guard containing a lock to an [`AdapterContext`]
#[derive(Debug)]
pub struct AdapterContextLock<'a> {
    reentrant_count: Share<AtomicUsize>,
    glow: ReentrantMutexGuard<'a, glow::Context>,
    egl: EglContextLock<'a>,
}

impl<'a> std::ops::Deref for AdapterContextLock<'a> {
    type Target = glow::Context;

    fn deref(&self) -> &Self::Target {
        &self.glow
    }
}

impl<'a> Drop for AdapterContextLock<'a> {
    #[inline]
    fn drop(&mut self) {
        self.reentrant_count.fetch_sub(1, Ordering::SeqCst);

        if self.reentrant_count.load(Ordering::SeqCst) == 0 {
            self.egl
                .instance
                .make_current(self.egl.display, None, None, None)
                .unwrap();
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
    #[allow(unused)]
    pub unsafe fn get_without_egl_lock(&self) -> ReentrantMutexGuard<glow::Context> {
        self.glow
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.")
    }

    /// Obtain a lock to the EGL context and get handle to the [`glow::Context`] that can be used to
    /// do rendering.
    #[track_caller]
    pub fn lock<'a>(&'a self) -> AdapterContextLock<'a> {
        let glow = self
            .glow
            // Don't lock forever. If it takes longer than 1 second to get the lock we've got a
            // deadlock and should panic to show where we got stuck
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.");

        if self.reentrant_count.load(Ordering::SeqCst) == 0 {
            self.egl.make_current();
        }
        self.reentrant_count.fetch_add(1, Ordering::SeqCst);

        let egl = EglContextLock {
            instance: &self.egl.instance,
            display: self.egl.display,
        };

        let reentrant_count = self.reentrant_count.clone();
        AdapterContextLock {
            glow,
            egl,
            reentrant_count,
        }
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
    #[allow(unused)] srgb_kind: SrgbFrameBufferKind,
) -> Result<egl::Config, InstanceError> {
    let mut attributes = Vec::with_capacity(20);

    attributes.extend_from_slice(&[
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_ES2_BIT,
    ]);

    // TODO 待定，看不到为什么 srgb就一定要有alpha通道
    // if srgb_kind != SrgbFrameBufferKind::None {
    //     attributes.push(egl::ALPHA_SIZE);
    //     attributes.push(8);
    // }

    #[cfg(not(target_os = "android"))]
    attributes.extend_from_slice(&[egl::NATIVE_RENDERABLE, egl::TRUE as _]);

    attributes.push(egl::NONE);

    let mut config = match egl.choose_first_config(display, &attributes[..]) {
        Ok(Some(config)) => Ok(config),
        Ok(None) => {
            log::error!("1 in choose_first_config: Missing config");
            Err(InstanceError)
        }
        Err(e) => {
            log::error!("1 error in choose_first_config: {:?}", e);
            Err(InstanceError)
        }
    };

    #[cfg(not(target_os = "android"))]
    if config.is_err() {
        attributes.clear();

        attributes.extend_from_slice(&[
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT, // 最低支持 gles2，到创建 context 才指定 gles3
        ]);

        // TODO 待定，看不到为什么 srgb就一定要有alpha通道
        // if srgb_kind != SrgbFrameBufferKind::None {
        //     attributes.push(egl::ALPHA_SIZE);
        //     attributes.push(8);
        // }

        attributes.push(egl::NONE);

        config = match egl.choose_first_config(display, &attributes[..]) {
            Ok(Some(config)) => Ok(config),
            Ok(None) => {
                log::error!("2 error in choose_first_config: Missing config");
                Err(InstanceError)
            }
            Err(e) => {
                log::error!("2 error in choose_first_config: {:?}", e);
                Err(InstanceError)
            }
        };
    }

    config
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
