// 设计：pi-wgpu-rebuild
//! formats 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::formats::GlParams;
use crate::types::{Extent3d, TextureDescriptor, TextureFormat};
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 查格式转换表取条目 → 用 caps 能力位过滤（压缩族/浮点渲染面按探测结果放行）。
// 2. 压缩格式做块布局归一（行距/偏移换算成块粒度）。
// 3. 表无条目或能力位未点亮=断言（报格式名与查询上下文）——绝不悄悄改用近似格式。
pub(crate) fn params_impl(_d: &TextureDescriptor<'_>, _f: TextureFormat) -> GlParams { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 上传完备后整链一次生成（当前附着+目标对象上执行）。
// 2. 成本合同：内容再更新=整链重生成，不提供局部刷新分支。
pub(crate) fn mip_impl(_e: Extent3d, _l: u32) { panic!("TODO-DECL") }
