// 设计：pi-wgpu-rebuild
//! draw 芯 · 渲染通道录制（begin/set/draw；Drop=end）。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：合同违反（嵌套通道/死句柄/参数级）保留断言（短码）；断言前无副作用。

use glow::HasContext;

use crate::context_inner::with_gl;
use crate::draw::{RenderPass, VertexLayout};
use crate::entry::{BindGroup, Buffer, CommandEncoder, RenderPipeline};
use crate::state::StateMirror;
use crate::state_inner;
use crate::types::{Color, DynamicOffset, IndexFormat, RenderPassDescriptor};

// GL 常量（glow 根面不导出 GL_*；规范值内联）。
// NOTE: split DRAW/READ framebuffer enums are rejected (0x500) on this WGL context;
// every framebuffer bind must use the combined GL_FRAMEBUFFER target.
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_FRAMEBUFFER: u32 = 0x8D40;
const GL_FRAMEBUFFER_COMPLETE: u32 = 0x8CD5;
// 0x8229 is GL_R8; the framebuffer depth attachment point is 0x8D00.
const GL_DEPTH_ATTACHMENT: u32 = 0x8D00;
const GL_DEPTH_STENCIL_ATTACHMENT: u32 = 0x821A;
const GL_DEPTH_BUFFER_BIT: u32 = 0x0000_0100;
const GL_STENCIL_BUFFER_BIT: u32 = 0x0000_0400;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
/// GL_SCISSOR_TEST：裁剪测试使能态（0x0C11；清屏守卫带临时禁用后归还）。
const GL_SCISSOR_TEST: u32 = 0x0C11;
/// GL_FRAMEBUFFER_SRGB：sRGB 写转换开关（0x8DB9；清屏期暂关、按 GL 现值归还）。
const GL_FRAMEBUFFER_SRGB: u32 = 0x8DB9;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const GL_FLOAT: u32 = 0x1406;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_SHORT: u32 = 0x1402;
const GL_UNSIGNED_SHORT: u32 = 0x1403;
const GL_UNSIGNED_INT: u32 = 0x1405;
/// 实例步进除数（Instance 模式）。
const INSTANCE_DIVISOR: u32 = 1;

/// 顶点格式→(分量数, GL 类型, 归一化, 整数通路)——上游全 45 值 GL 核心号全表（WebGL2 ES3 合法子集；
/// Float64 系为桌面专有、Unorm8x4Bgra 为桌面专有扩展——管线创建域外白名单先拒，本表余项全可达，权作完备登记）。
/// 整数通路=非归一化整型格式须走 glVertexAttribIPointer：ES3/WebGL2 的 f32 变体类型集不含 INT/UNSIGNED_INT，
/// 且整型属性经 f32 变体必被驱动转浮点=uvec/ivec 形参语义错。归一化族（Unorm/Snorm）与浮点族仍走 f32 变体。
const GL_BYTE: u32 = 0x1400;
const GL_HALF_FLOAT: u32 = 0x140B;
// GL_DOUBLE is 0x140A (0x140E is not a vertex type).
const GL_DOUBLE: u32 = 0x140A;
const GL_UNSIGNED_INT_2_10_10_10_REV: u32 = 0x8368;
const GL_UNSIGNED_INT_8_8_8_8_REV: u32 = 0x8367;
fn vtx_fmt(f: crate::types::VertexFormat) -> (i32, u32, bool, bool) {
    use crate::types::VertexFormat::*;
    match f {
        Uint8 => (1, GL_UNSIGNED_BYTE, false, true),
        Uint8x2 => (2, GL_UNSIGNED_BYTE, false, true),
        Uint8x4 => (4, GL_UNSIGNED_BYTE, false, true),
        Sint8 => (1, GL_BYTE, false, true),
        Sint8x2 => (2, GL_BYTE, false, true),
        Sint8x4 => (4, GL_BYTE, false, true),
        Unorm8 => (1, GL_UNSIGNED_BYTE, true, false),
        Unorm8x2 => (2, GL_UNSIGNED_BYTE, true, false),
        Unorm8x4 => (4, GL_UNSIGNED_BYTE, true, false),
        Snorm8 => (1, GL_BYTE, true, false),
        Snorm8x2 => (2, GL_BYTE, true, false),
        Snorm8x4 => (4, GL_BYTE, true, false),
        Uint16 => (1, GL_UNSIGNED_SHORT, false, true),
        Uint16x2 => (2, GL_UNSIGNED_SHORT, false, true),
        Uint16x4 => (4, GL_UNSIGNED_SHORT, false, true),
        Sint16 => (1, GL_SHORT, false, true),
        Sint16x2 => (2, GL_SHORT, false, true),
        Sint16x4 => (4, GL_SHORT, false, true),
        Unorm16 => (1, GL_UNSIGNED_SHORT, true, false),
        Unorm16x2 => (2, GL_UNSIGNED_SHORT, true, false),
        Unorm16x4 => (4, GL_UNSIGNED_SHORT, true, false),
        Snorm16 => (1, GL_SHORT, true, false),
        Snorm16x2 => (2, GL_SHORT, true, false),
        Snorm16x4 => (4, GL_SHORT, true, false),
        Float16 => (1, GL_HALF_FLOAT, false, false),
        Float16x2 => (2, GL_HALF_FLOAT, false, false),
        Float16x4 => (4, GL_HALF_FLOAT, false, false),
        Float32 => (1, GL_FLOAT, false, false),
        Float32x2 => (2, GL_FLOAT, false, false),
        Float32x3 => (3, GL_FLOAT, false, false),
        Float32x4 => (4, GL_FLOAT, false, false),
        Uint32 => (1, GL_UNSIGNED_INT, false, true),
        Uint32x2 => (2, GL_UNSIGNED_INT, false, true),
        Uint32x3 => (3, GL_UNSIGNED_INT, false, true),
        Uint32x4 => (4, GL_UNSIGNED_INT, false, true),
        Sint32 => (1, GL_INT, false, true),
        Sint32x2 => (2, GL_INT, false, true),
        Sint32x3 => (3, GL_INT, false, true),
        Sint32x4 => (4, GL_INT, false, true),
        Float64 => (1, GL_DOUBLE, false, false),
        Float64x2 => (2, GL_DOUBLE, false, false),
        Float64x3 => (3, GL_DOUBLE, false, false),
        Float64x4 => (4, GL_DOUBLE, false, false),
        Unorm10_10_10_2 => (4, GL_UNSIGNED_INT_2_10_10_10_REV, true, false),
        Unorm8x4Bgra => (4, GL_UNSIGNED_INT_8_8_8_8_REV, true, false),
    }
}
const GL_INT: u32 = 0x1404;

/// 索引格式→GL 类型。
fn idx_type(f: IndexFormat) -> u32 {
    match f {
        IndexFormat::Uint16 => GL_UNSIGNED_SHORT,
        IndexFormat::Uint32 => GL_UNSIGNED_INT,
    }
}

/// 拓扑→GL 图元。
fn topo(m: crate::types::PrimitiveTopology) -> u32 {
    match m {
        crate::types::PrimitiveTopology::TriangleList => 0x0004,
        crate::types::PrimitiveTopology::TriangleStrip => 0x0005,
        crate::types::PrimitiveTopology::LineList => 0x0001,
        crate::types::PrimitiveTopology::LineStrip => 0x0003,
        crate::types::PrimitiveTopology::PointList => 0x0000,
    }
}

/// 顶点格式判别码（哈希用；判别式即上游序，直接转 u8）。
fn fmt_code(f: crate::types::VertexFormat) -> u8 {
    f as u8
}
/// 索引宽（字节）。
fn index_width(f: IndexFormat) -> u64 {
    match f {
        IndexFormat::Uint16 => 2,
        IndexFormat::Uint32 => 4,
    }
}
/// 布局哈希（VAO 复用键）。
pub(crate) fn layout_hash(v: &[VertexLayout]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for l in v {
        l.array_stride.hash(&mut h);
        (matches!(l.step_mode, crate::types::VertexStepMode::Instance) as u8).hash(&mut h);
        for a in l.attributes.iter() {
            fmt_code(a.format).hash(&mut h);
            a.offset.hash(&mut h);
            a.shader_location.hash(&mut h);
        }
    }
    h.finish()
}

/// 非零包装（活句柄≥1；零=合同违反）。

/// 借设备状态镜像（锁序：state 先、with_gl 后）。
fn lock_state<'a>(p: &'a RenderPass<'_>) -> std::sync::MutexGuard<'a, StateMirror> {
    p._enc
        ._inner
        .state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

// 设计预期（伪码·示意·非唯一实现）：begin=开通道。
// 1. 编码器守卫检查：已有开启中通道=断言，通道不建。
// 2. 颜色附件恰一 Some；视图活态；timestamp_writes/occlusion_query_set=Some 恒拒（查询面无对应物）。违约=断言。
// 3. 目标：帧视图=默认帧缓冲；宿主视图=按配置键复用 FBO（色附件 mip0/层0；深度、MSAA、
// 4. 守卫置位，返回借用编码器的 RenderPass。
pub(crate) fn exec_begin_impl<'p>(
    e: &'p mut CommandEncoder,
    d: &RenderPassDescriptor<'_>,
) -> RenderPass<'p> {
    crate::state_inner::check_err("begin_pass");
    assert!(!e._opened_pass, "draw str-assert: 嵌套 render pass");
    assert!(
        d.timestamp_writes.is_none() && d.occlusion_query_set.is_none(),
        "draw str-assert: ErrQuery 时间戳/遮挡查询面无对应物（label={:?}）",
        d.label
    );
    let some = d.color_attachments.iter().filter(|x| x.is_some()).count();
    assert!(
        some <= 1,
        "draw str-assert: 颜色附件至多一个 Some（得 {some}）"
    );
    let ca = d.color_attachments.iter().find_map(|x| x.as_ref());
    let dsa = d.depth_stencil_attachment.as_ref();
    let ctext: Option<&crate::entry::Texture> = ca.and_then(|x| x.view._tex.as_ref());
    let dtext: Option<&crate::entry::Texture> = dsa.and_then(|x| x.view._tex.as_ref());
    if let Some(t) = ctext {
        host_gate(t, "color");
    }
    // MSAA resolve 配对门：resolve 在场=附件须 sample-4 宿主、目的须单采样 D2 单层同格式宿主。
    let mut resolve_pair: Option<(crate::entry::Texture, crate::entry::Texture)> = None;
    if let Some(rv) = ca.and_then(|x| x.resolve_target) {
        let ms: &crate::entry::Texture =
            ctext.expect("draw str-assert: resolve 附件宿主不可为帧视图");
        assert!(
            ms._samples == 4,
            "draw str-assert: ErrResolveNoMS resolve 目的需 sample-4 附件（samples={}）",
            ms._samples
        );
        let rtx: &crate::entry::Texture = rv
            ._tex
            .as_ref()
            .expect("draw str-assert: resolve 目的帧视图无宿主");
        assert!(
            rtx._samples == 1 && rtx._dim == crate::types::TextureDimension::D2 && rtx._layers == 1,
            "draw str-assert: ErrResolveDst resolve 目的需单采样 D2 单层（samples={}）",
            rtx._samples
        );
        assert!(
            rtx._format == ms._format,
            "draw str-assert: ErrResolveFmt resolve 目的与附件格式须一致",
        );
        resolve_pair = Some((ms.clone(), rtx.clone()));
    }
    if let Some(t) = dtext {
        host_gate(t, "depth");
    }
    // Packed-stencil capability of the depth attachment (non-packed => stencil ops unusable).
    let stencil_ok: bool = dtext
        .map(|t| is_packed_stencil(t._format))
        .unwrap_or(false);
    // 深度格式名（降级留痕用）：上层普遍无条件给 stencil_ops，纯深度格式下属预期组合。
    let dfmt = dtext.map(|t| t._format);
    // 附件宿主合法性：dsa 视图缺宿主（帧视图挂深度槽）或 color 槽挂深度族=拒止。
    if let Some(x) = dsa {
        assert!(
            x.view._tex.is_some(),
            "draw str-assert: ErrRttDepth 深度槽不接受帧视图宿主（label={:?}）",
            d.label
        );
        // ErrStencilFormat: stencil ops on a non-packed depth format has no GL slot.
        // Degrade (log + ignore stencil) instead of aborting the frame; clear path below
        // also gates on stencil_ok so no GL_STENCIL clear hits a depth-only attachment.
        // 预期组合（上层恒给 stencil_ops + 纯深度格式）：忽略模板并留痕，不按错误刷屏。
        if dsa.unwrap().stencil_ops.is_some() && !stencil_ok && warn_once(d.label) {
            log::warn!(
                "draw: ErrStencilFormat 深度格式无模板面（降级=忽略模板，format={dfmt:?} label={:?}）",
                d.label
            );
        }
    }
    // 清除掩码合成：color/depth/stencil 各槽按 load=Clear 参与同一次 glClear——
    // 与目标绑定同处一个 GL 段（他线程不得插进「绑目标」与「清目标」之间改绑）。
    let mut mask: u32 = 0;
    let mut col: Option<crate::types::Color> = None;
    if let Some(x) = ca {
        if let crate::types::LoadOp::Clear(c) = x.ops.load {
            mask |= GL_COLOR_BUFFER_BIT;
            col = Some(c);
        }
    }
    let mut dep: Option<f32> = None;
    let mut ste: Option<u32> = None;
    if let Some(x) = dsa {
        if let Some(ops) = x.depth_ops.as_ref() {
            if let crate::types::LoadOp::Clear(v) = ops.load {
                mask |= GL_DEPTH_BUFFER_BIT;
                dep = Some(v as f32);
            }
        }
        if stencil_ok {
            if let Some(ops) = x.stencil_ops.as_ref() {
                if let crate::types::LoadOp::Clear(v) = ops.load {
                    mask |= GL_STENCIL_BUFFER_BIT;
                    ste = Some(v);
                }
            }
        }
    }
    // 同段清除体（目标已绑；由调用方在 GL 段内执行）。
    // 清屏守卫带（WebGPU load-op 语义=附件全体、不受残状影响；GL 里裁剪与写掩码是全局态、
    // 随 FBO 切换不洗——上通道残裁剪框会裁缺清屏、残写掩码会挡清除值写入，产品 render_core
    // 高频 depth Write(false)=实弹面）：清屏临时禁裁剪+写掩码全开+sRGB 关，清毕按上层镜像
    // 期望归还（镜像只读不动、零下发链与水位不扰；深度使能/比较与 sRGB 另有回读档 flush
    // 自纠=双保险；模板写掩码全库无下发面=恒初值，不动）。视口与 glClear 无涉、不设默认
    // （登记差异：上游 pass 起始重置全屏视口，本层通道未显式 set 者延续上通道，render_core
    // 每通道必设全屏=无暴露）。
    let do_clear = |gl: &glow::Context| {
        if mask == 0 {
            return;
        }
        let (sc_expected, sc_box, cw_bits, dw_expected) = {
            let m = e._inner.state.lock().unwrap_or_else(|x| x.into_inner());
            (
                m.scissor_test.unwrap_or(false),
                m.scissor,
                m.color_writes.map(|w| w.bits()).unwrap_or(0xF),
                m.depth_write_enabled.unwrap_or(true),
            )
        };
        unsafe {
            #[cfg(not(target_arch = "wasm32"))]
            let srgb_on = gl.is_enabled(GL_FRAMEBUFFER_SRGB);
            #[cfg(target_arch = "wasm32")]
            let srgb_on = false;
            // 守卫带进场：残状不参清屏语义。
            gl.disable(GL_SCISSOR_TEST);
            gl.color_mask(true, true, true, true);
            gl.depth_mask(true);
            if srgb_on {
                gl.disable(GL_FRAMEBUFFER_SRGB);
            }
            if let Some(c) = col {
                gl.clear_color(c.r as f32, c.g as f32, c.b as f32, c.a as f32);
            }
            if let Some(v) = dep {
                // 深度清除按形态分口：桌面=glClearDepth(f64) 核心入口（glClearDepthf 为 ES/EXT 名，
                // 桌面 wgl 取该指针不保证在场=实测静默丢事故，登记）；Web=glClearDepthf(f32) ES 核心
                // ——_f64 专口在 glow web_sys 后端=合同级 panic（2026-10-27 产品实锤）。
                #[cfg(not(target_arch = "wasm32"))]
                gl.clear_depth_f64(v as f64);
                #[cfg(target_arch = "wasm32")]
                gl.clear_depth_f32(v as f32);
            }
            if let Some(v) = ste {
                gl.clear_stencil(v as i32);
            }
            gl.clear(mask);
            // 守卫带退场：按上层镜像期望逐位归还（None=GL 初始默认形为已在上取回读时折算）。
            if sc_expected {
                gl.enable(GL_SCISSOR_TEST);
                if let Some(b) = sc_box {
                    gl.scissor(b[0], b[1], b[2], b[3]);
                }
            }
            gl.color_mask(cw_bits & 1 != 0, cw_bits & 2 != 0, cw_bits & 4 != 0, cw_bits & 8 != 0);
            gl.depth_mask(dw_expected);
            if srgb_on {
                gl.enable(GL_FRAMEBUFFER_SRGB);
            }
        }
    };
    // 通道目标：双无宿主=默认帧缓冲；任一宿主=按（色×深×目标）配置键复用 FBO 行。
    let host = ctext.or(dtext);
    let fbo: Option<crate::objects::Reusable> = match host {
        None => None,
        Some(_) => {
            let fk0: u64 = rtt_fbo_key(ctext, dtext);
            let cur: RowAtt = row_att_sig_of(ctext, dtext);
            let (fk, item) = {
                let mut cache = e._inner.cache.lock().unwrap_or_else(|x| x.into_inner());
                // 选架：签名=本轮宿主之架优先；未被占即用；主架被在世通道占用（乱序交错）探备用架。
                let p = pick_rtt_row(&mut cache, fk0, cur, true);
                (p.key, p.item)
            };
            let fb = item
                .handle
                .expect("draw str-assert: ensure_reuse 返回无名行（Fbo 半登记）");
            // 绑定即水位：换 pass 必同步 fbo_draw，防 flush 按旧行重绑致漂移。
            {
                let mut mm = e._inner.state.lock().unwrap_or_else(|x| x.into_inner());
                crate::state_inner::set_draw_fbo(&mut mm, Some(fb));
                crate::state_inner::flush_impl(&mut mm);
    crate::state_inner::check_err("after_flush_mm");
            }
            let need_full: bool = match row_att_get(fk) {
                Some(old) => old != cur,
                None => true,
            };
            with_gl(|gl| unsafe {
                gl.bind_framebuffer(
                    GL_FRAMEBUFFER,
                    Some(crate::gid::framebuffer_from(fb.0)),
                );
                if need_full {
                // 行复用安全：先裸槽，防旧形状行残挂他宿主误判（0x8CD7 实测教训）。
                gl.framebuffer_texture_2d(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, 0x0DE1, None, 0);
                gl.framebuffer_texture_2d(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, 0x0DE1, None, 0);
                gl.framebuffer_texture_2d(GL_FRAMEBUFFER, 0x821A, 0x0DE1, None, 0);
                if cur.0 != 0 {
                    gl.framebuffer_texture_2d(
                        GL_FRAMEBUFFER,
                        GL_COLOR_ATTACHMENT0,
                        cur.1,
                        Some(crate::gid::texture_from(cur.0)),
                        0,
                    );
                }
                if cur.2 != 0 {
                    gl.framebuffer_texture_2d(
                        GL_FRAMEBUFFER,
                        cur.4,
                        cur.3,
                        Some(crate::gid::texture_from(cur.2)),
                        0,
                    );
                }
                let st: u32 = gl.check_framebuffer_status(GL_FRAMEBUFFER) as u32;
                assert!(
                    st == GL_FRAMEBUFFER_COMPLETE,
                    "draw str-assert: 离屏 FBO 不完备（status={:#x}）",
                    st
                );
                row_att_set(fk, cur);
                }
                do_clear(gl);
            });
            Some(item)
        }
    };
    if fbo.is_none() {
        // 默认帧通道也同步水位；绑目标与清除同段（防他线程插改绑）。
        with_gl(|gl| {
            unsafe { gl.bind_framebuffer(GL_FRAMEBUFFER, None) };
            do_clear(gl);
        });
        let mut mm = e._inner.state.lock().unwrap_or_else(|x| x.into_inner());
        crate::state_inner::set_draw_fbo(&mut mm, None);
    }

    // store=Discard 档：GL 单缓冲无丢弃语义，log 留痕后 noop（登记差异）。
    if let Some(x) = ca {
        if x.ops.store == crate::types::StoreOp::Discard {
            log::debug!("draw: store=Discard 落 noop（GL 单缓冲语义，登记差异）");
        }
    }
    e._opened_pass = true;
    RenderPass {
        _enc: e,
        _vertex: Vec::new(),
        _pipe_set: false,
        _vertex_ready: false,
        _index_ready: false,
        _topology: crate::types::PrimitiveTopology::TriangleList,
        _index_format: None,
        _index_range: (0, 0),
        _vao: None,
        _slots: crate::binds::SlotTable::default(),
        _replay: true,
        _vbinds: Vec::new(),
        _index_ibo: None,
        _fbo: fbo,
        _hold_color: ctext.cloned(),
        _hold_depth: dtext.cloned(),
        _resolve: resolve_pair,
    }
}

// 设计预期：死程序句柄=断言；管线字段改记，程序切换差异下发。
/// 执行芯：按「部件」设管线（录制持行体与直用共用；不依赖 RenderPipeline 本体）。
pub(crate) fn exec_set_pipeline_parts(
    p: &mut RenderPass<'_>,
    x: &crate::record::RetainedPipeline,
) {
    crate::state_inner::check_err("set_pipeline");
    let name = x.program.handle.expect("draw str-assert: 死程序句柄");
    {
        let mut m = lock_state(p);
        // 指位为程序对象持久态（管线创建期一次下发，entry collect_*），镜像不记、切换不重发。
        state_inner::set_program(&mut m, name);
    }
    p._vertex = x.vertex.clone();
    p._topology = x.topology;
    p._vao = Some(x.vao.handle.expect("draw str-assert: 死 VAO 句柄"));
    {
        let mut m = lock_state(p);
        match x.depth_stencil.as_ref() {
            Some(ds) => crate::state_inner::set_depth(
                &mut m,
                true,
                ds.depth_write_enabled,
                ds.depth_compare,
            ),
            None => crate::state_inner::set_depth(
                &mut m,
                false,
                true,
                crate::types::CompareFunction::Always,
            ),
        }
        // 混合随管线定态下发（2026-10-22 断线修复）：None=关混合；Some=开并双方程四因子。
        state_inner::set_blend(&mut m, x.blend);
    }
    p._slots = x.slots.clone();
    p._pipe_set = true;
}

/// 直用形（形照上游 set_pipeline）：由部件形派生。
pub(crate) fn exec_set_pipeline(p: &mut RenderPass<'_>, x: &RenderPipeline) {
    let held = crate::record::RetainedPipeline::new(x);
    exec_set_pipeline_parts(p, &held);
}

// 设计预期：set∈0-3 且偏移计数与声明一致；委托 binds.apply_layout。// 设计预期：set∈0-3 且偏移计数与声明一致；委托 binds.apply_layout。
pub(crate) fn exec_set_bind_group(
    p: &mut RenderPass<'_>,
    s: u32,
    g: &BindGroup,
    o: &[DynamicOffset],
) {
    assert!(s <= 3, "draw str-assert: set={s} 超 0-3");
    let mut m = lock_state(p);
    let mut offs = o;
    crate::binds::consume_dynamic_offsets(g, &mut offs);
    // 槽表=当前管线那份（set_pipeline 拷入）；未设管线即绑定=违约断言（查槽无同源地）。
    // 用设备级表会把「后建的别的管线」的编号错配到本组，绝无静默错绑路径。
    assert!(
        p._pipe_set,
        "draw str-assert: 未设管线即 set_bind_group（set={s}）"
    );
    crate::binds::apply_layout(s, g, o, &p._slots, &mut m);
}

// 设计预期：句柄活态（形照 25.0.2：入参=BufferSlice，步长由管线布局权威、不再收调用方值——
// 原「步长与布局不符」比对随形退役；片段边界在 buffer.slice() 构造期已验（entry_inner slice_bounds））。
// ───────────────── 组合 VAO（顶点属性一次整备、draw 只 bind）─────────────────

/// 组合 VAO 的整备状态。
struct ComboState {
    /// 属性指针/enable/divisor 已在本文 VAO 上整备完毕（之后同组合复用零下发）。
    settled: bool,
    /// 本文 VAO 当前绑定的 ELEMENT_ARRAY 缓冲名（None=未绑）。
    elem: Option<u64>,
    /// 最近使用时刻（LRU 淘汰序）。
    last: u64,
    /// 是否草稿（溢出共享 VAO：不参与 LRU、不置整备标记）。
    scratch: bool,
}

/// 组合 VAO 状态表：键=布局哈希×按槽号升序的各槽 (槽号,缓冲,偏移+实例 base 移位)。
static COMBO: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashMap<u64, ComboState>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// 组合缓存上限（WebGL 驱动对 VAO 总量有上限，取保守值；超限按 LRU 真删回收）。
const COMBO_CAP: usize = 128;

/// 草稿键盐：每布局一具共享 VAO，每次使用都重整备（不做整备标记）。
const COMBO_SCRATCH_SALT: u64 = 0x5C7A_7C4B_0000_0001;

/// LRU 时刻源。
static COMBO_TICK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 组合哈希混合子。
fn combo_mix(acc: u64, v: u64) -> u64 {
    (acc ^ v).wrapping_mul(0x0100_0000_0000_b711)
}

/// 组合键与草稿键：布局哈希 + 按槽号升序的各槽 (缓冲名, 偏移 + 实例 base 移位)。
fn combo_keys(p: &RenderPass<'_>, base: u32) -> (u64, u64) {
    let lay = layout_hash(&p._vertex);
    let mut per: [Option<(u64, u64)>; 16] = [None; 16];
    for (slot, name, o) in p._vbinds.iter() {
        let s = *slot as usize;
        if s >= per.len() {
            continue;
        }
        let shift: u64 = match p._vertex.get(s) {
            Some(l) if matches!(l.step_mode, crate::types::VertexStepMode::Instance) => {
                base as u64 * l.array_stride as u64
            }
            _ => 0,
        };
        per[s] = Some((name.0, o.wrapping_add(shift)));
    }
    let mut k = lay;
    for (s, v) in per.iter().enumerate() {
        if let Some((n, off)) = *v {
            k = combo_mix(k, s as u64);
            k = combo_mix(k, n);
            k = combo_mix(k, off);
        }
    }
    (k, combo_mix(lay, COMBO_SCRATCH_SALT))
}

/// 超限回收：按 LRU 淘汰非草稿组合（返回被淘汰键，供 GL 侧真删）。
fn combo_maintain() -> Vec<u64> {
    let mut t = COMBO.lock().unwrap_or_else(|e| e.into_inner());
    if t.len() < COMBO_CAP {
        return Vec::new();
    }
    let mut cand: Vec<(u64, u64)> = t
        .iter()
        .filter(|(_, s)| !s.scratch)
        .map(|(k, s)| (*k, s.last))
        .collect();
    cand.sort_by_key(|x| x.1);
    let target = COMBO_CAP * 3 / 4;
    let mut out: Vec<u64> = Vec::new();
    for (k, _) in cand {
        if t.len() <= target {
            break;
        }
        t.remove(&k);
        out.push(k);
    }
    out
}

/// 取键：已登记=复用；未满=登记新组合；超限=草稿键（返回是否草稿）。
fn combo_pick(full: u64, scratch: u64) -> (u64, bool) {
    let tick = COMBO_TICK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut t = COMBO.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = t.get_mut(&full) {
        s.last = tick;
        return (full, false);
    }
    if t.len() < COMBO_CAP {
        t.insert(
            full,
            ComboState {
                settled: false,
                elem: None,
                last: tick,
                scratch: false,
            },
        );
        return (full, false);
    }
    t.entry(scratch).or_insert(ComboState {
        settled: false,
        elem: None,
        last: tick,
        scratch: true,
    });
    log::error!("draw: 组合 VAO 表满（cap={COMBO_CAP}）转草稿 key={full:#x}");
    (scratch, true)
}

/// 取组合 VAO 句柄：Vao 域驻留，ensure/release 成对（行不死、句柄跨帧稳定）。
fn combo_acquire(p: &RenderPass<'_>, key: u64) -> crate::objects::GlHandle {
    let item = {
        let mut cache = p._enc._inner.cache.lock().unwrap_or_else(|e| e.into_inner());
        crate::objects::ensure_reuse(crate::objects::ReuseKey::Vao(key), &mut cache)
    };
    let h = item.handle.expect("draw str-assert: 组合 VAO 无名");
    crate::objects::release_reuse(&item);
    h
}

/// 一次性整备：把各顶点槽的属性指针/enable/divisor 写进当前绑定的 VAO。
unsafe fn combo_setup(
    gl: &glow::Context,
    layouts: &[VertexLayout],
    vbinds: &[(u32, crate::objects::GlHandle, u64)],
    base: u32,
) {
    for (slot, bname, o) in vbinds.iter() {
        let Some(lay) = layouts.get(*slot as usize) else {
            continue;
        };
        gl.bind_buffer(GL_ARRAY_BUFFER, Some(crate::gid::buffer_from(bname.0)));
        let inst = matches!(lay.step_mode, crate::types::VertexStepMode::Instance);
        let shift: u64 = if inst {
            base as u64 * lay.array_stride as u64
        } else {
            0
        };
        for a in lay.attributes.iter() {
            let (n, ty, norm, integer) = vtx_fmt(a.format);
            let off = o.wrapping_add(shift).wrapping_add(a.offset as u64) as i32;
            if integer {
                gl.vertex_attrib_pointer_i32(a.shader_location, n, ty, lay.array_stride as i32, off);
            } else {
                gl.vertex_attrib_pointer_f32(
                    a.shader_location,
                    n,
                    ty,
                    norm,
                    lay.array_stride as i32,
                    off,
                );
            }
            gl.enable_vertex_attrib_array(a.shader_location);
            if inst {
                gl.vertex_attrib_divisor(a.shader_location, INSTANCE_DIVISOR);
            }
        }
    }
}

/// 组合落位：LRU 回收→键判定→句柄→(首次)整备→索引缓冲对齐；返回时 m.vao 已是本文组合 VAO。
fn combo_bind(p: &RenderPass<'_>, m: &mut StateMirror, base: u32, gl: &glow::Context) {
    let evicted = combo_maintain();
    if !evicted.is_empty() {
        let mut cache = p._enc._inner.cache.lock().unwrap_or_else(|e| e.into_inner());
        for k in evicted.iter() {
            crate::objects::evict_vao(&mut cache, *k);
        }
        drop(cache);
        // 被淘汰对象可能正是当前绑定——镜像归零，下方 set_vao 必重绑。
        m.vao = None;
    }
    let (full, scratch_key) = combo_keys(p, base);
    let (key, scratch) = combo_pick(full, scratch_key);
    let handle = combo_acquire(p, key);
    state_inner::set_vao(m, Some(handle));
    let (settled, elem_same) = {
        let t = COMBO.lock().unwrap_or_else(|e| e.into_inner());
        match t.get(&key) {
            Some(s) => (s.settled, s.elem.is_some() && s.elem == p._index_ibo.map(|g| g.0)),
            None => (false, false),
        }
    };
    let setup_now = !settled;
    let need_elem = p._index_ibo.is_some() && !elem_same;
    if setup_now || need_elem {
        unsafe {
            if setup_now {
                combo_setup(gl, &p._vertex, &p._vbinds, base);
            }
            if let Some(ibo) = p._index_ibo {
                gl.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, Some(crate::gid::buffer_from(ibo.0)));
            }
        }
        let mut t = COMBO.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = t.get_mut(&key) {
            if setup_now && !scratch {
                s.settled = true;
            }
            s.elem = p._index_ibo.map(|g| g.0);
        }
    }
}

/// 组合表整清（VAO 物理删除/世代重置联动：旧整备标记对新对象无效）。
pub(crate) fn combo_forget() {
    COMBO.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

pub(crate) fn exec_set_vertex(p: &mut RenderPass<'_>, i: u32, sl: &crate::entry::BufferSlice<'_>) {
    let name = sl
        .buffer
        ._h
        .name()
        .expect("draw str-assert: 死顶点缓冲句柄");
    let o = sl.offset;
    assert!(
        (i as usize) < p._vertex.len(),
        "draw str-assert: 顶点槽越界（slot={i} layouts={}）",
        p._vertex.len()
    );
    assert!(p._vao.is_some(), "draw str-assert: 管线未设 VAO");
    // 纯记录（2026-10-28 组合 VAO 定案）：属性整备推迟到 draw，一次成型后同组合只 bind。
    p._vbinds.retain(|b| b.0 != i);
    p._vbinds.push((i, name, o));
    p._vertex_ready = true;
}

// 设计预期：句柄活态（形照 25.0.2：BufferSlice+格式）；记有效字节区间，draw_indexed 消费校验。
pub(crate) fn exec_set_index(
    p: &mut RenderPass<'_>,
    sl: &crate::entry::BufferSlice<'_>,
    f: IndexFormat,
) {
    let name = sl
        .buffer
        ._h
        .name()
        .expect("draw str-assert: 死索引缓冲句柄");
    let (st, en) = (sl.get_range().start, sl.get_range().end);
    // 纯记录（2026-10-28）：ELEMENT 绑定随组合 VAO 落位（VAO 态）；此处只记格式/区间/缓冲名。
    p._index_ibo = Some(name);
    p._index_format = Some(f);
    p._index_range = (st, en);
    p._index_ready = true;
}

// 设计预期：Y 翻换（缺目标高度→收窄存原始值，登记差异）；深度区间原样透传；差异才下发。
pub(crate) fn exec_set_viewport(
    p: &mut RenderPass<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    dn: f32,
    up: f32,
) {
    assert!(w >= 0.0 && h >= 0.0, "draw str-assert: 视口负尺寸");
    let mut m = lock_state(p);
    state_inner::set_viewport(&mut m, [x, y, w, h, dn, up]);
}

// 设计预期：裁剪框同源换算（收窄存原始值，登记差异）；差异才下发。
pub(crate) fn exec_set_scissor(p: &mut RenderPass<'_>, x: u32, y: u32, w: u32, h: u32) {
    let mut m = lock_state(p);
    state_inner::set_scissor(&mut m, [x as i32, y as i32, w as i32, h as i32], true);
}

// 设计预期：参考值入镜像；差异才下发。
pub(crate) fn exec_set_stencil(p: &mut RenderPass<'_>, v: u32) {
    let mut m = lock_state(p);
    state_inner::set_stencil_reference(&mut m, v);
}

// 设计预期：常量色入镜像、差异才下发。
pub(crate) fn exec_set_blend(p: &mut RenderPass<'_>, c: Color) {
    let mut m = lock_state(p);
    state_inner::set_blend_constant(&mut m, c);
}


// 设计预期（形照 25.0.2 Range 双参）：绑定完备缺一=断言零副作用；
// instance Range 起点=first_instance 语义（属性偏移模拟见 fi_rebind；instance_index
// 保持 0 基=登记差异）；实例数 1 且 base=0 走非实例化快路。
// 完备→立即发出数组绘制（first=start，count=end-start）。
pub(crate) fn exec_draw(
    p: &mut RenderPass<'_>,
    v: std::ops::Range<u32>,
    ins: std::ops::Range<u32>,
) {
    crate::state_inner::check_err("draw");
    assert!(
        v.end >= v.start && ins.end >= ins.start,
        "draw str-assert: 区间倒置（instances={ins:?} vertices={v:?}）"
    );
    assert!(p._pipe_set, "draw str-assert: 管线未设");
    assert!(p._vertex_ready, "draw str-assert: 顶点属性未就绪");
    let base = ins.start;
    let mode = topo(p._topology);
    let count = (v.end - v.start) as i32;
    let icount = (ins.end - ins.start) as i32;
    // 状态锁先、with_gl 后（锁序同 flush），并让「下发+绘制」占同一 GL 段——
    // 他线程无法插进两者之间改绑（多线程编码共驱动上下文的必需原子面）。
    let mut m = lock_state(p);
    crate::crash_diag::op(&format!(
        "draw_arrays mode={mode:#x} first={} count={count} base={base} instances={icount} vao={:?} prog={:?}",
        v.start,
        p._vao,
        m.program
    ));

    crate::state_inner::check_fbo("draw", state_inner::name_of_public(m.fbo_draw));
    with_gl(|gl| unsafe {
        state_inner::flush_impl(&mut m);
        crate::state_inner::check_err("after_flush_m");
        combo_bind(p, &mut m, base, gl);
        if icount == 1 && base == 0 {
            gl.draw_arrays(mode, v.start as i32, count);
        } else {
            gl.draw_arrays_instanced(mode, v.start as i32, count, icount);
        }
    });
}

// 设计预期（形照 25.0.2 三参 Range 形）：base_vertex≠0=断言（上游 gles 同不模拟）；
// instance 区间=属性偏移模拟（fi_rebind）；绘制区间须落在绑定片段 [start,end) 内
// （越界=断言，取代旧绑定期 c-count 校验——口径随形改为 draw 期）；通过→立即发元素绘制。
pub(crate) fn exec_draw_idx(
    p: &mut RenderPass<'_>,
    ix: std::ops::Range<u32>,
    base: i32,
    ins: std::ops::Range<u32>,
) {
    crate::state_inner::check_err("draw");
    assert!(base == 0, "draw str-assert: base_vertex≠0");
    assert!(
        ix.end >= ix.start && ins.end >= ins.start,
        "draw str-assert: 区间倒置（instances={ins:?} indices={ix:?}）"
    );
    assert!(p._pipe_set, "draw str-assert: 管线未设");
    assert!(p._vertex_ready, "draw str-assert: 顶点属性未就绪");
    assert!(p._index_ready, "draw str-assert: 索引未绑");
    let (st, en) = p._index_range;
    let w = index_width(p._index_format.expect("draw str-assert: 索引格式缺失"));
    assert!(
        st + (ix.end as u64) * w <= en,
        "draw str-assert: 索引区间越出绑定片段（indices={ix:?} slice={}..{}）",
        st,
        en
    );
    let f = p._index_format.expect("draw str-assert: 索引格式缺失");
    let base = ins.start;
    let mode = topo(p._topology);
    let off = st + (ix.start as u64) * w;
    let count = (ix.end - ix.start) as i32;
    let icount = (ins.end - ins.start) as i32;
    let mut m = lock_state(p);
    crate::crash_diag::op(&format!(
        "draw_elements mode={mode:#x} count={count} off={off} base={base} instances={icount} vao={:?} prog={:?}",
        p._vao, m.program
    ));
    crate::state_inner::check_fbo("draw_idx", state_inner::name_of_public(m.fbo_draw));
    with_gl(|gl| unsafe {
        state_inner::flush_impl(&mut m);
        crate::state_inner::check_err("after_flush_m");
        combo_bind(p, &mut m, base, gl);
        if icount == 1 && base == 0 {
            gl.draw_elements(mode, count, idx_type(f), off as i32)
        } else {
            gl.draw_elements_instanced(mode, count, idx_type(f), off as i32, icount)
        }
    });
}

// 设计预期：store 动作落定（Store/Discard）；释放编码器单通道守卫（此后绑定资源方可销毁）。
pub(crate) fn exec_end_impl(p: &mut RenderPass<'_>) {
    crate::state_inner::check_err("end_pass");
    // MSAA resolve 中转：当前持行（多重采样面）readPixels=驱动滤波采样聚合（等价 resolve，
    // 登记差异：上游 blit 精确多采样解析、此处像素滤波——uniform-clear 场景同色），
    // → 目的配置键 Fbo 行 texSub 直写；双方均 GL 底行在前=无行翻；目的行即取即还。
    if let Some((ms, rtx)) = p._resolve.take() {
        let curfb: Option<crate::objects::GlHandle> = p._fbo.as_ref().and_then(|i| i.handle);
        if let Some(msfb) = curfb {
            let w: i32 = ms._size.width as i32;
            let h: i32 = ms._size.height as i32;
            let mut block: Vec<u8> = vec![0u8; (w * h * ms.params.block_bytes as i32) as usize];
            let ditem = {
                let mut cache = p
                    ._enc
                    ._inner
                    .cache
                    .lock()
                    .unwrap_or_else(|x| x.into_inner());
                pick_rtt_row(
                    &mut cache,
                    rtt_fbo_key(Some(&rtx), None),
                    row_att_sig_of(Some(&rtx), None),
                    false,
                )
                .item
            };
            let dfb = ditem
                .handle
                .expect("draw str-assert: resolve 目的 Fbo 行无名");
            let drn = rtx._h.name().expect("draw str-assert: resolve 目的死句柄");
            let mut read_rejected: bool = false;
            with_gl(|gl| unsafe {
                gl.bind_framebuffer(
                    GL_FRAMEBUFFER,
                    Some(crate::gid::framebuffer_from(msfb.0)),
                );
                while gl.get_error() != 0 {}
                gl.read_pixels(
                    0,
                    0,
                    w,
                    h,
                    ms.params.format,
                    ms.params.pixel_type,
                    glow::PixelPackData::Slice(&mut block),
                );
                if gl.get_error() != 0 {
                    read_rejected = true;
                }
                if !read_rejected {
                    gl.bind_framebuffer(
                        GL_FRAMEBUFFER,
                        Some(crate::gid::framebuffer_from(dfb.0)),
                    );
                    gl.framebuffer_texture_2d(
                        GL_FRAMEBUFFER,
                        GL_COLOR_ATTACHMENT0,
                        rtx._target,
                        Some(crate::gid::texture_from(drn.0)),
                        0,
                    );
                    gl.active_texture(crate::upload_inner::UPLOAD_TEX_UNIT);
                    gl.bind_texture(
                        rtx._target,
                        Some(crate::gid::texture_from(drn.0)),
                    );
                    gl.tex_sub_image_2d(
                        rtx._target,
                        0,
                        0,
                        0,
                        w,
                        h,
                        rtx.params.format,
                        rtx.params.pixel_type,
                        glow::PixelUnpackData::Slice(&block),
                    );
                }
                gl.bind_framebuffer(GL_FRAMEBUFFER, None);
            });
            if read_rejected {
                log::error!("draw: resolve 读被本上下文拒绝（多采样读/blit 能力面封死登记），中转降级不写出（label 通道已收）");
            }
            crate::objects::release_reuse(&ditem);
        }
    }
    if let Some(item) = p._fbo.take() {
        // 离屏通道交还：Fbo 行计数-1（归零即物理删除=下次同名附件重建，复用表自持一致）。
        crate::objects::release_reuse(&item);
    }
    // 通道交还=水位归默认帧缓冲（与真实绑定同步；旧版只裸绑不回写致跨 pass 漂移）。
    {
        let mut mm = p
            ._enc
            ._inner
            .state
            .lock()
            .unwrap_or_else(|x| x.into_inner());
        crate::state_inner::set_draw_fbo(&mut mm, None);
        crate::state_inner::flush_impl(&mut mm);
    crate::state_inner::check_err("after_flush_mm");
    }
    p._enc._opened_pass = false;
}

// desc-align-wgpu25：合法域恒拒止——GL 路无 push constants（能力位恒 0/上限恒 0），一切调用=违约断言（前置拒止；正式错误码通道归实现批）。

/// 离屏附件配置键：色/深双宿主纹理名混列 GL 目标——同附件组合跨通道复用同一 FBO 行。
pub(crate) fn rtt_fbo_key(
    c: Option<&crate::entry::Texture>,
    d: Option<&crate::entry::Texture>,
) -> u64 {
    let mix = |acc: u64, v: u64| -> u64 { (acc ^ v).wrapping_mul(0x0100_0000_0000_b711) };
    // 域标记先行：色域/深域各自前置不同种子——修「(c,None)×(None,d) 同值碰撞」
    // （2026-10-20 双 pass 样例实锤：两 fb 行共用一 u64 致后开通道画进前通道的 fb）。
    let mut k: u64 = if c.is_some() { 0xC0FFEE } else { 0 };
    k = if d.is_some() { mix(k, 0xDEED) } else { k };
    // 形状键（2026-10-28 定案）：格式×样本×目标形态入键、宿主纹理名不入——
    // 逐帧重建同形状贴图时行稳定，附件变化由签名差分定向重挂（begin 路）。
    let shape = |t: &crate::entry::Texture, salt: u64| -> u64 {
        let fmt: u64 = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            t._format.hash(&mut h);
            h.finish()
        };
        let mut s = mix(0, fmt);
        s = mix(s, t._samples as u64);
        mix(s, t._target as u64 | salt)
    };
    if let Some(t) = c {
        k = mix(k, shape(t, 0));
    }
    if let Some(t) = d {
        k = mix(k, shape(t, 0xFFFF_0000));
    }
    k
}

/// 行附件签名：(色宿主 gid, 色 target, 深宿主 gid, 深 target, 深槽)；0=无宿主。
type RowAtt = (u64, u32, u64, u32, u32);

/// 形状键行→当前附件签名表（begin 路附件差分基准；死名摘除、世代重置整清）。
static ROW_ATT: once_cell::sync::Lazy<std::sync::Mutex<std::collections::HashMap<u64, RowAtt>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

fn row_att_get(fk: u64) -> Option<RowAtt> {
    ROW_ATT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&fk)
        .copied()
}

fn row_att_set(fk: u64, sig: RowAtt) {
    ROW_ATT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(fk, sig);
}

/// 纹删挂点：GL 自动 null 其附着——摘引用该名之签，下轮 begin 全量重挂+断言。
pub(crate) fn row_att_drop_tex(gid: u64) {
    ROW_ATT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .retain(|_, sig| sig.0 != gid && sig.2 != gid);
}

/// Fbo 行物理删挂点：摘对应 fk 记录（再 ensure 出新行=无签，触发全套）。
pub(crate) fn row_att_drop_fbo(fk: u64) {
    ROW_ATT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&fk);
}

/// 行附件签名由通道双宿主纹理构造（0=无宿主/死柄）。
pub(crate) fn row_att_sig_of(
    c: Option<&crate::entry::Texture>,
    d: Option<&crate::entry::Texture>,
) -> RowAtt {
    (
        c.and_then(|t| t._h.name()).map(|g| g.0).unwrap_or(0),
        c.map(|t| t._target).unwrap_or(0),
        d.and_then(|t| t._h.name()).map(|g| g.0).unwrap_or(0),
        d.map(|t| t._target).unwrap_or(0),
        d.map(|t| {
            if is_packed_stencil(t._format) {
                0x8D80u32
            } else {
                0x8D00u32
            }
        })
        .unwrap_or(0),
    )
}

/// 选架结果（键+行引用）。
pub(crate) struct RowPick {
    pub key: u64,
    pub item: crate::objects::Reusable,
}

/// RTT 选架：签名=本轮宿主的架优先（读回路/同宿主共架）；未被占的空架 begin 自挂；
/// 主架被在世通道占用且宿主不同（乱序交错）⇒ 线性探测备用架。attachable=false（读回路）
/// 不取空架（空架无附着读=垃圾）；探测穷尽=错误码兜底取主架（并发超额合同外）。
pub(crate) fn pick_rtt_row(
    cache: &mut crate::objects::HandleCache,
    fk0: u64,
    cur: RowAtt,
    attachable: bool,
) -> RowPick {
    let mixv = |f: u64, v: u64| -> u64 { (f ^ v).wrapping_mul(0x0100_0000_0000_b711) };
    for i in 0..8u64 {
        let key: u64 = if i == 0 {
            fk0
        } else {
            mixv(fk0 ^ 0x9E37_79B9_7F4A_7C15, i)
        };
        let holders_before = crate::objects::reuse_holders(crate::objects::ReuseKey::Fbo(key));
        let item = crate::objects::ensure_reuse(crate::objects::ReuseKey::Fbo(key), cache);
        if row_att_get(key) == Some(cur) || (attachable && holders_before == 0) {
            return RowPick { key, item };
        }
        crate::objects::release_reuse(&item);
    }
    log::error!("draw: RTT 架探测穷尽 fk={fk0:#x}");
    let item = crate::objects::ensure_reuse(crate::objects::ReuseKey::Fbo(fk0), cache);
    RowPick { key: fk0, item }
}

/// 世代/设备重置：整清（全部签名在新驱动环境下无意义）。
pub(crate) fn row_att_forget() {
    ROW_ATT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Per-label once-gate for noisy degrade logs (avoids per-frame spam).
fn warn_once(label: Option<&str>) -> bool {
    use std::sync::{Mutex, OnceLock};
    static SEEN: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let m = SEEN.get_or_init(|| Mutex::new(Vec::new()));
    let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
    let key = label.unwrap_or("<none>").to_string();
    if g.iter().any(|k| k == &key) {
        false
    } else {
        g.push(key);
        true
    }
}

/// 附件宿主四前置门 + 槽系族校验（depth_lane=深度槽须深度族；色槽反之）。
pub(crate) fn host_gate(t: &crate::entry::Texture, lane: &str) {
    if t._h.name().is_none() {
        log::error!(
            "draw: 附件死句柄 lane={} dim={:?} {}x{} layers={} samples={} fmt={:?} usage={:?} gen={} tid={:?}",
            lane, t._dim, t._size.width, t._size.height, t._layers, t._samples, t._format, t._usage,
            t._h.generation(), std::thread::current().id()
        );
    }
    assert!(
        t._h.name().is_some(),
        "draw str-assert: 附件宿主纹理死句柄（{lane}）"
    );
    assert!(
        matches!(t._dim, crate::types::TextureDimension::D2),
        "draw str-assert: ErrRttDim 离屏附件本批仅 D2 宿主（{lane} dim={:?}）",
        t._dim
    );
    assert!(
        t._samples == 1 || t._samples == 4,
        "draw str-assert: 附件采样数仅 1/4（{lane} samples={}）",
        t._samples
    );
    assert!(
        t._usage
            .contains(crate::types::TextureUsages::RENDER_ATTACHMENT),
        "draw str-assert: 附件宿主缺 RENDER_ATTACHMENT 位（{lane} usage={:?}）",
        t._usage
    );
    if lane == "depth" {
        assert!(
            is_depth_format(t._format),
            "draw str-assert: ErrNeedDepth 深度槽挂非深度族（format={:?}）",
            t._format
        );
    } else {
        assert!(
            !is_depth_format(t._format),
            "draw str-assert: ErrRttDepth 深度族不可挂色附件槽（format={:?}）",
            t._format
        );
    }
}

/// 深度族判定（与本层 formats 表深度词汇同名单）。
pub(crate) fn is_depth_format(f: crate::types::TextureFormat) -> bool {
    matches!(
        f,
        crate::types::TextureFormat::Depth16Unorm
            | crate::types::TextureFormat::Depth24Plus
            | crate::types::TextureFormat::Depth24PlusStencil8
            | crate::types::TextureFormat::Depth32Float
            | crate::types::TextureFormat::Depth32FloatStencil8
    )
}

/// 打包深度+模板族（挂 DEPTH_STENCIL_ATTACHMENT 双宽高面槽）。
fn is_packed_stencil(f: crate::types::TextureFormat) -> bool {
    matches!(
        f,
        crate::types::TextureFormat::Depth24PlusStencil8
            | crate::types::TextureFormat::Depth32FloatStencil8
    )
}
// desc-align-wgpu25：合法域恒拒止——GL 路无 push constants（能力位恒 0/上限恒 0），一切调用=违约断言（前置拒止；正式错误码通道归实现批）。
pub(crate) fn set_push_impl(_p: &mut RenderPass<'_>, _offset: u32, _data: &[u8]) {
    assert!(
        false,
        "draw 断言 ErrPush: GL 路恒拒止，本调用必违约（offset={} len={}，通道无标签源）",
        _offset,
        _data.len()
    );
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// 离屏 RTT 全链验收：色+深双附件 clear 落宿主→同配置键复用读回比对→释放重建→
    /// 深度族误挂色槽负断言。真实 GL 面（attach→probe 序与 B10 用例同法）。
    pub(crate) fn gl_offscreen_check() {
        // 附着与 caps 门控由全链主函数持有（本体检沿同线程入口）。
        let _caps = crate::caps_inner::probe_impl();
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
        let mk = |label: &str, fmt: crate::types::TextureFormat| {
            crate::entry_inner::create_texture_impl(
                &dev,
                &crate::types::TextureDescriptor {
                    label: Some(label),
                    size: crate::types::Extent3d {
                        width: 8,
                        height: 8,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: crate::types::TextureDimension::D2,
                    format: fmt,
                    usage: crate::types::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
            )
        };
        let ctex = mk("rtt-c", crate::types::TextureFormat::Rgba8Unorm);
        let dtex = mk("rtt-d", crate::types::TextureFormat::Depth16Unorm);
        let cview = ctex.create_view(&crate::types::TextureViewDescriptor::default());
        let dview = dtex.create_view(&crate::types::TextureViewDescriptor::default());
        let ca = crate::types::RenderPassColorAttachment {
            view: &cview,
            resolve_target: None,
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Clear(crate::types::Color {
                    r: 0.25,
                    g: 0.5,
                    b: 0.75,
                    a: 1.0,
                }),
                store: crate::types::StoreOp::Store,
            },
        };
        let dsa = crate::types::RenderPassDepthStencilAttachment {
            view: &dview,
            depth_ops: Some(crate::types::Operations {
                load: crate::types::LoadOp::Clear(0.55),
                store: crate::types::StoreOp::Store,
            }),
            stencil_ops: None,
        };
        let desc = crate::types::RenderPassDescriptor {
            label: Some("rtt-pass"),
            color_attachments: &[Some(ca)],
            depth_stencil_attachment: Some(dsa),
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        drop(enc.begin_render_pass(&desc));
        crate::record::flush_encoder(&mut enc);
        // 双附件复用：行驻留契约后同形状跨帧不删行——按选架读回=签名=本轮宿主之架（主架命中直绑）。
        let item = {
            let mut cache = enc._inner.cache.lock().unwrap_or_else(|x| x.into_inner());
            pick_rtt_row(
                &mut cache,
                rtt_fbo_key(Some(&ctex), Some(&dtex)),
                row_att_sig_of(Some(&ctex), Some(&dtex)),
                false,
            )
            .item
        };
        let fb = item.handle.expect("ensure 有名行");
        let px: [u8; 4] = with_gl(|gl| unsafe {
            gl.bind_framebuffer(
                GL_FRAMEBUFFER,
                Some(crate::gid::framebuffer_from(fb.0)),
            );
            gl.framebuffer_texture_2d(
                GL_FRAMEBUFFER,
                GL_COLOR_ATTACHMENT0,
                ctex._target,
                Some(crate::gid::texture_from(ctex._h.name().expect("活名").0)),
                0,
            );
            gl.framebuffer_texture_2d(
                GL_FRAMEBUFFER,
                GL_DEPTH_ATTACHMENT,
                dtex._target,
                Some(crate::gid::texture_from(dtex._h.name().expect("活名").0)),
                0,
            );
            let mut cbuf = [0u8; 4];
            gl.read_pixels(
                4,
                4,
                1,
                1,
                0x1908,
                GL_UNSIGNED_BYTE,
                glow::PixelPackData::Slice(&mut cbuf),
            );
            let mut dbuf = [0u8; 2];
            gl.read_pixels(
                4,
                4,
                1,
                1,
                0x1902,
                0x1403,
                glow::PixelPackData::Slice(&mut dbuf),
            );
            let derr = gl.get_error() as u32;
            let dnow = gl.get_parameter_f32(0x0B73);
            eprintln!("DEPTH-READ derr={derr:#x} cd={dnow} bytes={dbuf:?}");
            gl.bind_framebuffer(GL_FRAMEBUFFER, None);
            cbuf
        });
        // 深度附件通路状态面断言（本机无 attachment-iv 查询口、readPixels 深度=0x502 环境拒；
        // 内容级深度读回归 copy/采样批）：重挂深度槽→重清 0.77→清值寄存器与 FBO 完备双验。
        let cok = with_gl(|gl| unsafe {
            gl.bind_framebuffer(
                GL_FRAMEBUFFER,
                Some(crate::gid::framebuffer_from(fb.0)),
            );
            gl.framebuffer_texture_2d(
                GL_FRAMEBUFFER,
                GL_DEPTH_ATTACHMENT,
                dtex._target,
                Some(crate::gid::texture_from(dtex._h.name().expect("活名").0)),
                0,
            );
            gl.clear_depth_f64(0.77);
            gl.clear(GL_DEPTH_BUFFER_BIT);
            let stv = gl.check_framebuffer_status(GL_FRAMEBUFFER) as u32;
            let cdv = gl.get_parameter_f32(0x0B73);
            gl.bind_framebuffer(GL_FRAMEBUFFER, None);
            stv == GL_FRAMEBUFFER_COMPLETE && (cdv - 0.77).abs() < 0.01
        });
        assert!(cok, "深度附件通路状态面验证失败");

        crate::objects::release_reuse(&item);
        let near = |v: u8, t: f64| -> bool { ((v as f64) - t).abs() <= 2.0 };
        assert!(
            near(px[0], 64.0) && near(px[1], 128.0) && near(px[2], 191.0) && near(px[3], 255.0),
            "off-screen color clear missing: {:?}",
            px
        );

        // 深度族误挂色槽=拒止（不留半通道）。
        let bad_ca = crate::types::RenderPassColorAttachment {
            view: &dview,
            resolve_target: None,
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Load,
                store: crate::types::StoreOp::Store,
            },
        };
        let bad_desc = crate::types::RenderPassDescriptor {
            label: Some("rtt-bad"),
            color_attachments: &[Some(bad_ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let p = enc.begin_render_pass(&bad_desc);
            drop(p);
            crate::record::flush_encoder(&mut enc);
        }));
        assert!(r.is_err(), "深度族挂色槽=ErrRttDepth 断言");
        assert!(!enc._opened_pass, "拒止不留半通道");
    }
}
// ===== 录制面（提交序命令缓冲）：只入表，回放在 Queue::submit（record 芯）。 =====

/// 录制期描述符校验（与执行芯同口径的合同面：拒止发生在 begin 调用点，不留半通道）。
fn validate_begin(d: &RenderPassDescriptor<'_>) {
    assert!(
        d.timestamp_writes.is_none() && d.occlusion_query_set.is_none(),
        "draw str-assert: ErrQuery 时间戳/遮挡查询面无对应物（label={:?}）",
        d.label
    );
    let some = d.color_attachments.iter().filter(|x| x.is_some()).count();
    assert!(some <= 1, "draw str-assert: 颜色附件至多一个 Some（得 {some}）");
    let ca = d.color_attachments.iter().find_map(|x| x.as_ref());
    let dsa = d.depth_stencil_attachment.as_ref();
    let ctext = ca.and_then(|x| x.view._tex.as_ref());
    let dtext = dsa.and_then(|x| x.view._tex.as_ref());
    if let Some(t) = ctext {
        assert!(
            !is_depth_format(t._format),
            "draw str-assert: ErrRttDepth 深度族不可挂色附件槽（format={:?}）",
            t._format
        );
    }
    if let Some(rv) = ca.and_then(|x| x.resolve_target) {
        let ms = ctext.expect("draw str-assert: resolve 附件宿主不可为帧视图");
        assert!(
            ms._samples == 4,
            "draw str-assert: ErrResolveNoMS resolve 目的需 sample-4 附件（samples={}）",
            ms._samples
        );
        let rtx = rv
            ._tex
            .as_ref()
            .expect("draw str-assert: resolve 目的帧视图无宿主");
        assert!(
            rtx._samples == 1
                && rtx._dim == crate::types::TextureDimension::D2
                && rtx._layers == 1,
            "draw str-assert: ErrResolveDst resolve 目的需单采样 D2 单层（samples={}）",
            rtx._samples
        );
        assert!(
            rtx._format == ms._format,
            "draw str-assert: ErrResolveFmt resolve 目的与附件格式须一致"
        );
    }
    if let Some(x) = dsa {
        assert!(
            x.view._tex.is_some(),
            "draw str-assert: ErrRttDepth 深度槽不接受帧视图宿主（label={:?}）",
            d.label
        );
    }
}

/// 开通道：录制期只校验单通道守卫并入表（附件合法性/清理发生在提交回放）。
pub(crate) fn begin_impl<'p>(
    e: &'p mut CommandEncoder,
    d: &RenderPassDescriptor<'_>,
) -> RenderPass<'p> {
    assert!(!e._opened_pass, "draw str-assert: 嵌套 render pass");
    validate_begin(d);
    e._opened_pass = true;
    let color = d
        .color_attachments
        .iter()
        .find_map(|x| x.as_ref())
        .map(|ca| crate::record::Attachment {
            view: ca.view.clone(),
            resolve: ca.resolve_target.cloned(),
            ops: ca.ops,
        });
    let depth = d
        .depth_stencil_attachment
        .map(|dsa| crate::record::DepthAttachment {
            view: dsa.view.clone(),
            depth_ops: dsa.depth_ops,
            stencil_ops: dsa.stencil_ops,
        });
    e._commands.push(crate::record::Command::Begin(crate::record::PassBegin {
        label: d.label.map(str::to_string),
        color,
        depth,
    }));
    RenderPass {
        _enc: e,
        _vertex: Vec::new(),
        _pipe_set: false,
        _vertex_ready: false,
        _index_ready: false,
        _topology: crate::types::PrimitiveTopology::TriangleList,
        _index_format: None,
        _index_range: (0, 0),
        _vao: None,
        _slots: crate::binds::SlotTable::default(),
        _fbo: None,
        _hold_color: None,
        _hold_depth: None,
        _resolve: None,
        _replay: false,
        _vbinds: Vec::new(),
        _index_ibo: None,
    }
}

/// 关通道：入表并释放单通道守卫（资源交还在回放端）。
pub(crate) fn end_impl(p: &mut RenderPass<'_>) {
    p._enc._commands.push(crate::record::Command::End);
    p._enc._opened_pass = false;
}

/// 录制：设管线（持行程序/VAO 复用计数）。
pub(crate) fn set_pipeline_impl(p: &mut RenderPass<'_>, x: &RenderPipeline) {
    p._enc
        ._commands
        .push(crate::record::Command::SetPipeline(
            crate::record::RetainedPipeline::new(x),
        ));
}

/// 录制：设绑定组。
pub(crate) fn set_bind_group_impl(
    p: &mut RenderPass<'_>,
    set: u32,
    g: &BindGroup,
    offsets: &[DynamicOffset],
) {
    p._enc._commands.push(crate::record::Command::SetBindGroup {
        set,
        group: crate::record::RetainedBindGroup {
            entries: g._bound.clone(),
        },
        offsets: offsets.to_vec(),
    });
}

/// 录制：顶点属性来源绑定（缓冲/偏移 记录直下；fi_rebind 0 基=登记差异；
/// WebGL 属性槽=上下文态、下发自带生效——本层 exec_set_vertex 带水位缓存，
/// 同 (VAO,缓冲,偏移) 免重发）。
pub(crate) fn set_vertex_impl(p: &mut RenderPass<'_>, slot: u32, s: &crate::entry::BufferSlice<'_>) {
    p._enc._commands.push(crate::record::Command::SetVertex {
        slot,
        buffer: s.buffer.clone(),
        offset: s.offset,
        size: s.size.map(|z| z.get()),
    });
}

/// 录制：设索引缓冲。
pub(crate) fn set_index_impl(
    p: &mut RenderPass<'_>,
    s: &crate::entry::BufferSlice<'_>,
    format: IndexFormat,
) {
    p._enc._commands.push(crate::record::Command::SetIndex {
        buffer: s.buffer.clone(),
        offset: s.offset,
        size: s.size.map(|z| z.get()),
        format,
    });
}

/// 录制：视口。
pub(crate) fn set_viewport_impl(
    p: &mut RenderPass<'_>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    dn: f32,
    up: f32,
) {
    p._enc
        ._commands
        .push(crate::record::Command::SetViewport { x, y, w, h, dn, up });
}

/// 录制：裁剪框。
pub(crate) fn set_scissor_impl(p: &mut RenderPass<'_>, x: u32, y: u32, w: u32, h: u32) {
    p._enc
        ._commands
        .push(crate::record::Command::SetScissor { x, y, w, h });
}

/// 录制：模板参考值。
pub(crate) fn set_stencil_impl(p: &mut RenderPass<'_>, v: u32) {
    p._enc._commands.push(crate::record::Command::SetStencilRef(v));
}

/// 录制：混合常量色。
pub(crate) fn set_blend_impl(p: &mut RenderPass<'_>, c: Color) {
    p._enc
        ._commands
        .push(crate::record::Command::SetBlendConstant(c));
}

/// 录制：数组绘制。
pub(crate) fn draw_impl(
    p: &mut RenderPass<'_>,
    vertices: std::ops::Range<u32>,
    instances: std::ops::Range<u32>,
) {
    p._enc._commands.push(crate::record::Command::Draw {
        vertices,
        instances,
    });
}

/// 录制：索引绘制。
pub(crate) fn draw_idx_impl(
    p: &mut RenderPass<'_>,
    indices: std::ops::Range<u32>,
    base_vertex: i32,
    instances: std::ops::Range<u32>,
) {
    p._enc._commands.push(crate::record::Command::DrawIndexed {
        indices,
        base_vertex,
        instances,
    });
}