use std::{ffi, os::raw, ptr};

use glow::HasContext;
use pi_share::Share;

use super::{GLState, InstanceError, SrgbFrameBufferKind, VALIDATION_CANARY};

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

#[derive(Debug, Clone)]
pub(crate) struct EglContext(pub(crate) Share<EglContextImpl>);

impl EglContext {
    #[inline]
    pub(crate) fn new(
        flags: super::InstanceFlags,
        egl: Share<EglInstance>,
        display: egl::Display,
    ) -> Result<Self, InstanceError> {
        let imp = EglContextImpl::new(flags, egl, display)?;

        Ok(Self(Share::new(imp)))
    }
}

#[derive(Debug)]
pub(crate) struct EglContextImpl {
    pub(crate) state: GLState,
    pub(crate) egl: Share<EglInstance>,

    pub(crate) raw: egl::Context,
    pub(crate) display: egl::Display,

    pub(crate) version: (i32, i32),
    pub(crate) srgb_kind: SrgbFrameBufferKind,
}

unsafe impl Send for EglContextImpl {}

unsafe impl Sync for EglContextImpl {}

impl Drop for EglContextImpl {
    fn drop(&mut self) {
        if let Err(e) = self.egl.destroy_context(self.display, self.raw) {
            log::error!("Error in destroy_context: {:?}", e);
        }
        if let Err(e) = self.egl.terminate(self.display) {
            log::error!("Error in terminate: {:?}", e);
        }
    }
}

impl EglContextImpl {
    pub(crate) fn new(
        flags: super::InstanceFlags,
        egl: Share<EglInstance>,
        display: egl::Display,
    ) -> Result<Self, InstanceError> {
        // ========== 1. 初始化 EGL
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

        let (config, supports_native_window) = choose_config(&egl, display, srgb_kind)?;
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

        let gl = Self::create_gl_context(egl.as_ref(), raw, display, flags);
        let state = super::GLState::new(gl);

        Ok(Self {
            state,

            egl,
            display,

            raw,
            version,
            srgb_kind,
        })
    }

    pub(crate) fn create_gl_context(
        egl: &EglInstance,
        raw: egl::Context,
        display: egl::Display,
        flags: super::InstanceFlags,
    ) -> glow::Context {
        // =========== 1. 让当前的 Surface 起作用

        egl.make_current(display, None, None, Some(raw)).unwrap();

        // =========== 2. 取 glow 环境

        let gl = unsafe {
            glow::Context::from_loader_function(|name| {
                egl.get_proc_address(name)
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

        egl.make_current(display, None, None, None).unwrap();

        gl
    }
}

#[cfg(not(feature = "emscripten"))]
pub(crate) type EglInstance = egl::DynamicInstance<egl::EGL1_4>;

#[cfg(feature = "emscripten")]
type EglInstance = egl::Instance<egl::Static>;

impl EglContextImpl {
    pub(crate) fn make_current(&self) {
        self.egl
            .make_current(self.display, None, None, Some(self.raw))
            .unwrap();
    }

    pub(crate) fn unmake_current(&self) {
        self.egl
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
        log::info!("\tEGL choose_config: Trying {}", name);

        attributes.clear();

        for &(_, tier_attr) in tiers[..=tier_max].iter() {
            attributes.extend_from_slice(tier_attr);
        }

        // 如果 支持 SRGB，那么 必须有 ALpha8
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
                    1 // Android 或 Windows 的话，大于 第一层 就认为是 Native Renderable
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
