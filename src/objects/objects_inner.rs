// 设计：pi-wgpu-rebuild
//! objects 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::objects::{DirectHandle, HandleCache, Reusable, ReuseKey};
use crate::state::StateMirror;
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 查表命中同键项 → refcount+1，返回该项克隆。
// 2. 未命中 → 按 ReuseKey 域分派对应 glGen* 建对象并把配置即时写完（程序=编译链接/VAO=属性批/FBO=附件挂接/RBO=存储）。
// 3. 以 refcount=1 登记入表并返回；建对象中途报错=断言（报键域与哈希），表保持原状无半项。
pub(crate) fn ensure_reuse_impl(_k: ReuseKey, _c: &mut HandleCache) -> Reusable { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. refcount-1（Cell 自减；已为 0 则断言——引用方计数必须与获取严格成对）。
// 2. 减到 0 → 按域 glDelete* 物理删除 → 通知镜像作废引用该名的字段。
pub(crate) fn release_reuse_impl(_r: &Reusable) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. handle 置 None 先行——防重入：Drop 与手动 mark_dead 竞到一起时只删一次。
// 2. 即调该资源域 glDelete*（1:1 资源无延迟队列——不存在「稍后统一回收」路径）。
// 3. state.drop_invalidate 通知镜像作废引用该名的字段；此后句柄为死态，消费方使用=断言。
pub(crate) fn mark_dead_impl(_h: &mut DirectHandle) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 遍历给定三件套：缓存全项句柄置空、镜像引用字段作废——只改记账、不调驱动（上下文已丢，删除无意义）。
// 2. 幂等面：代号已失配则直接返回，重复调用效果同单次。
pub(crate) fn invalidate_impl(_c: &mut HandleCache, _s: &mut StateMirror) { panic!("TODO-DECL") }
