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

/// 纹理单元分配上限（GLES3 片元着色器 MAX_TEXTURE_IMAGE_UNITS 保底 16）。
const TEX_UNITS: usize = 16;
/// 采样器单元分配上限（同保底 16；与纹理各自独立计数，同一单元号可各持其一）。
const SMP_UNITS: usize = 16;

/// 域→槽表段：三族独立编号（紧凑，索引即 GL 绑定点/单元号，不叠加 set 基址）。
fn domain_array(t: &SlotTable, d: BindDomain) -> &[Option<u32>; crate::state::BIND_SLOTS] {
    match d {
        BindDomain::Uniform => &t.ubo,
        BindDomain::Texture => &t.tex,
        BindDomain::Sampler => &t.smp,
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

// 设计预期（伪码·示意·非唯一实现）：设计：sampler-unit-assign 切片 2。
// 线性扫描 domain_array(t, Texture) 求 pack(set,binding) 命中下标 i → bind_base(set, i)；
// 未命中 → None。**不得复用 lookup_slot**：其未命中即断言，本处合同要求可容错。
/// 查纹理域绑定占用单元（非断言版；契约见 `binds::unit_of` 壳注释）。 @internal
pub(crate) fn unit_of(t: &SlotTable, set: u32, binding: u32) -> Option<u32> {
    let key = pack(set, binding);
    for (i, v) in domain_array(t, BindDomain::Texture).iter().enumerate() {
        if *v == Some(key) {
            return Some(i as u32);
        }
    }
    None
}

/// 查 UBO 域绑定占用绑定点（非断言版；契约见 `binds::block_binding` 壳注释）。 @internal
pub(crate) fn block_binding(t: &SlotTable, set: u32, binding: u32) -> Option<u32> {
    let key = pack(set, binding);
    for (i, v) in t.ubo.iter().enumerate() {
        if *v == Some(key) {
            return Some(i as u32);
        }
    }
    None
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 遍历布局成员（set=序位）逐条目按域归类：Uniform→ubo 段；Texture/Sampler→单元段。
// 2. 逐域紧凑编号（0..）：编号即 GL 绑定点/单元号；三族独立计数、各有上限，越限=断言（报满的域），中途失败不半写。
// 3. 全部成功才整体提交（设备级槽表值=占槽 (set,binding) 打包键）。
pub(crate) fn alloc_impl(layout: &PipelineLayout, table: &mut SlotTable) {
    let mut ubo = [None; crate::state::BIND_SLOTS];
    let mut tex = [None; crate::state::BIND_SLOTS];
    let mut smp = [None; crate::state::BIND_SLOTS];
    let mut nu = 0usize;
    let mut nt = 0usize;
    let mut ns = 0usize;
    for (set, bgl) in layout._bgls.iter().enumerate() {
        for e in bgl._entries.iter() {
            let key = pack(set as u32, e.binding);
            match e.ty {
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    ..
                } => {
                    if nu >= crate::state::BIND_SLOTS {
                        panic!(
                            "binds 断言: UBO 槽满（{}）set={set} binding={}",
                            crate::state::BIND_SLOTS,
                            e.binding
                        );
                    }
                    ubo[nu] = Some(key);
                    nu += 1;
                }
                BindingType::Sampler(_) => {
                    if ns >= SMP_UNITS {
                        panic!(
                            "binds 断言: 采样器单元槽满（{SMP_UNITS}）set={set} binding={}",
                            e.binding
                        );
                    }
                    smp[ns] = Some(key);
                    ns += 1;
                }
                _ => {
                    if nt >= TEX_UNITS {
                        panic!(
                            "binds 断言: 纹理单元槽满（{TEX_UNITS}）set={set} binding={}",
                            e.binding
                        );
                    }
                    tex[nt] = Some(key);
                    nt += 1;
                }
            }
        }
    }
    *table = SlotTable {
        ubo,
        tex,
        smp,
        alias: Vec::new(),
    };
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 预检：全部句柄活态、槽齐备、动态偏移个数=声明数——任一违反=断言，未发任何 bind。
// 2. 逐条只改镜像期望（bind_* 差异才实际下发）；动态偏移按声明序消耗、叠加静态偏移。
// 3. 死句柄/槽缺=断言且绑定保持调用前原状（预检在动作前）。
pub(crate) fn apply_impl(
    set: u32,
    group: &BindGroup,
    offsets: &[DynamicOffset],
    table: &SlotTable,
    state: &mut StateMirror,
) {
    let need = group
        ._bound
        .iter()
        .filter(|e: &&BoundEntry| e.dynamic)
        .count();
    if offsets.len() != need {
        panic!(
            "binds 断言: 动态偏移个数 {} ≠ 组声明数 {need}",
            offsets.len()
        );
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

        let name = e
            .handle
            .name()
            .unwrap_or_else(|| panic!("binds 断言: 死句柄 set={set} binding={}", e.binding));
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
            BindDomain::Texture => {
                bind_texture(state, slot, e.target, Some(name));
            }
            BindDomain::Sampler => {
                // 组合采样器：采样器对象绑到配对纹理的单元（无配对则占自身槽）。
                let unit = match table
                    .alias
                    .iter()
                    .find(|(_, sk)| *sk == (set, e.binding))
                {
                    Some((tk, _)) => lookup_slot(table, BindDomain::Texture, tk.0, tk.1),
                    None => slot,
                };
                bind_sampler(state, unit, Some(name));
            }
        }
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 比较组登记声明数与切片长度：不符=断言，切片原样不动（绝不消耗半份再报错）。
// 2. 符合→按声明序从头取用，切片游标前移（顺序无特权=声明序）。
pub(crate) fn consume_impl(group: &BindGroup, offsets: &mut &[DynamicOffset]) {
    let need = group
        ._bound
        .iter()
        .filter(|e: &&BoundEntry| e.dynamic)
        .count();
    if offsets.len() != need {
        panic!(
            "binds 断言: 动态偏移个数 {} ≠ 组声明数 {need}（不消耗任何份额）",
            offsets.len()
        );
    }
    let (_head, rest) = offsets.split_at(need);
    *offsets = rest;
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{BindGroup, BindGroupLayout, PipelineLayout};
    use crate::objects::{DirectHandle, DirectKind, GlHandle};
    use crate::types::{
        BindGroupLayoutEntry, SamplerBindingType, ShaderStages, TextureSampleType,
        TextureViewDimension,
    };

    /// 布局条目夹具。
    fn ent(binding: u32, ty: BindingType) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX,
            count: None,
            ty,
        }
    }
    /// uniform 缓冲条目。
    fn uni(binding: u32) -> BindGroupLayoutEntry {
        ent(
            binding,
            BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        )
    }
    /// 采样器条目。
    fn smp(binding: u32) -> BindGroupLayoutEntry {
        ent(binding, BindingType::Sampler(SamplerBindingType::Filtering))
    }
    /// 纹理条目。
    fn tex(binding: u32) -> BindGroupLayoutEntry {
        ent(
            binding,
            BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
        )
    }
    /// 布局夹具。
    fn bgl(entries: Vec<BindGroupLayoutEntry>) -> BindGroupLayout {
        BindGroupLayout { _entries: entries }
    }
    /// 条目夹具（死句柄：consume 不读句柄；drop 早退不触 with_gl）。
    fn be(binding: u32, dynamic: bool) -> BoundEntry {
        BoundEntry {
            binding,
            domain: BindDomain::Uniform,
            handle: DirectHandle::placeholder(DirectKind::Buffer, 1),
            offset: 0,
            size: None,
            dynamic,
            target: 0,
        }
    }

    /// alloc：分域紧凑编号、键=打包(set,binding)、未占槽为 None。
    #[test]
    fn alloc_reorder_test() {
        let layout = PipelineLayout {
            _bgls: vec![bgl(vec![uni(0), smp(1)]), bgl(vec![tex(0)])],
        };
        let mut t = SlotTable::default();
        alloc_impl(&layout, &mut t);
        assert_eq!(t.ubo[0], Some(pack(0, 0)));
        assert_eq!(t.ubo[1], None);
        assert_eq!(t.tex[0], Some(pack(1, 0)), "纹理只数纹理条目");
        assert_eq!(t.tex[1], None);
        assert_eq!(t.smp[0], Some(pack(0, 1)), "采样器独立计数");
        assert_eq!(t.smp[1], None);
    }

    /// alloc：域满=断言，且中途失败不半写（哨兵保持）。
    #[test]
    fn alloc_full_asserts_no_half_write_test() {
        let mut entries: Vec<BindGroupLayoutEntry> = Vec::new();
        for i in 0..25u32 {
            entries.push(uni(i));
        }
        let layout = PipelineLayout {
            _bgls: vec![bgl(entries)],
        };
        let mut t = SlotTable::default();
        t.ubo[3] = Some(0xDEAD);
        let r =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| alloc_impl(&layout, &mut t)));
        assert!(r.is_err(), "槽满应断言");
        assert_eq!(t.ubo[3], Some(0xDEAD), "中途断言不半写");
    }

    /// consume：恰好相等=全消耗；个数不符=断言且不消耗。
    #[test]
    fn consume_exact_and_mismatch_test() {
        let g = BindGroup {
            _bound: vec![be(0, false), be(1, true), be(2, false), be(3, true)],
        };
        let mut offs: &[DynamicOffset] = &[10, 20];
        consume_impl(&g, &mut offs);
        assert!(offs.is_empty(), "恰好相等=全消耗");
        let g2 = BindGroup {
            _bound: vec![be(0, true)],
        };
        let mut offs2: &[DynamicOffset] = &[1, 2];
        let before = offs2.len();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            consume_impl(&g2, &mut offs2)
        }));
        assert!(r.is_err(), "不符应断言");
        assert_eq!(offs2.len(), before, "不符不消耗任何份额");
    }

    /// unit_of：与 apply_impl 同一算式（set*4+槽号）；不同次序各得自己单元；未命中=None 不断言。
    #[test]
    fn unit_of_matches_bind_unit_test() {
        let layout = PipelineLayout {
            _bgls: vec![bgl(vec![uni(0), tex(5)]), bgl(vec![tex(7)])],
        };
        let mut t = SlotTable::default();
        alloc_impl(&layout, &mut t);
        // 单元号=纹理域内紧凑序号（不再叠加 set 基址，跨 set 也不会跳号）。
        assert_eq!(unit_of(&t, 0, 5), Some(0), "set0 纹理占纹理段槽0 → 单元0");
        assert_eq!(unit_of(&t, 1, 7), Some(1), "set1 纹理接续 → 单元1（无 set 基址跳号）");
        assert_eq!(unit_of(&t, 9, 9), None, "未命中返回 None（不断言）");
        assert_eq!(unit_of(&t, 0, 0), None, "UBO 不在纹理域");
    }

    /// 配对采样器不占纹理单元：8 组 [tex,smp] 仍只用 0..7 号纹理单元。
    #[test]
    fn sampler_does_not_consume_texture_unit_test() {
        let mut entries: Vec<BindGroupLayoutEntry> = Vec::new();
        for i in 0..8u32 {
            entries.push(tex(i * 2));
            entries.push(smp(i * 2 + 1));
        }
        let layout = PipelineLayout {
            _bgls: vec![bgl(entries)],
        };
        let mut t = SlotTable::default();
        alloc_impl(&layout, &mut t);
        assert_eq!(t.tex[7], Some(pack(0, 14)), "第 8 个纹理落在槽 7");
        assert_eq!(t.tex[8], None, "采样器不占纹理槽");
        assert_eq!(unit_of(&t, 0, 14), Some(7), "纹理单元只数纹理条目");
        assert_eq!(t.smp[7], Some(pack(0, 15)), "采样器自成一段 0..7");
        assert_eq!(t.smp[8], None);
    }
}
