// 设计：pi-wgpu-rebuild
//! state 芯：桩+设计预期编号伪码（示意·不可验证）。
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
