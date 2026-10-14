// 设计：pi-wgpu-rebuild
//! shader 芯：naga 转译（源→校验→IR）+ 反射登记 + 按入口出码（glsl-out）。契约权威文本住壳面注释。
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use naga::AddressSpace;

use crate::binds::SlotTable;
use crate::entry::{PipelineLayout, ShaderModule};
use crate::shader::{BindingKind, CompiledEs300, RefEntry, ReflectionTable, ShaderIr};
use crate::types::{Label, ShaderSource};

/// 槽键打包（与 binds 同式：set 高 16 位/binding 低 16 位）。
fn pack(group: u32, binding: u32) -> u32 {
    (group << 16) | (binding & 0xFFFF)
}

/// 源文本哈希（复用键；同源同键）。
fn hash_text(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// 校验一模块并取信息（失败=断言）。
fn validate(module: &naga::Module, label: Label<'_>, what: &str) -> naga::valid::ModuleInfo {
    match naga::valid::Validator::new(naga::valid::ValidationFlags::default(), naga::valid::Capabilities::default()).validate(module) {
        Ok(info) => info,
        Err(e) => panic!("shader 断言: {what} 校验失败 label={label:?}: {e:?}"),
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. WGSL：前端解析校验（错误=codespan emit_to_string 后断言）→ 校验取信息。
// 2. GLSL：逐段（vertex/fragment）前端解析校验（语法错=断言含定位）。
// 3. 成功：源文本哈希=复用键；一/多个 IR 入产物；不产出失败源。
pub(crate) fn compile_impl<'a>(s: &ShaderSource<'a>, l: Label<'a>) -> CompiledEs300 {
    match s {
        ShaderSource::Wgsl(w) => {
            let module = match naga::front::wgsl::parse_str(w.code) {
                Ok(m) => m,
                Err(e) => panic!("shader 断言: WGSL 解析失败 label={l:?}\n{}", e.emit_to_string(w.code)),
            };
            let info = validate(&module, l, "WGSL");
            CompiledEs300 { hash: hash_text(w.code), modules: vec![ShaderIr { module, info }] }
        }
        ShaderSource::Glsl(g) => {
            let mut modules: Vec<ShaderIr> = Vec::new();
            if let Some(v) = g.vertex.as_ref() {
                modules.push(parse_glsl(v.code, naga::ShaderStage::Vertex, l));
            }
            if let Some(f) = g.fragment.as_ref() {
                modules.push(parse_glsl(f.code, naga::ShaderStage::Fragment, l));
            }
            if modules.is_empty() {
                panic!("shader 断言: GLSL 源至少需一段（vertex/fragment） label={l:?}");
            }
            let mut h = DefaultHasher::new();
            if let Some(v) = g.vertex.as_ref() { v.code.hash(&mut h); }
            if let Some(f) = g.fragment.as_ref() { f.code.hash(&mut h); }
            CompiledEs300 { hash: h.finish(), modules }
        }
    }
}

/// GLSL 单段：前端解析（错误=断言含定位）→ 校验。
fn parse_glsl(code: &str, stage: naga::ShaderStage, label: Label<'_>) -> ShaderIr {
    let mut front = naga::front::glsl::Frontend::default();
    let opts = naga::front::glsl::Options::from(stage);
    let module = match front.parse(&opts, code) {
        Ok(m) => m,
        Err(es) => panic!("shader 断言: GLSL 解析失败 stage={stage:?} label={label:?}: {es:?}"),
    };
    let info = validate(&module, label, "GLSL");
    ShaderIr { module, info }
}

/// 从模块收集反射条目（去重）。
fn collect_entries(module: &naga::Module, out: &mut Vec<RefEntry>) {
    for (_h, gv) in module.global_variables.iter() {
        let Some(rb) = gv.binding else { continue };
        let kind = match gv.space {
            AddressSpace::Uniform => BindingKind::UniformBuffer,
            AddressSpace::Handle => match module.types[gv.ty].inner {
                naga::TypeInner::Image { .. } => BindingKind::Texture,
                naga::TypeInner::Sampler { .. } => BindingKind::Sampler,
                _ => panic!("shader 断言: Handle 资源非图像/采样器 (set={},binding={})", rb.group, rb.binding),
            },
            _ => continue,
        };
        if !out.iter().any(|e| e.group == rb.group && e.binding == rb.binding) {
            out.push(RefEntry { group: rb.group, binding: rb.binding, kind, name: gv.name.clone().unwrap_or_default() });
        }
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 遍历产物的绑定资源表 → 填 (set,binding)↔类别/名 入登记（挂在 module 内）。
// 2. 交集原则：消费侧一律按块索引寻址，不按名字符串查询。
// 3. 与 layout 逐条对表：源缺条目/多条目=断言，模块与表保持调用前原状。
pub(crate) fn register_impl(m: &mut ShaderModule, p: &PipelineLayout, b: &CompiledEs300) {
    let mut entries: Vec<RefEntry> = Vec::new();
    for ir in b.modules.iter() {
        collect_entries(&ir.module, &mut entries);
    }
    let mut layout_pairs: Vec<(u32, u32)> = Vec::new();
    for (set, bgl) in p._bgls.iter().enumerate() {
        for e in bgl._entries.iter() {
            layout_pairs.push((set as u32, e.binding));
        }
    }
    for e in entries.iter() {
        if !layout_pairs.iter().any(|(s, bnd)| *s == e.group && *bnd == e.binding) {
            panic!("shader 断言: 源多绑定 (set={},binding={}) 不在布局", e.group, e.binding);
        }
    }
    for (s, bnd) in layout_pairs.iter() {
        if !entries.iter().any(|e| e.group == *s && e.binding == *bnd) {
            panic!("shader 断言: 源缺绑定 (set={s},binding={bnd})（布局声明但源未用）");
        }
    }
    m._refl = ReflectionTable { entries };
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 选含 (stage,entry_point) 的模块；未命中=断言。
// 2. binding_map=(set,binding)→GLSL 绑定位（按布局槽表定位；缺槽=断言）。
// 3. glsl-out 出码 ES 3.0 文本（std140）；任一失败=断言。
pub(crate) fn emit_stage_impl(b: &CompiledEs300, stage: naga::ShaderStage, entry_point: &str, slots: &SlotTable) -> String {
    let ir = b
        .modules
        .iter()
        .find(|ir| ir.module.entry_points.iter().any(|ep| ep.stage == stage && ep.name == entry_point))
        .unwrap_or_else(|| panic!("shader 断言: 产物无入口 stage={stage:?} entry_point={entry_point}"));
    let mut bmap = naga::back::glsl::BindingMap::new();
    for (_h, gv) in ir.module.global_variables.iter() {
        if let Some(rb) = gv.binding {
            let key = pack(rb.group, rb.binding);
            let seg = match gv.space {
                AddressSpace::Uniform => &slots.ubo,
                _ => &slots.sampler,
            };
            let slot = seg
                .iter()
                .position(|x| *x == Some(key))
                .unwrap_or_else(|| panic!("shader 断言: 槽表无 (set={},binding={})", rb.group, rb.binding));
            bmap.insert(rb, slot as u8);
        }
    }
    let options = naga::back::glsl::Options {
        version: naga::back::glsl::Version::new_gles(300),
        writer_flags: naga::back::glsl::WriterFlags::empty(),
        binding_map: bmap,
        zero_initialize_workgroup_memory: false,
    };
    let popts = naga::back::glsl::PipelineOptions { shader_stage: stage, entry_point: entry_point.to_string(), multiview: None };
    let policies = naga::proc::BoundsCheckPolicies::default();
    let mut out = String::new();
    let mut writer = naga::back::glsl::Writer::new(&mut out, &ir.module, &ir.info, &options, &popts, policies)
        .unwrap_or_else(|e| panic!("shader 断言: glsl 出码初始化失败 {e:?}"));
    writer.write().unwrap_or_else(|e| panic!("shader 断言: glsl 出码失败 {e:?}"));
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::{compile_to_es300, emit_stage, reflection_register};
    use crate::types::{WgslSource, ShaderSource as SS};

    const WGSL: &str = "@group(0) @binding(0) var<uniform> u: vec4<f32>;\n@vertex\nfn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> { return u; }";

    /// WGSL 转译：产物 IR + 反射条目 + 按入口出 ES300。
    #[test]
    fn wgsl_compile_and_emit_test() {
        let src = SS::Wgsl(WgslSource { code: WGSL });
        let blob = compile_to_es300(&src, Some("t"));
        assert_eq!(blob.modules.len(), 1, "WGSL 单模块");
        let mut entries = Vec::new();
        collect_entries(&blob.modules[0].module, &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].group, entries[0].binding), (0, 0));
        assert_eq!(entries[0].kind, BindingKind::UniformBuffer);
        let mut slots = SlotTable::default();
        slots.ubo[0] = Some(pack(0, 0));
        let glsl = emit_stage(&blob, naga::ShaderStage::Vertex, "vs", &slots);
        assert!(glsl.contains("#version 300 es"), "ES300 头: {glsl}");
    }

    /// 语法错=断言（含 codespan 定位）。
    #[test]
    fn wgsl_syntax_error_asserts_test() {
        let src = SS::Wgsl(WgslSource { code: "@vertex fn vs( { }" });
        let r = std::panic::catch_unwind(|| compile_to_es300(&src, Some("bad")));
        assert!(r.is_err(), "语法错=断言");
    }

    /// 反射对表：命中通过；源缺/多绑定=断言。
    #[test]
    fn register_cross_check_test() {
        use crate::entry::{BindGroupLayout, PipelineLayout, ShaderModule};
        use crate::shader::ReflectionTable;
        use crate::types::{BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};
        let src = SS::Wgsl(WgslSource { code: WGSL });
        let blob = compile_to_es300(&src, Some("t"));
        let mk = |binding: u32| PipelineLayout {
            _bgls: vec![BindGroupLayout { _entries: vec![BindGroupLayoutEntry {
                binding, visibility: ShaderStages::VERTEX, count: None,
                ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            }] }],
        };
        let mut m = ShaderModule { _refl: ReflectionTable::default(), _compiled: compile_to_es300(&src, Some("t")) };
        reflection_register(&mut m, &mk(0), &blob);
        assert_eq!(m._refl.entries.len(), 1, "对表通过后登记");
        let mut m2 = ShaderModule { _refl: ReflectionTable::default(), _compiled: compile_to_es300(&src, Some("t")) };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reflection_register(&mut m2, &mk(1), &blob)));
        assert!(r.is_err(), "缺/多绑定=断言");
    }
}

