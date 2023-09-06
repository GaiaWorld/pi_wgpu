use glow::HasContext;

use super::{db, GLState, PrivateCapabilities, Workarounds};
use crate::{wgt, AstcChannel};

#[derive(Debug)]
pub(crate) struct Adapter {
    pub(crate) state: GLState,
    pub(crate) private_caps: PrivateCapabilities,
    pub(crate) features: wgt::Features,
    pub(crate) workarounds: Workarounds,
    pub(crate) shading_language_version: naga::back::glsl::Version,
    pub(crate) max_texture_size: u32,
}

impl Adapter {
    // 枚举 gl 环境的 特性
    pub(crate) unsafe fn expose(state: &GLState) -> Option<crate::ExposedAdapter<super::GL>> {
        let gl = &state.0.borrow().gl;

        let extensions = gl.supported_extensions();

        let (vendor_const, renderer_const) = if extensions.contains("WEBGL_debug_renderer_info") {
            (GL_UNMASKED_VENDOR_WEBGL, GL_UNMASKED_RENDERER_WEBGL)
        } else {
            (glow::VENDOR, glow::RENDERER)
        };

        // ========== 1. 厂商
        let (vendor, renderer) = {
            let vendor = unsafe { gl.get_parameter_string(vendor_const) };
            let renderer = unsafe { gl.get_parameter_string(renderer_const) };

            (vendor, renderer)
        };

        // ========== 2. 版本，必须大于等于 3.0

        let version = unsafe { gl.get_parameter_string(glow::VERSION) };
        log::info!("GL Vendor: {}", vendor);
        log::info!("GL Renderer: {}", renderer);
        log::info!("GL Version: {}", version);

        log::info!("GL Extensions: {:#?}", extensions);

        let ver = Self::parse_version(&version).ok()?;
        if ver < (3, 0) {
            log::warn!(
                "Returned GLES context is {}.{}, when 3.0+ was requested",
                ver.0,
                ver.1
            );
            return None;
        }

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
            features.insert(wgt::Features::TEXTURE_COMPRESSION_ASTC_LDR);
            features.insert(wgt::Features::TEXTURE_COMPRESSION_ASTC_HDR);
        } else {
            features.set(
                wgt::Features::TEXTURE_COMPRESSION_ASTC_LDR,
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

        let mut workarounds = super::Workarounds::empty();

        let downlevel_defaults = wgt::DownlevelLimits {};

        let adapter = super::Adapter {
            state: state.clone(),
            private_caps,
            workarounds,
            features,
            shading_language_version,
            max_texture_size,
        };

        Some(super::ExposedAdapter {
            adapter,
            info: Self::make_info(vendor, renderer),
            features,
            limits,
            downlevel: wgt::DownlevelCapabilities {
                flags: downlevel_flags,
                limits: downlevel_defaults,
                shader_model: wgt::ShaderModel::Sm5,
            },
        })
    }

    pub(crate) unsafe fn open(
        &self,
        features: wgt::Features,
        _limits: &wgt::Limits,
    ) -> Result<super::OpenDevice<super::GL>, crate::DeviceError> {
        let gl = &self.state.0.borrow().gl;

        unsafe { gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1) };
        unsafe { gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1) };
        let main_vao =
            unsafe { gl.create_vertex_array() }.map_err(|_| crate::DeviceError::OutOfMemory)?;
        unsafe { gl.bind_vertex_array(Some(main_vao)) };

        let zero_buffer =
            unsafe { gl.create_buffer() }.map_err(|_| crate::DeviceError::OutOfMemory)?;
        unsafe { gl.bind_buffer(glow::COPY_READ_BUFFER, Some(zero_buffer)) };
        let zeroes = vec![0u8; super::ZERO_BUFFER_SIZE];
        unsafe { gl.buffer_data_u8_slice(glow::COPY_READ_BUFFER, &zeroes, glow::STATIC_DRAW) };

        Ok(super::OpenDevice {
            device: super::Device {
                state: self.state.clone(),
            },
            queue: super::Queue {
                state: self.state.clone(),
            },
        })
    }

    /// Return the set of supported capabilities for a texture format.
    pub(crate) unsafe fn texture_format_capabilities(
        &self,
        format: wgt::TextureFormat,
    ) -> super::TextureFormatCapabilities {
        use super::TextureFormatCapabilities as Tfc;
        use wgt::TextureFormat as Tf;

        let sample_count = {
            let max_samples = unsafe {
                self.state
                    .0
                    .borrow()
                    .gl
                    .get_parameter_i32(glow::MAX_SAMPLES)
            };
            if max_samples >= 8 {
                Tfc::MULTISAMPLE_X2 | Tfc::MULTISAMPLE_X4 | Tfc::MULTISAMPLE_X8
            } else if max_samples >= 4 {
                Tfc::MULTISAMPLE_X2 | Tfc::MULTISAMPLE_X4
            } else {
                Tfc::MULTISAMPLE_X2
            }
        };

        // Base types are pulled from the table in the OpenGLES 3.0 spec in section 3.8.
        //
        // The storage types are based on table 8.26, in section
        // "TEXTURE IMAGE LOADS AND STORES" of OpenGLES-3.2 spec.
        let empty = Tfc::empty();
        let base = Tfc::COPY_SRC | Tfc::COPY_DST;
        let unfilterable = base | Tfc::SAMPLED;
        let depth = base | Tfc::SAMPLED | sample_count | Tfc::DEPTH_STENCIL_ATTACHMENT;
        let filterable = unfilterable | Tfc::SAMPLED_LINEAR;
        let renderable =
            unfilterable | Tfc::COLOR_ATTACHMENT | sample_count | Tfc::MULTISAMPLE_RESOLVE;
        let filterable_renderable = filterable | renderable | Tfc::COLOR_ATTACHMENT_BLEND;
        let storage = base | Tfc::STORAGE | Tfc::STORAGE_READ_WRITE;

        let feature_fn = |f, caps| {
            if self.features.contains(f) {
                caps
            } else {
                empty
            }
        };

        let bcn_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_BC, filterable);
        let etc2_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ETC2, filterable);
        let astc_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ASTC_LDR, filterable);
        let astc_hdr_features = feature_fn(wgt::Features::TEXTURE_COMPRESSION_ASTC_HDR, filterable);

        let private_caps_fn = |f, caps| {
            if self.private_caps.contains(f) {
                caps
            } else {
                empty
            }
        };

        let half_float_renderable = private_caps_fn(
            super::PrivateCapabilities::COLOR_BUFFER_HALF_FLOAT,
            Tfc::COLOR_ATTACHMENT
                | Tfc::COLOR_ATTACHMENT_BLEND
                | sample_count
                | Tfc::MULTISAMPLE_RESOLVE,
        );

        let float_renderable = private_caps_fn(
            super::PrivateCapabilities::COLOR_BUFFER_FLOAT,
            Tfc::COLOR_ATTACHMENT
                | Tfc::COLOR_ATTACHMENT_BLEND
                | sample_count
                | Tfc::MULTISAMPLE_RESOLVE,
        );

        let texture_float_linear =
            private_caps_fn(super::PrivateCapabilities::TEXTURE_FLOAT_LINEAR, filterable);

        match format {
            Tf::R8Unorm => filterable_renderable,
            Tf::R8Snorm => filterable,
            Tf::R8Uint => renderable,
            Tf::R8Sint => renderable,
            Tf::R16Uint => renderable,
            Tf::R16Sint => renderable,
            Tf::R16Unorm => empty,
            Tf::R16Snorm => empty,
            Tf::R16Float => filterable | half_float_renderable,
            Tf::Rg8Unorm => filterable_renderable,
            Tf::Rg8Snorm => filterable,
            Tf::Rg8Uint => renderable,
            Tf::Rg8Sint => renderable,
            Tf::R32Uint => renderable | storage,
            Tf::R32Sint => renderable | storage,
            Tf::R32Float => unfilterable | storage | float_renderable | texture_float_linear,
            Tf::Rg16Uint => renderable,
            Tf::Rg16Sint => renderable,
            Tf::Rg16Unorm => empty,
            Tf::Rg16Snorm => empty,
            Tf::Rg16Float => filterable | half_float_renderable,
            Tf::Rgba8Unorm | Tf::Rgba8UnormSrgb => filterable_renderable | storage,
            Tf::Bgra8Unorm | Tf::Bgra8UnormSrgb => filterable_renderable,
            Tf::Rgba8Snorm => filterable,
            Tf::Rgba8Uint => renderable | storage,
            Tf::Rgba8Sint => renderable | storage,
            Tf::Rgb10a2Unorm => filterable_renderable,
            Tf::Rg11b10Float => filterable | float_renderable,
            Tf::Rg32Uint => renderable,
            Tf::Rg32Sint => renderable,
            Tf::Rg32Float => unfilterable | float_renderable | texture_float_linear,
            Tf::Rgba16Uint => renderable | storage,
            Tf::Rgba16Sint => renderable | storage,
            Tf::Rgba16Unorm => empty,
            Tf::Rgba16Snorm => empty,
            Tf::Rgba16Float => filterable | storage | half_float_renderable,
            Tf::Rgba32Uint => renderable | storage,
            Tf::Rgba32Sint => renderable | storage,
            Tf::Rgba32Float => unfilterable | storage | float_renderable | texture_float_linear,
            Tf::Stencil8
            | Tf::Depth16Unorm
            | Tf::Depth32Float
            | Tf::Depth32FloatStencil8
            | Tf::Depth24Plus
            | Tf::Depth24PlusStencil8 => depth,
            Tf::Rgb9e5Ufloat => filterable,
            Tf::Bc1RgbaUnorm
            | Tf::Bc1RgbaUnormSrgb
            | Tf::Bc2RgbaUnorm
            | Tf::Bc2RgbaUnormSrgb
            | Tf::Bc3RgbaUnorm
            | Tf::Bc3RgbaUnormSrgb
            | Tf::Bc4RUnorm
            | Tf::Bc4RSnorm
            | Tf::Bc5RgUnorm
            | Tf::Bc5RgSnorm
            | Tf::Bc6hRgbSfloat
            | Tf::Bc6hRgbUfloat
            | Tf::Bc7RgbaUnorm
            | Tf::Bc7RgbaUnormSrgb => bcn_features,
            Tf::Etc2Rgb8Unorm
            | Tf::Etc2Rgb8UnormSrgb
            | Tf::Etc2Rgb8A1Unorm
            | Tf::Etc2Rgb8A1UnormSrgb
            | Tf::Etc2Rgba8Unorm
            | Tf::Etc2Rgba8UnormSrgb
            | Tf::EacR11Unorm
            | Tf::EacR11Snorm
            | Tf::EacRg11Unorm
            | Tf::EacRg11Snorm => etc2_features,
            Tf::Astc {
                block: _,
                channel: AstcChannel::Unorm | AstcChannel::UnormSrgb,
            } => astc_features,
            Tf::Astc {
                block: _,
                channel: AstcChannel::Hdr,
            } => astc_hdr_features,
        }
    }

    /// Returns the capabilities of working with a specified surface.
    ///
    /// `None` means presentation is not supported for it.
    pub(crate) unsafe fn surface_capabilities(
        &self,
        surface: &super::Surface,
    ) -> Option<super::SurfaceCapabilities> {
        if surface.presentable {
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
                    width: self.max_texture_size,
                    height: self.max_texture_size,
                    depth_or_array_layers: 1,
                },
                usage: super::TextureUses::COLOR_TARGET,
            })
        } else {
            None
        }
    }
}

impl Adapter {
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
}

const GL_UNMASKED_VENDOR_WEBGL: u32 = 0x9245;
const GL_UNMASKED_RENDERER_WEBGL: u32 = 0x9246;
