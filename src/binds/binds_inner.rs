// 设计：pi-wgpu-rebuild
//! binds 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::binds::SlotTable;
use crate::entry::{BindGroup, PipelineLayout, RenderPipeline};
use crate::types::DynamicOffset;
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 遍历布局条目按域归类（uniform 缓冲/采样器）。
// 2. 分配策略：绑定相同资源的条目尽量共槽（无冲突即重合）；无法重合顺延取下一空位。
// 3. 域满=断言（报域名与已满的 16 槽水位），已分配部分整体作废（表回原状）。
pub(crate) fn alloc_impl(_l: &PipelineLayout, _t: &mut SlotTable) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 资源与布局对表取句柄；死句柄=断言，未发出任何 bind。
// 2. uniform 条目→绑定到记录槽号（偏移=静态+消耗所得动态值）；采样器条目→按槽绑到纹理单元。
// 3. 每条绑定先过状态镜像（相同则跳过）——热路径零驱动查询；随后 flush 差异下发。
pub(crate) fn apply_impl(_p: &RenderPipeline, _g: &BindGroup, _o: &[DynamicOffset]) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 比较组登记声明数与切片长度：不符=断言，切片原样不动（绝不消耗半份再报错）。
// 2. 符合→按声明序从头取用，切片游标前移（顺序无特权=声明序）。
pub(crate) fn consume_impl(_g: &BindGroup, _o: &mut &[DynamicOffset]) { panic!("TODO-DECL") }
