// 设计：pi-wgpu-rebuild
//! draw 芯 · 渲染通道录制（begin/set/draw；Drop=end）。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：合同违反（嵌套通道/死句柄/参数级）保留断言（短码）；断言前无副作用。
use std::num::NonZeroU32;

use pi_glow::HasContext;

use crate::context_inner::with_gl;
use crate::draw::{RenderPass, VertexLayout};
use crate::entry::{BindGroup, Buffer, CommandEncoder, RenderPipeline};
use crate::state::StateMirror;
use crate::state_inner;
use crate::types::{Color, DynamicOffset, IndexFormat, RenderPassDescriptor};

// GL 常量（pi_glow 根面不导出 GL_*；规范值内联）。
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

/// 顶点格式→(分量数, GL 类型, 归一化)。
fn vtx_fmt(f: crate::types::VertexFormat) -> (i32, u32, bool) {
    use crate::types::VertexFormat::*;
    match f {
        Float32 => (1, GL_FLOAT, false),
        Float32x2 => (2, GL_FLOAT, false),
        Float32x3 => (3, GL_FLOAT, false),
        Float32x4 => (4, GL_FLOAT, false),
        Uint8x4 => (4, GL_UNSIGNED_BYTE, false),
        Sint16x2 => (2, GL_SHORT, false),
    }
}

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

/// 顶点格式判别码（哈希用）。
fn fmt_code(f: crate::types::VertexFormat) -> u8 {
    use crate::types::VertexFormat::*;
    match f { Float32 => 0, Float32x2 => 1, Float32x3 => 2, Float32x4 => 3, Uint8x4 => 4, Sint16x2 => 5 }
}
/// 索引宽（字节）。
fn index_width(f: IndexFormat) -> u64 { match f { IndexFormat::Uint16 => 2, IndexFormat::Uint32 => 4 } }
/// 布局哈希（VAO 复用键）。
fn layout_hash(v: &[VertexLayout]) -> u64 {
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
fn nz(n: u32) -> NonZeroU32 { NonZeroU32::new(n).expect("draw str-assert: 零句柄名") }

/// 借设备状态镜像（锁序：state 先、with_gl 后）。
fn lock_state<'a>(p: &'a RenderPass<'_>) -> std::sync::MutexGuard<'a, StateMirror> {
    p._enc._inner.state.lock().unwrap_or_else(|e| e.into_inner())
}

// 设计预期（伪码·示意·非唯一实现）：begin=开通道。
// 1. 编码器守卫检查：已有开启中通道=断言，通道不建。
// 2. 颜色附件恰一 Some；视图活态。违约=断言。
// 3. 目标：帧视图=默认帧缓冲 0；离屏视图=登记差异（本批暂停，落错误日志后走 0）。load=Clear 即时清。
// 4. 守卫置位，返回借用编码器的 RenderPass。
pub(crate) fn begin_impl<'p>(e: &'p mut CommandEncoder, d: &RenderPassDescriptor<'p>) -> RenderPass<'p> {
    assert!(!e._opened_pass, "draw str-assert: 嵌套 render pass");
    let some = d.color_attachments.iter().filter(|x| x.is_some()).count();
    assert!(some == 1, "draw str-assert: 颜色附件须恰一 Some（得 {some}）");
    let ca = d.color_attachments.iter().find_map(|x| x.as_ref()).expect("恰一 Some（前置断言保证）");
    let is_frame = ca.view._tex.is_none();
    if !is_frame {
        log::error!("draw: 离屏附件 FBO 路径待后续批（登记差异；本批暂停，走默认帧缓冲）");
    }
    if let crate::types::LoadOp::Clear(c) = ca.ops.load {
        with_gl(|gl| unsafe {
            gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
            gl.clear_color(c.r, c.g, c.b, c.a);
            gl.clear(GL_COLOR_BUFFER_BIT);
        });
    }
    e._opened_pass = true;
    RenderPass { _enc: e, _vertex: Vec::new(), _pipe_set: false, _vertex_ready: false, _index_ready: false, _topology: crate::types::PrimitiveTopology::TriangleList, _index_format: None, _index_offset: 0, _vao: None }
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
    p._pipe_set = true;
}

// 设计预期：set∈0-3 且偏移计数与声明一致；委托 binds.apply_layout。
pub(crate) fn set_bind_group_impl(p: &mut RenderPass<'_>, s: u32, g: &BindGroup, o: &[DynamicOffset]) {
    assert!(s <= 3, "draw str-assert: set={s} 超 0-3");
    let mut m = lock_state(p);
    let mut offs = o;
    crate::binds::consume_dynamic_offsets(g, &mut offs);
    let slots = p._enc._inner.slots.lock().unwrap_or_else(|e| e.into_inner());
    crate::binds::apply_layout(s, g, o, &slots, &mut m);
}

// 设计预期：句柄活态+范围校验；槽 i 记录缓冲/偏移/步长 → 属性批配属性。
pub(crate) fn set_vertex_impl(p: &mut RenderPass<'_>, i: u32, b: &Buffer, o: u64, s: u64) {
    let name = b._h.handle.expect("draw str-assert: 死顶点缓冲句柄");
    let layout: VertexLayout = p._vertex.get(i as usize).expect("draw str-assert: 顶点槽越界").clone();
    assert!(s == layout.array_stride, "draw str-assert: 顶点步长与布局不符");
    // VAO 经 objects 复用通道（复用键=整管线布局哈希；通道内持有、end 释放）。
    let key = crate::objects::ReuseKey::Vao(layout_hash(&p._vertex));
    let va = {
        let mut cache = p._enc._inner.cache.lock().unwrap_or_else(|e| e.into_inner());
        crate::objects::ensure_reuse(key, &mut cache)
    };
    let vao_name = va.handle.expect("draw str-assert: VAO 无句柄");
    let mut m = lock_state(p);
    with_gl(|gl| unsafe {
        gl.bind_vertex_array(Some(pi_glow::NativeVertexArray(nz(vao_name.0))));
        gl.bind_buffer(GL_ARRAY_BUFFER, Some(pi_glow::NativeBuffer(nz(name.0))));
        for a in layout.attributes.iter() {
            let (n, ty, norm) = vtx_fmt(a.format);
            gl.vertex_attrib_pointer_f32(a.shader_location, n, ty, norm, layout.array_stride as i32, o as i32 + a.offset as i32);
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
    p._vao = Some(va);
    p._vertex_ready = true;
}

// 设计预期：句柄活态+偏移范围；镜像记索引绑定（格式仅记录）。
pub(crate) fn set_index_impl(p: &mut RenderPass<'_>, b: &Buffer, o: u64, f: IndexFormat, c: u64) {
    let name = b._h.handle.expect("draw str-assert: 死索引缓冲句柄");
    assert!(o + c * index_width(f) <= b.size || c == 0, "draw str-assert: 索引范围越界");
    with_gl(|gl| unsafe {
        gl.bind_buffer(GL_ELEMENT_ARRAY_BUFFER, Some(pi_glow::NativeBuffer(nz(name.0))));
    });
    p._index_format = Some(f);
    p._index_offset = o;
    p._index_ready = true;
}

// 设计预期：Y 翻换（缺目标高度→收窄存原始值，登记差异）；深度区间原样透传；差异才下发。
pub(crate) fn set_viewport_impl(p: &mut RenderPass<'_>, x: f32, y: f32, w: f32, h: f32, dn: f32, up: f32) {
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

// 设计预期：绑定完备检查（管线已设、属性就绪）缺一=断言零副作用；完备→立即发出数组绘制。
pub(crate) fn draw_impl(p: &mut RenderPass<'_>, s: u32, c: u32) {
    assert!(p._pipe_set, "draw str-assert: 管线未设");
    assert!(p._vertex_ready, "draw str-assert: 顶点属性未就绪");
    {
        let mut m = lock_state(p);
        state_inner::flush_impl(&mut m);
    }
    let mode = topo(p._topology);
    with_gl(|gl| unsafe { gl.draw_arrays(mode, s as i32, c as i32) });
}

// 设计预期：base≠0=断言；索引未绑/格式失配=断言；通过→立即发出元素绘制。
pub(crate) fn draw_idx_impl(p: &mut RenderPass<'_>, s: u32, c: u32, b: i32) {
    assert!(b == 0, "draw str-assert: base_vertex≠0");
    assert!(p._pipe_set, "draw str-assert: 管线未设");
    assert!(p._vertex_ready, "draw str-assert: 顶点属性未就绪");
    assert!(p._index_ready, "draw str-assert: 索引未绑");
    {
        let mut m = lock_state(p);
        state_inner::flush_impl(&mut m);
    }
    let f = p._index_format.expect("draw str-assert: 索引格式缺失");
    let mode = topo(p._topology);
    let off = p._index_offset + (s as u64) * index_width(f);
    with_gl(|gl| unsafe { gl.draw_elements(mode, c as i32, idx_type(f), off as i32) });
}

// 设计预期：store 动作落定（Store/Discard）；释放编码器单通道守卫（此后绑定资源方可销毁）。
pub(crate) fn end_impl(p: &mut RenderPass<'_>) {
    with_gl(|gl| unsafe { gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None) });
    if let Some(va) = p._vao.take() {
        crate::objects::release_reuse(&va);
    }
    p._enc._opened_pass = false;
}
