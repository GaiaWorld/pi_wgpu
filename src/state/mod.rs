// 设计：pi-wgpu-rebuild
//! state · 驱动状态镜像：只在「与驱动已知态不同」时才下发调用（避免重复 GL 调用的开销面）。
//! 隐藏决定=镜像字段集与比较粒度。契约权威文本住本面注释；本面全部 @internal。
use crate::objects::GlHandle;
/// 镜像快照：驱动侧可缓存副作用的字段逐组记录（字段集由实现批按驱动分组展开）。
/// 不变量：字段值=驱动已成功生效的态，绝不在调用发出时先改。比较策略=全字段逐组比对
/// （被否方案：脏位图——置位散落在各写入点，易漏置位导致镜像虚新）。 @internal
#[derive(Debug, Default)] pub(crate) struct StateMirror { pub(crate) program: Option<GlHandle>, pub(crate) generation: u32 }
impl StateMirror {
    /// 逐组比较→差异下发→成功后同步镜像。失败面：驱动报错的字段不改镜像、错误入队列，
    /// 下次比对自然重下发（自洽恢复，无谎报态）。 @internal
    pub(crate) fn flush_if_changed(&mut self) { crate::state_inner::flush_impl(self) }
    /// 对象死亡通知：置空引用该句柄名的镜像字段——下次下发按「镜像未知」强制重设，
    /// 不假设驱动仍持有已删对象。 @internal
    pub(crate) fn drop_invalidate(&mut self, dead: GlHandle) { crate::state_inner::invalidate_impl(self, dead) }
}
