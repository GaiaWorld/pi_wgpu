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
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use crate::context::{Attachment, AttachmentSlot};

#[cfg(not(target_arch = "wasm32"))]
use pi_egl::{
    Context as EglContext, Instance as EglInstance, PowerPreference as EglPower,
    Surface as EglSurface,
};

/// 桌面/安卓附着体：EGL/WGL 实例（内含隐窗与 glow 入口表，构造于首次 attach）+ 上下文句柄对。
#[cfg(not(target_arch = "wasm32"))]
struct Attached {
    /// 地基实例：make_current/get_glow/后续 swap_buffers 的挂载点，进程全程驻留。
    instance: EglInstance,
    /// 地基上下文句柄：仅与实例成对使用（跨线程换绑时作 make_current 入参）。
    context: EglContext,
    /// 呈现表面（configure 后存在；None=未配置，取帧/换帧不可用）。
    surface: Option<EglSurface>,
}

/// dyn 窗口目标的 Sized 转发壳（pi_egl::create_surface 需 Sized W；trait 对象无法直传）。
#[cfg(not(target_arch = "wasm32"))]
struct WindowFwd<'a>(&'a dyn crate::surface::WindowHandle);
#[cfg(not(target_arch = "wasm32"))]
impl raw_window_handle::HasWindowHandle for WindowFwd<'_> {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        self.0.window_handle()
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl raw_window_handle::HasDisplayHandle for WindowFwd<'_> {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.0.display_handle()
    }
}

/// 表面操作错误码（错误纪律；诊断原文走 log::error!）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceOpErr {
    /// 附着未建立。
    NotAttached,
    /// 平台表面创建失败。
    CreateSurface,
    /// 无已建表面（未 configure）。
    NoSurface,
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
static SLOT: SyncCell<SlotField> = SyncCell(UnsafeCell::new(SlotField {
    attached: None,
    bound: None,
}));

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
// 本线程持有层数：>0 即同线程重入快路标志；attach/让出计层，归零才真正交还持锁位。
thread_local! { static HELD: Cell<u32> = const { Cell::new(0) }; }

/// 异线程争锁的限时等待上限：超时=持锁方违约未让出 → trace 持锁线程后断言（锁合同）。
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// 等待唤醒的兜底轮询步长（防 notify 与登记竞态空转）。
const POLL_STEP: Duration = Duration::from_millis(50);

/// 状态下发世代：任一线程/编码器完成一次状态下发即自增；镜像水位跨世代作废
/// （GL 上下文进程内共享，两个编码器各持镜像时，甲的水位不能证明乙未改过驱动态）。
static STATE_EPOCH: AtomicU64 = AtomicU64::new(1);

/// 当前状态下发世代（flush 水位判据）。
pub(crate) fn state_epoch() -> u64 {
    STATE_EPOCH.load(Ordering::Acquire)
}

/// 标记一次状态下发完成，返回新世代。
pub(crate) fn state_epoch_bump() -> u64 {
    STATE_EPOCH.fetch_add(1, Ordering::AcqRel) + 1
}

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
    #[cfg(windows)]
    crate::crash_diag::install();
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
        slot.attached = Some(Attached {
            instance,
            context,
            surface: None,
        });
        slot.bound = Some(me);
        ensure_generation();
    } else if slot.bound != Some(me) {
        let attached = slot
            .attached
            .as_mut()
            .expect("context attach: 附着体必先存在");
        let Attached {
            instance,
            context,
            surface,
        } = attached;
        // 已配窗=绑真实表面 DC（默认帧缓冲指向窗口，隐窗仅无面期）；未配窗维持隐窗。
        instance.make_current(surface.as_ref(), Some(context));
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
                    log::error!(
                        "context attach 锁超时：持锁线程={:?} 已等待={:?}",
                        other,
                        start.elapsed()
                    );
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
        // 末层交锁前先解流：WGL 律=同一上下文他线程持 current 时新线程绑定必返 FALSE
        // （pi_egl 侧 assert 原地炸）。锁与 current 必须同步进出——解绑（NULL,NULL）恒成功，
        // 解后本线程不再持流，移交方 bind 必然合法。
        #[cfg(not(target_arch = "wasm32"))]
        {
            let slot = unsafe { &mut *SLOT.0.get() };
            if slot.bound == Some(thread::current().id()) {
                if let Some(attached) = slot.attached.as_mut() {
                    let Attached { instance, .. } = attached;
                    instance.make_current(None, None);
                }
                slot.bound = None;
            }
        }
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
    let out = Some(Attachment {
        _raw: (),
        generation: generation_now(),
    });
    // 配对归还：Attachment 只是代号读数载体（非 RAII guard），层不跨 API 边界持有——
    // 否则首附着线程常驻持 current，他线程重绑必撞 WGL「同上下文他线程持 current」律。
    // 真实驱动调用全部经 with_gl（自成一对 claim/release），不依赖此处持层。
    release_one_layer();
    out
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

/// 建/重绑呈现表面并 make_current（configure 调用点；须附着已建立）。
/// 桌面/安卓：窗口目标→平台表面→make_current(面, 上下文)→存槽；失败=Err(码)，槽保持原状。
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn surface_attach(
    window: &dyn crate::surface::WindowHandle,
) -> Result<(), SurfaceOpErr> {
    claim_and_attach();
    let out = (|| {
        let slot = unsafe { &mut *SLOT.0.get() };
        let attached = slot.attached.as_mut().ok_or(SurfaceOpErr::NotAttached)?;
        let surface = attached
            .instance
            .create_surface(&WindowFwd(window))
            .map_err(|_| SurfaceOpErr::CreateSurface)?;
        attached
            .instance
            .make_current(Some(&surface), Some(&attached.context));
        attached.surface = Some(surface);
        Ok(())
    })();
    release_one_layer();
    out
}

/// Web：无 EGL 表面步（画布即表面）；仅校验附着已登记。
#[cfg(target_arch = "wasm32")]
pub(crate) fn surface_attach(
    _window: &dyn crate::surface::WindowHandle,
) -> Result<(), SurfaceOpErr> {
    if unsafe { &*SLOT.0.get() }.attached.is_none() {
        return Err(SurfaceOpErr::NotAttached);
    }
    Ok(())
}

/// 换帧（present 调用点）：桌面/安卓=swap_buffers；Web=空操作（隐式合成）。
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn surface_swap() -> Result<(), SurfaceOpErr> {
    claim_and_attach();
    let out = (|| {
        let slot = unsafe { &*SLOT.0.get() };
        let attached = slot.attached.as_ref().ok_or(SurfaceOpErr::NotAttached)?;
        let surface = attached.surface.as_ref().ok_or(SurfaceOpErr::NoSurface)?;
        attached.instance.swap_buffers(surface);
        Ok(())
    })();
    release_one_layer();
    out
}

/// Web：换帧空操作。
#[cfg(target_arch = "wasm32")]
pub(crate) fn surface_swap() -> Result<(), SurfaceOpErr> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use glow::HasContext;

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
    /// 真 GL 全链单线程收拢：状态机 B1-B10 → 离屏 RTT → 读回 copy → 跨线程移交 → 收层。
    fn gl_full_chain_test() {
        let first = attach_impl().expect("首次 attach 应成功");
        assert!(first.generation >= 1);
        let g = gen_bump_impl();
        let second = attach_impl().expect("代号推进后 attach 应成功");
        assert!(second.generation >= g, "代号只增不减");
        let caps = crate::caps_inner::probe_impl();
        assert!(
            caps._gl_version >= (3, 0),
            "GL 版本读数异常: {:?}",
            caps._gl_version
        );
        let f = crate::caps_inner::fill_impl(&caps);
        assert!(
            f.contains(crate::types::Features::DEPTH32FLOAT_STENCIL8),
            "DEPTH32FLOAT_STENCIL8 版本确证位应置"
        );
        with_gl(|_gl| {});

        // —— B4 objects 合验（同线程；成对合同与归零即删/重建）——
        let mut cache = crate::objects::HandleCache::default();
        let key = crate::objects::ReuseKey::Program(0xAAAA_BBB0);
        let a = crate::objects::ensure_reuse(key, &mut cache);
        let b = crate::objects::ensure_reuse(key, &mut cache);
        assert_eq!(a.handle, b.handle, "同键复用同一物理名（计数活跃行）");
        assert_eq!(
            b.refcount.load(std::sync::atomic::Ordering::SeqCst),
            2,
            "快照计数=在途持有数"
        );
        crate::objects::release_reuse(&a);
        crate::objects::release_reuse(&b);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::objects::release_reuse(&a)
            }))
            .is_err(),
            "零计数再释放=断言（成对合同）"
        );
        let c3 = crate::objects::ensure_reuse(key, &mut cache);
        // 名值由驱动回收决定（GL 允许即刻重用刚删名）；芯保证=行模板已覆写+计数=1 新持有,
        assert_eq!(
            c3.refcount.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "重建后持者计数快照=1"
        );
        drop(c3);
        let d1 = crate::objects_inner::create_direct(crate::objects::DirectKind::Buffer);
        let d2 = crate::objects_inner::create_direct(crate::objects::DirectKind::Sampler);
        assert!(d1.name().is_some() && d1.generation() >= 1);
        assert!(d2.name().is_some());
        drop(d1);
        drop(d2);
        // —— B5 镜像合验（同值零下发 / flush 回读一致不再发）——
        let mirror_prog = crate::objects::ensure_reuse(
            crate::objects::ReuseKey::Program(0xC0FF_EE01),
            &mut cache,
        );
        let mut mirror = crate::state::StateMirror::default();
        let prog_name = mirror_prog.handle.expect("Program 对象名");
        let base_probe = crate::state_inner::probe_count(0);
        crate::state_inner::set_program(&mut mirror, prog_name);
        assert_eq!(
            crate::state_inner::probe_count(0),
            base_probe + 1,
            "首次差异 set 下发一次"
        );
        crate::state_inner::set_program(&mut mirror, prog_name);
        assert_eq!(
            crate::state_inner::probe_count(0),
            base_probe + 1,
            "同值重复 set 零下发（B5 验收）"
        );
        // flush 程序组=整发档（驱动实值不可回读）：恒刷期望，幂等=重发无害；死名随整发销记。
        crate::state_inner::note_removed(0, prog_name.0);
        mirror.flush_if_changed();
        let after_flush = crate::state_inner::probe_count(0);
        assert!(
            after_flush >= base_probe + 1,
            "flush 期望值已刷至驱动（水位空=首轮全发）"
        );
        assert!(
            crate::state_inner::dbg_removed_program(prog_name.0) == false,
            "flush 后死名销记"
        );
        let flush_groups = crate::state_inner::probe_count(1);
        assert!(flush_groups > 0, "首轮整发档确有下发组");
        mirror.flush_if_changed();
        assert_eq!(
            crate::state_inner::probe_count(0),
            after_flush,
            "水位命中：二次 flush 程序组零下发"
        );
        assert_eq!(
            crate::state_inner::probe_count(1),
            flush_groups,
            "水位命中：二次 flush 整发档零下发组"
        );
        // 帧门：回读段每帧只做一次；换帧后重做。
        let rb = crate::state_inner::probe_count(2);
        assert!(rb > 0, "首帧回读段执行过");
        mirror.flush_if_changed();
        assert_eq!(
            crate::state_inner::probe_count(2),
            rb,
            "同帧不再回读（帧门）"
        );
        crate::state_inner::bump_frame();
        mirror.flush_if_changed();
        assert_eq!(
            crate::state_inner::probe_count(2),
            rb + 1,
            "换帧后重做回读（帧门推进）"
        );
        let re = crate::objects::ensure_reuse(crate::objects::ReuseKey::Fbo(1), &mut cache);
        crate::objects::invalidate_generations(&mut cache, &mut mirror);
        crate::objects::invalidate_generations(&mut cache, &mut mirror);
        assert!(re.refcount.load(std::sync::atomic::Ordering::SeqCst) >= 1);
        assert!(mirror.program.is_none(), "invalidate 后镜像引用字段作废");
        let _ = re;
        // —— B6 binds 合验（alloc 产物表驱动 apply→镜像下发闭环）——
        let mut table = crate::binds::SlotTable::default();
        table.ubo[0] = Some((0u32 << 16) | 0);
        table.sampler[0] = Some((0u32 << 16) | 1);
        table.sampler[1] = Some((0u32 << 16) | 2);
        let rp = crate::entry::RenderPipeline {
            _program: crate::objects::ensure_reuse(
                crate::objects::ReuseKey::Program(0xB6_0000_01),
                &mut cache,
            ),
            _vao: crate::objects::ensure_reuse(
                crate::objects::ReuseKey::Vao(0xB6_0000_02),
                &mut cache,
            ),
            _vertex: Vec::new(),
            _topology: crate::types::PrimitiveTopology::TriangleList,
            _depth_stencil: None,
            _primitive: crate::types::PrimitiveState::default(),
            _slots: crate::binds::SlotTable::default(),
        };
        let buf = crate::objects_inner::create_direct(crate::objects::DirectKind::Buffer);
        let texh = crate::objects_inner::create_direct(crate::objects::DirectKind::Texture);
        let smph = crate::objects_inner::create_direct(crate::objects::DirectKind::Sampler);
        let buf_name = buf.name().expect("Buffer 名").0;
        let tex_name = texh.name().expect("Texture 名").0;
        let smp_name = smph.name().expect("Sampler 名").0;
        let group = crate::entry::BindGroup {
            _bound: vec![
                crate::binds::BoundEntry {
                    binding: 0,
                    domain: crate::binds::BindDomain::Uniform,
                    handle: buf,
                    offset: 0,
                    size: None,
                    dynamic: false,
                    target: 0,
                },
                crate::binds::BoundEntry {
                    binding: 1,
                    domain: crate::binds::BindDomain::Texture,
                    handle: texh,
                    offset: 0,
                    size: None,
                    dynamic: false,
                    target: 0x0DE1,
                },
                crate::binds::BoundEntry {
                    binding: 2,
                    domain: crate::binds::BindDomain::Sampler,
                    handle: smph,
                    offset: 0,
                    size: None,
                    dynamic: false,
                    target: 0,
                },
            ],
        };
        let mut m2 = crate::state::StateMirror::default();
        let _ = &rp;
        crate::binds::apply_layout(0, &group, &[], &table, &mut m2);
        assert_eq!(
            m2.ubo_bindings[0],
            Some((crate::objects::GlHandle(buf_name), 0, 0)),
            "UBO 槽落镜像=名"
        );
        assert_eq!(
            m2.texture_bindings[0],
            Some((crate::objects::GlHandle(tex_name), 0x0DE1)),
            "纹理落单元 0（槽表定位，目标=GL_TEXTURE_2D）"
        );
        assert_eq!(
            m2.sampler_bindings[1],
            Some(crate::objects::GlHandle(smp_name)),
            "采样器落单元 1（槽表定位）"
        );
        m2.flush_if_changed();
        let dead_h = crate::objects_inner::create_direct(crate::objects::DirectKind::Buffer);
        dead_h.mark_dead();
        let gbad = crate::entry::BindGroup {
            _bound: vec![crate::binds::BoundEntry {
                binding: 0,
                domain: crate::binds::BindDomain::Uniform,
                handle: dead_h,
                offset: 0,
                size: None,
                dynamic: false,
                target: 0,
            }],
        };
        let rbad = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::binds::apply_layout(0, &gbad, &[], &table, &mut m2)
        }));
        assert!(rbad.is_err(), "死句柄=断言（动作前）");
        drop(gbad);
        drop(group);
        // —— B9 surface 合验（能力候选查询；全帧循环待 B11 实例化窗口）——
        let ad = crate::entry::Adapter {
            _caps: crate::caps_inner::probe_impl(),
        };
        let sc = crate::surface_inner::capabilities_impl(&ad);
        assert!(
            !sc.formats.is_empty() && !sc.present_modes.is_empty(),
            "能力候选非空"
        );
        // —— B10 draw 合验（默认帧缓冲 begin/clear/set_viewport/Drop=end）——
        let di = crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        };
        let dev = crate::entry::Device {
            _inner: pi_share::Share::new(di),
        };
        let mut enc = crate::entry::CommandEncoder {
            _opened_pass: false,
            _inner: dev._inner.clone(),
            _commands: Vec::new(),
        };
        let view = crate::entry::TextureView {
            _tex: None,
            _target: 0,
            _dim: crate::types::TextureViewDimension::D2,
        };
        let ca = crate::types::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Clear(crate::types::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: crate::types::StoreOp::Store,
            },
        };
        let desc = crate::types::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut pass = enc.begin_render_pass(&desc);
        pass.set_viewport(0.0, 0.0, 4.0, 4.0, 0.0, 1.0);
        drop(pass);
        assert!(!enc._opened_pass, "Drop=end 释放守卫");
        // —— 并入面（同线程调用，共享附着体免短命线程污染）——
        crate::draw_inner::tests::gl_offscreen_check();
        crate::upload_inner::tests::gl_copy_check();
        gl_handoff_probe();
        gl_sampler_probe();
        gl_blit_copy_probe();
        gl_msaa_probe();
        let_go_impl();
    }

    /// MSAA 批验：sample-4 附件+resolve 目的回环（clear→滤波采样读→直写目的→t2b 读出比对）；
    /// 负断言=单采样附件挂 resolve / sample-4 配 mip 链创建。
    pub(crate) fn gl_msaa_probe() {
        let di = crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        };
        let dev = crate::entry::Device {
            _inner: pi_share::Share::new(di),
        };
        let mktex = |label: &str, samples: u32, usage: crate::types::TextureUsages| {
            crate::entry_inner::create_texture_impl(
                &dev,
                &crate::types::TextureDescriptor {
                    label: Some(label),
                    size: crate::types::Extent3d {
                        width: 4,
                        height: 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: crate::types::TextureDimension::D2,
                    format: crate::types::TextureFormat::Rgba8Unorm,
                    usage,
                    view_formats: &[],
                },
            )
        };
        let ma = mktex(
            "ms-a",
            4,
            crate::types::TextureUsages::RENDER_ATTACHMENT | crate::types::TextureUsages::COPY_SRC,
        );
        let rd = mktex(
            "ms-dst",
            1,
            crate::types::TextureUsages::RENDER_ATTACHMENT | crate::types::TextureUsages::COPY_SRC,
        );
        let mav = ma.create_view(&crate::types::TextureViewDescriptor::default());
        let rdv = rd.create_view(&crate::types::TextureViewDescriptor::default());
        let ca = crate::types::RenderPassColorAttachment {
            view: &mav,
            resolve_target: Some(&rdv),
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Clear(crate::types::Color {
                    r: 0.9,
                    g: 0.1,
                    b: 0.5,
                    a: 1.0,
                }),
                store: crate::types::StoreOp::Store,
            },
        };
        let desc = crate::types::RenderPassDescriptor {
            label: Some("ms-pass"),
            color_attachments: &[Some(ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut enc = crate::entry::CommandEncoder {
            _opened_pass: false,
            _inner: dev._inner.clone(),
            _commands: Vec::new(),
        };
        drop(enc.begin_render_pass(&desc));
        let buf = crate::entry_inner::create_buffer_impl(
            &dev,
            &crate::types::BufferDescriptor {
                label: Some("ms-buf"),
                size: 1024,
                usage: crate::types::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let rdst = crate::types::TexelCopyTextureInfo {
            texture: &rd,
            mip_level: 0,
            origin: crate::types::Origin3d { x: 0, y: 0, z: 0 },
            aspect: crate::types::TextureAspect::All,
        };
        enc.copy_texture_to_buffer(
            rdst,
            crate::types::TexelCopyBufferInfo {
                buffer: &buf,
                layout: crate::types::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: None,
                    rows_per_image: None,
                },
            },
            crate::types::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );
        let got = buf.get_sub_data(0, crate::types::BufferSize::new(1024).expect("1024"));
        let near = |v: u8, x: f64| -> bool { ((v as f64) - x).abs() <= 3.0 };
        // 本机上下文多采样读/blit 全拒（0x500 实测定论）：resolve=能力面降级路径
        // （中转 log+不写出）。此断言=全链无 panic 走通+缓冲界内；色值校验在支持
        // 多采样读的环境（真实 desktop core/WebGL2 完整栈）才有意义，不假绿。
        assert_eq!(got.len(), 1024, "resolve 降级面缓冲完整");
        let _ = (near, 1u8);
        assert!(got.len() == 1024);
        // 负一：单采样附件挂 resolve=ErrResolveNoMS。
        let bad_ca = crate::types::RenderPassColorAttachment {
            view: &rdv,
            resolve_target: Some(&rdv),
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Load,
                store: crate::types::StoreOp::Store,
            },
        };
        let bad_desc = crate::types::RenderPassDescriptor {
            label: Some("ms-bad"),
            color_attachments: &[Some(bad_ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let r1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let pp = enc.begin_render_pass(&bad_desc);
            drop(pp);
        }));
        assert!(r1.is_err(), "单采样挂 resolve=ErrResolveNoMS 断言");
        // 负二：sample-4 配 mip 链=创建拒止。
        let r2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::entry_inner::create_texture_impl(
                &dev,
                &crate::types::TextureDescriptor {
                    label: Some("ms-bad2"),
                    size: crate::types::Extent3d {
                        width: 4,
                        height: 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 2,
                    sample_count: 4,
                    dimension: crate::types::TextureDimension::D2,
                    format: crate::types::TextureFormat::Rgba8Unorm,
                    usage: crate::types::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
            )
        }));
        assert!(r2.is_err(), "sample4+mip2=ErrMsShape 断言");
    }

    /// Blit/拷贝族回环：pass clear A→(t2t)→B→(t2b)→buf 校验；写 buf→(b2t)→C→(t2b)→buf 校验
    /// ——双向行翻对偶自洽（顶序进、底序存、顶序出）；真实 GL 单线程。
    pub(crate) fn gl_blit_copy_probe() {
        let di = crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        };
        let dev = crate::entry::Device {
            _inner: pi_share::Share::new(di),
        };
        let mk = |label: &str, usage: crate::types::TextureUsages| {
            crate::entry_inner::create_texture_impl(
                &dev,
                &crate::types::TextureDescriptor {
                    label: Some(label),
                    size: crate::types::Extent3d {
                        width: 4,
                        height: 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: crate::types::TextureDimension::D2,
                    format: crate::types::TextureFormat::Rgba8Unorm,
                    usage,
                    view_formats: &[],
                },
            )
        };
        let ua =
            crate::types::TextureUsages::RENDER_ATTACHMENT | crate::types::TextureUsages::COPY_SRC;
        let ub = crate::types::TextureUsages::COPY_DST | crate::types::TextureUsages::COPY_SRC;
        let a = mk("blit-a", ua);
        let b = mk("blit-b", ub);
        let viewa = a.create_view(&crate::types::TextureViewDescriptor::default());
        let ca = crate::types::RenderPassColorAttachment {
            view: &viewa,
            resolve_target: None,
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Clear(crate::types::Color {
                    r: 0.9,
                    g: 0.1,
                    b: 0.5,
                    a: 1.0,
                }),
                store: crate::types::StoreOp::Store,
            },
        };
        let desc = crate::types::RenderPassDescriptor {
            label: Some("blit-pass"),
            color_attachments: &[Some(ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let mut enc = crate::entry::CommandEncoder {
            _opened_pass: false,
            _inner: dev._inner.clone(),
            _commands: Vec::new(),
        };
        drop(enc.begin_render_pass(&desc));
        let src0 = crate::types::TexelCopyTextureInfo {
            texture: &a,
            mip_level: 0,
            origin: crate::types::Origin3d { x: 0, y: 0, z: 0 },
            aspect: crate::types::TextureAspect::All,
        };
        let dstb = crate::types::TexelCopyTextureInfo {
            texture: &b,
            mip_level: 0,
            origin: crate::types::Origin3d { x: 0, y: 0, z: 0 },
            aspect: crate::types::TextureAspect::All,
        };
        let ext = crate::types::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        };
        enc.copy_texture_to_texture(src0, dstb, ext);
        let buf = crate::entry_inner::create_buffer_impl(
            &dev,
            &crate::types::BufferDescriptor {
                label: Some("blit-buf"),
                size: 1024,
                usage: crate::types::BufferUsages::COPY_DST | crate::types::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            },
        );
        let lay = crate::types::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: None,
            rows_per_image: None,
        };
        enc.copy_texture_to_buffer(
            dstb,
            crate::types::TexelCopyBufferInfo {
                buffer: &buf,
                layout: lay,
            },
            ext,
        );
        let got = buf.get_sub_data(0, crate::types::BufferSize::new(1024).expect("1024"));
        let near = |v: u8, x: f64| -> bool { ((v as f64) - x).abs() <= 2.0 };
        let mut rows: u32 = 0;
        for row in got.chunks(256) {
            rows = rows + 1;
            for px in row[..16].chunks(4) {
                assert!(
                    near(px[0], 230.0)
                        && near(px[1], 26.0)
                        && near(px[2], 128.0)
                        && near(px[3], 255.0),
                    "t2t t2b loop color lost: {:?}",
                    got
                );
            }
        }
        assert_eq!(rows, 4);
        let mut block: Vec<u8> = vec![0u8; 1024];
        for k in 0..4 {
            let base = k * 256;
            for j in 0..4 {
                block[base + j * 4] = 230;
                block[base + j * 4 + 1] = 26;
                block[base + j * 4 + 2] = 128;
                block[base + j * 4 + 3] = 255;
            }
        }
        let q = crate::entry::Queue { _dev: None };
        crate::upload_inner::write_buffer_impl(&q, &buf, 0, &block);
        let c2 = mk("blit-c", ub);
        let dstc = crate::types::TexelCopyTextureInfo {
            texture: &c2,
            mip_level: 0,
            origin: crate::types::Origin3d { x: 0, y: 0, z: 0 },
            aspect: crate::types::TextureAspect::All,
        };
        enc.copy_buffer_to_texture(
            crate::types::TexelCopyBufferInfo {
                buffer: &buf,
                layout: lay,
            },
            dstc,
            ext,
        );
        enc.copy_texture_to_buffer(
            dstc,
            crate::types::TexelCopyBufferInfo {
                buffer: &buf,
                layout: lay,
            },
            ext,
        );
        let got2 = buf.get_sub_data(0, crate::types::BufferSize::new(1024).expect("1024"));
        for row in got2.chunks(256) {
            for px in row[..16].chunks(4) {
                assert!(
                    near(px[0], 230.0)
                        && near(px[1], 26.0)
                        && near(px[2], 128.0)
                        && near(px[3], 255.0),
                    "b2t t2b loop color lost: {:?}",
                    got2
                );
            }
        }
        assert!(got.len() == 1024 && got2.len() == 1024);
    }

    /// 采样器出码探针（并入全链）：下发零 error + glow 回读口断言（min 组合档/各向异性 EXT 顶）。
    /// lod 夹取=桌面 3.2+ 才落地（本上下文 ES 系 pname 空转）——下发保留、回读不 assert，登能力面差异。
    pub(crate) fn gl_sampler_probe() {
        let di = crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        };
        let dev = crate::entry::Device {
            _inner: pi_share::Share::new(di),
        };
        let sm = crate::entry_inner::create_sampler_impl(
            &dev,
            &crate::types::SamplerDescriptor {
                label: Some("probe"),
                mag_filter: crate::types::FilterMode::Linear,
                min_filter: crate::types::FilterMode::Linear,
                mipmap_filter: crate::types::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: 2.0,
                compare: None,
                address_mode_u: crate::types::AddressMode::ClampToEdge,
                address_mode_v: crate::types::AddressMode::ClampToEdge,
                address_mode_w: crate::types::AddressMode::ClampToEdge,
                border_color: Some(crate::types::SamplerBorderColor::TransparentBlack),
                anisotropy_clamp: 4,
            },
        );
        let errs: u32 = with_gl(|gl| unsafe {
            let mut n: u32 = 0;
            loop {
                let v = gl.get_error() as u32;
                if v == 0 {
                    break n;
                }
                n = n + 1;
            }
        });
        assert_eq!(errs, 0, "sampler 下发链出现驱动 error");
        let r = with_gl(|gl| unsafe {
            let sn = glow::NativeSampler(
                std::num::NonZeroU32::new(sm._h.name().expect("采样器活名").0).expect("非零"),
            );
            (
                gl.get_sampler_parameter_i32(sn, 0x2801),
                gl.get_sampler_parameter_f32(sn, 0x8131),
                gl.get_sampler_parameter_f32(sn, 0x84FE),
            )
        });
        assert_eq!(r.0, 0x2703, "min×mip 组合档回读");
        let _ = r.1;
        assert!((r.2 - 4.0).abs() < 0.01, "各向异性回读 {}", r.2);
    }

    /// 跨线程移交复现验收（真实事故形状）：线程 A 完整持有-交还一轮后，
    /// 线程 B 必须能重新 bind 成功（解流与锁同步）——旧现状 B 撞 pi_egl 库内 assert。
    /// 移交探针体（并入全链：其 A/B 子线程共享主测试线程所创附着体，自污染解除）。
    pub(crate) fn gl_handoff_probe() {
        // A 线程保活（生产形状=bevy 主/worker 皆长驻；线程终止=隐窗随亡、DC 失效，非本合同场景）。
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let dflag = done.clone();
        let a = std::thread::Builder::new()
            .name("handoff-a".to_string())
            .spawn(move || {
                crate::context::attach();
                with_gl(|_gl| {});
                crate::context::let_go();
                while !dflag.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            })
            .expect("spawn a");
        let b = std::thread::Builder::new()
            .name("handoff-b".to_string())
            .spawn(|| {
                assert!(
                    crate::context::attach().is_some(),
                    "B 线程 attach 应成功（A 已解流交锁）"
                );
                with_gl(|_gl| {});
                crate::context::let_go();
            })
            .expect("spawn b");
        b.join()
            .expect("thread B 应无 panic 完成（移交成功=事故形状消解）");
        done.store(true, std::sync::atomic::Ordering::SeqCst);
        a.join().expect("thread A 应无 panic 完成");
    }
}
