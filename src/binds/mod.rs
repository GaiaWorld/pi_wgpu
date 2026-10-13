// 设计：pi-wgpu-rebuild
//! binds · 绑定槽分配：把布局条目映射到 GL 物理绑定槽。隐藏决定=槽位分配/重排规则。
//! 变更点：UBO/采样器槽上限 16 的放宽只动本模块。契约权威文本住本面注释；本面全部 @internal。
use crate::entry::{BindGroup, PipelineLayout, RenderPipeline};
use crate::types::DynamicOffset;

/// 槽表：UBO 与采样器各 16 个定长槽（槽号即 GL 绑定点号，无运行期扩表）。
/// 不变量：满槽或越界引用=断言——不存在「静默丢绑定」路径。 @internal
#[derive(Debug)] pub(crate) struct SlotTable { pub(crate) ubo: [Option<u32>; 16], pub(crate) sampler: [Option<u32>; 16] }
impl Default for SlotTable { fn default() -> Self { Self { ubo: [None; 16], sampler: [None; 16] } } }
/// 建管线时为布局分配全局槽位：(set,binding)→物理槽号，结果写入设备级槽表；绑定应用时按槽表取用。
/// 域满/越界=断言（报满的域与布局 label），分配中途失败不回写半张表。 @internal
pub(crate) fn alloc_slots(layout: &PipelineLayout, table: &mut SlotTable) { crate::binds_inner::alloc_impl(layout, table) }
/// set_bind_group 的实现底座：按管线记录的槽映射查槽 → 动态偏移按声明序消耗 →
/// 绑定经状态镜像差异下发。任一违约（死句柄/槽对不上）=断言，且在任何 bind 调用发出之前，绑定保持调用前原状。
/// TODO-DECL：槽表与镜像实体在设备侧——连接批在本芯形参加入其借用（芯声明面可加工，公开壳的委托调用点随批同步），骨架桩形参不携带为已知欠账。 @internal
pub(crate) fn apply_layout(pipe: &RenderPipeline, group: &BindGroup, offsets: &[DynamicOffset]) { crate::binds_inner::apply_impl(pipe, group, offsets) }
/// 动态偏移消耗：入参个数必须与绑定组登记的声明数恰好相等——不符=断言且不消耗任何份额；
/// 消耗顺序=布局声明顺序（合同名即行为名，无重排特权）。 @internal
pub(crate) fn consume_dynamic_offsets(group: &BindGroup, offsets: &mut &[DynamicOffset]) { crate::binds_inner::consume_impl(group, offsets) }
