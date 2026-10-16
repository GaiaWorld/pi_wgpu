// 设计：pi-wgpu-rebuild
//! draw 芯 · 渲染通道录制（begin/set/draw；Drop=end）。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：合同违反（嵌套通道/死句柄/参数级）保留断言（短码）；断言前无副作用。
use std::num::NonZeroU32;

use glow::HasContext;

use crate::context_inner::with_gl;
use crate::draw::{RenderPass, VertexLayout};
use crate::entry::{BindGroup, Buffer, CommandEncoder, RenderPipeline};
use crate::state::StateMirror;
use crate::state_inner;
use crate::types::{Color, DynamicOffset, IndexFormat, RenderPassDescriptor};

// GL 常量（glow 根面不导出 GL_*；规范值内联）。
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const GL_FLOAT: u32 = 0x1406;
const GL_UNSIGNED_BYTE: u32 = 0x1401;
const GL_SHORT: u32 = 0x1402;
const GL_UNSIGNED_SHORT: u32 = 0x1403;
const GL_UNSIGNED_INT: u32 = 0x1405;
/// 实例步进除数（Instance 模式）。
const INSTANCE_DIVISOR: u32 = 1;

/// 顶点格式→(分量数, GL 类型, 归一化)——上游全 45 值 GL 核心号全表（WebGL2 ES3 合法子集；
/// Float64 系/双源类为完备性收录——管线创建域外白名单先拒，本表实际只达六型，余路不可达）。
const GL_BYTE: u32 = 0x1400;
const GL_HALF_FLOAT: u32 = 0x140B;
const GL_DOUBLE: u32 = 0x140E;
const GL_UNSIGNED_INT_2_10_10_10_REV: u32 = 0x8368;
const GL_UNSIGNED_INT_8_8_8_8_REV: u32 = 0x8367;
fn vtx_fmt(f: crate::types::VertexFormat) -> (i32, u32, bool) {
    use crate::types::VertexFormat::*;
    match f {
        Uint8 => (1, GL_UNSIGNED_BYTE, false),
        Uint8x2 => (2, GL_UNSIGNED_BYTE, false),
        Uint8x4 => (4, GL_UNSIGNED_BYTE, false),
        Sint8 => (1, GL_BYTE, false),
        Sint8x2 => (2, GL_BYTE, false),
        Sint8x4 => (4, GL_BYTE, false),
        Unorm8 => (1, GL_UNSIGNED_BYTE, true),
        Unorm8x2 => (2, GL_UNSIGNED_BYTE, true),
        Unorm8x4 => (4, GL_UNSIGNED_BYTE, true),
        Snorm8 => (1, GL_BYTE, true),
        Snorm8x2 => (2, GL_BYTE, true),
        Snorm8x4 => (4, GL_BYTE, true),
        Uint16 => (1, GL_UNSIGNED_SHORT, false),
        Uint16x2 => (2, GL_UNSIGNED_SHORT, false),
        Uint16x4 => (4, GL_UNSIGNED_SHORT, false),
        Sint16 => (1, GL_SHORT, false),
        Sint16x2 => (2, GL_SHORT, false),
        Sint16x4 => (4, GL_SHORT, false),
        Unorm16 => (1, GL_UNSIGNED_SHORT, true),
        Unorm16x2 => (2, GL_UNSIGNED_SHORT, true),
        Unorm16x4 => (4, GL_UNSIGNED_SHORT, true),
        Snorm16 => (1, GL_SHORT, true),
        Snorm16x2 => (2, GL_SHORT, true),
        Snorm16x4 => (4, GL_SHORT, true),
        Float16 => (1, GL_HALF_FLOAT, false),
        Float16x2 => (2, GL_HALF_FLOAT, false),
        Float16x4 => (4, GL_HALF_FLOAT, false),
        Float32 => (1, GL_FLOAT, false),
        Float32x2 => (2, GL_FLOAT, false),
        Float32x3 => (3, GL_FLOAT, false),
        Float32x4 => (4, GL_FLOAT, false),
        Uint32 => (1, GL_UNSIGNED_INT, false),
        Uint32x2 => (2, GL_UNSIGNED_INT, false),
        Uint32x3 => (3, GL_UNSIGNED_INT, false),
        Uint32x4 => (4, GL_UNSIGNED_INT, false),
        Sint32 => (1, GL_INT, false),
        Sint32x2 => (2, GL_INT, false),
        Sint32x3 => (3, GL_INT, false),
        Sint32x4 => (4, GL_INT, false),
        Float64 => (1, GL_DOUBLE, false),
        Float64x2 => (2, GL_DOUBLE, false),
        Float64x3 => (3, GL_DOUBLE, false),
        Float64x4 => (4, GL_DOUBLE, false),
        Unorm10_10_10_2 => (4, GL_UNSIGNED_INT_2_10_10_10_REV, true),
        Unorm8x4Bgra => (4, GL_UNSIGNED_INT_8_8_8_8_REV, true),
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
fn nz(n: u32) -> NonZeroU32 {
    NonZeroU32::new(n).expect("draw str-assert: 零句柄名")
}

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
// 3. 目标：帧视图=默认帧缓冲 0；离屏视图=登记差异（本批暂停，落错误日志后走 0）。load=Clear 即时清。
// 4. 守卫置位，返回借用编码器的 RenderPass。
pub(crate) fn begin_impl<'p>(
    e: &'p mut CommandEncoder,
    d: &RenderPassDescriptor<'_>,
) -> RenderPass<'p> {
    assert!(!e._opened_pass, "draw str-assert: 嵌套 render pass");
    assert!(
        d.timestamp_writes.is_none() && d.occlusion_query_set.is_none(),
        "draw str-assert: ErrQuery 时间戳/遮挡查询面无对应物（label={:?}）",
        d.label
    );
    let some = d.color_attachments.iter().filter(|x| x.is_some()).count();
    assert!(
        some == 1,
        "draw str-assert: 颜色附件须恰一 Some（得 {some}）"
    );
    let ca = d
        .color_attachments
        .iter()
        .find_map(|x| x.as_ref())
        .expect("恰一 Some（前置断言保证）");
    let is_frame = ca.view._tex.is_none();
    if !is_frame {
        log::error!("draw: 离屏附件 FBO 路径待后续批（登记差异；本批暂停，走默认帧缓冲）");
    }
    if let crate::types::LoadOp::Clear(c) = ca.ops.load {
        with_gl(|gl| unsafe {
            gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
            gl.clear_color(c.r as f32, c.g as f32, c.b as f32, c.a as f32);
            gl.clear(GL_COLOR_BUFFER_BIT);
        });
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
    }
}

// 设计预期：死程序句柄=断言；管线字段改记，程序切换差异下发。
pub(crate) fn set_pipeline_impl(p: &mut RenderPass<'_>, x: &RenderPipeline) {
    let name = x._program.handle.expect("draw str-assert: 死程序句柄");
    {
        let mut m = lock_state(p);
        state_inner::set_program(&mut m, name);
    }
    p._vertex = x._vertex.clone();
    p._topology = x._topology;
    p._vao = Some(x._vao.handle.expect("draw str-assert: 死 VAO 句柄"));
    p._pipe_set = true;
}

// 设计预期：set∈0-3 且偏移计数与声明一致；委托 binds.apply_layout。
pub(crate) fn set_bind_group_impl(
    p: &mut RenderPass<'_>,
    s: u32,
    g: &BindGroup,
    o: &[DynamicOffset],
) {
    assert!(s <= 3, "draw str-assert: set={s} 超 0-3");
    let mut m = lock_state(p);
    let mut offs = o;
    crate::binds::consume_dynamic_offsets(g, &mut offs);
    let slots = p
        ._enc
        ._inner
        .slots
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    crate::binds::apply_layout(s, g, o, &slots, &mut m);
}

// 设计预期：句柄活态（形照 25.0.2：入参=BufferSlice，步长由管线布局权威、不再收调用方值——
// 原「步长与布局不符」比对随形退役；片段边界在 buffer.slice() 构造期已验（entry_inner slice_bounds））。
pub(crate) fn set_vertex_impl(p: &mut RenderPass<'_>, i: u32, sl: &crate::entry::BufferSlice<'_>) {
    let name = sl
        .buffer
        ._h
        .name()
        .expect("draw str-assert: 死顶点缓冲句柄");
    let o = sl.offset;
    let layout: VertexLayout = p
        ._vertex
        .get(i as usize)
        .expect("draw str-assert: 顶点槽越界")
        .clone();
    // VAO=管线级复用对象（由 set_pipeline 提供；本处只绑 + 重设属性，不建删）。
    let vao_name = p._vao.expect("draw str-assert: 管线未设 VAO");
    let mut m = lock_state(p);
    with_gl(|gl| unsafe {
        gl.bind_vertex_array(Some(glow::NativeVertexArray(nz(vao_name.0))));
        gl.bind_buffer(GL_ARRAY_BUFFER, Some(glow::NativeBuffer(nz(name.0))));
        for a in layout.attributes.iter() {
            let (n, ty, norm) = vtx_fmt(a.format);
            gl.vertex_attrib_pointer_f32(
                a.shader_location,
                n,
                ty,
                norm,
                layout.array_stride as i32,
                o as i32 + a.offset as i32,
            );
            gl.enable_vertex_attrib_array(a.shader_location);
        }
        if matches!(layout.step_mode, crate::types::VertexStepMode::Instance) {
            for a in layout.attributes.iter() {
                gl.vertex_attrib_divisor(a.shader_location, INSTANCE_DIVISOR);
            }
        }
    });
    state_inner::set_vao(&mut m, Some(vao_name));
    drop(m);
    p._vertex_ready = true;
}

// 设计预期：句柄活态（形照 25.0.2：BufferSlice+格式）；记有效字节区间，draw_indexed 消费校验。
pub(crate) fn set_index_impl(
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
    with_gl(|gl| unsafe {
        gl.bind_buffer(
            GL_ELEMENT_ARRAY_BUFFER,
            Some(glow::NativeBuffer(nz(name.0))),
        );
    });
    p._index_format = Some(f);
    p._index_range = (st, en);
    p._index_ready = true;
}

// 设计预期：Y 翻换（缺目标高度→收窄存原始值，登记差异）；深度区间原样透传；差异才下发。
pub(crate) fn set_viewport_impl(
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
pub(crate) fn set_scissor_impl(p: &mut RenderPass<'_>, x: u32, y: u32, w: u32, h: u32) {
    let mut m = lock_state(p);
    state_inner::set_scissor(&mut m, [x as i32, y as i32, w as i32, h as i32], true);
}

// 设计预期：参考值入镜像；差异才下发。
pub(crate) fn set_stencil_impl(p: &mut RenderPass<'_>, v: u32) {
    let mut m = lock_state(p);
    state_inner::set_stencil_reference(&mut m, v);
}

// 设计预期：常量色入镜像、差异才下发。
pub(crate) fn set_blend_impl(p: &mut RenderPass<'_>, c: Color) {
    let mut m = lock_state(p);
    state_inner::set_blend_constant(&mut m, c);
}

// 设计预期（形照 25.0.2 Range 双参）：绑定完备缺一=断言零副作用；instances 恒 0..1（实例化未接）；
// 完备→立即发出数组绘制（first=start，count=end-start）。
pub(crate) fn draw_impl(
    p: &mut RenderPass<'_>,
    v: std::ops::Range<u32>,
    ins: std::ops::Range<u32>,
) {
    assert!(
        ins == (0..1) && v.end >= v.start,
        "draw str-assert: ErrInstances/区间倒置（instances={ins:?} vertices={v:?}）"
    );
    assert!(p._pipe_set, "draw str-assert: 管线未设");
    assert!(p._vertex_ready, "draw str-assert: 顶点属性未就绪");
    {
        let mut m = lock_state(p);
        state_inner::flush_impl(&mut m);
    }
    let mode = topo(p._topology);
    with_gl(|gl| unsafe { gl.draw_arrays(mode, v.start as i32, (v.end - v.start) as i32) });
}

// 设计预期（形照 25.0.2 三参 Range 形）：base≠0=断言；instances 恒 0..1；绘制区间须落在
// 绑定片段 [start,end) 内（越界=断言，取代旧绑定期 c-count 校验——口径随形改为 draw 期）；
// 通过→立即发元素绘制。
pub(crate) fn draw_idx_impl(
    p: &mut RenderPass<'_>,
    ix: std::ops::Range<u32>,
    base: i32,
    ins: std::ops::Range<u32>,
) {
    assert!(base == 0, "draw str-assert: base_vertex≠0");
    assert!(
        ins == (0..1) && ix.end >= ix.start,
        "draw str-assert: ErrInstances/区间倒置（instances={ins:?} indices={ix:?}）"
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
    {
        let mut m = lock_state(p);
        state_inner::flush_impl(&mut m);
    }
    let f = p._index_format.expect("draw str-assert: 索引格式缺失");
    let mode = topo(p._topology);
    let off = st + (ix.start as u64) * w;
    with_gl(|gl| unsafe {
        gl.draw_elements(mode, (ix.end - ix.start) as i32, idx_type(f), off as i32)
    });
}

// 设计预期：store 动作落定（Store/Discard）；释放编码器单通道守卫（此后绑定资源方可销毁）。
pub(crate) fn end_impl(p: &mut RenderPass<'_>) {
    with_gl(|gl| unsafe { gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None) });
    p._enc._opened_pass = false;
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
