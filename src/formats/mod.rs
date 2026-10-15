// 设计：pi-wgpu-rebuild
//! formats · 格式管道：词汇格式↔驱动参数的单一转换点 + mipmap 链策略。
//! 隐藏决定=格式支持集的存放（新增格式只动本表与 caps 能力面）。契约权威文本住本面注释；本面全部 @internal。
use crate::types::{Extent3d, TextureDescriptor, TextureFormat};

/// 描述符→驱动参数组查询：内部格式/像素型/压缩块尺寸与行距偏移换算。
/// 能力位过滤后才可返回：表里有条目 ≠ 本机可用。不支持=断言（报格式名），不做近似替代。 @internal
pub(crate) fn to_gl_params(desc: &TextureDescriptor<'_>, fmt: TextureFormat) -> GlParams {
    crate::formats_inner::params_impl(desc, fmt)
}
/// mipmap 链生成合同：上传完成后整链重生成——内容更新的成本=全链重来（无仅刷一级的路径，登记为差异）。 @internal
pub(crate) fn mip_convert(extent: Extent3d, levels: u32) {
    crate::formats_inner::mip_impl(extent, levels)
}
/// 纹理 GL 目标选择（镜像 wgpu get_info_from_desc）：D1/D2(层=1)→GL_TEXTURE_2D；D2(层>1)：cube 兼容→GL_TEXTURE_CUBE_MAP(层6)/GL_TEXTURE_CUBE_MAP_ARRAY，否则 GL_TEXTURE_2D_ARRAY；D3→GL_TEXTURE_3D。 @internal
pub(crate) fn target_of(desc: &TextureDescriptor<'_>) -> u32 {
    crate::formats_inner::target_impl(desc)
}
/// cube 兼容判定（镜像 wgpu is_cube_compatible）：D2 且 层数%6==0 且 sample_count==1 且 宽==高。 @internal
pub(crate) fn is_cube_compatible(desc: &TextureDescriptor<'_>) -> bool {
    crate::formats_inner::cube_compat_impl(desc)
}
/// 驱动参数组：内部格式/像素格式/数据类型/压缩块粒度五元组（L7 解冻展齐）。可作为对象复用域缓存键的组成部分（见 objects 面）。 @internal
#[derive(Debug, Clone)]
pub(crate) struct GlParams {
    /// GL 内部格式枚举值（glTexImage/glTexStorage 首参形态）。
    pub(crate) internal_format: u32,
    /// 像素格式枚举值（compressed 时为块格式名；未压缩=RGBA/BGRA/Depth 族）。
    pub(crate) format: u32,
    /// 数据类型枚举值（压缩直通与深度模板组合类型均在此收口；depth surface 写入被驱动拒）。
    pub(crate) pixel_type: u32,
    /// 压缩块宽高（未压缩恒 (1,1)；供 upload 块粒度换算消费）。
    pub(crate) block: (u32, u32),
    /// 每块字节数（未压缩=单像素字节数）。
    pub(crate) block_bytes: u32,
}
