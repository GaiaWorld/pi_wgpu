use super::super::wgt;

pub(crate) const MAX_SAMPLERS: usize = 16;
pub(crate) const MAX_TEXTURE_SLOTS: usize = 16;
pub(crate) const MAX_VERTEX_ATTRIBUTES: usize = 16;

pub(crate) type BindTarget = u32;

#[derive(Clone, Debug)]
pub(crate) struct TextureFormatDesc {
    pub internal: u32,
    pub external: u32,
    pub data_type: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PrimitiveState {
    front_face: u32,
    cull_face: u32,
    unclipped_depth: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ColorTargetDesc {
    mask: wgt::ColorWrites,
    blend: Option<BlendDesc>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct BlendComponent {
    pub(crate) equation: u32, // glow::FUNC_ADD,

    pub(crate) src_factor: u32, // glow::SRC_ALPHA,
    pub(crate) dst_factor: u32, // glow::ONE_MINUS_SRC_ALPHA,
}

impl Default for BlendComponent {
    fn default() -> Self {
        Self {
            equation: glow::FUNC_ADD,
            src_factor: glow::ONE,
            dst_factor: glow::ZERO,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BlendDesc {
    pub(crate) alpha: BlendComponent,
    pub(crate) color: BlendComponent,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VertexFormatDesc {
    pub(crate) element_count: i32,
    pub(crate) element_format: u32,
    pub(crate) attrib_kind: VertexAttribKind,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) enum VertexAttribKind {
    Float,   // glVertexAttribPointer
    Integer, // glVertexAttribIPointer
}

impl Default for VertexAttribKind {
    fn default() -> Self {
        Self::Float
    }
}

bitflags::bitflags! {
    /// Flags that affect internal code paths but do not
    /// change the exposed feature set.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub(crate) struct PrivateCapabilities: u32 {
        /// Indicates support for `glBufferStorage` allocation.
        const BUFFER_ALLOCATION = 1 << 0;
        /// Support explicit layouts in shader.
        const SHADER_BINDING_LAYOUT = 1 << 1;
        /// Support extended shadow sampling instructions.
        const SHADER_TEXTURE_SHADOW_LOD = 1 << 2;
        /// Support memory barriers.
        const MEMORY_BARRIERS = 1 << 3;
        /// Vertex buffer layouts separate from the data.
        const VERTEX_BUFFER_LAYOUT = 1 << 4;
        /// Indicates that buffers used as `GL_ELEMENT_ARRAY_BUFFER` may be created / initialized / used
        /// as other targets, if not present they must not be mixed with other targets.
        const INDEX_BUFFER_ROLE_CHANGE = 1 << 5;
        /// Supports `glGetBufferSubData`
        const GET_BUFFER_SUB_DATA = 1 << 7;
        /// Supports `f16` color buffers
        const COLOR_BUFFER_HALF_FLOAT = 1 << 8;
        /// Supports `f11/f10` and `f32` color buffers
        const COLOR_BUFFER_FLOAT = 1 << 9;
        /// Supports query buffer objects.
        const QUERY_BUFFERS = 1 << 11;
        /// Supports 64 bit queries via `glGetQueryObjectui64v`
        const QUERY_64BIT = 1 << 12;
        /// Supports `glTexStorage2D`, etc.
        const TEXTURE_STORAGE = 1 << 13;
        /// Supports `push_debug_group`, `pop_debug_group` and `debug_message_insert`.
        const DEBUG_FNS = 1 << 14;
        /// Supports framebuffer invalidation.
        const INVALIDATE_FRAMEBUFFER = 1 << 15;
        /// Indicates support for `glDrawElementsInstancedBaseVertexBaseInstance` and `ARB_shader_draw_parameters`
        ///
        /// When this is true, instance offset emulation via vertex buffer rebinding and a shader uniform will be disabled.
        const FULLY_FEATURED_INSTANCING = 1 << 16;
    }
}

pub(crate) mod db {
    pub mod amd {
        pub const VENDOR: u32 = 0x1002;
    }
    pub mod apple {
        pub const VENDOR: u32 = 0x106B;
    }
    pub mod arm {
        pub const VENDOR: u32 = 0x13B5;
    }
    pub mod broadcom {
        pub const VENDOR: u32 = 0x14E4;
    }
    pub mod imgtec {
        pub const VENDOR: u32 = 0x1010;
    }
    pub mod intel {
        pub const VENDOR: u32 = 0x8086;
    }
    pub mod mesa {
        // Mesa does not actually have a PCI vendor id.
        //
        // To match Vulkan, we use the VkVendorId for Mesa in the gles backend so that lavapipe (Vulkan) and
        // llvmpipe (OpenGL) have the same vendor id.
        pub const VENDOR: u32 = 0x10005;
    }
    pub mod nvidia {
        pub const VENDOR: u32 = 0x10DE;
    }
    pub mod qualcomm {
        pub const VENDOR: u32 = 0x5143;
    }
}
