// 设计：pi-wgpu-rebuild
//! objects · GL 对象句柄层：一对一资源与可复用缓存两种生命周期。隐藏决定=哪些对象允许复用（仅四类）。
//! 契约权威文本住本面注释；本面全部 @internal。
use crate::state::StateMirror;
use pi_share::{Share, ShareU32, ShareU64};

/// GL 对象归一化柄（gid 层的存形）。只做类型区分，数值本身不含生命周期信息。
/// 元组字段：桌面=驱动原始名号（u32 加宽零损）；Web=glow slot-key as_ffi 全 64 位
/// （idx+version 无损——截 32 位即删后混叠，禁截）。进出驱动面必须过 objects::gid。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GlHandle(pub u64);
/// 一对一资源域：直建三件套=全部合法域（复用域另列四类合同）。供删除分派与域校验。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DirectKind {
    /// 顶点/索引/统一缓冲对象。
    Buffer,
    /// 纹理对象（直建单附件；MIP 链经 formats 管道）。
    Texture,
    /// 采样器对象。
    Sampler,
}
/// 一对一资源句柄（缓冲/纹理/采样器）。不变量：活态⟺generation 与当前附着代号一致；
/// **共享计数**（Share，对齐 pi_render 的 Share/Arc 强引用）：Clone 共享同一驱动名，末引用 Drop 才删——
/// 绑定组与用户可双持，互不删对方；死亡即删驱动对象并登记死名（state 芯驻留，下次 flush 前作废镜像=强发）。 @internal
#[derive(Debug, Clone)]
pub(crate) struct DirectHandle {
    /// 共享内芯（名字/代号/域）。
    pub(crate) inner: Share<DirectHandleState>,
}
/// 共享句柄可变内芯：末引用析构=按域删驱动对象+死名登记（防重入：先 take 名置 None）。 @internal
#[derive(Debug)]
pub(crate) struct DirectHandleState {
    /// 资源域：删除入口按此分派（buffer/texture/sampler 三路 glDelete*）。
    pub(crate) kind: DirectKind,
    /// 句柄名（原子形：0=死态或已释放——非零即活名；gid 归一化 u64：桌面=驱动名，Web=slot-key ffi）。
    pub(crate) handle: ShareU64,
    /// 构造时附着代号快照；使用死代号句柄由消费方按失配断言拦截。
    pub(crate) generation: u32,
}
impl DirectHandle {
    /// 读当前驱动名（None=死态；原子装载，跨线程观察合法——驱动调用仍受附着门控）。 @internal
    pub(crate) fn name(&self) -> Option<GlHandle> {
        std::num::NonZeroU64::new(self.inner.handle.load(std::sync::atomic::Ordering::SeqCst))
            .map(|v| GlHandle(v.get()))
    }
    /// 读附着代号快照。 @internal
    pub(crate) fn generation(&self) -> u32 {
        self.inner.generation
    }
    /// 占位构造：给定域与代号、句柄名初始 None（不走驱动、不建对象；供无 GL 环境单测）。 @internal
    pub(crate) fn placeholder(kind: DirectKind, generation: u32) -> Self {
        Self {
            inner: Share::new(DirectHandleState {
                kind,
                handle: ShareU64::new(0),
                generation,
            }),
        }
    }
    /// 置死并物理删除：先置空句柄名防重入，再即调驱动删除（本库无延迟删除队列、无等待 GC），
    /// 并把（域,名）登记进 state 芯死名表——镜像对已删名不谎报持有，下次 flush 强制重下发后销记。
    /// 注意（共享计数）：对任一 clone 调本方法=连坐删除（对全部持有者生效），仅适用单持有/整面失效；常规死亡走末引用 Drop。 @internal
    pub(crate) fn mark_dead(&self) {
        crate::objects_inner::mark_dead_impl(self)
    }
}
impl Drop for DirectHandleState {
    fn drop(&mut self) {
        crate::objects_inner::drop_impl(self)
    }
}
/// 可复用对象的键=配置哈希。四个域：程序/顶点属性布局(VAO)/帧缓冲/渲染缓冲——
/// 仅此四类允许复用，一对一资源不得进此域（复用范围本身即合同）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ReuseKey {
    /// 程序域；载荷=着色器反射+管线固定功能的合成配置哈希（B7 产出源）。
    Program(u64),
    /// 顶点属性布局域；载荷=属性批+缓冲布局的合成哈希。
    Vao(u64),
    /// 帧缓冲域；载荷=附件组合哈希。
    Fbo(u64),
    /// 渲染缓冲域；载荷=存储形态哈希。
    Rbo(u64),
}
/// 复用句柄。不变量：计数>0=物理存在且被至少一方引用；引用归零=物理删除+死名登记刷失。
/// 计数操作与 acquire/release 严格成对，不得旁路手改。 @internal
#[derive(Debug)]
pub(crate) struct Reusable {
    /// 复用键（域+配置哈希；release 定位驻留计数表的锚）。
    pub(crate) key: ReuseKey,
    /// 物理名快照（None=行已死待重建；活态真相判定=objects 芯计数表，非本字段）。
    pub(crate) handle: Option<GlHandle>,
    /// 在途计数快照（真相=objects 芯全局计数表；本原子位仅观测/调试承载，不参与 release 运算）。
    pub(crate) refcount: ShareU32,
}
/// 复用对象缓存表（L7 解冻重落定：约定哈希=pi_hash::XHashMap/FxHash64；计数真相与死名刷失驻留 objects/state 芯，行仅持物理名模板）。 @internal
#[derive(Debug, Default)]
pub(crate) struct HandleCache {
    pub(crate) map: pi_hash::XHashMap<ReuseKey, Vec<Reusable>>,
}

/// 按键取对象：命中=计数+1 后返回；未命中=按键域建物理对象并登记。
/// 任一阶段失败：断言（报键域与哈希），表不留半登记项，保持调用前原状。 @internal
pub(crate) fn ensure_reuse(key: ReuseKey, cache: &mut HandleCache) -> Reusable {
    crate::objects_inner::ensure_reuse_impl(key, cache)
}
/// 克隆持行：计数+1（录制命令持有复用对象用；与 release_reuse 成对）。
pub(crate) fn retain_reuse(item: &Reusable) -> Reusable {
    crate::objects_inner::retain_reuse_impl(item)
}

/// 一方引用退出：计数-1；归零即物理删+死名登记（镜像刷失=下次 flush 强制重发，非即时改写镜像）。
/// 计数已为零再释放=断言（成对合同被破）。 @internal
pub(crate) fn release_reuse(item: &Reusable) {
    crate::objects_inner::release_reuse_impl(item)
}
/// 显式预清指定三件套：复用缓存全项名置空、芯计数表清零、镜像整面重置、死名登记表全清
/// （旧世代登记在新附着下无意义）。正常丢失路径不需要本调用（旧三件套随 Device 整体废弃、消费端代号失配自判）；
/// 保留给同设备重建时的显式清理。幂等——重复调用与单次效果相同。 @internal
pub(crate) fn invalidate_generations(cache: &mut HandleCache, state: &mut StateMirror) {
    crate::objects_inner::invalidate_impl(cache, state)
}
