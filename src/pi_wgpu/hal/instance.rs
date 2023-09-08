use std::sync::atomic::AtomicUsize;

use bitflags::bitflags;
use parking_lot::ReentrantMutex;
use pi_share::Share;
use thiserror::Error;

use crate::pi_wgpu::hal::AdapterContext;

use super::super::{hal::egl_debug_proc, wgt};
use super::egl_impl::EglContext;

#[derive(Debug)]
pub(crate) struct Instance {
    context: Share<AdapterContext>,
    flags: InstanceFlags,
}

unsafe impl Sync for Instance {}
unsafe impl Send for Instance {}

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
            Ok(egl) => egl,
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

        // ========= 3. 优先使用 EGL 1.5 接口

        #[cfg(not(feature = "emscripten"))]
        let egl1_5 = egl.upcast::<egl::EGL1_5>();

        #[cfg(feature = "emscripten")]
        let egl1_5: Option<&EglInstance> = Some(&egl);

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

        let gl = context.create_glow_context(desc.flags);

        let context = AdapterContext {
            egl: context,
            reentrant_count: Share::new(AtomicUsize::new(0)),
            glow: ReentrantMutex::new(gl),
        };

        Ok(Instance {
            flags: desc.flags,
            context: Share::new(context),
        })
    }

    // EGL 所谓的 枚举显卡，实际上是 取 系统默认设置的显卡！
    // 这里的迭代器，只返回一个值
    #[inline]
    pub(crate) unsafe fn enumerate_adapters(&self) -> Vec<super::super::ExposedAdapter<super::GL>> {
        unsafe { super::Adapter::expose(self.context.clone()) }
            .into_iter()
            .collect()
    }

    pub(crate) unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<super::Surface, super::InstanceError> {
        use raw_window_handle::RawWindowHandle as Rwh;

        let inner = self.context.egl_ref();

        match (window_handle, display_handle) {
            (Rwh::Win32(_), _) => {}
            #[cfg(target_os = "android")]
            (Rwh::AndroidNdk(handle), _) => {
                let format = inner
                    .instance
                    .get_config_attrib(inner.display, inner.config, egl::NATIVE_VISUAL_ID)
                    .unwrap();

                let ret = unsafe {
                    ANativeWindow_setBuffersGeometry(handle.a_native_window, 0, 0, format)
                };

                if ret != 0 {
                    log::error!("Error returned from ANativeWindow_setBuffersGeometry");
                    return Err(crate::InstanceError);
                }
            }
            #[cfg(feature = "emscripten")]
            (Rwh::Web(_), _) => {}
            other => {
                log::error!("Unsupported window: {:?}", other);
                return Err(super::InstanceError);
            }
        };

        inner.unmake_current();

        todo!()
        // Ok(super::Surface {
        //     egl: self.context.clone(),
        //     config: inner.config,
        //     presentable: inner.supports_native_window,
        //     raw_window_handle: window_handle,
        //     swapchain: None,
        //     srgb_kind: inner.srgb_kind,
        // })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("Not supported")]
pub(crate) struct InstanceError;

bitflags!(
    /// Instance initialization flags.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
