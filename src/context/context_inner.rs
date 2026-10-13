// 设计：pi-wgpu-rebuild
//! context 芯 · 全库唯一跨线程 GL 附着点。
//! 重入锁形制=持锁线程登记+TLS 层数计数（std 原语，无额外依赖）：attach 计层；
//! 异线程等待限时（超时=持锁方未让出 → trace 持锁线程后断言）；同线程重入=零开销快路。
//! single_thread feature 下锁步与搬运步编译期整体退化（恒通过、零开销）。
//! 全库不变量：一切驱动调用只发生在附着持有期内；本文件 with_gl 为后续各芯的驱动调用入口闸。
//! Web 形态无 EGL：attach 只登记，GL 入口由呈现配置链注入（web_bind_glow，链10 接线）。

use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use crate::context::{Attachment, AttachmentSlot};

use pi_glow as glow;

#[cfg(not(target_arch = "wasm32"))]
use pi_egl::{Context as EglContext, Instance as EglInstance, PowerPreference as EglPower};

#[cfg(not(target_arch = "wasm32"))]
struct Attached {
    instance: EglInstance,
    context: EglContext,
}

#[cfg(target_arch = "wasm32")]
struct Attached {
    glow: Option<glow::Context>,
}

/// 附着槽：仅当前持锁线程可读写（重入门锁控=全库不变量的机制层）。
struct SlotField {
    attached: Option<Attached>,
    bound: Option<ThreadId>,
}

/// 不可变借用旁路壳：仅承载「单持有者协议」下的附着槽（访问纪律见下方 SLOT 注释）。
struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncCell<T> {}
/// 附着槽本体：读写只允许发生在当前持锁（claim 获胜或同线程重入）的持有期内——
/// 这是「一切驱动调用在重入锁内」全库不变量的机制层；生命周期=进程全程（骨架泄漏式驻留，图形上下文随程存续）。
static SLOT: SyncCell<SlotField> = SyncCell(UnsafeCell::new(SlotField { attached: None, bound: None }));

/// 重入锁登记册：当前持有驱动锁的线程（None=无人持有）。
struct Owner {
    thread: Option<ThreadId>,
}
/// 持锁位：跨线程争用 attach 的排队点（锁本身只护登记，不跨驱动调用持有）。
static OWNER: Mutex<Owner> = Mutex::new(Owner { thread: None });
/// 让位广播：最后层释放持锁位时唤醒等待者（免纯轮询）。
static READY: Condvar = Condvar::new();
/// 附着代号：0=未初始化；首附着 compare_exchange 立基线 1，此后仅 gen_bump 单调增；锁外可读（失配判定用）。
static GENERATION: AtomicU32 = AtomicU32::new(0);
/// 本线程持有层数：>0 即同线程重入快路标志；attach/让出计层，归零才真正交还持锁位。
thread_local! { static HELD: Cell<u32> = const { Cell::new(0) }; }

/// 异线程争锁的限时等待上限：超时=持锁方违约未让出 → trace 持锁线程后断言（锁合同）。
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// 等待唤醒的兜底轮询步长（防 notify 与登记竞态空转）。
const POLL_STEP: Duration = Duration::from_millis(50);

pub(crate) fn generation_now() -> u32 {
    GENERATION.load(Ordering::SeqCst)
}

fn ensure_generation() {
    let _ = GENERATION.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst);
}

/// 桌面/安卓附着体：首次=建实例（内部隐窗，无表面亦可附着）→建上下文→make_current 绑当前线程
/// →GL 入口表在 make_current 内初始化；非首次且换了线程=重取 make_current（旧线程绑定自失效）。
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

#[cfg(feature = "single_thread")]
fn claim_and_attach() {
    bind_current_thread();
}

#[cfg(feature = "single_thread")]
fn release_one_layer() {}

/// 驱动调用入口闸（库内公开）：持有期内借出 GL 入口，同线程可重入嵌套。
pub(crate) fn with_gl<R>(f: impl FnOnce(&glow::Context) -> R) -> R {
    claim_and_attach();
    let out = f(current_glow());
    release_one_layer();
    out
}

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
            .expect("context: Web 形态先配置呈现（链10）注入 GL 入口后才可驱动调用")
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

// 1. 同线程已在持有期→计层直接返回现附着，不重复建上下文。
// 2. 桌面/安卓：实例（隐窗）→上下文→make_current；Web：登记制（隐式，无 EGL 步）。
// 3. 代号首建=1、此后只随 gen_bump 单调增。
pub(crate) fn attach_impl() -> AttachmentSlot {
    claim_and_attach();
    Some(Attachment { _raw: (), generation: generation_now() })
}

// 让出=解绑当前线程绑定，不销毁上下文本体。
// 非 attach 线程调用=幂等空操作；末层让出时置空 make_current（Web 形态无此步恒空操作；
// single_thread feature 下锁步与搬运步编译期退化=整体空操作）。
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

// 只动代号：+1 写回（无附着时先立基线 1 再增）；不动对象层账目——
// 旧句柄/镜像随代号失配集体退场，公开重建路径=request_device 重入。
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

    #[test]
    #[ignore = "需真实 GL 环境（桌面/安卓/Web 三形态手工验证）：cargo test -- --ignored"]
    fn attach_roundtrip_test() {
        let first = attach_impl().expect("首次 attach 应成功");
        assert!(first.generation >= 1);
        let g = gen_bump_impl();
        let second = attach_impl().expect("让出/换绑后 attach 应成功");
        assert_eq!(second.generation, g);
        with_gl(|_gl| {});
        let_go_impl();
    }
}
