// 设计：pi-wgpu-rebuild
//! shader · 着色器桥：源→GLSL ES 3.0 转译 + 资源反射登记。隐藏决定=转译输出形态与反射契约。
//! 契约权威文本住本面注释；本面全部 @internal。
use crate::entry::{PipelineLayout, ShaderModule};
use crate::types::{ShaderSource, Label};

/// 反射登记表：(set,binding)↔location/name 的双向对应。不变量：条目与程序转译哈希同生命周期——
/// 哈希失效则表同步作废。 TODO-DECL：字段形态=实现批。 @internal
#[derive(Debug, Default)] pub(crate) struct ReflectionTable;
/// 把源转译为 GLSL ES 3.0 产物（std140 目标，不用宏 define 迁就）。
/// 失败面：语法/转译错误=先 codespan 输出定位（含源 label），再断言终止，绝不返回半产物。 @internal
pub(crate) fn compile_to_es300<'a>(src: &ShaderSource<'a>, label: Label<'a>) -> CompiledEs300 { crate::shader_inner::compile_impl(src, label) }
/// 转译产物的绑定资源反思进表并挂模块。调用时机=模块创建内、管线创建前（先登记后引用）。
/// 与给定管线布局逐条对表：源里缺绑定/多绑定=断言含 label。 @internal
pub(crate) fn reflection_register(module: &mut ShaderModule, layout: &PipelineLayout, blob: &CompiledEs300) { crate::shader_inner::register_impl(module, layout, blob) }
/// 转译产物。不变量：hash=程序复用域的缓存键；转译输出形态变更时哈希必须随动
/// （否则旧程序被错误键命中——复用正确性的根）。 @internal
#[derive(Debug)] pub(crate) struct CompiledEs300 { pub(crate) hash: u64 }
