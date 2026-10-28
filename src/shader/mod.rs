// 设计：pi-wgpu-rebuild
//! shader · 着色器桥：源→GLSL ES 3.0 转译 + 资源反射登记。隐藏决定=转译输出形态与反射契约。
//! 契约权威文本住本面注释；本面全部 @internal。
//! 错误处理（错误纪律）：解析/校验/出码失败=短错误码上递，诊断原文走 log::error!，不 panic。
use crate::binds::SlotTable;
use crate::entry::{PipelineLayout, ShaderModule};
use crate::types::{Label, ShaderSource};

/// 绑定类别（反射条目分类：决定应用期走 UBO 槽还是纹理单元）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingKind {
    /// uniform 缓冲（AddressSpace::Uniform）。
    UniformBuffer,
    /// 纹理（Handle + Image 类型）。
    Texture,
    /// 采样器（Handle + Sampler 类型）。
    Sampler,
}

/// 反射条目：(set,binding) 的类别与源名（应用期按块索引寻址的登记单位，不按名字符串查询）。 @internal
#[derive(Debug, Clone)]
pub(crate) struct RefEntry {
    /// set 归属（0–3）。
    pub(crate) group: u32,
    /// 组内绑定号。
    pub(crate) binding: u32,
    /// 绑定类别（域判定）。
    pub(crate) kind: BindingKind,
    /// 源侧变量名（诊断/对表用；应用期不以名寻址）。
    pub(crate) name: String,
}

/// shader 错误码（上递；诊断原文走 log::error!，不进错误值——短码，错误纪律）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShaderErr {
    /// WGSL 前端解析失败。
    ParseWgsl,
    /// GLSL 前端解析失败。
    ParseGlsl,
    /// 模块校验失败。
    Validate,
    /// glsl-out 出码失败。
    Emit,
    /// 产物无匹配入口（stage/entry_point）。
    NoEntry,
    /// 槽表缺 (set,binding) 定位。
    SlotMissing,
    /// 源与布局对表不符（缺/多绑定）。
    LayoutMismatch,
    /// 绑定资源类型不受支持（Handle 非图像/采样器）。
    UnsupportedBinding,
}

/// 反射登记表：转译产物的绑定资源条目。不变量：条目与程序转译哈希同生命周期——哈希失效则表同步作废。 @internal
#[derive(Debug, Default)]
pub(crate) struct ReflectionTable {
    pub(crate) entries: Vec<RefEntry>,
}

/// 单个转译单元的 naga 中间产物（模块 + 校验信息）。 @internal
#[derive(Debug)]
pub(crate) struct ShaderIr {
    /// naga 模块（含入口点与全局资源）。
    pub(crate) module: naga::Module,
    /// 校验信息（出码所需）。
    pub(crate) info: naga::valid::ModuleInfo,
}

/// 转译产物：复用键 + 一/多个 naga 中间产物（WGSL=一模块多入口；GLSL=每段一模块）。
/// 不变量：hash=程序复用域的缓存键；出码（emit_stage）延迟到管线创建（entry_point 那时才知）。 @internal
#[derive(Debug)]
pub(crate) struct CompiledEs300 {
    /// 复用键（源文本哈希；同源同键）。
    pub(crate) hash: u64,
    /// 中间产物集合。
    pub(crate) modules: Vec<ShaderIr>,
}

/// 把源转译为 GLSL 产物素材（WGSL：前端解析+校验；GLSL：按阶段单源校验采纳，形制=desc-align-wgpu25）。
/// 失败面：错误码上递（ParseWgsl/ParseGlsl/Validate/UnsupportedBinding）；诊断原文走 log::error!。 @internal
pub(crate) fn compile_to_es300<'a>(
    src: &ShaderSource<'a>,
    label: Label<'a>,
) -> Result<CompiledEs300, ShaderErr> {
    crate::shader_inner::compile_impl(src, label)
}
/// 转译产物的绑定资源反思进表并挂模块。调用时机=模块创建内、管线创建前（先登记后引用）。
/// 与给定管线布局逐条对表：源里缺绑定/多绑定=Err(LayoutMismatch)（不 panic）。
/// 失败后状态：模块反射表保持调用前原状（不半写）。 @internal
pub(crate) fn reflection_register(
    module: &mut ShaderModule,
    layout: &PipelineLayout,
    blob: &CompiledEs300,
) -> Result<(), ShaderErr> {
    crate::shader_inner::register_impl(module, layout, blob)
}
/// 按 (stage,entry_point) 从产物出码一份 GLSL ES 3.0 文本（glsl-out；binding_map 按当前布局槽表定位）。
/// entry_point=None=该 stage 唯一入口自动选；缺失或歧义=Err(NoEntry)。
/// 未命中入口=Err(NoEntry)；槽缺=Err(SlotMissing)；出码失败=Err(Emit)。 @internal
/// 取「纹理↔采样器」配对表（GLSL 组合采样器同源判定；供槽表携带）。 @internal
pub(crate) fn sampler_pairs(b: &CompiledEs300) -> Vec<((u32, u32), (u32, u32))> {
    crate::shader_inner::sampler_pairs_impl(b)
}
pub(crate) fn emit_stage(
    blob: &CompiledEs300,
    stage: naga::ShaderStage,
    entry_point: Option<&str>,
    slots: &crate::binds::SlotTable,
) -> Result<String, ShaderErr> {
    crate::shader_inner::emit_stage_impl(blob, stage, entry_point, slots)
}
/// 取「会长成 GLSL 组合采样入口」的 (group,binding) 集（供程序侧指位定位）。
/// 纯函数，不失败；空表=该程序无采样入口。无副作用、幂等。 @internal
pub(crate) fn sampler_targets(refl: &ReflectionTable) -> Vec<(u32, u32)> {
    crate::shader_inner::sampler_targets_impl(refl)
}
/// 从转译产物直接收集反射条目（同登记面，但**不与布局对表**——管线创建期求采样入口候选用）。
/// 失败面：产物含不受支持的 Handle 类型=Err(UnsupportedBinding)；诊断原文走 log::error!。
/// 失败后状态：不写任何模块字段（本操作只产出局部表）。 @internal
pub(crate) fn reflection_of(blob: &CompiledEs300) -> Result<ReflectionTable, ShaderErr> {
    crate::shader_inner::reflection_of_impl(blob)
}
