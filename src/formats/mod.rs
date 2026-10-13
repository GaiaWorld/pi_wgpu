// 设计：pi-wgpu-rebuild
//! formats · 格式管道：词汇格式↔驱动参数的单一转换点 + mipmap 链策略。
//! 隐藏决定=格式支持集的存放（新增格式只动本表与 caps 能力面）。契约权威文本住本面注释；本面全部 @internal。
use crate::types::{Extent3d, TextureDescriptor, TextureFormat};

/// 描述符→驱动参数组查询：内部格式/像素型/压缩块尺寸与行距偏移换算。
/// 能力位过滤后才可返回：表里有条目 ≠ 本机可用。不支持=断言（报格式名），不做近似替代。 @internal
pub(crate) fn to_gl_params(desc: &TextureDescriptor<'_>, fmt: TextureFormat) -> GlParams { crate::formats_inner::params_impl(desc, fmt) }
/// mipmap 链生成合同：上传完成后整链重生成——内容更新的成本=全链重来（无仅刷一级的路径，登记为差异）。 @internal
pub(crate) fn mip_convert(extent: Extent3d, levels: u32) { crate::formats_inner::mip_impl(extent, levels) }
/// 驱动参数组（内部格式等；字段由实现批展开）。可作为对象复用域缓存键的组成部分（见 objects 面）。 @internal
#[derive(Debug)] pub(crate) struct GlParams { pub(crate) internal_format: u32 }
