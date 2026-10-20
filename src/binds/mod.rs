// 设计：pi-wgpu-rebuild
//! binds · 绑定槽分配：把布局条目映射到 GL 物理绑定槽。隐藏决定=槽位分配/重排规则。
//! 变更点：UBO/采样器槽上限 16 的放宽只动本模块。契约权威文本住本面注释；本面全部 @internal。
use crate::entry::{BindGroup, PipelineLayout, RenderPipeline};
use crate::state::StateMirror;
use crate::types::DynamicOffset;

/// 绑定域：决定条目落哪张槽表。Uniform=UBO 绑定点槽；Texture/Sampler=纹理单元槽（同一 16 单元空间）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindDomain {
    /// uniform 缓冲绑定点（glBindBufferBase/Range 的 index）。
    Uniform,
    /// 纹理条目（占纹理单元）。
    Texture,
    /// 采样器条目（占纹理单元；纹理×采样器配对由反射批收口，本批各自占槽）。
    Sampler,
}

/// 绑定组条目（应用期单位）：定位键+驱动名来源，兼作资源强引用（生命周期合同：资源先于组销毁=断言）。 @internal
#[derive(Debug, Clone)]
pub(crate) struct BoundEntry {
    /// 布局声明号（与布局条目 (set,binding) 对表；槽查找键的一半，set 在应用点给定）。
    pub(crate) binding: u32,
    /// 资源域：选 ubo 槽或单元槽。
    pub(crate) domain: BindDomain,
    /// 资源强引用句柄（活态判定+驱动名；Drop 即删）。
    pub(crate) handle: crate::objects::DirectHandle,
    /// 缓冲静态偏移（非缓冲=0）。
    pub(crate) offset: u32,
    /// 缓冲绑定尺寸（None=整段；UBO 带偏移/动态时必须声明，否则应用期断言）。
    pub(crate) size: Option<u32>,
    /// 是否动态偏移条目（consume 计数来源；应用期按声明序消耗偏移值）。
    pub(crate) dynamic: bool,
    /// 纹理条目的 GL 绑定目标（自 TextureView；非纹理=0）。
    pub(crate) target: u32,
}

/// 槽表：UBO 与纹理单元各 16 个定长槽；值=占槽条目的 (set,binding) 打包键（set<<16|binding），索引即 GL 绑定点号/单元号。
/// 不变量：满槽或越界引用=断言——不存在「静默丢绑定」路径。
/// 归属：分配结果与「创建它的那份布局」绑定——管线持有自身那份（出码期与绘制期同源），
/// 设备级槽表只作「最近一次分配」的查询面（未设管线即绑定时的唯一可依信息来源）。 @internal
#[derive(Debug, Clone)]
pub(crate) struct SlotTable {
    pub(crate) ubo: [Option<u32>; 16],
    pub(crate) sampler: [Option<u32>; 16],
}
impl Default for SlotTable {
    fn default() -> Self {
        Self {
            ubo: [None; 16],
            sampler: [None; 16],
        }
    }
}
/// 建管线时为布局分配全局槽位：(set,binding)→物理槽号，结果写入设备级槽表；绑定应用时按槽表取用。
/// 域满/越界=断言（报满的域），分配中途失败不回写半张表。 @internal
pub(crate) fn alloc_slots(layout: &PipelineLayout, table: &mut SlotTable) {
    crate::binds_inner::alloc_impl(layout, table)
}
/// set_bind_group 的实现底座：按设备级槽表查槽 → 动态偏移按声明序消耗 → 绑定经状态镜像差异下发。
/// 任一违约（死句柄/槽对不上/动态计数不符）=断言，且在任何 bind 调用发出之前，绑定保持调用前原状。 @internal
pub(crate) fn apply_layout(
    set: u32,
    group: &BindGroup,
    offsets: &[DynamicOffset],
    table: &SlotTable,
    state: &mut StateMirror,
) {
    crate::binds_inner::apply_impl(set, group, offsets, table, state)
}
/// 动态偏移消耗：入参个数必须与绑定组登记的声明数恰好相等——不符=断言且不消耗任何份额；
/// 消耗顺序=布局声明顺序（合同名即行为名，无重排特权）。 @internal
pub(crate) fn consume_dynamic_offsets(group: &BindGroup, offsets: &mut &[DynamicOffset]) {
    crate::binds_inner::consume_impl(group, offsets)
}
