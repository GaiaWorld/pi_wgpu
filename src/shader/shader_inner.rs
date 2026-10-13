// 设计：pi-wgpu-rebuild
//! shader 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::entry::{PipelineLayout, ShaderModule};
use crate::shader::CompiledEs300;
use crate::types::{Label, ShaderSource};
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. WGSL：前端解析校验 → 后端出 GLSL ES 3.0（std140 约定，不用 define 改布局）。GLSL 源：直接语法校验后采纳。
// 2. 任一阶段报错：codespan 带行列定位输出（含源 label）→ 断言终止；不产出、不缓存失败源。
// 3. 成功：对产物文本计哈希 → CompiledEs300{ hash }（哈希=程序复用键的唯一来源）。
pub(crate) fn compile_impl<'a>(_s: &ShaderSource<'a>, _l: Label<'a>) -> CompiledEs300 { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 遍历产物的绑定资源表 → 填 (set,binding)↔location/name 入登记（挂在 module 内）。
// 2. 交集原则：消费侧一律按块索引寻址，不按名字符串查询（三形态可用的前提）。
// 3. 与 layout 逐条对表：源缺条目/多条目=断言含 label，模块与表保持调用前原状。
pub(crate) fn register_impl(_m: &mut ShaderModule, _p: &PipelineLayout, _b: &CompiledEs300) { panic!("TODO-DECL") }
