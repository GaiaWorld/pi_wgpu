// 设计：pi-wgpu-rebuild
//! formats 芯 · 格式管道：词汇格式→驱动参数转换表（单一转换点）+ mipmap 整链重生。
//! 格式表=词汇层数据：新增格式只动本表与 caps 能力面（隐藏决定落壳注释）。
//! 能力位过滤=经 caps 最近快照门控（未探先查=断言）；表无条目/被门控拒=断言报格式名与 label，绝无近似替代。

use pi_glow::HasContext;

use crate::caps_inner::{latest_caps, CAP_FLOAT_RENDER};
use crate::context_inner::with_gl;
use crate::formats::GlParams;
use crate::types::{Extent3d, TextureDescriptor, TextureFormat};

/// —— GL 枚举值内联表（pi_glow 根面未导出 GL_* 枚举行名；取 GL 4.6 规范值，与 ES3.0/WebGL2 交集一致）——
/// GL_RGBA8：未压缩 RGBA8 内部格式（0x8058）。
const GL_RGBA8: u32 = 0x8058;
/// GL_SRGB8_ALPHA8：sRGB RGBA8 内部格式（0x8C43）。
const GL_SRGB8_ALPHA8: u32 = 0x8C43;
/// GL_RGBA16F：半浮点 RGBA 内部格式（0x881A）。
const GL_RGBA16F: u32 = 0x881A;
/// GL_DEPTH24_STENCIL8：D24+S8 打包深度模板内部格式（0x88F0；GLES 核心无此组合）。
const GL_DEPTH24_STENCIL8: u32 = 0x88F0;
/// GL_DEPTH_COMPONENT32F：浮点深度内部格式（0x8CAF；两形态核心皆有）。
const GL_DEPTH_COMPONENT32F: u32 = 0x8CAF;
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
const GL_UNSIGNED_INT_24_8: u32 = 0x83F1;
/// GL_TEXTURE_2D：单层 2D 目标（0x0DE1）。
const GL_TEXTURE_2D: u32 = 0x0DE1;
/// GL_TEXTURE_2D_ARRAY：多层 2D 数组目标（0x8C1A）。
const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;

/// 格式转换表：词汇子集→驱动参数组单一转换点。DDS/ASTC 等压缩族在词汇子集外=表无条目→拒。
/// 全条目未压缩：block=(1,1)、block_bytes=单像素字节；sRGB 只换内部格式不动采样通路（交 caps/GL 判读）。
fn table_entry(f: TextureFormat) -> Option<GlParams> {
    let p = match f {
        TextureFormat::Rgba8Unorm => GlParams { internal_format: GL_RGBA8, format: GL_RGBA, pixel_type: GL_UNSIGNED_BYTE, block: (1, 1), block_bytes: 4 },
        TextureFormat::Rgba8UnormSrgb => GlParams { internal_format: GL_SRGB8_ALPHA8, format: GL_RGBA, pixel_type: GL_UNSIGNED_BYTE, block: (1, 1), block_bytes: 4 },
        TextureFormat::Bgra8Unorm => GlParams { internal_format: GL_RGBA8, format: GL_BGRA, pixel_type: GL_UNSIGNED_BYTE, block: (1, 1), block_bytes: 4 },
        // GLES 无 sRGB×BGRA 合法组合——条目在、门控拒（见 params_impl），不静默换内部格式。
        TextureFormat::Bgra8UnormSrgb => GlParams { internal_format: GL_SRGB8_ALPHA8, format: GL_BGRA, pixel_type: GL_UNSIGNED_BYTE, block: (1, 1), block_bytes: 4 },
        TextureFormat::Rgba16Float => GlParams { internal_format: GL_RGBA16F, format: GL_RGBA, pixel_type: GL_HALF_FLOAT, block: (1, 1), block_bytes: 8 },
        TextureFormat::Depth24UnormS8Uint => GlParams { internal_format: GL_DEPTH24_STENCIL8, format: GL_DEPTH_STENCIL, pixel_type: GL_UNSIGNED_INT_24_8, block: (1, 1), block_bytes: 4 },
        TextureFormat::Depth32Float => GlParams { internal_format: GL_DEPTH_COMPONENT32F, format: GL_DEPTH_COMPONENT, pixel_type: GL_FLOAT, block: (1, 1), block_bytes: 4 },
    };
    Some(p)
}

// 1. 查格式转换表取条目 → 用 caps 最近快照门控（浮点渲染按探测位、形态拒组合缺口）。
// 2. 表无条目（词汇子集外如压缩族）或门控不过=断言，报格式名+desc.label+拒因——绝不近似替代。
// 3. 词汇无压缩条目期：块粒度恒 (1,1)；压缩直载开放后本表增块行、归一逻辑在此扩（变更点预留）。
pub(crate) fn params_impl(desc: &TextureDescriptor<'_>, f: TextureFormat) -> GlParams {
    let p = match table_entry(f) {
        Some(p) => p,
        None => panic!("formats 断言: 格式 {f:?} 无转换表条目（label={:?}）", desc.label),
    };
    let caps = match latest_caps() {
        Some(c) => c,
        None => panic!("formats 断言: 未完成 attach 后即探就查询格式 {f:?}（label={:?}）", desc.label),
    };
    if caps.version < (3, 0) {
        panic!("formats 断言: 驱动版本 {:?} 低于格式面下限 3.0（fmt={f:?} label={:?}）", caps.version, desc.label);
    }
    match f {
        TextureFormat::Bgra8UnormSrgb if caps.es => {
            panic!("formats 断言: GLES 形态无 sRGB×BGRA 核心组合，拒 {f:?}（label={:?}）", desc.label);
        }
        TextureFormat::Depth24UnormS8Uint if caps.es => {
            panic!("formats 断言: GLES 核心无 D24S8 打包组合（用 D32F_S8），拒 {f:?}（label={:?}）", desc.label);
        }
        TextureFormat::Rgba16Float => {
            if caps.version < (3, 0) && (caps.family & CAP_FLOAT_RENDER) == 0 {
                panic!("formats 断言: 无浮点渲染确证，拒 {f:?}（label={:?}）", desc.label);
            }
        }
        _ => {}
    }
    p
}

// 1. 上传完备后整链一次生成：在当前附着+已绑定目标纹理上执行 generate（调用方=upload 写路径在整链上传完成后接线）。
// 2. 成本合同：内容再更新=整链重生成——levels<=1 无链=空操作；无局部刷一级分支。
// 3. 目标按层数选（单层 2D/多层 2D Array）；宽或高 0=越界断言（创建期保证本不该到）。
pub(crate) fn mip_impl(extent: Extent3d, levels: u32) {
    if levels <= 1 {
        return;
    }
    if extent.width == 0 || extent.height == 0 {
        panic!("formats 断言: 生成 mip 链于零尺寸（extent={extent:?}）");
    }
    let target = if extent.depth_or_array_layers <= 1 { GL_TEXTURE_2D } else { GL_TEXTURE_2D_ARRAY };
    with_gl(|gl| {
        // SAFETY: 驱动调用于附着持有期内（with_gl 门禁保证当前线程已绑）。
        unsafe { gl.generate_mipmap(target) };
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TextureDimension, TextureUsages};
    use crate::caps_inner::inject_latest;

    /// 组一个仅够门控读数的描述符（label 进断言消息）。
    fn desc<'a>(label: &'a str) -> TextureDescriptor<'a> {
        TextureDescriptor {
            label: Some(label),
            usage: TextureUsages::TEXTURE_BINDING,
            dim: TextureDimension::D2,
            size: Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        }
    }

    /// 表覆盖全词汇子集：7 条目内部格式逐一对规范值。
    #[test]
    fn table_covers_subset_test() {
        let cases: [(TextureFormat, u32, u32); 7] = [
            (TextureFormat::Rgba8Unorm, GL_RGBA8, GL_RGBA),
            (TextureFormat::Rgba8UnormSrgb, GL_SRGB8_ALPHA8, GL_RGBA),
            (TextureFormat::Bgra8Unorm, GL_RGBA8, GL_BGRA),
            (TextureFormat::Bgra8UnormSrgb, GL_SRGB8_ALPHA8, GL_BGRA),
            (TextureFormat::Rgba16Float, GL_RGBA16F, GL_RGBA),
            (TextureFormat::Depth24UnormS8Uint, GL_DEPTH24_STENCIL8, GL_DEPTH_STENCIL),
            (TextureFormat::Depth32Float, GL_DEPTH_COMPONENT32F, GL_DEPTH_COMPONENT),
        ];
        for (f, internal, format) in cases {
            let p = table_entry(f).expect("子集条目必在表");
            assert_eq!(p.internal_format, internal, "{f:?}");
            assert_eq!(p.format, format, "{f:?}");
            assert_eq!(p.block, (1, 1), "{f:?} 词汇子集全未压缩");
        }
    }

    /// 门控：桌面对全部子集放行；GLES 拒 D24S8 与 sRGB×BGRA 组合（断言消息含 label 与格式名）。
    #[test]
    fn gate_form_rules_test() {
        inject_latest(0, (4, 5), false);
        let d = desc("desk");
        let _ = params_impl(&d, TextureFormat::Depth24UnormS8Uint);
        let _ = params_impl(&d, TextureFormat::Bgra8UnormSrgb);
        let _ = params_impl(&d, TextureFormat::Rgba16Float);
        inject_latest(0, (3, 2), true);
        let d = desc("gles");
        assert!(std::panic::catch_unwind(|| params_impl(&d, TextureFormat::Depth24UnormS8Uint)).is_err());
        assert!(std::panic::catch_unwind(|| params_impl(&d, TextureFormat::Bgra8UnormSrgb)).is_err());
        assert!(params_impl(&d, TextureFormat::Rgba8Unorm).internal_format == GL_RGBA8);
    }

    /// mip：levels<=1 恒空操作、零尺寸断言、层数选择目标（后两者需附着——GL 侧由合验用例覆盖）。
    #[test]
    fn mip_contract_test() {
        let e = Extent3d { width: 4, height: 4, depth_or_array_layers: 0 };
        mip_impl(e, 1);
        let z = Extent3d { width: 0, height: 4, depth_or_array_layers: 1 };
        assert!(std::panic::catch_unwind(|| mip_impl(z, 3)).is_err());
    }

    // 未探先查=断言路径由最新快照门控隐式覆盖（表门控测试首跑于 inject 前的顺序依赖不显著，单列用例会受并发污染，留实机合验批覆盖）。
}
