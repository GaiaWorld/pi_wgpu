// 设计：pi-wgpu-rebuild
//! state 芯 · 驱动状态镜像——两段式差异下发：
//! ① set_* 组 helper（B6/B8/B10 消费）：先比对镜像期望值，相同=零下发；差异=下发、成功后落镜像（不变量执行体）。
//! ② flush_if_changed（pass begin / reset_state 消费）：可回读组向驱动实值比对、差异才重下发；
//!    驱动侧不可回读组（WebGL 对象绑定查询返回 JS 对象、UBO 绑定点无标量回读）按「驱动未知」处理=幂等整组重发。
//! 死名驻留：objects 删除路径已删名登记；回读比对命中=视同差异强制重发，重下发后销记。

use std::sync::Mutex;

use once_cell::sync::Lazy;
use pi_hash::XHashSet;

use pi_glow::HasContext;

use crate::context_inner::with_gl;
use crate::objects::GlHandle;
use crate::state::{StateMirror, BIND_SLOTS};
use crate::types::{BlendFactor, BlendOperation, BlendState, Color, ColorWrites, CompareFunction, CullMode, DepthBiasState, Face, FrontFace, PolygonMode, StencilFaceState, StencilOperation};

/// GL_CURRENT_PROGRAM 回读参数号。
const GL_CURRENT_PROGRAM: u32 = 0x8B8D;
/// DRAW framebuffer 目标号与回读号（规范同源 0x8CA9）。
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
/// PIXEL_UNPACK_BUFFER 目标号。
const GL_PIXEL_UNPACK_BUFFER: u32 = 0x88EB;
/// UNIFORM_BUFFER 目标号。
const GL_UNIFORM_BUFFER: u32 = 0x8A11;
/// 纹理单元基号与 ACTIVE_TEXTURE 回读参数号。
const GL_TEXTURE0: u32 = 0x84C0;
const GL_ACTIVE_TEXTURE: u32 = 0x8480;
/// TEXTURE_2D 目标号。
const GL_TEXTURE_2D: u32 = 0x0DE1;
/// 深度开关/写掩码/比较参数号。
const GL_DEPTH_TEST: u32 = 0x0B71;
const GL_DEPTH_FUNC: u32 = 0x0B74;
/// 剔除开关与模式参数号。
const GL_CULL_FACE: u32 = 0x0B44;
const GL_CULL_FACE_MODE: u32 = 0x0B45;
/// 正面绕序参数号。
const GL_FRONT_FACE: u32 = 0x0B46;
/// 混合开关参数号。
const GL_BLEND: u32 = 0x0BE2;
/// 裁剪开关与框参数号。
const GL_SCISSOR_TEST: u32 = 0x0C11;
const GL_SCISSOR_BOX: u32 = 0x0C22;
/// 视口与深度区间参数号。
const GL_VIEWPORT: u32 = 0x8472;
const GL_DEPTH_RANGE: u32 = 0x0B70;
/// 多边形偏置开关参数号。
const GL_POLYGON_OFFSET_FILL: u32 = 0x8E30;
/// 覆盖采样开关参数号。
const GL_SAMPLE_ALPHA_TO_COVERAGE: u32 = 0x809E;
/// 模板朝向参数号。
const GL_FRONT: u32 = 0x0404;
const GL_BACK: u32 = 0x0405;
/// 双面朝向号（polygon_mode 落发用）。
const GL_FRONT_AND_BACK: u32 = 0x0408;

/// 测试构建下发观测计数器：0=useProgram 下发次数、1=flush 整组重发次数（产品路径不读）。
static TEST_PROBE: Lazy<Mutex<[u32; 2]>> = Lazy::new(|| Mutex::new([0; 2]));

/// 计数观测（test 构建生效；产品构建编译期为空）。
#[cfg(test)]
fn probe(slot: usize) {
    let mut p = TEST_PROBE.lock().unwrap_or_else(|e| e.into_inner());
    p[slot] = p[slot].wrapping_add(1);
}

#[cfg(not(test))]
#[inline(always)]
fn probe(_slot: usize) {}

/// 读观测计数（合验用例断言用）。
#[cfg(test)]
pub(crate) fn probe_count(slot: usize) -> u32 {
    TEST_PROBE.lock().unwrap_or_else(|e| e.into_inner())[slot]
}

/// —— 死名刷失驻留：「镜像不假设驱动仍持有已删对象」的机制层（objects 删除路径登记）——
/// 已删对象名登记表（域编号×名字防跨域串扰；域编号约定见 objects 芯 kind_num_* 函数注释）。
static REMOVED_NAMES: Lazy<Mutex<XHashSet<(u8, u32)>>> = Lazy::new(|| Mutex::new(XHashSet::default()));

/// 登记一个被物理删除的对象名（objects 芯 mark_dead/release 删除后调用）。
pub(crate) fn note_removed(domain: u8, name: u32) {
    REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner()).insert((domain, name));
}

/// 查询某域对象名是否已被删除（回读比对命中=该组按驱动未知处理，强制重发）。
pub(crate) fn is_removed(domain: u8, name: u32) -> bool {
    REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner()).contains(&(domain, name))
}

/// 重下发成功后销记（GL 名可被驱动回收再用，滞留登记会误判新对象为死）。
pub(crate) fn forget_removed(domain: u8, name: u32) {
    REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner()).remove(&(domain, name));
}

/// 整面清空（invalidate_generations 路径：旧世代名集在新附着下无意义）。
pub(crate) fn clear_removed() {
    REMOVED_NAMES.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// 镜像名读数（None=驱动侧 0 名=无绑定语义）。
fn name_of(o: Option<GlHandle>) -> u32 {
    match o {
        Some(g) => g.0,
        None => 0,
    }
}

/// 进入绑定面的句柄名必须非零（0 名=非法状态断言）。
fn nz(n: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(n).unwrap_or_else(|| panic!("state 断言: 非法零句柄名进入绑定面"))
}

// —— 枚举映射（词汇无 PartialEq：比较一律经映射原语；全表值对规范注释）——
/// 比较函数映射（NEVER=0x0200..ALWAYS=0x0207 标准序）。
fn cmp_to_gl(c: CompareFunction) -> u32 {
    match c {
        CompareFunction::Never => 0x0200, CompareFunction::Less => 0x0201, CompareFunction::Equal => 0x0202,
        CompareFunction::LessEqual => 0x0203, CompareFunction::Greater => 0x0204, CompareFunction::NotEqual => 0x0205,
        CompareFunction::GreaterEqual => 0x0206, CompareFunction::Always => 0x0207,
    }
}

/// 混合等式映射（ADD=0x8006/SUB=0x800A/REV=0x800B/MIN=0x8007/MAX=0x8008）。
fn blend_op_to_gl(op: BlendOperation) -> u32 {
    match op {
        BlendOperation::Add => 0x8006, BlendOperation::Subtract => 0x800A, BlendOperation::ReverseSubtract => 0x800B,
        BlendOperation::Min => 0x8007, BlendOperation::Max => 0x8008,
    }
}

/// 混合因子映射（子集六值官方号）。
fn blend_factor_to_gl(f: BlendFactor) -> u32 {
    match f {
        BlendFactor::Zero => 0, BlendFactor::One => 1, BlendFactor::SrcAlpha => 0x0302,
        BlendFactor::OneMinusSrcAlpha => 0x0303, BlendFactor::DstAlpha => 0x0304, BlendFactor::OneMinusDstAlpha => 0x0305,
    }
}

/// 正面绕序映射（GL_CW=0x0900/GL_CCW=0x0901）。
fn front_face_to_gl(f: FrontFace) -> u32 {
    match f { FrontFace::Ccw => 0x0901, FrontFace::Cw => 0x0900 }
}

/// 剔除模式映射（FRONT=0x0404/BACK=0x0405；None/Undef=无剔除号=开关关）。
fn cull_mode_to_gl(m: CullMode) -> Option<u32> {
    match m {
        Some(Face::Front) => Some(0x0404), Some(Face::Back) => Some(0x0405), Some(Face::Undef) | None => None,
    }
}

/// 填充模式映射（FILL=0x1B01/LINE=0x1B02/POINT=0x1B03；非 Fill 能力断言在 draw 侧）。
fn polygon_to_gl(m: PolygonMode) -> u32 {
    match m { PolygonMode::Fill => 0x1B01, PolygonMode::Line => 0x1B02, PolygonMode::Point => 0x1B03 }
}

/// 模板操作映射（KEEP=0x1E00/ZERO=0x1E01/REPLACE=0x1E02/INCR=0x1E03/DECR=0x1E04/INVERT=0x1E05/WRAP=0x8507/0x8508）。
fn stencil_op_to_gl(op: StencilOperation) -> u32 {
    match op {
        StencilOperation::Keep => 0x1E00, StencilOperation::Zero => 0x1E01, StencilOperation::Replace => 0x1E02,
        StencilOperation::IncrementClamp => 0x1E03, StencilOperation::DecrementClamp => 0x1E04, StencilOperation::Invert => 0x1E05,
        StencilOperation::IncrementWrap => 0x8507, StencilOperation::DecrementWrap => 0x8508,
    }
}

/// 深度偏置常量与斜率下发值对（驱动侧 POLYGON_OFFSET_FACTOR=斜率/UNITS=常数）。
fn bias_pair(b: DepthBiasState) -> (f32, f32) {
    (b.slope, b.constant)
}

// —— 相等比较器（全经映射/分量拆解；None=从未设 恒不等任何实值，首轮必下发）——
/// 混合态整体相等（两等式+四因子经原语拆量；外层哨兵 Some(None)=关混合与 None=未设 可区分）。
fn blend_same(cur: Option<Option<BlendState>>, next: Option<BlendState>) -> bool {
    match (cur, next) {
        (None, _) => false,
        (Some(None), None) => true,
        (Some(Some(a)), Some(b)) => {
            blend_op_to_gl(a.color.operation) == blend_op_to_gl(b.color.operation)
                && blend_op_to_gl(a.alpha.operation) == blend_op_to_gl(b.alpha.operation)
                && blend_factor_to_gl(a.color.color_factor) == blend_factor_to_gl(b.color.color_factor)
                && blend_factor_to_gl(a.color.alpha_factor) == blend_factor_to_gl(b.color.alpha_factor)
                && blend_factor_to_gl(a.alpha.color_factor) == blend_factor_to_gl(b.alpha.color_factor)
                && blend_factor_to_gl(a.alpha.alpha_factor) == blend_factor_to_gl(b.alpha.alpha_factor)
        }
        _ => false,
    }
}

/// 颜色相等（f32 分量直较）。
fn color_same(cur: Option<Color>, next: Color) -> bool {
    match cur {
        Some(c) => c.r == next.r && c.g == next.g && c.b == next.b && c.a == next.a,
        None => false,
    }
}

/// 深度比较函数相等（映射值比较）。
fn cmp_same(cur: Option<CompareFunction>, next: CompareFunction) -> bool {
    match cur {
        Some(c) => cmp_to_gl(c) == cmp_to_gl(next),
        None => false,
    }
}

/// 模板单面相等（比较函数+三操作拆原语）。
fn stencil_face_same(cur: Option<StencilFaceState>, next: StencilFaceState) -> bool {
    match cur {
        Some(a) => {
            cmp_to_gl(a.compare) == cmp_to_gl(next.compare)
                && stencil_op_to_gl(a.fail_op) == stencil_op_to_gl(next.fail_op)
                && stencil_op_to_gl(a.depth_fail_op) == stencil_op_to_gl(next.depth_fail_op)
                && stencil_op_to_gl(a.pass_op) == stencil_op_to_gl(next.pass_op)
        }
        None => false,
    }
}

/// 剔除模式相等（None/未设可区分）。
fn cull_same(cur: Option<CullMode>, next: CullMode) -> bool {
    match (cur, next) {
        (Some(a), b) => match (cull_mode_to_gl(a), cull_mode_to_gl(b)) {
            (Some(x), Some(y)) => x == y,
            (None, None) => true,
            _ => false,
        },
        (None, _) => false,
    }
}

/// 正面绕序相等。
fn front_same(cur: Option<FrontFace>, next: FrontFace) -> bool {
    match cur {
        Some(f) => front_face_to_gl(f) == front_face_to_gl(next),
        None => false,
    }
}

/// 填充模式相等。
fn polygon_same(cur: Option<PolygonMode>, next: PolygonMode) -> bool {
    match cur {
        Some(p) => polygon_to_gl(p) == polygon_to_gl(next),
        None => false,
    }
}

/// 深度偏置相等（三分量）。
fn bias_same(cur: Option<DepthBiasState>, next: DepthBiasState) -> bool {
    match cur {
        Some(b) => b.constant == next.constant && b.slope == next.slope && b.clamp == next.clamp,
        None => false,
    }
}

/// 写掩码相等（字节直较）。
fn writes_same(cur: Option<ColorWrites>, next: ColorWrites) -> bool {
    match cur {
        Some(w) => w.0 == next.0,
        None => false,
    }
}

// —— 组下发体（把镜像当前期望刷向驱动；set helper 与 flush 整发档共用）——
/// 程序组下发（PROBE(0)=useProgram 实际下发次数，供零下发验收观测）。
fn apply_program(m: &StateMirror) {
    with_gl(|gl| unsafe {
        probe(0);
        match m.program {
            Some(g) => gl.use_program(Some(pi_glow::NativeProgram(nz(g.0)))),
            None => gl.use_program(None),
        }
    });
}

/// 属性组下发。
fn apply_vao(m: &StateMirror) {
    with_gl(|gl| unsafe {
        match m.vao {
            Some(g) => gl.bind_vertex_array(Some(pi_glow::NativeVertexArray(nz(g.0)))),
            None => gl.bind_vertex_array(None),
        }
    });
}

/// 目标组下发。
fn apply_fbo(m: &StateMirror) {
    with_gl(|gl| unsafe {
        match m.fbo_draw {
            Some(g) => gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, Some(pi_glow::NativeFramebuffer(nz(g.0)))),
            None => gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None),
        }
    });
}

/// 上传组下发。
fn apply_unpack(m: &StateMirror) {
    with_gl(|gl| unsafe {
        match m.pixel_unpack_buffer {
            Some(g) => gl.bind_buffer(GL_PIXEL_UNPACK_BUFFER, Some(pi_glow::NativeBuffer(nz(g.0)))),
            None => gl.bind_buffer(GL_PIXEL_UNPACK_BUFFER, None),
        }
    });
}

/// 开/关统一下发体（调用者在 gl 门禁内）。
unsafe fn set_enable(gl: &pi_glow::Context, group: u32, on: bool) {
    if on {
        gl.enable(group);
    } else {
        gl.disable(group);
    }
}

// —— set_* 组 helper（消费端=B6/B8/B10；差异才下发）——
/// 程序组 set：同值零下发；差异→下发→落镜像。
pub(crate) fn set_program(m: &mut StateMirror, name: GlHandle) {
    if m.program == Some(name) {
        return;
    }
    m.program = Some(name);
    apply_program(m);
}

/// 属性组 set（None=解绑回无名状态）。
pub(crate) fn set_vao(m: &mut StateMirror, name: Option<GlHandle>) {
    if m.vao == name {
        return;
    }
    m.vao = name;
    apply_vao(m);
}

/// 目标组 set。
pub(crate) fn set_draw_fbo(m: &mut StateMirror, name: Option<GlHandle>) {
    if m.fbo_draw == name {
        return;
    }
    m.fbo_draw = name;
    apply_fbo(m);
}

/// 上传组 set。
pub(crate) fn set_pixel_unpack_buffer(m: &mut StateMirror, name: Option<GlHandle>) {
    if m.pixel_unpack_buffer == name {
        return;
    }
    m.pixel_unpack_buffer = name;
    apply_unpack(m);
}

/// uniform 槽 set：差异才重绑（键=名+偏移+尺寸）；无偏移无尺寸走 bindBufferBase，带偏移/尺寸走 bindBufferRange；
/// 解绑走 bindBufferRange(None) 低频路径。带偏移但未声明尺寸=断言（range 需合法 size）。
pub(crate) fn bind_ubo(m: &mut StateMirror, index: u32, name: Option<GlHandle>, offset: u32, size: u32) {
    let idx = index as usize;
    if idx >= BIND_SLOTS {
        panic!("state 断言: UBO 槽 {index} 超上限 {BIND_SLOTS}");
    }
    if name.is_some() && offset != 0 && size == 0 {
        panic!("state 断言: UBO 槽 {index} 带偏移 {offset} 但未声明尺寸（range 需合法 size）");
    }
    let key = name.map(|g| (g, offset, size));
    if m.ubo_bindings[idx] == key {
        return;
    }
    m.ubo_bindings[idx] = key;
    with_gl(|gl| unsafe {
        match (name, offset, size) {
            (Some(g), 0, 0) => gl.bind_buffer_base(GL_UNIFORM_BUFFER, index, Some(pi_glow::NativeBuffer(nz(g.0)))),
            (Some(g), off, sz) => gl.bind_buffer_range(GL_UNIFORM_BUFFER, index, Some(pi_glow::NativeBuffer(nz(g.0))), off as i32, sz as i32),
            (None, _, _) => gl.bind_buffer_range(GL_UNIFORM_BUFFER, index, None, 0, 0),
        };
    });
}

/// 纹理槽 set：必要时先切 ACTIVE_TEXTURE（单元号比对防冗余切换），再绑定并落槽表。
pub(crate) fn bind_texture(m: &mut StateMirror, unit: u32, target: u32, name: Option<GlHandle>) {
    let idx = unit as usize;
    if idx >= BIND_SLOTS {
        panic!("state 断言: 纹理单元 {unit} 超上限 {BIND_SLOTS}");
    }
    let want = name.map(|g| (g, target));
    if m.texture_bindings[idx] == want {
        return;
    }
    m.texture_bindings[idx] = want;
    with_gl(|gl| unsafe {
        if m.active_texture_unit != Some(unit) {
            gl.active_texture(GL_TEXTURE0 + unit);
            m.active_texture_unit = Some(unit);
        }
        match name {
            Some(g) => gl.bind_texture(target, Some(pi_glow::NativeTexture(nz(g.0)))),
            None => gl.bind_texture(target, None),
        };
    });
}

/// 采样器槽 set（绑定到单元；WebGL2 兼容面）。
pub(crate) fn bind_sampler(m: &mut StateMirror, unit: u32, name: Option<GlHandle>) {
    let idx = unit as usize;
    if idx >= BIND_SLOTS {
        panic!("state 断言: 采样器单元 {unit} 超上限 {BIND_SLOTS}");
    }
    if m.sampler_bindings[idx] == name {
        return;
    }
    m.sampler_bindings[idx] = name;
    with_gl(|gl| unsafe {
        match name {
            Some(g) => gl.bind_sampler(unit, Some(pi_glow::NativeSampler(nz(g.0)))),
            None => gl.bind_sampler(unit, None),
        };
    });
}

/// 视口组 set（六元组=draw 汇合层内翻后的最终下发值；交集面=GLint 视口+f32 深度区间两段下发，比对防冗余）。
pub(crate) fn set_viewport(m: &mut StateMirror, v: [f32; 6]) {
    if m.viewport == Some(v) {
        return;
    }
    m.viewport = Some(v);
    with_gl(|gl| unsafe {
        gl.viewport(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
        gl.depth_range_f32(v[4], v[5]);
    });
}

/// 裁剪组 set（框+使能一次落齐）。
pub(crate) fn set_scissor(m: &mut StateMirror, box4: [i32; 4], enabled: bool) {
    if m.scissor == Some(box4) && m.scissor_test == Some(enabled) {
        return;
    }
    m.scissor = Some(box4);
    m.scissor_test = Some(enabled);
    with_gl(|gl| unsafe {
        set_enable(gl, GL_SCISSOR_TEST, enabled);
        gl.scissor(box4[0], box4[1], box4[2], box4[3]);
    });
}

/// 深度三面 set（整体比对、一次刷齐；使能/写掩码/比较函数）。
pub(crate) fn set_depth(m: &mut StateMirror, enabled: bool, write: bool, compare: CompareFunction) {
    if m.depth_test_enabled == Some(enabled) && m.depth_write_enabled == Some(write) && cmp_same(m.depth_compare, compare) {
        return;
    }
    m.depth_test_enabled = Some(enabled);
    m.depth_write_enabled = Some(write);
    m.depth_compare = Some(compare);
    with_gl(|gl| unsafe {
        set_enable(gl, GL_DEPTH_TEST, enabled);
        gl.depth_mask(write);
        gl.depth_func(cmp_to_gl(compare));
    });
}

/// 混合组 set（哨兵面：Some(None)=关混合；enable+方程+因子一次刷）。
pub(crate) fn set_blend(m: &mut StateMirror, blend: Option<BlendState>) {
    if blend_same(m.blend, blend) {
        return;
    }
    m.blend = Some(blend);
    with_gl(|gl| unsafe {
        set_enable(gl, GL_BLEND, blend.is_some());
        if let Some(bs) = blend {
            gl.blend_equation_separate(blend_op_to_gl(bs.color.operation), blend_op_to_gl(bs.alpha.operation));
            gl.blend_func_separate(
                blend_factor_to_gl(bs.color.color_factor),
                blend_factor_to_gl(bs.color.alpha_factor),
                blend_factor_to_gl(bs.alpha.color_factor),
                blend_factor_to_gl(bs.alpha.alpha_factor),
            );
        }
    });
}

/// 混合常量色 set。
pub(crate) fn set_blend_constant(m: &mut StateMirror, c: Color) {
    if color_same(m.blend_constant, c) {
        return;
    }
    m.blend_constant = Some(c);
    with_gl(|gl| unsafe { gl.blend_color(c.r, c.g, c.b, c.a) });
}

/// 写掩码 set（位拆四 bool）。
pub(crate) fn set_color_writes(m: &mut StateMirror, w: ColorWrites) {
    if writes_same(m.color_writes, w) {
        return;
    }
    m.color_writes = Some(w);
    with_gl(|gl| unsafe { gl.color_mask(w.0 & 1 != 0, w.0 & 2 != 0, w.0 & 4 != 0, w.0 & 8 != 0) });
}

/// 图元三面 set（剔除开关+模式/绕序/填充一次刷齐）。
pub(crate) fn set_primitive(m: &mut StateMirror, cull: CullMode, front: FrontFace, polygon: PolygonMode) {
    if cull_same(m.cull_mode, cull) && front_same(m.front_face, front) && polygon_same(m.polygon_mode, polygon) {
        return;
    }
    m.cull_mode = Some(cull);
    m.front_face = Some(front);
    m.polygon_mode = Some(polygon);
    with_gl(|gl| unsafe {
        set_enable(gl, GL_CULL_FACE, cull.is_some());
        if let Some(mode) = cull_mode_to_gl(cull) {
            gl.cull_face(mode);
        }
        gl.front_face(front_face_to_gl(front));
        gl.polygon_mode(GL_FRONT_AND_BACK, polygon_to_gl(polygon));
    });
}

/// 深度偏置 set（使能=任一非零；clamp 词汇位无驱动对应=不落地已登记差异）。
pub(crate) fn set_depth_bias(m: &mut StateMirror, bias: DepthBiasState) {
    if bias_same(m.depth_bias, bias) {
        return;
    }
    m.depth_bias = Some(bias);
    let on = bias.constant != 0.0 || bias.slope != 0.0;
    let (slope, constant) = bias_pair(bias);
    with_gl(|gl| unsafe {
        set_enable(gl, GL_POLYGON_OFFSET_FILL, on);
        gl.polygon_offset(slope, constant);
    });
}

/// 模板两面+参考值 set（前后各刷 func/op；掩码恒全 1=交集面）。
pub(crate) fn set_stencil(m: &mut StateMirror, front: StencilFaceState, back: StencilFaceState, reference: u32) {
    if stencil_face_same(m.stencil_front, front) && stencil_face_same(m.stencil_back, back) && m.stencil_reference == Some(reference) {
        return;
    }
    m.stencil_front = Some(front);
    m.stencil_back = Some(back);
    m.stencil_reference = Some(reference);
    with_gl(|gl| unsafe {
        for (face, st) in [(GL_FRONT, front), (GL_BACK, back)] {
            gl.stencil_func_separate(face, cmp_to_gl(st.compare), reference as i32, !0);
            gl.stencil_op_separate(face, stencil_op_to_gl(st.fail_op), stencil_op_to_gl(st.depth_fail_op), stencil_op_to_gl(st.pass_op));
        }
    });
}

/// 模板参考值 set：入镜像；两面状态已知时即时重下 func（差异才发）。
pub(crate) fn set_stencil_reference(m: &mut StateMirror, reference: u32) {
    if m.stencil_reference == Some(reference) {
        return;
    }
    m.stencil_reference = Some(reference);
    if let (Some(fs), Some(bs)) = (m.stencil_front, m.stencil_back) {
        with_gl(|gl| unsafe {
            for (face, st) in [(GL_FRONT, fs), (GL_BACK, bs)] {
                gl.stencil_func_separate(face, cmp_to_gl(st.compare), reference as i32, !0);
            }
        });
    }
}

/// 覆盖采样使能 set。
pub(crate) fn set_alpha_to_coverage(m: &mut StateMirror, on: bool) {
    if m.sample_alpha_to_coverage == Some(on) {
        return;
    }
    m.sample_alpha_to_coverage = Some(on);
    with_gl(|gl| unsafe { set_enable(gl, GL_SAMPLE_ALPHA_TO_COVERAGE, on) });
}

/// 采样掩码值 set：本 fork glow 未暴露掩码下发面——镜像仅记录期望值（供 B10 采样合同校验），
/// 不产生驱动下发；差异登记见设计登记（词汇有能力、交集驱动面无=保守不谎报）。
pub(crate) fn set_sample_mask(m: &mut StateMirror, mask: u64) {
    m.sample_mask = Some(mask);
}

// 1. 可回读档：程序/单元/开关/函数/视口/裁剪框 与驱动实值比对，差异（或死名命中）才重下发镜像期望。
// 2. 整发档：句柄绑定类在 WebGL 面不可回读 + UBO 槽无标量回读——恒幂等刷镜像期望（下发内容=镜像值，不新增谎报）。
// 3. 下发后镜像不动（镜像即期望）；死名命中组下发后 forget_removed 销记。
pub(crate) fn flush_impl(m: &mut StateMirror) {
    with_gl(|gl| unsafe {
        // —— 程序组属整发档：GL_CURRENT_PROGRAM 无 glGetIntegerv 回读路径（规范未提供）——
        // 期望比对不可行：恒向驱动刷镜像名（幂等）。
        apply_program(m);
        // 死名登记表的域=objects kind_num_*（0..3 复用域、4..6 直建域）。
        // 整发档已把镜像全期望重刷给驱动（绑定=重指到镜像名，被删名已不可能进驱动当前态），
        // 故末尾整面清表：登记使命（强制本次重发）已兑现，防名单调增长（B6 消费前收口）。
        clear_removed();
        let exp_active = m.active_texture_unit.unwrap_or(0);
        let real_active = (gl.get_parameter_i32(GL_ACTIVE_TEXTURE) as u32).wrapping_sub(GL_TEXTURE0);
        if exp_active != real_active {
            gl.active_texture(GL_TEXTURE0 + exp_active);
        }
        let exp_depth = m.depth_test_enabled.unwrap_or(false);
        if (gl.get_parameter_i32(GL_DEPTH_TEST) != 0) != exp_depth {
            set_enable(gl, GL_DEPTH_TEST, exp_depth);
        }
        if let Some(cmp) = m.depth_compare {
            if (gl.get_parameter_i32(GL_DEPTH_FUNC) as u32) != cmp_to_gl(cmp) {
                gl.depth_func(cmp_to_gl(cmp));
            }
        }
        if let Some(wr) = m.depth_write_enabled {
            gl.depth_mask(wr);
        }
        let exp_cull = cull_mode_to_gl(m.cull_mode.unwrap_or(None));
        let real_cull_mode = gl.get_parameter_i32(GL_CULL_FACE_MODE) as u32;
        if (gl.get_parameter_i32(GL_CULL_FACE) != 0) != exp_cull.is_some() {
            set_enable(gl, GL_CULL_FACE, exp_cull.is_some());
        }
        if let Some(mode) = exp_cull {
            if mode != real_cull_mode {
                gl.cull_face(mode);
            }
        }
        if let Some(f) = m.front_face {
            if (gl.get_parameter_i32(GL_FRONT_FACE) as u32) != front_face_to_gl(f) {
                gl.front_face(front_face_to_gl(f));
            }
        }
        let exp_scissor = m.scissor_test.unwrap_or(false);
        if (gl.get_parameter_i32(GL_SCISSOR_TEST) != 0) != exp_scissor {
            set_enable(gl, GL_SCISSOR_TEST, exp_scissor);
        }
        let exp_blend = m.blend.map(|b| b.is_some()).unwrap_or(false);
        if (gl.get_parameter_i32(GL_BLEND) != 0) != exp_blend {
            set_enable(gl, GL_BLEND, exp_blend);
        }
        let mut box4 = [0i32; 4];
        gl.get_parameter_i32_slice(GL_SCISSOR_BOX, &mut box4);
        if m.scissor != Some(box4) {
            if let Some(b) = m.scissor {
                gl.scissor(b[0], b[1], b[2], b[3]);
            }
        }
        if let Some(v) = m.viewport {
            let mut vp = [0i32; 4];
            gl.get_parameter_i32_slice(GL_VIEWPORT, &mut vp);
            let mut dr = [0f32; 2];
            gl.get_parameter_f32_slice(GL_DEPTH_RANGE, &mut dr);
            let want = [v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32];
            if vp != want {
                gl.viewport(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32);
            }
            if dr != [v[4], v[5]] {
                gl.depth_range_f32(v[4], v[5]);
            }
        }
        // 整发档（驱动态不可回读的组，逐一口播镜像期望）
        probe(1);
        apply_vao(m);
        apply_fbo(m);
        apply_unpack(m);
        for i in 0..BIND_SLOTS {
            if let Some((g, off, sz)) = m.ubo_bindings[i] {
                if off == 0 && sz == 0 {
                    gl.bind_buffer_base(GL_UNIFORM_BUFFER, i as u32, Some(pi_glow::NativeBuffer(nz(g.0))));
                } else {
                    gl.bind_buffer_range(GL_UNIFORM_BUFFER, i as u32, Some(pi_glow::NativeBuffer(nz(g.0))), off as i32, sz as i32);
                }
            }
            gl.active_texture(GL_TEXTURE0 + i as u32);
            match m.texture_bindings[i] {
                Some((g, tg)) => gl.bind_texture(tg, Some(pi_glow::NativeTexture(nz(g.0)))),
                None => gl.bind_texture(GL_TEXTURE_2D, None),
            };
            match m.sampler_bindings[i] {
                Some(g) => gl.bind_sampler(i as u32, Some(pi_glow::NativeSampler(nz(g.0)))),
                None => gl.bind_sampler(i as u32, None),
            };
        }
        gl.active_texture(GL_TEXTURE0 + exp_active);
        if let Some(Some(bs)) = m.blend {
            gl.blend_equation_separate(blend_op_to_gl(bs.color.operation), blend_op_to_gl(bs.alpha.operation));
            gl.blend_func_separate(
                blend_factor_to_gl(bs.color.color_factor),
                blend_factor_to_gl(bs.color.alpha_factor),
                blend_factor_to_gl(bs.alpha.color_factor),
                blend_factor_to_gl(bs.alpha.alpha_factor),
            );
        }
        if let Some(c) = m.blend_constant {
            gl.blend_color(c.r, c.g, c.b, c.a);
        }
        if let Some(w) = m.color_writes {
            gl.color_mask(w.0 & 1 != 0, w.0 & 2 != 0, w.0 & 4 != 0, w.0 & 8 != 0);
        }
        if let Some(p) = m.polygon_mode {
            gl.polygon_mode(GL_FRONT_AND_BACK, polygon_to_gl(p));
        }
        if let Some(bias) = m.depth_bias {
            let on = bias.constant != 0.0 || bias.slope != 0.0;
            let (slope, constant) = bias_pair(bias);
            set_enable(gl, GL_POLYGON_OFFSET_FILL, on);
            gl.polygon_offset(slope, constant);
        }
        if let (Some(fs), Some(bs)) = (m.stencil_front, m.stencil_back) {
            let r = m.stencil_reference.unwrap_or(0);
            for (face, st) in [(GL_FRONT, fs), (GL_BACK, bs)] {
                gl.stencil_func_separate(face, cmp_to_gl(st.compare), r as i32, !0);
                gl.stencil_op_separate(face, stencil_op_to_gl(st.fail_op), stencil_op_to_gl(st.depth_fail_op), stencil_op_to_gl(st.pass_op));
            }
        }
        if let Some(on) = m.sample_alpha_to_coverage {
            set_enable(gl, GL_SAMPLE_ALPHA_TO_COVERAGE, on);
        }

    });
}

// 1. 引用该句柄名的全部镜像字段置「未知」（None）——不发任何驱动调用。
// 2. 后续 flush 对这些组按未知处理重同步=强制重下发；幂等（重复调用同单次）。
pub(crate) fn invalidate_impl(m: &mut StateMirror, dead: GlHandle) {
    let hit = |o: &mut Option<GlHandle>| {
        if o.map(|g| g == dead).unwrap_or(false) {
            *o = None;
        }
    };
    hit(&mut m.program);
    hit(&mut m.vao);
    hit(&mut m.fbo_draw);
    hit(&mut m.pixel_unpack_buffer);
    let hit_buf = |o: &mut Option<(GlHandle, u32, u32)>| {
        if o.map(|(g, _, _)| g == dead).unwrap_or(false) {
            *o = None;
        }
    };
    for slot in m.ubo_bindings.iter_mut() {
        hit_buf(slot);
    }
    let hit_tex = |o: &mut Option<(GlHandle, u32)>| {
        if o.map(|(g, _)| g == dead).unwrap_or(false) {
            *o = None;
        }
    };
    for slot in m.texture_bindings.iter_mut() {
        hit_tex(slot);
    }
    for slot in m.sampler_bindings.iter_mut() {
        hit(slot);
    }
}

/// 调试读口（仅测试）：程序域死名登记查询。
#[cfg(test)]
pub(crate) fn dbg_removed_program(name: u32) -> bool {
    is_removed(0, name)
}

/// 整面重置（invalidate_generations 显式清理路径：全部期望值丢弃，下轮 flush 即重探测基线）。
/// 幂等；generation 由调用侧随新附着落值。
pub(crate) fn reset_all(m: &mut StateMirror) {
    *m = StateMirror::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 映射表完整性：每枚举全臂可映射即覆盖（值对规范见各表注释；此处验证穷举与抽查）。
    #[test]
    fn enum_maps_test() {
        assert_eq!(cmp_to_gl(CompareFunction::Never), 0x0200);
        assert_eq!(cmp_to_gl(CompareFunction::Always), 0x0207);
        assert_eq!(blend_op_to_gl(BlendOperation::Add), 0x8006);
        assert_eq!(front_face_to_gl(FrontFace::Ccw), 0x0901);
        assert_eq!(polygon_to_gl(PolygonMode::Fill), 0x1B01);
        assert_eq!(stencil_op_to_gl(StencilOperation::Replace), 0x1E02);
    }

    /// 比较器语义：未设(None)对任何实值恒不等→首轮必下发；哨兵可区分关混合与未设。
    #[test]
    fn same_helpers_semantics_test() {
        let off: Option<BlendState> = None;
        assert!(blend_same(Some(off), off), "关混合哨兵相等");
        assert!(!blend_same(None, off), "未设≠关混合");
        assert!(!cull_same(None, None));
        assert!(cull_same(Some(None), None));
        assert!(name_of(None) == 0);
    }

    /// nz：合法名通过、零名断言路径。
    #[test]
    fn nz_test() {
        assert_eq!(nz(5).get(), 5);
        assert!(std::panic::catch_unwind(|| nz(0)).is_err());
    }
}
