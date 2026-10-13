// 设计：pi-wgpu-rebuild
//! context 芯 · 全库唯一跨线程 GL 附着点。
//! 重入锁形制=持锁线程登记+TLS 层数计数（std 原语，无额外依赖）：attach 计层；
//! 异线程等待限时（超时=持锁方未让出 → trace 持锁线程后断言）；同线程重入=零开销快路。
//! single_thread feature 下锁步与搬运步编译期整体退化（恒通过、零开销）。
//! 全库不变量：一切驱动调用只发生在附着持有期内；本文件 with_gl 为后续各芯的驱动调用入口闸。
//! Web 形态无 EGL：attach 只登记，GL 入口由呈现配置链注入（web_bind_glow，呈现批接线）。
//! Windows 形态事实约束（实现批诊断坐实）：隐窗 DC 同创建线程生命周期——创建线程存活期间，
//! 跨线程让出/重绑全通；该线程终止后重绑由地基断言拒绝（合同=附着创建线程须存活至进程末）。

use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use pi_glow as glow;

use crate::context::{Attachment, AttachmentSlot};

#[cfg(not(target_arch = "wasm32"))]
use pi_egl::{Context as EglContext, Instance as EglInstance, PowerPreference as EglPower};

/// 桌面/安卓附着体：EGL/WGL 实例（内含隐窗与 glow 入口表，构造于首次 attach）+ 上下文句柄对。
#[cfg(not(target_arch = "wasm32"))]
struct Attached {
    /// 地基实例：make_current/get_glow/后续 swap_buffers 的挂载点，进程全程驻留。
    instance: EglInstance,
    /// 地基上下文句柄：仅与实例成对使用（跨线程换绑时作 make_current 入参）。
    context: EglContext,
}

/// Web 形态附着体：无 EGL 部件，仅画布 GL 入口占位（由呈现配置链注入后才可用）。
#[cfg(target_arch = "wasm32")]
struct Attached {
    /// 画布 WebGL2 入口（None=尚未配置呈现；驱动调用=断言提示先走配置）。
    glow: Option<glow::Context>,
}

/// 不可变借用旁路壳：承载「单持有者协议」下的附着槽（读写纪律见 SLOT 注释）。
struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}

/// 附着槽：仅当前持锁线程可读写（重入门锁控=全库不变量的机制层）。
struct SlotField {
    /// 附着体本体（None=未初始化；失败半程不留=无半附着残留）；仅持锁线程可读写。
    attached: Option<Attached>,
    /// 当前 GL 绑定归属线程（None=未绑）：换绑/让出/防重复绑定的判定基准。
    bound: Option<ThreadId>,
}

/// 附着槽本体：读写只允许发生在当前持锁（claim 获胜或同线程重入）的持有期内。
static SLOT: SyncCell<SlotField> = SyncCell(UnsafeCell::new(SlotField { attached: None, bound: None }));

/// 重入锁登记册：当前持有驱动锁的线程（None=无人持有）。
struct Owner {
    /// 持锁线程登记（None=无人持有）；跨线程 attach 的争用/超时判定对象。
    thread: Option<ThreadId>,
}
/// 持锁位：跨线程争用 attach 的排队点（锁本身只护登记，不跨驱动调用持有）。
static OWNER: Mutex<Owner> = Mutex::new(Owner { thread: None });
/// 让位广播：末层释放持锁位时唤醒等待者（免纯轮询）。
static READY: Condvar = Condvar::new();
/// 附着代号：0=未初始化哨兵；首附着 compare_exchange 立基线 1，此后仅 gen_bump 单调增；锁外可读（失配判定用）。
static GENERATION: AtomicU32 = AtomicU32::new(0);
/// 本线程持有层数：>0 即同线程重入快路标志；attach/让出计层，归零才真正交还持锁位。
thread_local! { static HELD: Cell<u32> = const { Cell::new(0) }; }

/// 异线程争锁的限时等待上限：超时=持锁方违约未让出 → trace 持锁线程后断言（锁合同）。
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// 等待唤醒的兜底轮询步长（防 notify 与登记竞态空转）。
const POLL_STEP: Duration = Duration::from_millis(50);

/// 当前附着代号（锁外可读的失配判定基准；0=未初始化，首附着/首 bump 立基线 1）。
pub(crate) fn generation_now() -> u32 {
    GENERATION.load(Ordering::SeqCst)
}

/// 代号基线：首建写 1（0=未初始化哨兵）；此后只增不减，由 gen_bump 推进。
fn ensure_generation() {
    let _ = GENERATION.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst);
}

/// 桌面/安卓附着：首次=建实例（内部隐窗，无表面亦可附着）→建上下文→make_current 绑当前线程
/// （GL 入口表在首次 make_current 内初始化）；非首次且换了线程=重取 make_current（旧线程绑定自失效）。
/// 任一步失败即 panic，槽未写入半步状态（无半附着残留）。
#[cfg(not(target_arch = "wasm32"))]
fn bind_current_thread() {
    let me = thread::current().id();
    // SAFETY: 与并发读写的互斥由重入门锁控——只有刚赢得持锁位的线程走到这里。
    let slot = unsafe { &mut *SLOT.0.get() };
    if slot.attached.is_none() {
        let mut instance = EglInstance::new(EglPower::HighPerformance, true)
            .unwrap_or_else(|e| panic!("context attach: 实例创建失败 err={e:?}"));
        let context = instance
            .create_context()
            .unwrap_or_else(|e| panic!("context attach: 上下文创建失败 err={e:?}"));
        instance.make_current(None, Some(&context));
        slot.attached = Some(Attached { instance, context });
        slot.bound = Some(me);
        ensure_generation();
    } else if slot.bound != Some(me) {
        let attached = slot.attached.as_mut().expect("context attach: 附着体必先存在");
        let Attached { instance, context } = attached;
        instance.make_current(None, Some(context));
        slot.bound = Some(me);
    }
}

/// Web：零 EGL 步——只登记代号与隐式绑定的占位附着体；GL 入口后到（呈现配置注入）。
#[cfg(target_arch = "wasm32")]
fn bind_current_thread() {
    let slot = unsafe { &mut *SLOT.0.get() };
    if slot.attached.is_none() {
        slot.attached = Some(Attached { glow: None });
        ensure_generation();
    }
    slot.bound = Some(thread::current().id());
}

/// 取得一层持有（非重入路径限时争锁）：获胜后同线程重入=TLS 层数快路，零成本。
#[cfg(not(feature = "single_thread"))]
fn claim_and_attach() {
    if HELD.with(|c| c.get()) > 0 {
        HELD.with(|c| c.set(c.get() + 1));
        return;
    }
    let me = thread::current().id();
    let mut guard = OWNER.lock().unwrap_or_else(|e| e.into_inner());
    let start = Instant::now();
    loop {
        match guard.thread {
            None => {
                guard.thread = Some(me);
                drop(guard);
                bind_current_thread();
                HELD.with(|c| c.set(1));
                return;
            }
            Some(other) => {
                if start.elapsed() >= LOCK_WAIT {
                    drop(guard);
                    log::error!("context attach 锁超时：持锁线程={:?} 已等待={:?}", other, start.elapsed());
                    panic!("context attach 断言：他线程占锁未让出，holder={other:?}");
                }
                let (next, _wait) = READY
                    .wait_timeout(guard, POLL_STEP.min(LOCK_WAIT - start.elapsed()))
                    .unwrap_or_else(|e| e.into_inner());
                guard = next;
            }
        }
    }
}

/// 交还一层持有：末层（层数归零）才真正交还持锁位并唤醒等待者。
#[cfg(not(feature = "single_thread"))]
fn release_one_layer() {
    let prev = HELD.with(|c| {
        let v = c.get();
        if v > 0 {
            c.set(v - 1);
        }
        v
    });
    if prev == 1 {
        let mut guard = OWNER.lock().unwrap_or_else(|e| e.into_inner());
        guard.thread = None;
        drop(guard);
        READY.notify_all();
    }
}

/// single_thread 合同：锁步与搬运步编译期退化=附着即建、无争用。
#[cfg(feature = "single_thread")]
fn claim_and_attach() {
    bind_current_thread();
}

/// single_thread 合同：交还恒空操作（零开销）。
#[cfg(feature = "single_thread")]
fn release_one_layer() {}

/// 驱动调用入口闸（库内公开）：持有期内借出 GL 入口，同线程可重入嵌套。
pub(crate) fn with_gl<R>(f: impl FnOnce(&glow::Context) -> R) -> R {
    claim_and_attach();
    let out = f(current_glow());
    release_one_layer();
    out
}

/// 借出 GL 入口（仅持有期内调用——全库唯一借出点，两形态字段面 cfg 分叉）。
fn current_glow() -> &'static glow::Context {
    // SAFETY: 调用点均在 claim_and_attach 之后的持有期内，单持有者协议保证无并发访问。
    let slot = unsafe { &*SLOT.0.get() };
    #[cfg(not(target_arch = "wasm32"))]
    {
        slot.attached
            .as_ref()
            .expect("context: 驱动调用前必须先 attach 成功")
            .instance
            .get_glow()
    }
    #[cfg(target_arch = "wasm32")]
    {
        slot.attached
            .as_ref()
            .and_then(|a| a.glow.as_ref())
            .expect("context: Web 形态先配置呈现注入 GL 入口后才可驱动调用")
    }
}

/// Web 形态 GL 入口注入点（surface 配置链调用；桌面/安卓形态编译期不存在）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_bind_glow(gl: glow::Context) {
    let slot = unsafe { &mut *SLOT.0.get() };
    if let Some(attached) = slot.attached.as_mut() {
        attached.glow = Some(gl);
    } else {
        slot.attached = Some(Attached { glow: Some(gl) });
        ensure_generation();
    }
}

/// attach 芯实现。设计预期（编号步）：
/// 1. 同线程已在持有期→计层直接返回现附着代号，不重复建上下文。
/// 2. 桌面/安卓：实例（隐窗，无需表面）→上下文→make_current 绑当前线程；Web：登记制（隐式，无 EGL 步）。
/// 3. 代号首建=1、此后只随 gen_bump 单调增；返回的附着体=代号读数。
pub(crate) fn attach_impl() -> AttachmentSlot {
    claim_and_attach();
    Some(Attachment { _raw: (), generation: generation_now() })
}

/// let_go 芯实现。让出=解绑当前线程与上下文的绑定，不销毁上下文本体。
/// 非 attach 线程调用=幂等空操作；末层让出时置空 make_current（Web 形态无此步恒空操作；
/// single_thread feature 下锁步与搬运步编译期退化=整体空操作）。
pub(crate) fn let_go_impl() {
    if cfg!(feature = "single_thread") {
        return;
    }
    let held = HELD.with(|c| c.get());
    if held == 0 {
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    if held == 1 {
        let slot = unsafe { &mut *SLOT.0.get() };
        if slot.bound == Some(thread::current().id()) {
            if let Some(attached) = slot.attached.as_mut() {
                attached.instance.make_current(None, None);
            }
            slot.bound = None;
        }
    }
    release_one_layer();
}

/// gen_bump 芯实现。只动代号：+1 写回（无附着时先立基线再增），返回新代号；
/// 不动对象层账目——旧句柄/镜像随代号失配集体退场，公开重建路径=request_device 重入。
pub(crate) fn gen_bump_impl() -> u32 {
    ensure_generation();
    GENERATION.fetch_add(1, Ordering::SeqCst) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_bump_monotonic_test() {
        let a = gen_bump_impl();
        let b = gen_bump_impl();
        assert_eq!(b, a + 1);
        assert!(generation_now() >= b);
    }

    /// GL 状态机全链路（attach 往返+代号推进+探活投影）收拢为单用例：
    /// WGL 隐窗 DC 与附着创建线程同生命周期（诊断坐实），跨测试线程会撞死线程资源——
    /// 实现层合同=创建线程存活期间跨绑全通；跨模块合验在本线程内完成（B1+B2 联合验收）。
    #[test]
    #[ignore = "需真实 GL 环境（桌面/安卓/Web 三形态手工验证）：cargo test -- --ignored"]
    fn gl_state_machine_test() {
        let first = attach_impl().expect("首次 attach 应成功");
        assert!(first.generation >= 1);
        let g = gen_bump_impl();
        let second = attach_impl().expect("代号推进后 attach 应成功");
        assert!(second.generation >= g, "代号只增不减");
        let caps = crate::caps_inner::probe_impl();
        assert!(caps._gl_version >= (3, 0), "GL 版本读数异常: {:?}", caps._gl_version);
        let f = crate::caps_inner::fill_impl(&caps).0;
        assert_ne!(f & (1 << 1), 0, "DEPTH32FLOAT_STENCIL8 版本确证位应置");
        with_gl(|_gl| {});

        // —— B4 objects 合验（同线程；成对合同与归零即删/重建）——
        let mut cache = crate::objects::HandleCache::default();
        let key = crate::objects::ReuseKey::Program(0xAAAA_BBB0);
        let a = crate::objects::ensure_reuse(key, &mut cache);
        let b = crate::objects::ensure_reuse(key, &mut cache);
        assert_eq!(a.handle, b.handle, "同键复用同一物理名（计数活跃行）");
        assert_eq!(b.refcount.get(), 2, "快照计数=在途持有数");
        crate::objects::release_reuse(&a);
        crate::objects::release_reuse(&b);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::objects::release_reuse(&a))).is_err(),
            "零计数再释放=断言（成对合同）"
        );
        let c3 = crate::objects::ensure_reuse(key, &mut cache);
        // 名值由驱动回收决定（GL 允许即刻重用刚删名）；芯保证=行模板已覆写+计数=1 新持有,
        assert_eq!(c3.refcount.get(), 1, "重建后持者计数快照=1");
        drop(c3);
        let d1 = crate::objects_inner::create_direct(crate::objects::DirectKind::Buffer);
        let d2 = crate::objects_inner::create_direct(crate::objects::DirectKind::Sampler);
        assert!(d1.handle.is_some() && d1.generation >= 1);
        assert!(d2.handle.is_some());
        drop(d1);
        drop(d2);
        let mut mirror = state_mirror_probe();
        let re = crate::objects::ensure_reuse(crate::objects::ReuseKey::Fbo(1), &mut cache);
        crate::objects::invalidate_generations(&mut cache, &mut mirror);
        crate::objects::invalidate_generations(&mut cache, &mut mirror);
        assert!(re.refcount.get() >= 1);
        assert!(mirror.program.is_none(), "invalidate 后镜像引用字段作废");
        let _ = re;
        let_go_impl();
    }

    /// 组一个最小镜像供 invalidate 合验（字段面=当前壳形制）。
    fn state_mirror_probe() -> crate::state::StateMirror {
        crate::state::StateMirror { program: Some(crate::objects::GlHandle(1)), generation: 1 }
    }
}
