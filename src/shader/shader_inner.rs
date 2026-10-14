// 设计：pi-wgpu-rebuild
//! shader 芯：naga 转译（源→校验→IR）+ 反射登记 + 按入口出码（glsl-out）。
//! 错误处理（错误纪律）：解析/校验/出码失败=短错误码上递；诊断原文走 log::error!。
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use naga::AddressSpace;

use crate::binds::SlotTable;
use crate::entry::{PipelineLayout, ShaderModule};
use crate::shader::{BindingKind, CompiledEs300, RefEntry, ReflectionTable, ShaderErr, ShaderIr};
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

/// 校验一模块并取信息（失败=log 诊断 + 上递码）。
fn validate(module: &naga::Module, label: Label<'_>, what: &str) -> Result<naga::valid::ModuleInfo, ShaderErr> {
    naga::valid::Validator::new(naga::valid::ValidationFlags::default(), naga::valid::Capabilities::default())
        .validate(module)
        .map_err(|e| {
            log::error!("shader {:?} label={label:?} what={what}: {e:?}", ShaderErr::Validate);
            ShaderErr::Validate
        })
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. WGSL：前端解析校验（错误=emit_to_string 记 log 后上递 ParseWgsl）→ 校验取信息。
// 2. GLSL：逐段（vertex/fragment）前端解析校验（语法错=上递 ParseGlsl，诊断记 log）。
// 3. 成功：源文本哈希=复用键；一/多个 IR 入产物；不产出失败源。
pub(crate) fn compile_impl<'a>(s: &ShaderSource<'a>, l: Label<'a>) -> Result<CompiledEs300, ShaderErr> {
    match s {
        ShaderSource::Wgsl(w) => {
            let module = naga::front::wgsl::parse_str(w.code).map_err(|e| {
                log::error!("shader {:?} label={l:?}\n{}", ShaderErr::ParseWgsl, e.emit_to_string(w.code));
                ShaderErr::ParseWgsl
            })?;
            let info = validate(&module, l, "WGSL")?;
            Ok(CompiledEs300 { hash: hash_text(w.code), modules: vec![ShaderIr { module, info }] })
        }
        ShaderSource::Glsl(g) => {
            let mut modules: Vec<ShaderIr> = Vec::new();
            if let Some(v) = g.vertex.as_ref() {
                modules.push(parse_glsl(v.code, naga::ShaderStage::Vertex, l)?);
            }
            if let Some(f) = g.fragment.as_ref() {
                modules.push(parse_glsl(f.code, naga::ShaderStage::Fragment, l)?);
            }
            if modules.is_empty() {
                log::error!("shader {:?} label={l:?}", ShaderErr::EmptyGlsl);
                return Err(ShaderErr::EmptyGlsl);
            }
            let mut h = DefaultHasher::new();
            if let Some(v) = g.vertex.as_ref() { v.code.hash(&mut h); }
            if let Some(f) = g.fragment.as_ref() { f.code.hash(&mut h); }
            Ok(CompiledEs300 { hash: h.finish(), modules })
        }
    }
}

/// GLSL 单段：前端解析（错误=上递 ParseGlsl，诊断记 log）→ 校验。
fn parse_glsl(code: &str, stage: naga::ShaderStage, label: Label<'_>) -> Result<ShaderIr, ShaderErr> {
    let mut front = naga::front::glsl::Frontend::default();
    let opts = naga::front::glsl::Options::from(stage);
    let module = front.parse(&opts, code).map_err(|es| {
        log::error!("shader {:?} stage={stage:?} label={label:?}: {es:?}", ShaderErr::ParseGlsl);
        ShaderErr::ParseGlsl
    })?;
    let info = validate(&module, label, "GLSL")?;
    Ok(ShaderIr { module, info })
}

/// 从模块收集反射条目（去重；不支持类型=上递 UnsupportedBinding）。
fn collect_entries(module: &naga::Module, out: &mut Vec<RefEntry>) -> Result<(), ShaderErr> {
    for (_h, gv) in module.global_variables.iter() {
        let Some(rb) = gv.binding else { continue };
        let kind = match gv.space {
            AddressSpace::Uniform => BindingKind::UniformBuffer,
            AddressSpace::Handle => match module.types[gv.ty].inner {
                naga::TypeInner::Image { .. } => BindingKind::Texture,
                naga::TypeInner::Sampler { .. } => BindingKind::Sampler,
                _ => {
                    log::error!("shader {:?} set={} binding={}", ShaderErr::UnsupportedBinding, rb.group, rb.binding);
                    return Err(ShaderErr::UnsupportedBinding);
                }
            },
            _ => continue,
        };
        if !out.iter().any(|e| e.group == rb.group && e.binding == rb.binding) {
            out.push(RefEntry { group: rb.group, binding: rb.binding, kind, name: gv.name.clone().unwrap_or_default() });
        }
    }
    Ok(())
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 遍历产物的绑定资源表 → 填 (set,binding)↔类别/名 入登记（挂在 module 内）。
// 2. 交集原则：消费侧一律按块索引寻址，不按名字符串查询。
// 3. 与 layout 逐条对表：源缺/多绑定=Err(LayoutMismatch)，模块与表保持调用前原状。
pub(crate) fn register_impl(m: &mut ShaderModule, p: &PipelineLayout, b: &CompiledEs300) -> Result<(), ShaderErr> {
    let mut entries: Vec<RefEntry> = Vec::new();
    for ir in b.modules.iter() {
        collect_entries(&ir.module, &mut entries)?;
    }
    let mut layout_pairs: Vec<(u32, u32)> = Vec::new();
    for (set, bgl) in p._bgls.iter().enumerate() {
        for e in bgl._entries.iter() {
            layout_pairs.push((set as u32, e.binding));
        }
    }
    for e in entries.iter() {
        if !layout_pairs.iter().any(|(s, bnd)| *s == e.group && *bnd == e.binding) {
            log::error!("shader {:?} 多绑定 set={} binding={}", ShaderErr::LayoutMismatch, e.group, e.binding);
            return Err(ShaderErr::LayoutMismatch);
        }
    }
    for (s, bnd) in layout_pairs.iter() {
        if !entries.iter().any(|e| e.group == *s && e.binding == *bnd) {
            log::error!("shader {:?} 缺绑定 set={s} binding={bnd}", ShaderErr::LayoutMismatch);
            return Err(ShaderErr::LayoutMismatch);
        }
    }
    m._refl = ReflectionTable { entries };
    Ok(())
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 选含 (stage,entry_point) 的模块；未命中=Err(NoEntry)。
// 2. binding_map=(set,binding)→GLSL 绑定位（按布局槽表定位；缺槽=Err(SlotMissing)）。
// 3. glsl-out 出码 ES 3.0 文本（std140）；任一失败=Err(Emit)。
pub(crate) fn emit_stage_impl(b: &CompiledEs300, stage: naga::ShaderStage, entry_point: &str, slots: &SlotTable) -> Result<String, ShaderErr> {
    let ir = b
        .modules
        .iter()
        .find(|ir| ir.module.entry_points.iter().any(|ep| ep.stage == stage && ep.name == entry_point))
        .ok_or_else(|| {
            log::error!("shader {:?} stage={stage:?} entry_point={entry_point}", ShaderErr::NoEntry);
            ShaderErr::NoEntry
        })?;
    let mut bmap = naga::back::glsl::BindingMap::new();
    for (_h, gv) in ir.module.global_variables.iter() {
        if let Some(rb) = gv.binding {
            let key = pack(rb.group, rb.binding);
            let seg = match gv.space {
                AddressSpace::Uniform => &slots.ubo,
                _ => &slots.sampler,
            };
            let slot = seg.iter().position(|x| *x == Some(key)).ok_or_else(|| {
                log::error!("shader {:?} set={} binding={}", ShaderErr::SlotMissing, rb.group, rb.binding);
                ShaderErr::SlotMissing
            })?;
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
    let mut writer = naga::back::glsl::Writer::new(&mut out, &ir.module, &ir.info, &options, &popts, policies).map_err(|e| {
        log::error!("shader {:?} init: {e:?}", ShaderErr::Emit);
        ShaderErr::Emit
    })?;
    writer.write().map_err(|e| {
        log::error!("shader {:?}: {e:?}", ShaderErr::Emit);
        ShaderErr::Emit
    })?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::{compile_to_es300, emit_stage, reflection_register};
    use crate::types::{ShaderSource as SS, WgslSource};

    const WGSL: &str = "@group(0) @binding(0) var<uniform> u: vec4<f32>;\n@vertex\nfn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> { return u; }";

    /// WGSL 转译：产物 IR + 反射条目 + 按入口出 ES300。
    #[test]
    fn wgsl_compile_and_emit_test() {
        let src = SS::Wgsl(WgslSource { code: WGSL });
        let blob = compile_to_es300(&src, Some("t")).expect("转译应成功");
        assert_eq!(blob.modules.len(), 1, "WGSL 单模块");
        let mut entries = Vec::new();
        collect_entries(&blob.modules[0].module, &mut entries).expect("收条目");
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].group, entries[0].binding), (0, 0));
        assert_eq!(entries[0].kind, BindingKind::UniformBuffer);
        let mut slots = SlotTable::default();
        slots.ubo[0] = Some(pack(0, 0));
        let glsl = emit_stage(&blob, naga::ShaderStage::Vertex, "vs", &slots).expect("出码");
        assert!(glsl.contains("#version 300 es"), "ES300 头: {glsl}");
    }

    /// 语法错=错误码上递（不 panic）。
    #[test]
    fn wgsl_syntax_error_returns_err_test() {
        let src = SS::Wgsl(WgslSource { code: "@vertex fn vs( { }" });
        assert_eq!(compile_to_es300(&src, Some("bad")).err(), Some(ShaderErr::ParseWgsl));
    }

    /// 反射对表：命中 Ok；源缺/多绑定=Err(LayoutMismatch)。
    #[test]
    fn register_cross_check_test() {
        use crate::entry::{BindGroupLayout, PipelineLayout, ShaderModule};
        use crate::shader::ReflectionTable;
        use crate::types::{BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages};
        let src = SS::Wgsl(WgslSource { code: WGSL });
        let blob = compile_to_es300(&src, Some("t")).expect("转译");
        let mk = |binding: u32| PipelineLayout {
            _bgls: vec![BindGroupLayout { _entries: vec![BindGroupLayoutEntry {
                binding, visibility: ShaderStages::VERTEX, count: None,
                ty: BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            }] }],
        };
        let mut m = ShaderModule { _refl: ReflectionTable::default(), _compiled: compile_to_es300(&src, Some("t")).expect("转译") };
        reflection_register(&mut m, &mk(0), &blob).expect("对表应通过");
        assert_eq!(m._refl.entries.len(), 1, "对表通过后登记");
        let mut m2 = ShaderModule { _refl: ReflectionTable::default(), _compiled: compile_to_es300(&src, Some("t")).expect("转译") };
        assert_eq!(reflection_register(&mut m2, &mk(1), &blob).err(), Some(ShaderErr::LayoutMismatch), "缺/多绑定=Err");
    }
}
