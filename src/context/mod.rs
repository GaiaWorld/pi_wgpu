// 设计：pi-wgpu-rebuild
//! context · 上下文附着：GL 上下文的获取、让出与失效代号。隐藏决定=线程合同（单线程为默认合同）。
//! 契约权威文本住本面注释；本面全部 @internal（仅库内可达）。

/// 附着体=库内私有句柄包装：EGL 显示/上下文/表面三元组与 GL 入口的聚合（Web 形态退化为画布上下文变体）。
/// 不变量：generation 全局单调增；任何驱动调用只允许在重入锁内发生。 @internal
#[derive(Debug)]
pub struct Attachment {
    /// 附着体代表占位（骨架期 ()）：真实附着资源驻留 context 芯全局槽，本结构=代号读数载体；
    /// 官方挂接批如引入跨线程引用链在此形制换位（签名不变表示替换=合法）。
    pub(crate) _raw: (),
    /// 读取时的附着代号（失配判定基准；单调增）。
    pub(crate) generation: u32,
}
/// 附着槽占位声明：实现批改为共享引用链（EGL 实例/上下文/表面 + GL 入口 + 重入锁 + 代号）；
/// Web 形态按 cfg 换画布内变体。骨架期用 Option<Attachment> 维持引用网。 @internal
pub type AttachmentSlot = Option<Attachment>;

/// 重入取锁并附着当前线程。single_thread feature 下锁与搬运整体退化为零开销恒通过。
/// 搬运前提（Windows 形态）：首次建立附着体的线程须存活至附着存续期——隐窗 DC 等
/// 资源随其线程生命周期，该线程终止后再跨线程重绑由地基断言拒绝（无半附着副作用）；
/// 安卓 EGL 与 Web 形态天然满足此前提（无线程绑定句柄）。 @internal
pub(crate) fn attach() -> AttachmentSlot {
    crate::context_inner::attach_impl()
}
/// 让出当前线程的上下文绑定（跨线程搬运底座；unmake_current 的实现端）。 @internal
pub(crate) fn let_go() {
    crate::context_inner::let_go_impl()
}
/// 上下文丢失登记：附着代号+1、返回新代号。本步只记账号——旧句柄与镜像靠代号失配自然退出（活态判）；公开重建路径=对同一适配器重走 request_device 得新三件套。 @internal
pub(crate) fn gen_bump() -> u32 {
    crate::context_inner::gen_bump_impl()
}
