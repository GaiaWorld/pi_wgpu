// 设计：pi-wgpu-rebuild
//! formats 芯 · 格式管道：词汇格式→驱动参数转换表（单一转换点）+ mipmap 整链重生。
//! 格式表=词汇层数据：新增格式只动本表与 caps 能力面（隐藏决定落壳注释）。
//! 能力位过滤=经 caps 最近快照门控（未探先查=断言）；表无条目/被门控拒=断言报格式名与 label，绝无近似替代。

use glow::HasContext;

use crate::caps_inner::{latest_caps, CAP_FLOAT_RENDER, CAP_RENDER_HALF};
use crate::context_inner::with_gl;
use crate::formats::GlParams;
use crate::types::{Extent3d, TextureDescriptor, TextureFormat};

/// —— GL 枚举值内联表（glow 根面未导出 GL_* 枚举行名；取 GL 4.6 规范值，与 ES3.0/WebGL2 交集一致）——
/// GL_R8：单通道 u8 归一化内部格式（0x8229；GL3.0 与 ES3.0 核心均在场=无平台差）。
const GL_R8: u32 = 0x8229;
/// GL_RG8：双通道 u8 归一化内部格式（0x822B；同上，两形态核心同在）。
const GL_RG8: u32 = 0x822B;
/// GL_RGBA32F：全浮点 RGBA 内部格式（0x8814；两形态核心同在，过滤能力另按 FLOAT_LINEAR 判读）。
const GL_RGBA32F: u32 = 0x8814;
/// GL_RGBA8：未压缩 RGBA8 内部格式（0x8058）。
const GL_RGBA8: u32 = 0x8058;
/// GL_SRGB8_ALPHA8：sRGB RGBA8 内部格式（0x8C43）。
const GL_SRGB8_ALPHA8: u32 = 0x8C43;
/// GL_RGBA16F：半浮点 RGBA 内部格式（0x881A）。
const GL_RGBA16F: u32 = 0x881A;
/// GL_DEPTH24_STENCIL8：D24+S8 打包深度模板内部格式（0x88F0；GLES 核心无此组合）。
const GL_DEPTH24_STENCIL8: u32 = 0x88F0;
/// GL_DEPTH_COMPONENT32F：浮点深度内部格式（0x8CAC；GL 3.0+/ES3 皆有）。
/// 勘误：曾用 0x8CAF（非法）→ 0x8C1A（实为 GL_TEXTURE_2D_ARRAY，产品 800x600/1024x1024
/// D32 全数 0x502 实跑定论）→ 现 0x8CAC（glcorearb.h/GLES3 头一致）。
const GL_DEPTH_COMPONENT32F: u32 = 0x8CAC;
/// GL_DEPTH_COMPONENT16：16 位深度内部格式（0x81A5；两形态核心皆有）。
const GL_DEPTH_COMPONENT16: u32 = 0x81A5;
/// GL_DEPTH_COMPONENT24：24 位深度内部格式（0x81A6；两形态核心皆有）。
const GL_DEPTH_COMPONENT24: u32 = 0x81A6;
/// GL_DEPTH32F_STENCIL8：32F 深度+8 位模板打包内部格式（0x8CAD；两形态核心皆有）。
const GL_DEPTH32F_STENCIL8: u32 = 0x8CAD;
/// GL_RED：像素格式单通道（0x1903）。
const GL_RED: u32 = 0x1903;
/// GL_RG：像素格式双通道（0x8227）。
const GL_RG: u32 = 0x8227;
/// GL_RGBA：像素格式 RGBA 元组（0x1908）。
const GL_RGBA: u32 = 0x1908;
/// GL_BGRA（=GL_BGRA_EXT 同值）：像素格式 BGRA 元组（0x80E1，ES 侧同名扩展值）。
const GL_BGRA: u32 = 0x80E1;
/// GL_DEPTH_STENCIL：深度+模板双平面像素格式（0x84F9）。
const GL_DEPTH_STENCIL: u32 = 0x84F9;
/// GL_DEPTH_COMPONENT：深度单平面像素格式（0x1902）。
const GL_DEPTH_COMPONENT: u32 = 0x1902;
/// GL_UNSIGNED_BYTE：u8 归一化数据类型（0x1401）。
const GL_UNSIGNED_BYTE: u32 = 0x1401;
/// GL_HALF_FLOAT：f16 数据类型（0x140B）。
const GL_HALF_FLOAT: u32 = 0x140B;
/// GL_FLOAT：f32 数据类型（0x1406；深度侧=可写但采样过滤按 FLOAT_LINEAR）。
const GL_FLOAT: u32 = 0x1406;
/// GL_UNSIGNED_INT_24_8：D24S8 打包数据类型（0x83F1）。
// Packed depth-stencil upload type is 0x84FA (0x83F1 is UNSIGNED_INT_8_8_8_8).
const GL_UNSIGNED_INT_24_8: u32 = 0x84FA;
/// GL_FLOAT_32_UNSIGNED_INT_24_8_REV：D32F_S8 打包数据类型（0x8DAD）。
const GL_FLOAT_32_UNSIGNED_INT_24_8_REV: u32 = 0x8DAD;
/// GL_UNSIGNED_SHORT：u16 数据类型（0x1403）。
const GL_UNSIGNED_SHORT: u32 = 0x1403;
/// GL_UNSIGNED_INT：u32 数据类型（0x1405）。
const GL_UNSIGNED_INT: u32 = 0x1405;
/// GL_TEXTURE_2D：单层 2D 目标（0x0DE1）。
const GL_TEXTURE_2D: u32 = 0x0DE1;
/// GL_TEXTURE_2D_MULTISAMPLE（多重采样 2D 目标面）。
const GL_TEXTURE_2D_MULTISAMPLE: u32 = 0x9100;
/// GL_TEXTURE_2D_ARRAY：多层 2D 数组目标（0x8C1A）。
const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;

/// 格式转换表：词汇→驱动参数组单一转换点。词汇全表 75 项在场（desc-align 后续批），本表当前
/// 收录已接 GL 映射的 13 项——表无条目（压缩族与未接非色族）=params_impl 断言拒止（形宽法窄）。
/// 全条目未压缩：block=(1,1)、block_bytes=单像素字节；sRGB 只换内部格式不动采样通路（交 caps/GL 判读）。
/// 收录口径=GL3.0∨ES3.0 核心**共有**且映射唯一的格式（无平台分歧；有分歧者留表外断言，不近似替代）。
fn table_entry(f: TextureFormat) -> Option<GlParams> {
    let p = match f {
        // R/RG 单双通道 u8：两形态核心同名同参，映射唯一。
        TextureFormat::R8Unorm => GlParams {
            internal_format: GL_R8,
            format: GL_RED,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 1,
        },
        TextureFormat::Rg8Unorm => GlParams {
            internal_format: GL_RG8,
            format: GL_RG,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 2,
        },
        // 全浮点 RGBA：内部格式两形态同名（过滤能力另行按 caps FLOAT_LINEAR 判读）。
        TextureFormat::Rgba32Float => GlParams {
            internal_format: GL_RGBA32F,
            format: GL_RGBA,
            pixel_type: GL_FLOAT,
            block: (1, 1),
            block_bytes: 16,
        },
        TextureFormat::Rgba8Unorm => GlParams {
            internal_format: GL_RGBA8,
            format: GL_RGBA,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 4,
        },
        TextureFormat::Rgba8UnormSrgb => GlParams {
            internal_format: GL_SRGB8_ALPHA8,
            format: GL_RGBA,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 4,
        },
        TextureFormat::Bgra8Unorm => GlParams {
            internal_format: GL_RGBA8,
            format: GL_BGRA,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 4,
        },
        // GLES 无 sRGB×BGRA 合法组合——条目在、门控拒（见 params_impl），不静默换内部格式。
        TextureFormat::Bgra8UnormSrgb => GlParams {
            internal_format: GL_SRGB8_ALPHA8,
            format: GL_BGRA,
            pixel_type: GL_UNSIGNED_BYTE,
            block: (1, 1),
            block_bytes: 4,
        },
        TextureFormat::Rgba16Float => GlParams {
            internal_format: GL_RGBA16F,
            format: GL_RGBA,
            pixel_type: GL_HALF_FLOAT,
            block: (1, 1),
            block_bytes: 8,
        },
        TextureFormat::Depth24PlusStencil8 => GlParams {
            internal_format: GL_DEPTH24_STENCIL8,
            format: GL_DEPTH_STENCIL,
            pixel_type: GL_UNSIGNED_INT_24_8,
            block: (1, 1),
            block_bytes: 4,
        },
        TextureFormat::Depth32Float => GlParams {
            internal_format: GL_DEPTH_COMPONENT32F,
            format: GL_DEPTH_COMPONENT,
            pixel_type: GL_FLOAT,
            block: (1, 1),
            block_bytes: 4,
        },
        // 深度族余项：内部格式/打包类型两形态核心同名同参，映射唯一（无平台分歧）。
        TextureFormat::Depth16Unorm => GlParams {
            internal_format: GL_DEPTH_COMPONENT16,
            format: GL_DEPTH_COMPONENT,
            pixel_type: GL_UNSIGNED_SHORT,
            block: (1, 1),
            block_bytes: 2,
        },
        TextureFormat::Depth24Plus => GlParams {
            internal_format: GL_DEPTH_COMPONENT24,
            format: GL_DEPTH_COMPONENT,
            pixel_type: GL_UNSIGNED_INT,
            block: (1, 1),
            block_bytes: 4,
        },
        TextureFormat::Depth32FloatStencil8 => GlParams {
            internal_format: GL_DEPTH32F_STENCIL8,
            format: GL_DEPTH_STENCIL,
            pixel_type: GL_FLOAT_32_UNSIGNED_INT_24_8_REV,
            block: (1, 1),
            block_bytes: 8,
        },
        _ => return None,
    };
    Some(p)
}

// 1. 查格式转换表取条目 → 用 caps 最近快照门控（浮点渲染按探测位、形态拒组合缺口）。
// 2. 表无条目（词汇子集外如压缩族）或门控不过=断言，报格式名+desc.label+拒因——绝不近似替代。
// 3. 压缩直载开放后本表增块行、归一逻辑在此扩（变更点预留）；表外词汇由第 2 步断言路径恒拒。
pub(crate) fn params_impl(desc: &TextureDescriptor<'_>, f: TextureFormat) -> GlParams {
    let p = match table_entry(f) {
        Some(p) => p,
        None => panic!(
            "formats 断言: 格式 {f:?} 无转换表条目（label={:?}）",
            desc.label
        ),
    };
    let caps = match latest_caps() {
        Some(c) => c,
        None => panic!(
            "formats 断言: 未完成 attach 后即探就查询格式 {f:?}（label={:?}）",
            desc.label
        ),
    };
    if caps.version < (3, 0) {
        panic!(
            "formats 断言: 驱动版本 {:?} 低于格式面下限 3.0（fmt={f:?} label={:?}）",
            caps.version, desc.label
        );
    }
    match f {
        TextureFormat::Bgra8UnormSrgb if caps.es => {
            panic!(
                "formats 断言: GLES 形态无 sRGB×BGRA 核心组合，拒 {f:?}（label={:?}）",
                desc.label
            );
        }
        TextureFormat::Depth24PlusStencil8 if caps.es => {
            panic!(
                "formats 断言: GLES 核心无 D24S8 打包组合（用 D32F_S8），拒 {f:?}（label={:?}）",
                desc.label
            );
        }
        TextureFormat::Rgba16Float => {
            // ES3 core 的 RGBA16F 仅可采样：作颜色附件须 EXT_color_buffer_half_float / 浮点扩展。
            let render = desc
                .usage
                .contains(crate::types::TextureUsages::RENDER_ATTACHMENT);
            if render && caps.es && (caps.family & (CAP_FLOAT_RENDER | CAP_RENDER_HALF)) == 0 {
                panic!(
                    "formats 断言: ES 无半浮点颜色附件确证，拒 {f:?} 作渲染目标（label={:?}）",
                    desc.label
                );
            }
        }
        _ => {}
    }
    p
}

// 1. 上传完备后整链一次生成：在当前附着+已绑定目标纹理上执行 generate（调用方=upload 写路径在整链上传完成后接线）。
// 2. 成本合同：内容再更新=整链重生成——levels<=1 无链=空操作；无局部刷一级分支。
// 3. 目标按层数选（单层 2D/多层 2D Array）；宽或高 0=越界断言（创建期保证本不该到）。
/// GL 纹理目标常量（GL_* 名面未导出故内联规范值；2D/2D_ARRAY 已在上方定义）。
const GL_TEXTURE_3D: u32 = 0x806F;
const GL_TEXTURE_CUBE_MAP: u32 = 0x8513;
const GL_TEXTURE_CUBE_MAP_ARRAY: u32 = 0x9009;

/// cube 兼容判定（镜像 wgpu is_cube_compatible）。
pub(crate) fn cube_compat_impl(desc: &TextureDescriptor<'_>) -> bool {
    matches!(desc.dimension, crate::types::TextureDimension::D2)
        && desc.size.depth_or_array_layers % 6 == 0
        && desc.sample_count == 1
        && desc.size.width == desc.size.height
}

/// 纹理 GL 目标选择（镜像 wgpu get_info_from_desc）。
pub(crate) fn target_impl(desc: &TextureDescriptor<'_>) -> u32 {
    use crate::types::TextureDimension;
    match desc.dimension {
        TextureDimension::D1 => GL_TEXTURE_2D,
        TextureDimension::D3 => GL_TEXTURE_3D,
        TextureDimension::D2 => {
            let layers = desc.size.depth_or_array_layers;
            match (cube_compat_impl(desc), layers) {
                (false, 1) => {
                    if desc.sample_count > 1 {
                        GL_TEXTURE_2D_MULTISAMPLE
                    } else {
                        GL_TEXTURE_2D
                    }
                }
                (false, _) => GL_TEXTURE_2D_ARRAY,
                (true, 6) => GL_TEXTURE_CUBE_MAP,
                (true, _) => GL_TEXTURE_CUBE_MAP_ARRAY,
            }
        }
    }
}

pub(crate) fn mip_impl(extent: Extent3d, levels: u32) {
    if levels <= 1 {
        return;
    }
    if extent.width == 0 || extent.height == 0 {
        panic!("formats 断言: 生成 mip 链于零尺寸（extent={extent:?}）");
    }
    let target = if extent.depth_or_array_layers <= 1 {
        GL_TEXTURE_2D
    } else {
        GL_TEXTURE_2D_ARRAY
    };
    with_gl(|gl| {
        // SAFETY: 驱动调用于附着持有期内（with_gl 门禁保证当前线程已绑）。
        unsafe { gl.generate_mipmap(target) };
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps_inner::{inject_latest, CAP_FLOAT_RENDER, CAP_RENDER_HALF};
    use crate::types::{TextureDimension, TextureUsages};

    /// 组一个仅够门控读数的描述符（label 进断言消息）。
    fn desc<'a>(label: &'a str) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label: Some(label),
            usage: TextureUsages::TEXTURE_BINDING,
            dimension: TextureDimension::D2,
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        }
    }

    /// 表覆盖全词汇子集：13 条目内部格式逐一对规范值。
    #[test]
    fn table_covers_subset_test() {
        let cases: [(TextureFormat, u32, u32); 13] = [
            (TextureFormat::R8Unorm, GL_R8, GL_RED),
            (TextureFormat::Rg8Unorm, GL_RG8, GL_RG),
            (TextureFormat::Rgba8Unorm, GL_RGBA8, GL_RGBA),
            (TextureFormat::Rgba8UnormSrgb, GL_SRGB8_ALPHA8, GL_RGBA),
            (TextureFormat::Bgra8Unorm, GL_RGBA8, GL_BGRA),
            (TextureFormat::Bgra8UnormSrgb, GL_SRGB8_ALPHA8, GL_BGRA),
            (TextureFormat::Rgba16Float, GL_RGBA16F, GL_RGBA),
            (TextureFormat::Rgba32Float, GL_RGBA32F, GL_RGBA),
            (
                TextureFormat::Depth24PlusStencil8,
                GL_DEPTH24_STENCIL8,
                GL_DEPTH_STENCIL,
            ),
            (
                TextureFormat::Depth32Float,
                GL_DEPTH_COMPONENT32F,
                GL_DEPTH_COMPONENT,
            ),
            (
                TextureFormat::Depth16Unorm,
                GL_DEPTH_COMPONENT16,
                GL_DEPTH_COMPONENT,
            ),
            (
                TextureFormat::Depth24Plus,
                GL_DEPTH_COMPONENT24,
                GL_DEPTH_COMPONENT,
            ),
            (
                TextureFormat::Depth32FloatStencil8,
                GL_DEPTH32F_STENCIL8,
                GL_DEPTH_STENCIL,
            ),
        ];
        for (f, internal, format) in cases {
            let p = table_entry(f).expect("子集条目必在表");
            assert_eq!(p.internal_format, internal, "{f:?}");
            assert_eq!(p.format, format, "{f:?}");
            assert_eq!(p.block, (1, 1), "{f:?} 词汇子集全未压缩");
        }
    }

    /// 表外词汇=断言拒止：全表新增名（压缩族/未接非色族）无转换条目，创建路径恒拒并报格式名。
    #[test]
    fn untabled_format_rejects_test() {
        inject_latest(0, (4, 5), false);
        let d = desc("ut");
        for f in [
            TextureFormat::Bc1RgbaUnorm,
            TextureFormat::R8Snorm,
            TextureFormat::Rgba16Unorm,
        ] {
            assert!(
                std::panic::catch_unwind(|| params_impl(&d, f)).is_err(),
                "{f:?} 表无条目应拒"
            );
        }
    }

    /// 门控：桌面对全部有目条目放行；GLES 拒 D24S8 与 sRGB×BGRA 组合（断言消息含 label 与格式名）。
    #[test]
    fn gate_form_rules_test() {
        inject_latest(0, (4, 5), false);
        let d = desc("desk");
        let _ = params_impl(&d, TextureFormat::Depth24PlusStencil8);
        let _ = params_impl(&d, TextureFormat::Bgra8UnormSrgb);
        let _ = params_impl(&d, TextureFormat::Rgba16Float);
        inject_latest(0, (3, 2), true);
        let d = desc("gles");
        assert!(
            std::panic::catch_unwind(|| params_impl(&d, TextureFormat::Depth24PlusStencil8))
                .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| params_impl(&d, TextureFormat::Bgra8UnormSrgb)).is_err()
        );
        assert!(params_impl(&d, TextureFormat::Rgba8Unorm).internal_format == GL_RGBA8);
    }

    /// RGBA16F 作颜色附件：ES 形态无 half-float 扩展=拒；有 EXT_color_buffer_half_float 或浮点扩展=放行；
    /// 仅作采样（TEXTURE_BINDING）=不受门控。
    #[test]
    fn rgba16f_render_gate_es_test() {
        let mut rt = desc("gles-rt");
        rt.usage = TextureUsages::RENDER_ATTACHMENT;
        let mut tex_only = desc("gles-tex");
        tex_only.usage = TextureUsages::TEXTURE_BINDING;

        inject_latest(0, (3, 2), true);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| params_impl(&rt, TextureFormat::Rgba16Float)))
                .is_err(),
            "ES 无扩展时 RGBA16F 不可作渲染目标"
        );
        let _ = params_impl(&tex_only, TextureFormat::Rgba16Float);

        inject_latest(CAP_RENDER_HALF, (3, 2), true);
        let _ = params_impl(&rt, TextureFormat::Rgba16Float);

        inject_latest(CAP_FLOAT_RENDER, (3, 2), true);
        let _ = params_impl(&rt, TextureFormat::Rgba16Float);
    }

    /// mip：levels<=1 恒空操作、零尺寸断言、层数选择目标（后两者需附着——GL 侧由合验用例覆盖）。
    #[test]
    fn mip_contract_test() {
        let e = Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 0,
        };
        mip_impl(e, 1);
        let z = Extent3d {
            width: 0,
            height: 4,
            depth_or_array_layers: 1,
        };
        assert!(std::panic::catch_unwind(|| mip_impl(z, 3)).is_err());
    }

    // 未探先查=断言路径由最新快照门控隐式覆盖（表门控测试首跑于 inject 前的顺序依赖不显著，单列用例会受并发污染，留实机合验批覆盖）。
}
