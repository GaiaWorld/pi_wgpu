use crate::wgt;

#[inline]
pub(crate) fn map_compare_func(fun: wgt::CompareFunction) -> u32 {
    use wgt::CompareFunction as Cf;

    match fun {
        Cf::Never => glow::NEVER,
        Cf::Less => glow::LESS,
        Cf::LessEqual => glow::LEQUAL,
        Cf::Equal => glow::EQUAL,
        Cf::GreaterEqual => glow::GEQUAL,
        Cf::Greater => glow::GREATER,
        Cf::NotEqual => glow::NOTEQUAL,
        Cf::Always => glow::ALWAYS,
    }
}

#[inline]
pub(crate) fn map_stencil_op(operation: wgt::StencilOperation) -> u32 {
    use wgt::StencilOperation as So;

    match operation {
        So::Keep => glow::KEEP,
        So::Zero => glow::ZERO,
        So::Replace => glow::REPLACE,
        So::Invert => glow::INVERT,
        So::IncrementClamp => glow::INCR,
        So::DecrementClamp => glow::DECR,
        So::IncrementWrap => glow::INCR_WRAP,
        So::DecrementWrap => glow::DECR_WRAP,
    }
}

#[inline]
pub(crate) fn map_primitive_topology(topology: wgt::PrimitiveTopology) -> u32 {
    use wgt::PrimitiveTopology as Pt;
    match topology {
        Pt::PointList => glow::POINTS,
        Pt::LineList => glow::LINES,
        Pt::LineStrip => glow::LINE_STRIP,
        Pt::TriangleList => glow::TRIANGLES,
        Pt::TriangleStrip => glow::TRIANGLE_STRIP,
    }
}

#[inline]
pub(crate) fn map_index_format(format: wgt::IndexFormat) -> (u32, u32) {
    match format {
        wgt::IndexFormat::Uint16 => (2, glow::UNSIGNED_SHORT),
        wgt::IndexFormat::Uint32 => (4, glow::UNSIGNED_INT),
    }
}

#[inline]
pub(crate) fn map_blend_factor(factor: wgt::BlendFactor) -> u32 {
    use wgt::BlendFactor as Bf;
    match factor {
        Bf::Zero => glow::ZERO,
        Bf::One => glow::ONE,
        Bf::Src => glow::SRC_COLOR,
        Bf::OneMinusSrc => glow::ONE_MINUS_SRC_COLOR,
        Bf::Dst => glow::DST_COLOR,
        Bf::OneMinusDst => glow::ONE_MINUS_DST_COLOR,
        Bf::SrcAlpha => glow::SRC_ALPHA,
        Bf::OneMinusSrcAlpha => glow::ONE_MINUS_SRC_ALPHA,
        Bf::DstAlpha => glow::DST_ALPHA,
        Bf::OneMinusDstAlpha => glow::ONE_MINUS_DST_ALPHA,
        Bf::Constant => glow::CONSTANT_COLOR,
        Bf::OneMinusConstant => glow::ONE_MINUS_CONSTANT_COLOR,
        Bf::SrcAlphaSaturated => glow::SRC_ALPHA_SATURATE,
    }
}

#[inline]
pub(crate) fn map_texture_usage(
    usage: wgt::TextureUsages,
    aspect: super::FormatAspects,
) -> super::TextureUses {
    let mut u = super::TextureUses::empty();
    // u.set(
    //     super::TextureUses::COPY_SRC,
    //     usage.contains(wgt::TextureUsages::COPY_SRC),
    // );
    // u.set(
    //     super::TextureUses::COPY_DST,
    //     usage.contains(wgt::TextureUsages::COPY_DST),
    // );
    // u.set(
    //     super::TextureUses::STORAGE_READ | super::TextureUses::STORAGE_READ_WRITE,
    //     usage.contains(wgt::TextureUsages::STORAGE_BINDING),
    // );

    assert!(!usage.contains(wgt::TextureUsages::COPY_DST));
    assert!(!usage.contains(wgt::TextureUsages::COPY_SRC));
    assert!(!usage.contains(wgt::TextureUsages::STORAGE_BINDING));

    u.set(
        super::TextureUses::RESOURCE,
        usage.contains(wgt::TextureUsages::TEXTURE_BINDING),
    );

    let is_color = aspect.contains(super::FormatAspects::COLOR);
    u.set(
        super::TextureUses::COLOR_TARGET,
        usage.contains(wgt::TextureUsages::RENDER_ATTACHMENT) && is_color,
    );
    u.set(
        super::TextureUses::DEPTH_STENCIL_READ | super::TextureUses::DEPTH_STENCIL_WRITE,
        usage.contains(wgt::TextureUsages::RENDER_ATTACHMENT) && !is_color,
    );
    u
}

#[inline]
pub(crate) fn map_blend_component(component: &wgt::BlendComponent) -> super::BlendComponent {
    super::BlendComponent {
        src_factor: map_blend_factor(component.src_factor),
        dst_factor: map_blend_factor(component.dst_factor),
        equation: match component.operation {
            wgt::BlendOperation::Add => glow::FUNC_ADD,
            wgt::BlendOperation::Subtract => glow::FUNC_SUBTRACT,
            wgt::BlendOperation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
            wgt::BlendOperation::Min => glow::MIN,
            wgt::BlendOperation::Max => glow::MAX,
        },
    }
}

pub(super) fn describe_vertex_format(vertex_format: wgt::VertexFormat) -> super::VertexFormatDesc {
    use super::VertexAttribKind as Vak;
    use wgt::VertexFormat as Vf;

    let (element_count, element_format, attrib_kind) = match vertex_format {
        Vf::Unorm8x2 => (2, glow::UNSIGNED_BYTE, Vak::Float),
        Vf::Snorm8x2 => (2, glow::BYTE, Vak::Float),
        Vf::Uint8x2 => (2, glow::UNSIGNED_BYTE, Vak::Integer),
        Vf::Sint8x2 => (2, glow::BYTE, Vak::Integer),
        Vf::Unorm8x4 => (4, glow::UNSIGNED_BYTE, Vak::Float),
        Vf::Snorm8x4 => (4, glow::BYTE, Vak::Float),
        Vf::Uint8x4 => (4, glow::UNSIGNED_BYTE, Vak::Integer),
        Vf::Sint8x4 => (4, glow::BYTE, Vak::Integer),
        Vf::Unorm16x2 => (2, glow::UNSIGNED_SHORT, Vak::Float),
        Vf::Snorm16x2 => (2, glow::SHORT, Vak::Float),
        Vf::Uint16x2 => (2, glow::UNSIGNED_SHORT, Vak::Integer),
        Vf::Sint16x2 => (2, glow::SHORT, Vak::Integer),
        Vf::Float16x2 => (2, glow::HALF_FLOAT, Vak::Float),
        Vf::Unorm16x4 => (4, glow::UNSIGNED_SHORT, Vak::Float),
        Vf::Snorm16x4 => (4, glow::SHORT, Vak::Float),
        Vf::Uint16x4 => (4, glow::UNSIGNED_SHORT, Vak::Integer),
        Vf::Sint16x4 => (4, glow::SHORT, Vak::Integer),
        Vf::Float16x4 => (4, glow::HALF_FLOAT, Vak::Float),
        Vf::Uint32 => (1, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32 => (1, glow::INT, Vak::Integer),
        Vf::Float32 => (1, glow::FLOAT, Vak::Float),
        Vf::Uint32x2 => (2, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x2 => (2, glow::INT, Vak::Integer),
        Vf::Float32x2 => (2, glow::FLOAT, Vak::Float),
        Vf::Uint32x3 => (3, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x3 => (3, glow::INT, Vak::Integer),
        Vf::Float32x3 => (3, glow::FLOAT, Vak::Float),
        Vf::Uint32x4 => (4, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x4 => (4, glow::INT, Vak::Integer),
        Vf::Float32x4 => (4, glow::FLOAT, Vak::Float),
        Vf::Float64 | Vf::Float64x2 | Vf::Float64x3 | Vf::Float64x4 => unimplemented!(),
    };

    super::VertexFormatDesc {
        element_count,
        element_format,
        attrib_kind,
    }
}

#[inline]
pub(crate) fn map_filter_modes(
    min: wgt::FilterMode,
    mag: wgt::FilterMode,
    mip: wgt::FilterMode,
) -> (u32, u32) {
    use wgt::FilterMode as Fm;

    let mag_filter = match mag {
        Fm::Nearest => glow::NEAREST,
        Fm::Linear => glow::LINEAR,
    };

    let min_filter = match (min, mip) {
        (Fm::Nearest, Fm::Nearest) => glow::NEAREST_MIPMAP_NEAREST,
        (Fm::Nearest, Fm::Linear) => glow::NEAREST_MIPMAP_LINEAR,
        (Fm::Linear, Fm::Nearest) => glow::LINEAR_MIPMAP_NEAREST,
        (Fm::Linear, Fm::Linear) => glow::LINEAR_MIPMAP_LINEAR,
    };

    (min_filter, mag_filter)
}

#[inline]
pub(crate) fn map_address_mode(mode: wgt::AddressMode) -> u32 {
    match mode {
        wgt::AddressMode::Repeat => glow::REPEAT,
        wgt::AddressMode::MirrorRepeat => glow::MIRRORED_REPEAT,
        wgt::AddressMode::ClampToEdge => glow::CLAMP_TO_EDGE,
        _ => unimplemented!(),
    }
}

pub(super) fn map_texture_format(
    texture_format: wgt::TextureFormat,
) -> super::TextureFormatDesc {
    use wgt::TextureFormat as Tf;
    use wgt::{AstcBlock, AstcChannel};

    let (internal, external, data_type) = match texture_format {
        Tf::R8Unorm => (glow::R8, glow::RED, glow::UNSIGNED_BYTE),
        Tf::R8Snorm => (glow::R8_SNORM, glow::RED, glow::BYTE),
        Tf::R8Uint => (glow::R8UI, glow::RED_INTEGER, glow::UNSIGNED_BYTE),
        Tf::R8Sint => (glow::R8I, glow::RED_INTEGER, glow::BYTE),
        Tf::R16Uint => (glow::R16UI, glow::RED_INTEGER, glow::UNSIGNED_SHORT),
        Tf::R16Sint => (glow::R16I, glow::RED_INTEGER, glow::SHORT),
        Tf::R16Unorm => (glow::R16, glow::RED, glow::UNSIGNED_SHORT),
        Tf::R16Snorm => (glow::R16_SNORM, glow::RED, glow::SHORT),
        Tf::R16Float => (glow::R16F, glow::RED, glow::HALF_FLOAT),
        Tf::Rg8Unorm => (glow::RG8, glow::RG, glow::UNSIGNED_BYTE),
        Tf::Rg8Snorm => (glow::RG8_SNORM, glow::RG, glow::BYTE),
        Tf::Rg8Uint => (glow::RG8UI, glow::RG_INTEGER, glow::UNSIGNED_BYTE),
        Tf::Rg8Sint => (glow::RG8I, glow::RG_INTEGER, glow::BYTE),
        Tf::R32Uint => (glow::R32UI, glow::RED_INTEGER, glow::UNSIGNED_INT),
        Tf::R32Sint => (glow::R32I, glow::RED_INTEGER, glow::INT),
        Tf::R32Float => (glow::R32F, glow::RED, glow::FLOAT),
        Tf::Rg16Uint => (glow::RG16UI, glow::RG_INTEGER, glow::UNSIGNED_SHORT),
        Tf::Rg16Sint => (glow::RG16I, glow::RG_INTEGER, glow::SHORT),
        Tf::Rg16Unorm => (glow::RG16, glow::RG, glow::UNSIGNED_SHORT),
        Tf::Rg16Snorm => (glow::RG16_SNORM, glow::RG, glow::SHORT),
        Tf::Rg16Float => (glow::RG16F, glow::RG, glow::HALF_FLOAT),
        Tf::Rgba8Unorm => (glow::RGBA8, glow::RGBA, glow::UNSIGNED_BYTE),
        Tf::Rgba8UnormSrgb => (glow::SRGB8_ALPHA8, glow::RGBA, glow::UNSIGNED_BYTE),
        Tf::Bgra8UnormSrgb => (glow::SRGB8_ALPHA8, glow::BGRA, glow::UNSIGNED_BYTE), //TODO?
        Tf::Rgba8Snorm => (glow::RGBA8_SNORM, glow::RGBA, glow::BYTE),
        Tf::Bgra8Unorm => (glow::RGBA8, glow::BGRA, glow::UNSIGNED_BYTE), //TODO?
        Tf::Rgba8Uint => (glow::RGBA8UI, glow::RGBA_INTEGER, glow::UNSIGNED_BYTE),
        Tf::Rgba8Sint => (glow::RGBA8I, glow::RGBA_INTEGER, glow::BYTE),
        Tf::Rgb10a2Unorm => (
            glow::RGB10_A2,
            glow::RGBA,
            glow::UNSIGNED_INT_2_10_10_10_REV,
        ),
        Tf::Rg11b10Float => (
            glow::R11F_G11F_B10F,
            glow::RGB,
            glow::UNSIGNED_INT_10F_11F_11F_REV,
        ),
        Tf::Rg32Uint => (glow::RG32UI, glow::RG_INTEGER, glow::UNSIGNED_INT),
        Tf::Rg32Sint => (glow::RG32I, glow::RG_INTEGER, glow::INT),
        Tf::Rg32Float => (glow::RG32F, glow::RG, glow::FLOAT),
        Tf::Rgba16Uint => (glow::RGBA16UI, glow::RGBA_INTEGER, glow::UNSIGNED_SHORT),
        Tf::Rgba16Sint => (glow::RGBA16I, glow::RGBA_INTEGER, glow::SHORT),
        Tf::Rgba16Unorm => (glow::RGBA16, glow::RGBA, glow::UNSIGNED_SHORT),
        Tf::Rgba16Snorm => (glow::RGBA16_SNORM, glow::RGBA, glow::SHORT),
        Tf::Rgba16Float => (glow::RGBA16F, glow::RGBA, glow::HALF_FLOAT),
        Tf::Rgba32Uint => (glow::RGBA32UI, glow::RGBA_INTEGER, glow::UNSIGNED_INT),
        Tf::Rgba32Sint => (glow::RGBA32I, glow::RGBA_INTEGER, glow::INT),
        Tf::Rgba32Float => (glow::RGBA32F, glow::RGBA, glow::FLOAT),
        Tf::Stencil8 => (
            glow::STENCIL_INDEX8,
            glow::STENCIL_COMPONENTS,
            glow::UNSIGNED_BYTE,
        ),
        Tf::Depth16Unorm => (
            glow::DEPTH_COMPONENT16,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        ),
        Tf::Depth32Float => (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT),
        Tf::Depth32FloatStencil8 => {
            (glow::DEPTH32F_STENCIL8, glow::DEPTH_COMPONENT, glow::FLOAT)
        }
        Tf::Depth24Plus => (
            glow::DEPTH_COMPONENT24,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_NORMALIZED,
        ),
        Tf::Depth24PlusStencil8 => (
            glow::DEPTH24_STENCIL8,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_INT,
        ),
        Tf::Rgb9e5Ufloat => (glow::RGB9_E5, glow::RGB, glow::UNSIGNED_INT_5_9_9_9_REV),
        Tf::Bc1RgbaUnorm => (glow::COMPRESSED_RGBA_S3TC_DXT1_EXT, glow::RGBA, 0),
        Tf::Bc1RgbaUnormSrgb => (glow::COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT, glow::RGBA, 0),
        Tf::Bc2RgbaUnorm => (glow::COMPRESSED_RGBA_S3TC_DXT3_EXT, glow::RGBA, 0),
        Tf::Bc2RgbaUnormSrgb => (glow::COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT, glow::RGBA, 0),
        Tf::Bc3RgbaUnorm => (glow::COMPRESSED_RGBA_S3TC_DXT5_EXT, glow::RGBA, 0),
        Tf::Bc3RgbaUnormSrgb => (glow::COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT, glow::RGBA, 0),
        Tf::Bc4RUnorm => (glow::COMPRESSED_RED_RGTC1, glow::RED, 0),
        Tf::Bc4RSnorm => (glow::COMPRESSED_SIGNED_RED_RGTC1, glow::RED, 0),
        Tf::Bc5RgUnorm => (glow::COMPRESSED_RG_RGTC2, glow::RG, 0),
        Tf::Bc5RgSnorm => (glow::COMPRESSED_SIGNED_RG_RGTC2, glow::RG, 0),
        Tf::Bc6hRgbUfloat => (glow::COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT, glow::RGB, 0),
        Tf::Bc6hRgbSfloat => (glow::COMPRESSED_RGB_BPTC_SIGNED_FLOAT, glow::RGB, 0),
        Tf::Bc7RgbaUnorm => (glow::COMPRESSED_RGBA_BPTC_UNORM, glow::RGBA, 0),
        Tf::Bc7RgbaUnormSrgb => (glow::COMPRESSED_SRGB_ALPHA_BPTC_UNORM, glow::RGBA, 0),
        Tf::Etc2Rgb8Unorm => (glow::COMPRESSED_RGB8_ETC2, glow::RGB, 0),
        Tf::Etc2Rgb8UnormSrgb => (glow::COMPRESSED_SRGB8_ETC2, glow::RGB, 0),
        Tf::Etc2Rgb8A1Unorm => (
            glow::COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2,
            glow::RGBA,
            0,
        ),
        Tf::Etc2Rgb8A1UnormSrgb => (
            glow::COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2,
            glow::RGBA,
            0,
        ),
        Tf::Etc2Rgba8Unorm => (
            //TODO: this is a lie, it's not sRGB
            glow::COMPRESSED_SRGB8_ALPHA8_ETC2_EAC,
            glow::RGBA,
            0,
        ),
        Tf::Etc2Rgba8UnormSrgb => (glow::COMPRESSED_SRGB8_ALPHA8_ETC2_EAC, glow::RGBA, 0),
        Tf::EacR11Unorm => (glow::COMPRESSED_R11_EAC, glow::RED, 0),
        Tf::EacR11Snorm => (glow::COMPRESSED_SIGNED_R11_EAC, glow::RED, 0),
        Tf::EacRg11Unorm => (glow::COMPRESSED_RG11_EAC, glow::RG, 0),
        Tf::EacRg11Snorm => (glow::COMPRESSED_SIGNED_RG11_EAC, glow::RG, 0),
        Tf::Astc { block, channel } => match channel {
            AstcChannel::Unorm | AstcChannel::Hdr => match block {
                AstcBlock::B4x4 => (glow::COMPRESSED_RGBA_ASTC_4x4_KHR, glow::RGBA, 0),
                AstcBlock::B5x4 => (glow::COMPRESSED_RGBA_ASTC_5x4_KHR, glow::RGBA, 0),
                AstcBlock::B5x5 => (glow::COMPRESSED_RGBA_ASTC_5x5_KHR, glow::RGBA, 0),
                AstcBlock::B6x5 => (glow::COMPRESSED_RGBA_ASTC_6x5_KHR, glow::RGBA, 0),
                AstcBlock::B6x6 => (glow::COMPRESSED_RGBA_ASTC_6x6_KHR, glow::RGBA, 0),
                AstcBlock::B8x5 => (glow::COMPRESSED_RGBA_ASTC_8x5_KHR, glow::RGBA, 0),
                AstcBlock::B8x6 => (glow::COMPRESSED_RGBA_ASTC_8x6_KHR, glow::RGBA, 0),
                AstcBlock::B8x8 => (glow::COMPRESSED_RGBA_ASTC_8x8_KHR, glow::RGBA, 0),
                AstcBlock::B10x5 => (glow::COMPRESSED_RGBA_ASTC_10x5_KHR, glow::RGBA, 0),
                AstcBlock::B10x6 => (glow::COMPRESSED_RGBA_ASTC_10x6_KHR, glow::RGBA, 0),
                AstcBlock::B10x8 => (glow::COMPRESSED_RGBA_ASTC_10x8_KHR, glow::RGBA, 0),
                AstcBlock::B10x10 => (glow::COMPRESSED_RGBA_ASTC_10x10_KHR, glow::RGBA, 0),
                AstcBlock::B12x10 => (glow::COMPRESSED_RGBA_ASTC_12x10_KHR, glow::RGBA, 0),
                AstcBlock::B12x12 => (glow::COMPRESSED_RGBA_ASTC_12x12_KHR, glow::RGBA, 0),
            },
            AstcChannel::UnormSrgb => match block {
                AstcBlock::B4x4 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR, glow::RGBA, 0),
                AstcBlock::B5x4 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR, glow::RGBA, 0),
                AstcBlock::B5x5 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR, glow::RGBA, 0),
                AstcBlock::B6x5 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR, glow::RGBA, 0),
                AstcBlock::B6x6 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR, glow::RGBA, 0),
                AstcBlock::B8x5 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR, glow::RGBA, 0),
                AstcBlock::B8x6 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR, glow::RGBA, 0),
                AstcBlock::B8x8 => (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR, glow::RGBA, 0),
                AstcBlock::B10x5 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR, glow::RGBA, 0)
                }
                AstcBlock::B10x6 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR, glow::RGBA, 0)
                }
                AstcBlock::B10x8 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR, glow::RGBA, 0)
                }
                AstcBlock::B10x10 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR, glow::RGBA, 0)
                }
                AstcBlock::B12x10 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR, glow::RGBA, 0)
                }
                AstcBlock::B12x12 => {
                    (glow::COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR, glow::RGBA, 0)
                }
            },
        },
    };

    super::TextureFormatDesc {
        internal,
        external,
        data_type,
    }
}
