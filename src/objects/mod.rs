// 设计：pi-wgpu-rebuild
//! objects · GL 对象句柄层：一对一资源与可复用缓存两种生命周期。隐藏决定=哪些对象允许复用（仅四类）。
//! 契约权威文本住本面注释；本面全部 @internal。
use crate::state::StateMirror;

/// GL 对象名（glGen* 原始产物）。只做类型区分，数值本身不含生命周期信息。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub(crate) struct GlHandle(pub u32);
/// 一对一资源句柄（缓冲/纹理/采样器）。不变量：活态⟺generation 与当前附着代号一致；
/// 使用死句柄由消费方断言拦截；死亡即删驱动对象并通知镜像刷失。 @internal
#[derive(Debug)] pub(crate) struct DirectHandle { pub(crate) handle: Option<GlHandle>, pub(crate) generation: u32 }
impl DirectHandle {
    /// 置死并物理删除：先置空句柄名防重入，再即调驱动删除（本库无延迟删除队列、无等待 GC），
    /// 并通知镜像作废引用该名的字段。调用后句柄即死态。 @internal
    pub(crate) fn mark_dead(&mut self) { crate::objects_inner::mark_dead_impl(self) }
}
impl Drop for DirectHandle { fn drop(&mut self) { self.mark_dead() } }
/// 可复用对象的键=配置哈希。四个域：程序/顶点属性布局(VAO)/帧缓冲/渲染缓冲——
/// 仅此四类允许复用，一对一资源不得进此域（复用范围本身即合同）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub(crate) enum ReuseKey { Program(u64), Vao(u64), Fbo(u64), Rbo(u64) }
/// 复用句柄。不变量：refcount>0=物理存在且被至少一方引用；引用归零=物理删除+通知镜像刷失。
/// 计数操作与 acquire/release 严格成对，不得旁路手改。 @internal
#[derive(Debug)] pub(crate) struct Reusable { pub(crate) key: ReuseKey, pub(crate) handle: Option<GlHandle>, pub(crate) refcount: std::cell::Cell<u32> }
/// 复用对象缓存表。 TODO-DECL：表容器由实现批替换为约定哈希方案；骨架期用 std HashMap 维持引用网。 @internal
#[derive(Debug, Default)] pub(crate) struct HandleCache { pub(crate) map: std::collections::HashMap<ReuseKey, Vec<Reusable>> }

/// 按键取对象：命中=计数+1 后返回；未命中=按键域建物理对象并登记。
/// 任一阶段失败：断言（报键域与哈希），表不留半登记项，保持调用前原状。 @internal
pub(crate) fn ensure_reuse(key: ReuseKey, cache: &mut HandleCache) -> Reusable { crate::objects_inner::ensure_reuse_impl(key, cache) }
/// 一方引用退出：计数-1；归零即物理删并刷失镜像。计数已为零再释放=断言（成对合同被破）。 @internal
pub(crate) fn release_reuse(item: &Reusable) { crate::objects_inner::release_reuse_impl(item) }
/// 显式预清指定三件套：遍历复用缓存置空句柄并作废旧镜像字段。正常丢失路径不需要本调用（旧三件套随 Device 整体废弃、消费端代号失配自判）；保留给同设备重建时的显式清理。幂等——重复调用与单次效果相同。 @internal
pub(crate) fn invalidate_generations(cache: &mut HandleCache, state: &mut StateMirror) { crate::objects_inner::invalidate_impl(cache, state) }
