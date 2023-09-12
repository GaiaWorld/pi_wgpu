use std::{
    ffi,
    os::raw,
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use glow::HasContext;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use pi_share::{Share, ShareCell};

use super::{db, InstanceError, PrivateCapabilities, SrgbFrameBufferKind};
use crate::{pi_wgpu::wgt, AdapterInfo};

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
    pub(crate) config: egl::Config,
    pub(crate) display: egl::Display,
    pub(crate) version: (i32, i32), // EGL 版本, (1, 5) 或者 (1, 4)
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

        gl
    }
}

impl EglContext {
    pub(crate) fn make_current(&self, surface: Option<egl::Surface>) {
        self.instance
            .make_current(self.display, surface, surface, Some(self.raw))
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
struct AdapterContextImpl {
    reentrant_count: Share<AtomicUsize>,
    glow: ReentrantMutex<glow::Context>, // 可重入锁，为了性能

    egl: EglContext,
    private_caps: PrivateCapabilities,
    features: wgt::Features,
    limits: wgt::Limits,
    downlevel: wgt::DownlevelCapabilities,

    max_texture_size: u32,
    shading_language_version: naga::back::glsl::Version,
    info: AdapterInfo,
}

unsafe impl Sync for AdapterContextImpl {}
unsafe impl Send for AdapterContextImpl {}

#[derive(Debug, Clone)]
pub(crate) struct AdapterContext {
    imp: Share<AdapterContextImpl>,
    surface: Share<ShareCell<Option<egl::Surface>>>,
}

unsafe impl Sync for AdapterContext {}
unsafe impl Send for AdapterContext {}

impl AdapterContext {
    #[inline]
    pub(crate) fn new(gl: glow::Context, context: EglContext) -> Option<Self> {
        let extensions = gl.supported_extensions();
        log::info!("GL Extensions: {:#?}", extensions);

        log::info!("glow version: {:#?}", gl.version());

        // ========== 1. 版本，必须大于等于 3.0

        let version = unsafe { gl.get_parameter_string(glow::VERSION) };
        log::info!("GL Version: {}", version);

        let ver = Self::parse_version(&version).ok()?;
        if ver < (3, 0) {
            log::info!(
                "Returned GLES context is {}.{}, when 3.0+ was requested",
                ver.0,
                ver.1
            );
            return None;
        }

        // ========== 2. 厂商

        let vendor_const = glow::VENDOR;
        let renderer_const = glow::RENDERER;

        let (vendor, renderer) = {
            let vendor = unsafe { gl.get_parameter_string(vendor_const) };
            let renderer = unsafe { gl.get_parameter_string(renderer_const) };

            (vendor, renderer)
        };
        log::info!("GL Renderer: {}", renderer);
        log::info!("GL Vendor: {}", vendor);

        // ========== 3. glsl shader 版本

        let shading_language_version = {
            let sl_version = unsafe { gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION) };

            log::info!("GLSL version: {}", &sl_version);

            let (sl_major, sl_minor) = Self::parse_version(&sl_version).ok()?;

            let value = sl_major as u16 * 100 + sl_minor as u16 * 10;
            naga::back::glsl::Version::Embedded {
                version: value,
                is_webgl: cfg!(target_arch = "wasm32"),
            }
        };

        // ANGLE provides renderer strings like: "ANGLE (Apple, Apple M1 Pro, OpenGL 4.1)"
        let is_angle = renderer.contains("ANGLE");

        // 和 storage 有关的，webgl2 都不支持
        let supports_storage = false;
        let vertex_ssbo_false_zero = false;
        let vertex_shader_storage_blocks = 0;
        let fragment_shader_storage_blocks = 0;
        let vertex_shader_storage_textures = 0;
        let fragment_shader_storage_textures = 0;
        let max_storage_block_size = 0;
        let max_storage_buffers_per_shader_stage = 0;
        let max_storage_textures_per_shader_stage = 0;

        // draw_index 能使用的最大索引的数量
        let max_element_index = unsafe { gl.get_parameter_i32(glow::MAX_ELEMENT_INDEX) } as u32;

        let mut downlevel_flags = wgt::DownlevelFlags::empty()
            | wgt::DownlevelFlags::NON_POWER_OF_TWO_MIPMAPPED_TEXTURES
            // TODO | wgt::DownlevelFlags::CUBE_ARRAY_TEXTURES
            | wgt::DownlevelFlags::COMPARISON_SAMPLERS;

        // WebGL2 不支持: 计算着色器
        downlevel_flags.set(wgt::DownlevelFlags::COMPUTE_SHADERS, false);
        // WebGL2 不支持: Storage
        downlevel_flags.set(wgt::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE, false);
        // WebGL2 不支持: Storage
        downlevel_flags.set(wgt::DownlevelFlags::VERTEX_STORAGE, false);
        // WebGL2 不支持: Storage
        downlevel_flags.set(wgt::DownlevelFlags::FRAGMENT_STORAGE, false);
        // WebGL2 不支持: 间接渲染
        downlevel_flags.set(wgt::DownlevelFlags::INDIRECT_EXECUTION, false);
        // WebGL2 不支持: base_vertex
        downlevel_flags.set(wgt::DownlevelFlags::BASE_VERTEX, false);
        // WebGL2 不支持: 为每个ColorAttachment 单独指定 Blend
        downlevel_flags.set(wgt::DownlevelFlags::INDEPENDENT_BLEND, false);
        // 各向异性过滤，看扩展
        downlevel_flags.set(
            wgt::DownlevelFlags::ANISOTROPIC_FILTERING,
            extensions.contains("EXT_texture_filter_anisotropic"),
        );
        // 绑定 buffer时，offset 和 size 不必 16字节对齐
        // 现在设置的是：webgl 和 angle 必须 16B 对齐！
        downlevel_flags.set(
            wgt::DownlevelFlags::BUFFER_BINDINGS_NOT_16_BYTE_ALIGNED,
            !(cfg!(target_arch = "wasm32") || is_angle),
        );
        // see https://registry.khronos.org/webgl/specs/latest/2.0/#BUFFER_OBJECT_BINDING
        downlevel_flags.set(wgt::DownlevelFlags::UNRESTRICTED_INDEX_BUFFER, false);
        downlevel_flags.set(
            wgt::DownlevelFlags::UNRESTRICTED_EXTERNAL_TEXTURE_COPIES,
            false,
        );
        // 索引取全部的 u32
        downlevel_flags.set(
            wgt::DownlevelFlags::FULL_DRAW_INDEX_UINT32,
            max_element_index == u32::MAX,
        );

        // TODO | wgt::Features::CLEAR_TEXTURE
        // TODO | wgt::Features::PUSH_CONSTANTS
        let mut features =
            wgt::Features::empty() | wgt::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        // 不支持：纹理坐标寻址 Border
        features.set(
            wgt::Features::ADDRESS_MODE_CLAMP_TO_BORDER | wgt::Features::ADDRESS_MODE_CLAMP_TO_ZERO,
            false,
        );
        // 不支持：Depth Clip
        features.set(wgt::Features::DEPTH_CLIP_CONTROL, false);
        // 不支持：Storage
        features.set(wgt::Features::VERTEX_WRITABLE_STORAGE, false);
        // 不支持: 扩展 "OVR_multiview2"
        features.set(wgt::Features::MULTIVIEW, false);
        // 不支持: 集合着色器
        features.set(wgt::Features::SHADER_PRIMITIVE_INDEX, false);
        // DDS 支持
        let gles_bcn_exts = [
            "GL_EXT_texture_compression_s3tc_srgb",
            "GL_EXT_texture_compression_rgtc",
            "GL_EXT_texture_compression_bptc",
        ];
        let webgl_bcn_exts = [
            "WEBGL_compressed_texture_s3tc",
            "WEBGL_compressed_texture_s3tc_srgb",
            "EXT_texture_compression_rgtc",
            "EXT_texture_compression_bptc",
        ];
        let bcn_exts = if cfg!(target_arch = "wasm32") {
            &webgl_bcn_exts[..]
        } else {
            &gles_bcn_exts[..]
        };
        features.set(
            wgt::Features::TEXTURE_COMPRESSION_BC,
            bcn_exts.iter().all(|&ext| extensions.contains(ext)),
        );
        // 不支持 ETC2 压缩纹理
        features.set(
            wgt::Features::TEXTURE_COMPRESSION_ETC2,
            false,
            // This is a part of GLES-3 but not WebGL2 core
            // !cfg!(target_arch = "wasm32") || extensions.contains("WEBGL_compressed_texture_etc"),
        );
        // 支持 ASTC 压缩纹理
        // `OES_texture_compression_astc` provides 2D + 3D, LDR + HDR support
        if extensions.contains("WEBGL_compressed_texture_astc")
            || extensions.contains("GL_OES_texture_compression_astc")
        {
            features.insert(wgt::Features::TEXTURE_COMPRESSION_ASTC);
            features.insert(wgt::Features::TEXTURE_COMPRESSION_ASTC_HDR);
        } else {
            features.set(
                wgt::Features::TEXTURE_COMPRESSION_ASTC,
                extensions.contains("GL_KHR_texture_compression_astc_ldr"),
            );
            features.set(
                wgt::Features::TEXTURE_COMPRESSION_ASTC_HDR,
                extensions.contains("GL_KHR_texture_compression_astc_hdr"),
            );
        }

        let mut private_caps = super::PrivateCapabilities::empty();
        private_caps.set(
            super::PrivateCapabilities::BUFFER_ALLOCATION,
            extensions.contains("GL_EXT_buffer_storage"),
        );
        private_caps.set(super::PrivateCapabilities::SHADER_BINDING_LAYOUT, false);
        private_caps.set(
            super::PrivateCapabilities::SHADER_TEXTURE_SHADOW_LOD,
            false,
            // extensions.contains("GL_EXT_texture_shadow_lod"),
        );
        private_caps.set(super::PrivateCapabilities::MEMORY_BARRIERS, false);
        private_caps.set(super::PrivateCapabilities::VERTEX_BUFFER_LAYOUT, false);
        private_caps.set(super::PrivateCapabilities::INDEX_BUFFER_ROLE_CHANGE, false);
        private_caps.set(super::PrivateCapabilities::CAN_DISABLE_DRAW_BUFFER, false);
        private_caps.set(super::PrivateCapabilities::GET_BUFFER_SUB_DATA, false);
        let color_buffer_float = extensions.contains("GL_EXT_color_buffer_float")
            || extensions.contains("EXT_color_buffer_float");

        let color_buffer_half_float = extensions.contains("GL_EXT_color_buffer_half_float");

        private_caps.set(
            super::PrivateCapabilities::COLOR_BUFFER_HALF_FLOAT,
            // color_buffer_half_float || color_buffer_float,
            false,
        );
        private_caps.set(
            super::PrivateCapabilities::COLOR_BUFFER_FLOAT,
            false, // color_buffer_float,
        );
        private_caps.set(
            super::PrivateCapabilities::TEXTURE_FLOAT_LINEAR,
            false, // extensions.contains("OES_texture_float_linear"),
        );

        let max_texture_size = unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) } as u32;

        let max_texture_3d_size = unsafe { gl.get_parameter_i32(glow::MAX_3D_TEXTURE_SIZE) } as u32;

        let min_uniform_buffer_offset_alignment =
            (unsafe { gl.get_parameter_i32(glow::UNIFORM_BUFFER_OFFSET_ALIGNMENT) } as u32);

        let min_storage_buffer_offset_alignment = 0;

        let max_uniform_buffers_per_shader_stage =
            unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_UNIFORM_BLOCKS) }
                .min(unsafe { gl.get_parameter_i32(glow::MAX_FRAGMENT_UNIFORM_BLOCKS) })
                as u32;

        let supports_work_group_params = false;
        let max_compute_workgroups_per_dimension = 0;

        let limits = wgt::Limits {
            max_texture_dimension_1d: max_texture_size,
            max_texture_dimension_2d: max_texture_size,
            max_texture_dimension_3d: max_texture_3d_size,
            max_texture_array_layers: unsafe {
                gl.get_parameter_i32(glow::MAX_ARRAY_TEXTURE_LAYERS)
            } as u32,
            max_bind_groups: super::MAX_BIND_GROUPS as u32,
            max_bindings_per_bind_group: 65535,
            max_dynamic_uniform_buffers_per_pipeline_layout: max_uniform_buffers_per_shader_stage,
            max_dynamic_storage_buffers_per_pipeline_layout: max_storage_buffers_per_shader_stage,
            max_sampled_textures_per_shader_stage: super::MAX_TEXTURE_SLOTS as u32,
            max_samplers_per_shader_stage: super::MAX_SAMPLERS as u32,
            max_storage_buffers_per_shader_stage,
            max_storage_textures_per_shader_stage,
            max_uniform_buffers_per_shader_stage,
            max_uniform_buffer_binding_size: unsafe {
                gl.get_parameter_i32(glow::MAX_UNIFORM_BLOCK_SIZE)
            } as u32,
            max_storage_buffer_binding_size: 0,
            max_vertex_buffers: if private_caps
                .contains(super::PrivateCapabilities::VERTEX_BUFFER_LAYOUT)
            {
                unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIB_BINDINGS) as u32 }
            } else {
                unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) as u32 }
            },
            max_vertex_attributes: (unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) }
                as u32)
                .min(super::MAX_VERTEX_ATTRIBUTES as u32),
            max_vertex_buffer_array_stride: if private_caps
                .contains(super::PrivateCapabilities::VERTEX_BUFFER_LAYOUT)
            {
                (unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIB_STRIDE) } as u32)
            } else {
                !0
            },
            max_push_constant_size: 0, // super::MAX_PUSH_CONSTANTS as u32 * 4,
            min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment,
            max_inter_stage_shader_components: unsafe {
                gl.get_parameter_i32(glow::MAX_VARYING_COMPONENTS)
            } as u32,
            max_compute_workgroup_storage_size: 0,
            max_compute_invocations_per_workgroup: 0,
            max_compute_workgroup_size_x: 0,
            max_compute_workgroup_size_y: 0,
            max_compute_workgroup_size_z: 0,
            max_compute_workgroups_per_dimension,
            max_buffer_size: i32::MAX as u64,
        };

        let downlevel_defaults = wgt::DownlevelLimits {};

        let info = Self::make_info(vendor, renderer);

        let downlevel = wgt::DownlevelCapabilities {
            flags: downlevel_flags,
            limits: downlevel_defaults,
            shader_model: wgt::ShaderModel::Sm5,
        };

        let imp = AdapterContextImpl {
            egl: context,
            reentrant_count: Share::new(AtomicUsize::new(0)),
            glow: ReentrantMutex::new(gl),

            private_caps,
            features,
            limits,
            downlevel,
            max_texture_size,
            shading_language_version,
            info,
        };

        Some(Self {
            imp: Share::new(imp),
            surface: Share::new(ShareCell::new(None)),
        })
    }

    #[inline]
    pub(crate) fn set_surface(&self, surface: Option<egl::Surface>) {
        *self.surface.as_ref().borrow_mut() = surface;
    }

    #[inline]
    pub(crate) fn remove_surface(&self, surface: egl::Surface) {
        let need_remove = match self.surface.as_ref().borrow().as_ref() {
            Some(s) => *s == surface,
            None => false,
        };

        if need_remove {
            self.set_surface(None);
        }
    }

    #[inline]
    pub(crate) fn swap_buffers(&self) -> Result<(), egl::Error> {
        if let Some(surface) = self.surface.as_ref().borrow().as_ref() {
            self.egl_ref().make_current(Some(*surface));

            let display = self.egl_display();

            let r = self.egl_instance().swap_buffers(*display, *surface);

            self.egl_ref().make_current(None);

            r
        } else {
            Ok(())
        }
    }

    #[inline]
    pub(crate) fn egl_ref(&self) -> &EglContext {
        &self.imp.egl
    }

    #[inline]
    pub(crate) fn egl_instance(&self) -> &EglInstance {
        &self.egl_ref().instance
    }

    #[inline]
    pub(crate) fn egl_context(&self) -> &egl::Context {
        &self.egl_ref().raw
    }

    #[inline]
    pub(crate) fn egl_config(&self) -> &egl::Config {
        &self.egl_ref().config
    }

    #[inline]
    pub(crate) fn egl_display(&self) -> &egl::Display {
        &self.egl_ref().display
    }

    // EGL 版本, (1, 5) 或者 (1, 4)
    #[inline]
    pub(crate) fn egl_version(&self) -> (i32, i32) {
        self.egl_ref().version
    }

    #[inline]
    pub(crate) fn egl_srgb_support(&self) -> SrgbFrameBufferKind {
        self.egl_ref().srgb_kind
    }

    #[inline]
    pub(crate) fn private_caps(&self) -> PrivateCapabilities {
        self.imp.private_caps
    }

    #[inline]
    pub(crate) fn features(&self) -> wgt::Features {
        self.imp.features
    }

    #[inline]
    pub(crate) fn limits(&self) -> &wgt::Limits {
        &self.imp.limits
    }

    #[inline]
    pub(crate) fn downlevel(&self) -> &wgt::DownlevelCapabilities {
        &self.imp.downlevel
    }

    #[inline]
    pub(crate) fn max_texture_size(&self) -> u32 {
        self.imp.max_texture_size
    }

    #[inline]
    pub(crate) fn shading_language_version(&self) -> naga::back::glsl::Version {
        self.imp.shading_language_version
    }

    #[inline]
    pub(crate) fn info(&self) -> &AdapterInfo {
        &self.imp.info
    }

    /// Returns the capabilities of working with a specified surface.
    ///
    /// `None` means presentation is not supported for it.
    #[inline]
    pub(crate) fn surface_capabilities(
        &self,
        surface: &super::Surface,
    ) -> Option<super::SurfaceCapabilities> {
        let mut formats = vec![
            wgt::TextureFormat::Rgba8Unorm,
            #[cfg(not(target_arch = "wasm32"))]
            wgt::TextureFormat::Bgra8Unorm,
        ];
        if surface.supports_srgb() {
            formats.extend([
                wgt::TextureFormat::Rgba8UnormSrgb,
                #[cfg(not(target_arch = "wasm32"))]
                wgt::TextureFormat::Bgra8UnormSrgb,
            ])
        }
        if self
            .imp
            .private_caps
            .contains(super::PrivateCapabilities::COLOR_BUFFER_HALF_FLOAT)
        {
            formats.push(wgt::TextureFormat::Rgba16Float)
        }

        Some(super::SurfaceCapabilities {
            formats,
            present_modes: vec![wgt::PresentMode::Fifo], //TODO
            composite_alpha_modes: vec![wgt::CompositeAlphaMode::Opaque], //TODO
            swap_chain_sizes: 2..=2,
            current_extent: None,
            extents: wgt::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            }..=wgt::Extent3d {
                width: self.imp.max_texture_size,
                height: self.imp.max_texture_size,
                depth_or_array_layers: 1,
            },
            usage: super::TextureUses::COLOR_TARGET,
        })
    }
}

impl AdapterContext {
    /// Obtain a lock to the EGL context and get handle to the [`glow::Context`] that can be used to
    /// do rendering.
    #[track_caller]
    pub(crate) fn lock<'a>(&'a self) -> AdapterContextLock<'a> {
        let glow = self
            .imp
            .glow
            // Don't lock forever. If it takes longer than 1 second to get the lock we've got a
            // deadlock and should panic to show where we got stuck
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.");

        if self.imp.reentrant_count.load(Ordering::SeqCst) == 0 {
            let surface = self.surface.as_ref().borrow().as_ref().map(|s| *s);
            self.imp.egl.make_current(surface);
        }
        self.imp.reentrant_count.fetch_add(1, Ordering::SeqCst);

        let egl = EglContextLock {
            instance: &self.imp.egl,
        };

        let reentrant_count = self.imp.reentrant_count.clone();
        AdapterContextLock {
            glow,
            egl,
            reentrant_count,
        }
    }
}

impl AdapterContext {
    fn make_info(vendor_orig: String, renderer_orig: String) -> wgt::AdapterInfo {
        let vendor = vendor_orig.to_lowercase();
        let renderer = renderer_orig.to_lowercase();

        // opengl has no way to discern device_type, so we can try to infer it from the renderer string
        let strings_that_imply_integrated = [
            " xpress", // space here is on purpose so we don't match express
            "amd renoir",
            "radeon hd 4200",
            "radeon hd 4250",
            "radeon hd 4290",
            "radeon hd 4270",
            "radeon hd 4225",
            "radeon hd 3100",
            "radeon hd 3200",
            "radeon hd 3000",
            "radeon hd 3300",
            "radeon(tm) r4 graphics",
            "radeon(tm) r5 graphics",
            "radeon(tm) r6 graphics",
            "radeon(tm) r7 graphics",
            "radeon r7 graphics",
            "nforce", // all nvidia nforce are integrated
            "tegra",  // all nvidia tegra are integrated
            "shield", // all nvidia shield are integrated
            "igp",
            "mali",
            "intel",
            "v3d",
            "apple m", // all apple m are integrated
        ];
        let strings_that_imply_cpu = ["mesa offscreen", "swiftshader", "llvmpipe"];

        //TODO: handle Intel Iris XE as discreet
        let inferred_device_type = if vendor.contains("qualcomm")
            || vendor.contains("intel")
            || strings_that_imply_integrated
                .iter()
                .any(|&s| renderer.contains(s))
        {
            wgt::DeviceType::IntegratedGpu
        } else if strings_that_imply_cpu.iter().any(|&s| renderer.contains(s)) {
            wgt::DeviceType::Cpu
        } else {
            // At this point the Device type is Unknown.
            // It's most likely DiscreteGpu, but we do not know for sure.
            // Use "Other" to avoid possibly making incorrect assumptions.
            // Note that if this same device is available under some other API (ex: Vulkan),
            // It will mostly likely get a different device type (probably DiscreteGpu).
            wgt::DeviceType::Other
        };

        // source: Sascha Willems at Vulkan
        let vendor_id = if vendor.contains("amd") {
            db::amd::VENDOR
        } else if vendor.contains("imgtec") {
            db::imgtec::VENDOR
        } else if vendor.contains("nvidia") {
            db::nvidia::VENDOR
        } else if vendor.contains("arm") {
            db::arm::VENDOR
        } else if vendor.contains("qualcomm") {
            db::qualcomm::VENDOR
        } else if vendor.contains("intel") {
            db::intel::VENDOR
        } else if vendor.contains("broadcom") {
            db::broadcom::VENDOR
        } else if vendor.contains("mesa") {
            db::mesa::VENDOR
        } else if vendor.contains("apple") {
            db::apple::VENDOR
        } else {
            0
        };

        wgt::AdapterInfo {
            name: renderer_orig,
            vendor: vendor_id as usize,
            device: 0,
            device_type: inferred_device_type,
            driver: String::new(),
            driver_info: String::new(),
            backend: wgt::Backend::Gl,
        }
    }

    /// According to the OpenGL specification, the version information is
    /// expected to follow the following syntax:
    ///
    /// ~~~bnf
    /// <major>       ::= <number>
    /// <minor>       ::= <number>
    /// <revision>    ::= <number>
    /// <vendor-info> ::= <string>
    /// <release>     ::= <major> "." <minor> ["." <release>]
    /// <version>     ::= <release> [" " <vendor-info>]
    /// ~~~
    ///
    /// Note that this function is intentionally lenient in regards to parsing,
    /// and will try to recover at least the first two version numbers without
    /// resulting in an `Err`.
    /// # Notes
    /// `WebGL 2` version returned as `OpenGL ES 3.0`
    fn parse_version(mut src: &str) -> Result<(u8, u8), super::InstanceError> {
        let webgl_sig = "WebGL ";
        // According to the WebGL specification
        // VERSION  WebGL<space>1.0<space><vendor-specific information>
        // SHADING_LANGUAGE_VERSION WebGL<space>GLSL<space>ES<space>1.0<space><vendor-specific information>
        let is_webgl = src.starts_with(webgl_sig);
        if is_webgl {
            let pos = src.rfind(webgl_sig).unwrap_or(0);
            src = &src[pos + webgl_sig.len()..];
        } else {
            let es_sig = " ES ";
            match src.rfind(es_sig) {
                Some(pos) => {
                    src = &src[pos + es_sig.len()..];
                }
                None => {
                    log::warn!("ES not found in '{}'", src);
                    return Err(super::InstanceError);
                }
            }
        };

        let glsl_es_sig = "GLSL ES ";
        let is_glsl = match src.find(glsl_es_sig) {
            Some(pos) => {
                src = &src[pos + glsl_es_sig.len()..];
                true
            }
            None => false,
        };

        let (version, _vendor_info) = match src.find(' ') {
            Some(i) => (&src[..i], src[i + 1..].to_string()),
            None => (src, String::new()),
        };

        // TODO: make this even more lenient so that we can also accept
        // `<major> "." <minor> [<???>]`
        let mut it = version.split('.');
        let major = it.next().and_then(|s| s.parse().ok());
        let minor = it.next().and_then(|s| {
            let trimmed = if s.starts_with('0') {
                "0"
            } else {
                s.trim_end_matches('0')
            };
            trimmed.parse().ok()
        });

        match (major, minor) {
            (Some(major), Some(minor)) => Ok((
                // Return WebGL 2.0 version as OpenGL ES 3.0
                if is_webgl && !is_glsl {
                    major + 1
                } else {
                    major
                },
                minor,
            )),
            _ => {
                log::warn!("Unable to extract the version from '{}'", version);
                Err(super::InstanceError)
            }
        }
    }

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
    unsafe fn get_without_egl_lock(&self) -> ReentrantMutexGuard<glow::Context> {
        self.imp
            .glow
            .try_lock_for(Duration::from_secs(CONTEXT_LOCK_TIMEOUT_SECS))
            .expect("Could not lock adapter context. This is most-likely a deadlcok.")
    }
}

#[derive(Debug)]
struct EglContextLock<'a> {
    instance: &'a EglContext,
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
            self.egl.instance.unmake_current();
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
    if srgb_kind != SrgbFrameBufferKind::None {
        attributes.push(egl::ALPHA_SIZE);
        attributes.push(8);
    }

    attributes.push(egl::NONE);

    match egl.choose_first_config(display, &attributes[..]) {
        Ok(Some(config)) => Ok(config),
        Ok(None) => {
            log::error!("choose_first_config: Missing config");
            Err(InstanceError)
        }
        Err(e) => {
            log::error!("choose_first_config error = {:?}", e);
            Err(InstanceError)
        }
    }
}

const GL_UNMASKED_VENDOR_WEBGL: u32 = 0x9245;
const GL_UNMASKED_RENDERER_WEBGL: u32 = 0x9246;
