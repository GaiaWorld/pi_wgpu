// 设计：pi-wgpu-rebuild
//! binds 芯：槽分配（reorder）+ 动态偏移消耗 + 应用下发闭环。契约权威文本住壳面注释。
use crate::binds::{BindDomain, BoundEntry, SlotTable};
use crate::entry::{BindGroup, PipelineLayout, RenderPipeline};
use crate::state::StateMirror;
use crate::state_inner::{bind_sampler, bind_texture, bind_ubo};
use crate::types::{BindingType, BufferBindingType, DynamicOffset};

/// 槽键打包：set 与 binding 合成定位键（set<<16 | binding）。
fn pack(set: u32, binding: u32) -> u32 {
    (set << 16) | (binding & 0xFFFF)
}

/// 域→槽表段（Texture/Sampler 共用同一 16 单元空间）。
fn domain_array(t: &SlotTable, d: BindDomain) -> &[Option<u32>; 16] {
    match d {
        BindDomain::Uniform => &t.ubo,
        BindDomain::Texture | BindDomain::Sampler => &t.sampler,
    }
}

/// 查槽：返回该 (set,binding) 占用槽号；未找到=断言（布局与组不对表）。
fn lookup_slot(t: &SlotTable, d: BindDomain, set: u32, binding: u32) -> u32 {
    let key = pack(set, binding);
    for (i, v) in domain_array(t, d).iter().enumerate() {
        if *v == Some(key) {
            return i as u32;
        }
    }
    panic!("binds 断言: 槽缺 (set={set},binding={binding}) 域={d:?}（布局与组不对表）");
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 遍历布局成员（set=序位）逐条目按域归类：Uniform→ubo 段；Texture/Sampler→单元段。
// 2. 逐域紧凑编号（0..）：编号即 GL 绑定点/单元号；超 16=断言（报满的域），中途失败不半写。
// 3. 全部成功才整体提交（设备级槽表值=占槽 (set,binding) 打包键）。
pub(crate) fn alloc_impl(layout: &PipelineLayout, table: &mut SlotTable) {
    let mut ubo: [Option<u32>; 16] = [None; 16];
    let mut unit: [Option<u32>; 16] = [None; 16];
    let mut nu = 0usize;
    let mut nt = 0usize;
    for (set, bgl) in layout._bgls.iter().enumerate() {
        for e in bgl._entries.iter() {
            let key = pack(set as u32, e.binding);
            let is_uniform = matches!(e.ty, BindingType::Buffer { ty: BufferBindingType::Uniform, .. });
            if is_uniform {
                if nu >= 16 {
                    panic!("binds 断言: UBO 槽满（16）set={set} binding={}", e.binding);
                }
                ubo[nu] = Some(key);
                nu += 1;
            } else {
                if nt >= 16 {
                    panic!("binds 断言: 纹理单元槽满（16）set={set} binding={}", e.binding);
                }
                unit[nt] = Some(key);
                nt += 1;
            }
        }
    }
    *table = SlotTable { ubo, sampler: unit };
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 预检：全部句柄活态、槽齐备、动态偏移个数=声明数——任一违反=断言，未发任何 bind。
// 2. 逐条只改镜像期望（bind_* 差异才实际下发）；动态偏移按声明序消耗、叠加静态偏移。
// 3. 死句柄/槽缺=断言且绑定保持调用前原状（预检在动作前）。
pub(crate) fn apply_impl(set: u32, group: &BindGroup, offsets: &[DynamicOffset], table: &SlotTable, state: &mut StateMirror) {
    let need = group._bound.iter().filter(|e: &&BoundEntry| e.dynamic).count();
    if offsets.len() != need {
        panic!("binds 断言: 动态偏移个数 {} ≠ 组声明数 {need}", offsets.len());
    }
    for e in group._bound.iter() {
        if e.handle.name().is_none() {
            panic!("binds 断言: 死句柄 set={set} binding={}", e.binding);
        }
        let _ = lookup_slot(table, e.domain, set, e.binding);
    }
    let mut di = 0usize;
    for e in group._bound.iter() {
        let slot = lookup_slot(table, e.domain, set, e.binding);
        let name = e.handle.name().unwrap_or_else(|| panic!("binds 断言: 死句柄 set={set} binding={}", e.binding));
        match e.domain {
            BindDomain::Uniform => {
                let dyn_off = if e.dynamic {
                    let d = offsets[di];
                    di += 1;
                    d
                } else {
                    0
                };
                let size = e.size.unwrap_or(0);
                bind_ubo(state, slot, Some(name), e.offset + dyn_off, size);
            }
            BindDomain::Texture => bind_texture(state, slot, Some(name)),
            BindDomain::Sampler => bind_sampler(state, slot, Some(name)),
        }
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 比较组登记声明数与切片长度：不符=断言，切片原样不动（绝不消耗半份再报错）。
// 2. 符合→按声明序从头取用，切片游标前移（顺序无特权=声明序）。
pub(crate) fn consume_impl(group: &BindGroup, offsets: &mut &[DynamicOffset]) {
    let need = group._bound.iter().filter(|e: &&BoundEntry| e.dynamic).count();
    if offsets.len() != need {
        panic!("binds 断言: 动态偏移个数 {} ≠ 组声明数 {need}（不消耗任何份额）", offsets.len());
    }
    let (_head, rest) = offsets.split_at(need);
    *offsets = rest;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{BindGroup, BindGroupLayout, PipelineLayout};
    use crate::objects::{DirectHandle, DirectKind, GlHandle};
    use crate::types::{BindGroupLayoutEntry, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension};

    /// 布局条目夹具。
    fn ent(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry { binding, visibility: ShaderStages::VERTEX, count: None, ty }
    }
    /// uniform 缓冲条目。
    fn uni(binding: u32) -> BindGroupLayoutEntry {
        ent(binding, BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None })
    }
    /// 采样器条目。
    fn smp(binding: u32) -> BindGroupLayoutEntry { ent(binding, BindingType::Sampler(SamplerBindingType::Filtering)) }
    /// 纹理条目。
    fn tex(binding: u32) -> BindGroupLayoutEntry {
        ent(binding, BindingType::Texture { sample_type: TextureSampleType::Float { filterable: true }, view_dimension: TextureViewDimension::D2, multisampled: false })
    }
    /// 布局夹具。
    fn bgl(entries: Vec<BindGroupLayoutEntry>) -> BindGroupLayout { BindGroupLayout { _entries: entries } }
    /// 条目夹具（死句柄：consume 不读句柄；drop 早退不触 with_gl）。
    fn be(binding: u32, dynamic: bool) -> BoundEntry {
        BoundEntry { binding, domain: BindDomain::Uniform, handle: DirectHandle::placeholder(DirectKind::Buffer, 1), offset: 0, size: None, dynamic }
    }

    /// alloc：分域紧凑编号、键=打包(set,binding)、未占槽为 None。
    #[test]
    fn alloc_reorder_test() {
        let layout = PipelineLayout { _bgls: vec![bgl(vec![uni(0), smp(1)]), bgl(vec![tex(0)])] };
        let mut t = SlotTable::default();
        alloc_impl(&layout, &mut t);
        assert_eq!(t.ubo[0], Some(pack(0, 0)));
        assert_eq!(t.ubo[1], None);
        assert_eq!(t.sampler[0], Some(pack(0, 1)));
        assert_eq!(t.sampler[1], Some(pack(1, 0)));
        assert_eq!(t.sampler[2], None);
    }

    /// alloc：域满=断言，且中途失败不半写（哨兵保持）。
    #[test]
    fn alloc_full_asserts_no_half_write_test() {
        let mut entries: Vec<BindGroupLayoutEntry> = Vec::new();
        for i in 0..17u32 {
            entries.push(uni(i));
        }
        let layout = PipelineLayout { _bgls: vec![bgl(entries)] };
        let mut t = SlotTable::default();
        t.ubo[3] = Some(0xDEAD);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| alloc_impl(&layout, &mut t)));
        assert!(r.is_err(), "槽满应断言");
        assert_eq!(t.ubo[3], Some(0xDEAD), "中途断言不半写");
    }

    /// consume：恰好相等=全消耗；个数不符=断言且不消耗。
    #[test]
    fn consume_exact_and_mismatch_test() {
        let g = BindGroup { _bound: vec![be(0, false), be(1, true), be(2, false), be(3, true)] };
        let mut offs: &[DynamicOffset] = &[10, 20];
        consume_impl(&g, &mut offs);
        assert!(offs.is_empty(), "恰好相等=全消耗");
        let g2 = BindGroup { _bound: vec![be(0, true)] };
        let mut offs2: &[DynamicOffset] = &[1, 2];
        let before = offs2.len();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| consume_impl(&g2, &mut offs2)));
        assert!(r.is_err(), "不符应断言");
        assert_eq!(offs2.len(), before, "不符不消耗任何份额");
    }
}

