use std::{os::raw, ptr};

use bitflags::bitflags;
use glow::HasContext;
use pi_share::Share;
use thiserror::Error;

use super::{egl_impl::EglContext, GLState, SrgbFrameBufferKind, VALIDATION_CANARY};
use crate::{hal::egl_debug_proc, wgt};

#[derive(Debug)]
pub(crate) struct Instance {
    context: EglContext,
    flags: InstanceFlags,
}

impl Instance {
    pub(crate) unsafe fn init(desc: &InstanceDescriptor) -> Result<Instance, InstanceError> {
        // ========= 1. 加载 EGL 库，初始化 EGL

        #[cfg(not(feature = "emscripten"))]
        let egl_result = if cfg!(windows) {
            unsafe {
                // windows 会到走这里来
                egl::DynamicInstance::<egl::EGL1_4>::load_required_from_filename("libEGL.dll")
            }
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            unsafe {
                egl::DynamicInstance::<egl::EGL1_4>::load_required_from_filename("libEGL.dylib")
            }
        } else {
            // Android 会到走这里来
            // TODO wasm32-unknown-unknown 大概会走到这里来？
            unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
        };

        let egl = match egl_result {
            Ok(egl) => Share::new(egl),
            Err(e) => {
                log::info!("Unable to open libEGL: {:?}", e);
                return Err(InstanceError);
            }
        };

        // ========= 2. 查询 EGL扩展

        let client_extensions = egl.query_string(None, egl::EXTENSIONS);

        let client_ext_str = match client_extensions {
            Ok(ext) => ext.to_string_lossy().into_owned(),
            Err(_) => String::new(),
        };
        log::debug!(
            "EGL Extensions: {:#?}",
            client_ext_str.split_whitespace().collect::<Vec<_>>()
        );

        if client_ext_str.contains("EGL_EXT_platform_wayland") {
            unimplemented!("Linux / Unix: Wayland is not supported yet");
        }

        if client_ext_str.contains("EGL_EXT_platform_x11") {
            unimplemented!("Linux / Unix: X11 is not supported yet");
        }

        // ========= 3. 使用 EGL 1.5 接口

        #[cfg(not(feature = "emscripten"))]
        let egl1_5 = egl.upcast::<egl::EGL1_5>();

        #[cfg(feature = "emscripten")]
        let egl1_5: Option<&Arc<EglInstance>> = Some(&egl);

        // ========= 4. 取 EglDisplay

        let display = egl.get_display(egl::DEFAULT_DISPLAY).unwrap();

        // ========= 5. 如果参数带验证，而且 egl 含 Debug 扩展，就使用它

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

        // ========= 6. 创建 EglContext

        let context = EglContext::new(desc.flags, egl, display)?;

        Ok(Instance {
            flags: desc.flags,
            context,
        })
    }

    // EGL 所谓的 枚举显卡，实际上是 取 系统默认设置的显卡！
    // 这里的迭代器，只返回一个值
    #[inline]
    pub(crate) unsafe fn enumerate_adapters(&self) -> Vec<crate::ExposedAdapter<super::GL>> {
        let inner = self.context.0.as_ref();

        unsafe { super::Adapter::expose(&inner.state) }
            .into_iter()
            .collect()
    }
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

#[inline]
unsafe fn find_library(paths: &[&str]) -> Option<libloading::Library> {
    for path in paths {
        match unsafe { libloading::Library::new(path) } {
            Ok(lib) => return Some(lib),
            _ => continue,
        };
    }
    None
}

