// 设计：pi-wgpu-rebuild
//! state 芯：驱动状态镜像——本批落死名刷失驻留面（objects 删除路径登记、B5 flush 消费）；
//! 镜像分组比较与差异下发的桩仍由后续批次按伪码落地。
use std::sync::Mutex;

use once_cell::sync::Lazy;
use pi_hash::XHashSet;

use crate::objects::GlHandle;
use crate::state::StateMirror;

// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 逐组比较镜像字段与驱动当前值（全字段扫描；不依赖脏位——漏置位即谎报）。
// 2. 仅差异字段经 GL 入口表下发。
// 3. 下发成功才把该字段写入镜像；报错字段保持旧值并入错误队列（下轮自然重试，绝不「调用即改镜像」）。
pub(crate) fn flush_impl(_m: &mut StateMirror) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 把引用该句柄名的全部镜像字段置「未知」（None/失效标记）——不发任何驱动调用。
// 2. 效果=下一次 flush 对该组必然强制下发一次，等价重同步；幂等。
pub(crate) fn invalidate_impl(_m: &mut StateMirror, _d: GlHandle) { panic!("TODO-DECL") }

/// —— 死名刷失驻留：「镜像不假设驱动仍持有已删对象」的机制层（objects 芯删除路径登记）——
/// 已删对象名登记表（域编号×名字防跨域同名串扰；域编号约定见 objects 芯 kind_num_* 函数注释）。
static REMOVED_NAMES: Lazy<Mutex<XHashSet<(u8, u32)>>> = Lazy::new(|| Mutex::new(XHashSet::default()));

/// 登记一个被物理删除的对象名（objects 芯 mark_dead/release 删除后调用）。
pub(crate) fn note_removed(domain: u8, name: u32) {
    let mut set = REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner());
    set.insert((domain, name));
}

/// 查询某域对象名是否已被删除（flush 下发前置判：命中=该镜像引用按未知处理，强制重下发）。
pub(crate) fn is_removed(domain: u8, name: u32) -> bool {
    let set = REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner());
    set.contains(&(domain, name))
}

/// 重下发成功后销记（GL 名可被驱动回收再用，滞留登记会误判新对象为死）。
pub(crate) fn forget_removed(domain: u8, name: u32) {
    let mut set = REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner());
    set.remove(&(domain, name));
}

/// 整面清空（invalidate_generations 路径：旧世代名集在新附着下无意义）。
pub(crate) fn clear_removed() {
    let mut set = REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner());
    set.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 死名登记/查询/销记/清空四路。
    #[test]
    fn removed_set_lifecycle_test() {
        note_removed(7, 1234);
        assert!(is_removed(7, 1234));
        assert!(!is_removed(6, 1234), "跨域不串扰");
        forget_removed(7, 1234);
        assert!(!is_removed(7, 1234));
        note_removed(0, 9);
        clear_removed();
        assert!(!is_removed(0, 9));
    }
}
