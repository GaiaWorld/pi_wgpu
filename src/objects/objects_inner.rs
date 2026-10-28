// 设计：pi-wgpu-rebuild
//! objects 芯 · GL 对象句柄层：一对一直建与四类可复用缓存。契约权威文本住壳面注释。
//! 计数真相=本芯全局驻留表（键→在途引用数，单设备合同下与附着同寿命）；缓存表行持物理名模板，
//! 零计数行=已删待重建位（ensure 命中换新名覆写；GL 名可回收，保守换代不复用死名）。
//! Lost 世代失配后的 release 由 invalidate_generations 显式清场兜底（只改记账、不触驱动）。

use std::sync::atomic::Ordering;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use pi_hash::XHashMap;

use glow::HasContext;

use crate::context_inner::{generation_now, with_gl};
use crate::objects::{
    DirectHandle, DirectHandleState, DirectKind, GlHandle, HandleCache, Reusable, ReuseKey,
};
use crate::state::StateMirror;
use crate::state_inner::note_removed;
use pi_share::{Share, ShareU32, ShareU64};

/// 复用在途计数真相表（键→持有者数）。仅经 bump/drop_count 访问，禁止旁路手改（成对合同）。
static REFCOUNTS: Lazy<Mutex<XHashMap<ReuseKey, u32>>> =
    Lazy::new(|| Mutex::new(XHashMap::default()));

/// 增计数并返回新值（键首次出现=1）。不触驱动。
fn bump(key: ReuseKey) -> u32 {
    let mut t = REFCOUNTS.lock().unwrap_or_else(|e| e.into_inner());
    let slot = t.entry(key).or_insert(0);
    *slot += 1;
    *slot
}

/// 取用/交还串行闸：`count_active` 与 `bump` 分锁查询+自增之间存在他线程清零删除的窗口——
/// 两函数整体串行化（每通道各一次，非热路径；锁序=cache→本闸→驱动）。
static CACHE_OP: Mutex<()> = Mutex::new(());

/// 复用域持有者数只读（RTT 选架占用判据）。
pub(crate) fn holders_impl(key: ReuseKey) -> u32 {
    REFCOUNTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&key)
        .copied()
        .unwrap_or(0)
}

/// 活物理判据：当前计数>0 即活；纯读判定不改计数。
fn count_active(key: ReuseKey) -> bool {
    let t = REFCOUNTS.lock().unwrap_or_else(|e| e.into_inner());
    matches!(t.get(&key), Some(v) if *v > 0)
}

/// 显式淘汰 VAO 复用行（组合缓存 LRU 回收）：移表行、物理删、清计数条目。
pub(crate) fn evict_vao_impl(cache: &mut HandleCache, key: u64) {
    let k = ReuseKey::Vao(key);
    if let Some(row) = cache.map.remove(&k) {
        for item in row {
            if let Some(name) = item.handle {
                with_gl(|gl| unsafe {
                    gl.delete_vertex_array(crate::gid::vao_from(name.0));
                });
            }
        }
    }
    REFCOUNTS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&k);
}

/// 减计数并返回新值；已为零或无键=断言（引用与获取必须严格成对——成对合同被破=调用方违约）。
fn drop_count(key: ReuseKey) -> u32 {
    let mut t = REFCOUNTS.lock().unwrap_or_else(|e| e.into_inner());
    match t.get_mut(&key) {
        Some(v) if *v > 0 => {
            *v -= 1;
            *v
        }
        _ => panic!("objects 断言: 键 {key:?} 计数已零再释放（成对合同被破）"),
    }
}

/// 键域编号（死名刷失表防跨域同名串扰：0..3=复用域，4..6=直建域）。
fn kind_num_reuse(key: ReuseKey) -> u8 {
    match key {
        ReuseKey::Program(_) => 0,
        ReuseKey::Vao(_) => 1,
        ReuseKey::Fbo(_) => 2,
        ReuseKey::Rbo(_) => 3,
    }
}

/// 直建域编号（与复用域编号段错开）。
fn kind_num_direct(kind: DirectKind) -> u8 {
    4 + match kind {
        DirectKind::Buffer => 0,
        DirectKind::Texture => 1,
        DirectKind::Sampler => 2,
    }
}

/// 按域生成复用物理对象柄（glCreate* 分派经 gid 身份层取归一化 u64；Web=slot-key ffi 全 64 位）。
fn gen_by_key(key: ReuseKey) -> GlHandle {
    with_gl(|gl| {
        let name: u64 = unsafe {
            match key {
                ReuseKey::Program(h) => crate::gid::id_of_program(
                    gl.create_program()
                        .unwrap_or_else(|e| panic!("objects 断言: Program 域创建失败 err={e} hash={h}")),
                ),
                ReuseKey::Vao(h) => crate::gid::id_of_vao(
                    gl.create_vertex_array()
                        .unwrap_or_else(|e| panic!("objects 断言: Vao 域创建失败 err={e} hash={h}")),
                ),
                ReuseKey::Fbo(h) => crate::gid::id_of_framebuffer(
                    gl.create_framebuffer()
                        .unwrap_or_else(|e| panic!("objects 断言: Fbo 域创建失败 err={e} hash={h}")),
                ),
                ReuseKey::Rbo(h) => crate::gid::id_of_renderbuffer(
                    gl.create_renderbuffer()
                        .unwrap_or_else(|e| panic!("objects 断言: Rbo 域创建失败 err={e} hash={h}")),
                ),
            }
        };
        // 名可被驱动回收再用：新建即权威「此名为活对象」，销去死名登记（否则守卫永久吞下发）。
        crate::state_inner::forget_removed(kind_num_reuse(key), name);
        GlHandle(name)
    })
}

/// 按域物理删除复用对象并登记死名（镜像按「引用名已死」强制重下发，B5 flush 消费）。
/// 在途 GL 态若正引用该名，删除即失配——合同上 release 只发生在通道锁释放后（RenderPass Drop 之后）。
fn delete_by_key(key: ReuseKey, name: GlHandle) {
    with_gl(|gl| {
        unsafe {
            match key {
                ReuseKey::Program(_) => gl.delete_program(crate::gid::program_from(name.0)),
                ReuseKey::Vao(_) => {
                    crate::draw_inner::combo_forget();
                    gl.delete_vertex_array(crate::gid::vao_from(name.0))
                }
                ReuseKey::Fbo(h) => {
                    crate::draw_inner::row_att_drop_fbo(h);
                    gl.delete_framebuffer(crate::gid::framebuffer_from(name.0))
                }
                ReuseKey::Rbo(_) => gl.delete_renderbuffer(crate::gid::renderbuffer_from(name.0)),
            }
        };
    });
    note_removed(kind_num_reuse(key), name.0);
}

/// 克隆持行：计数+1 并复制句柄快照（录制期把管线等复用对象引用延到提交后）。
/// 与 release_reuse 严格成对；不做 GL 调用（对象已存在）。
pub(crate) fn retain_reuse_impl(item: &Reusable) -> Reusable {
    let _op = CACHE_OP.lock().unwrap_or_else(|e| e.into_inner());
    let c = bump(item.key);
    Reusable {
        key: item.key,
        handle: item.handle,
        refcount: ShareU32::new(c),
    }
}

// 1. 查表行且计数活跃 → 计数+1，行模板计数覆写快照，返回持有者副本（原子快照读数，真相在驻留表）。
// 2. 行存在但计数已零（release 已物理删）→ 重新 glGen 原地覆写行（不复用缓存旧名——驱动可回收该名，行只做覆写不假定新名≠旧名）。
// 3. 未命中 → 建物理、以计数=1 登记行与驻留表后返回；创建失败=断言，全表保持调用前原状。
pub(crate) fn ensure_reuse_impl(key: ReuseKey, cache: &mut HandleCache) -> Reusable {
    let _op = CACHE_OP.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(row) = cache.map.get_mut(&key) {
        if let Some(item) = row.first_mut() {
            if count_active(key) {
                let c = bump(key);
                item.refcount.store(c, Ordering::SeqCst);
                return Reusable {
                    key,
                    handle: item.handle,
                    refcount: ShareU32::new(c),
                };
            }
            // Fbo 域驻留（2026-10-28）：计数归零不再物理删（release 侧豁免）——有名行直接沿用，
            // 跨帧同形状=同对象同附着；重建 genGl 仅无名/世代失效行。
            let fresh = match (key, item.handle) {
                (ReuseKey::Fbo(_) | ReuseKey::Vao(_), Some(existing)) => existing,
                _ => {
                    let g = gen_by_key(key);
                    item.handle = Some(g);
                    g
                }
            };
            let c = bump(key);
            item.refcount.store(c, Ordering::SeqCst);
            return Reusable {
                key,
                handle: Some(fresh),
                refcount: ShareU32::new(c),
            };
        }
    }
    let fresh = gen_by_key(key);
    let c = bump(key);
    cache.map.insert(
        key,
        vec![Reusable {
            key,
            handle: Some(fresh),
            refcount: ShareU32::new(c),
        }],
    );
    Reusable {
        key,
        handle: Some(fresh),
        refcount: ShareU32::new(c),
    }
}

// 1. 全局计数-1（本地原子快照不参与真相）；已为零 → 断言（成对合同被破）。
// 2. 减到零 → 按域物理删除 + 登记死名；缓存行留死名原样待 ensure 换新覆写。
//    Fbo 域豁免删除（驻留复用）——pass 级归还不再触发整行重建，附件与对象跨帧稳定。
pub(crate) fn release_reuse_impl(item: &Reusable) {
    let _op = CACHE_OP.lock().unwrap_or_else(|e| e.into_inner());
    let left = drop_count(item.key);
    if left == 0 && !matches!(item.key, ReuseKey::Fbo(_) | ReuseKey::Vao(_)) {
        if let Some(name) = item.handle {
            delete_by_key(item.key, name);
        }
    }
}

// 1. 先置 None（防重入：Drop 与手动 mark_dead 竞到一起只删一次）。
// 2. 即调该资源域 glDelete*（1:1 资源无延迟队列、无等待 GC）。
// 3. 登记死名刷失（引用名集=state 芯驻留）；此后句柄为死态，消费方对死代号的使用=其自身断言拦截。
pub(crate) fn mark_dead_impl(h: &DirectHandle) {
    let raw = h.inner.handle.swap(0, Ordering::SeqCst);
    if raw != 0 {
        let name = GlHandle(raw);
        delete_direct(h.inner.kind, name);
    }
}

// 末引用析构入口（Drop for DirectHandleState）：与显式 mark_dead 同语义（先置 None 防重入，再删+登记）。
pub(crate) fn drop_impl(s: &mut DirectHandleState) {
    let raw = s.handle.swap(0, Ordering::SeqCst);
    if raw != 0 {
        let name = GlHandle(raw);
        delete_direct(s.kind, name);
    }
}

/// 直建资源物理删除 + 死名登记（mark_dead 与末引用析构共用）。
fn delete_direct(kind: DirectKind, name: GlHandle) {
    with_gl(|gl| {
        unsafe {
            match kind {
                DirectKind::Buffer => gl.delete_buffer(crate::gid::buffer_from(name.0)),
                DirectKind::Texture => {
                    crate::draw_inner::row_att_drop_tex(name.0);
                    gl.delete_texture(crate::gid::texture_from(name.0))
                }
                DirectKind::Sampler => gl.delete_sampler(crate::gid::sampler_from(name.0)),
            }
        };
    });
    note_removed(kind_num_direct(kind), name.0);
}

// 1. 只改记账、不调驱动（上下文已丢，删除无意义）：缓存全项名置空、计数归零；驻留计数表清空。
// 2. 死名刷失集整面清空（旧世代名集在新附着下无意义）。
// 3. 镜像整面重置（芯 reset_all，字段面随 state 壳展齐演进）+ 死名登记表整面清空；幂等——重复调用与单次效果相同。
pub(crate) fn invalidate_impl(c: &mut HandleCache, s: &mut StateMirror) {
    for (_k, row) in c.map.iter_mut() {
        for item in row.iter_mut() {
            item.handle = None;
            item.refcount.store(0, Ordering::SeqCst);
        }
    }
    let mut t = REFCOUNTS.lock().unwrap_or_else(|e| e.into_inner());
    t.clear();
    drop(t);
    crate::draw_inner::row_att_forget();
    crate::draw_inner::combo_forget();
    crate::state_inner::clear_removed();
    crate::state_inner::reset_all(s);
}

/// 直建一对一资源（entry 创建链 B11 消费）：glGen*/glCreate* 得活态名，代号=当前附着快照。
/// 域删除在末引用 Drop 自动发生（共享计数；合同=死亡即删）；跨线程搬运前提见 context 壳 attach 条款。
pub(crate) fn create_direct(kind: DirectKind) -> DirectHandle {
    let name = with_gl(|gl| -> u64 {
        unsafe {
            match kind {
                DirectKind::Buffer => crate::gid::id_of_buffer(
                    gl.create_buffer()
                        .unwrap_or_else(|e| panic!("objects 断言: Buffer 直建失败 err={e}")),
                ),
                DirectKind::Texture => crate::gid::id_of_texture(
                    gl.create_texture()
                        .unwrap_or_else(|e| panic!("objects 断言: Texture 直建失败 err={e}")),
                ),
                DirectKind::Sampler => crate::gid::id_of_sampler(
                    gl.create_sampler()
                        .unwrap_or_else(|e| panic!("objects 断言: Sampler 直建失败 err={e}")),
                ),
            }
        }
    });
    crate::state_inner::forget_removed(kind_num_direct(kind), name);
    DirectHandle {
        inner: Share::new(DirectHandleState {
            kind,
            handle: ShareU64::new(name),
            generation: generation_now(),
        }),
    }
}
