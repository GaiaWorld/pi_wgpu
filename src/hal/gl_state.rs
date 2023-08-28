use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use glow::HasContext;
use twox_hash::RandomXxHashBuilder64;

use super::EglContext;
use crate::{wgt, BufferAddress};

const RESET_INTERVAL_SECS: u64 = 5;

#[derive(Debug)]
pub(crate) struct GLState {
    // 每次Draw 之后，都会 重置 为 Default
    curr_geometry: Geometry,

    // 10s 清空一次

    vao: Option<glow::VertexArray>,
    vao_map: HashMap<Geometry, glow::VertexArray, RandomXxHashBuilder64>,

    last_reset_vao_time: Instant,

    pub(crate) copy_fbo: glow::Framebuffer,
    pub(crate) draw_fbo: glow::Framebuffer,

    gl: glow::Context,
    egl: EglContext,

    topology: u32,

    blend_color: [f32; 4],

    viewport: Viewport,
    scissor: Scissor,

    stencil: StencilState,
}

impl GLState {
    pub fn new(gl: glow::Context, egl: EglContext) -> Self {
        let copy_fbo = unsafe { gl.create_framebuffer().unwrap() };

        let draw_fbo = unsafe { gl.create_framebuffer().unwrap() };

        Self {
            gl,
            egl,

            copy_fbo,
            draw_fbo,

            topology: glow::TRIANGLES,

            vao: Default::default(),
            vao_map: Default::default(),
            curr_geometry: Default::default(),

            blend_color: [0.0; 4],
            viewport: Default::default(),
            scissor: Default::default(),

            stencil: Default::default(),
        }
    }

    pub fn set_topology(&mut self, topology: wgt::PrimitiveTopology) {
        let t = map_primitive_topology(topology);
        if self.topology != t {
            self.topology = t;
        }
    }

    pub fn set_vertex_buffer(&mut self, index: usize, buffer: glow::Buffer, offset: BufferAddress) {
        let g = &mut self.curr_geometry;
        if g.vertexs[index].is_none() {
            g.vertexs[index] = Some(Default::default());
        }

        let v = g.vertexs[index].as_mut().unwrap();
        v.buffer = Some(buffer);
        v.offset = offset;
    }

    pub fn set_index_buffer(
        &mut self,
        buffer: glow::Buffer,
        offset: BufferAddress,
        format: wgt::IndexFormat,
    ) {
        let g = &mut self.curr_geometry;

        g.index_buffer = Some(buffer);
        g.index_offset = offset;

        let (index_size, index_type) = map_index_format(format);
        g.index_size = index_size;
        g.index_type = index_type;
    }

    pub fn draw(&mut self, start_vertex: u32, vertex_count: u32, instance_count: u32) {
        profiling::scope!("hal::CommandEncoder::")
        self.before_draw();

        if instance_count == 1 {
            unsafe {
                self.gl
                    .draw_arrays(self.topology, start_vertex as i32, vertex_count as i32)
            };
        } else {
            unsafe {
                self.gl.draw_arrays_instanced(
                    self.topology,
                    start_vertex as i32,
                    vertex_count as i32,
                    instance_count as i32,
                )
            };
        }

        self.after_draw();
    }

    pub fn draw_indexed(&mut self, start_index: u32, index_count: u32, instance_count: u32) {
        self.before_draw();

        let g = &self.curr_geometry;

        let index_offset = g.index_offset + g.index_size as u64 * start_index as u64;

        if instance_count == 1 {
            unsafe {
                self.gl.draw_elements(
                    self.topology,
                    index_count as i32,
                    g.index_type,
                    index_offset as i32,
                )
            }
        } else {
            unsafe {
                self.gl.draw_elements_instanced(
                    self.topology,
                    index_count as i32,
                    g.index_type,
                    index_offset as i32,
                    instance_count as i32,
                )
            }
        }

        self.after_draw();
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let vp = &mut self.viewport;

        if x != vp.x || y != vp.y || w != vp.w || h != vp.h {
            unsafe { self.gl.viewport(x, y, w, h) };
            vp.x = x;
            vp.y = y;
            vp.w = w;
            vp.h = h;
        }
    }

    pub fn set_scissor(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let s = &mut self.scissor;

        if !s.is_enable {
            unsafe { self.gl.enable(glow::SCISSOR_TEST) };

            s.is_enable = true;
        }

        if x != s.x || y != s.y || w != s.w || h != s.h {
            unsafe { self.gl.scissor(x, y, w, h) };

            s.x = x;
            s.y = y;
            s.w = w;
            s.h = h;
        }
    }

    pub fn set_depth_range(&mut self, min_depth: f32, max_depth: f32) {
        let vp = &mut self.viewport;

        if min_depth != vp.min_depth || max_depth != vp.max_depth {
            unsafe { self.gl.depth_range_f32(min_depth, max_depth) };

            vp.min_depth = min_depth;
            vp.max_depth = max_depth;
        }
    }

    pub fn set_blend_color(&mut self, color: &[f32; 4]) {
        if self.blend_color[0] != color[0]
            || self.blend_color[1] != color[1]
            || self.blend_color[2] != color[2]
            || self.blend_color[3] != color[3]
        {
            unsafe { self.gl.blend_color(color[0], color[1], color[2], color[3]) };

            self.blend_color[0] = color[0];
            self.blend_color[1] = color[1];
            self.blend_color[2] = color[2];
            self.blend_color[3] = color[3];
        }
    }

    pub fn set_stencil_reference(&mut self, front: i32, back: i32) {
        let s = &mut self.stencil;

        if front == back
            && s.front.test_func == s.back.test_func
            && s.front.mask_read == s.back.mask_read
        {
            if front != self.stencil.front.reference {
                unsafe {
                    self.gl.stencil_func_separate(
                        glow::FRONT_AND_BACK,
                        s.front.test_func,
                        front,
                        s.front.mask_read,
                    )
                };

                s.front.reference = front;
                s.back.reference = back;
            }
        } else {
            if front != s.front.reference {
                unsafe {
                    self.gl.stencil_func_separate(
                        glow::FRONT,
                        s.front.test_func,
                        front,
                        s.front.mask_read,
                    )
                };
                s.front.reference = front;
            }

            if back != s.back.reference {
                unsafe {
                    self.gl.stencil_func_separate(
                        glow::BACK,
                        s.back.test_func,
                        back,
                        s.back.mask_read,
                    )
                };
                s.back.reference = back;
            }
        }
    }
}

impl GLState {
    fn before_draw(&mut self) {
        self.update_and_bind_vao();
    }

    fn after_draw(&mut self) {
        self.curr_geometry = Default::default();

        // 必须 清空 VAO 绑定，否则 之后 如果 bind_buffer 修改 vb / ib 的话 就会 误操作了
        unsafe {
            self.gl.bind_vertex_array(None);
            self.vao = None;
        }

        self.reset_vao();
    }

    // 每过 几秒 就 清空一次 vao
    fn reset_vao(&mut self) {
        let now = Instant::now();
        if now - self.last_reset_vao_time < Duration::from_secs(RESET_INTERVAL_SECS) {
            return;
        }

        self.last_reset_vao_time = now;

        self.vao = None;

        for (geometry, vao) in &self.vao_map {
            unsafe {
                self.gl.delete_vertex_array(vao.clone());
            }
        }

        self.vao_map.clear();
    }

    // 根据 curr_geometry 更新 vao
    fn update_and_bind_vao(&mut self) {
        self.vao = match self.vao_map.get(&self.curr_geometry) {
            Some(vao) => {
                unsafe {
                    self.gl.bind_vertex_array(Some(vao.clone()));
                }

                Some(vao.clone())
            }
            None => unsafe {
                let g = &self.curr_geometry;

                let vao = self.gl.create_vertex_array().unwrap();

                self.gl.bind_vertex_array(Some(vao));

                if let Some(ib) = &g.index_buffer {
                    self.gl
                        .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ib.clone()));
                }

                for (i, vb) in g.vertexs.iter().enumerate() {
                    if vb.is_none() {
                        self.gl.disable_vertex_attrib_array(i as u32);
                    } else {
                        let vb = vb.as_ref().unwrap();

                        self.gl.enable_vertex_attrib_array(i as u32);

                        self.gl.bind_buffer(glow::ARRAY_BUFFER, vb.buffer);

                        // TODO: 如果有 int 作为 attribute 的话，需要加上 i32 函数
                        self.gl.vertex_attrib_pointer_f32(
                            i as u32,
                            vb.item_count,
                            vb.data_type,
                            false,
                            0,
                            vb.offset as i32,
                        );
                    }
                }

                self.vao_map.insert(self.curr_geometry.clone(), vao.clone());

                Some(vao)
            },
        };
    }
}

#[derive(Debug, Default)]
struct Viewport {
    x: i32,
    y: i32,
    w: i32,
    h: i32,

    min_depth: f32,
    max_depth: f32,
}

#[derive(Debug)]
struct Scissor {
    is_enable: bool,

    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Default for Scissor {
    fn default() -> Self {
        Self {
            is_enable: false,
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        }
    }
}

#[derive(Debug)]
struct StencilState {
    is_enable: bool,

    front: StencilFaceState,

    back: StencilFaceState,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            is_enable: false,
            front: Default::default(),
            back: Default::default(),
        }
    }
}

#[derive(Debug)]
struct StencilFaceState {
    pub test_func: u32, // wgt::CompareFunction, map_compare_func
    pub reference: i32,
    pub mask_read: u32,
    pub mask_write: u32,

    pub fail_op: u32,  // wgt::StencilOperation, map_stencil_op
    pub zfail_op: u32, // wgt::StencilOperation, map_stencil_op
    pub zpass_op: u32, // wgt::StencilOperation, map_stencil_op
}

impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            test_func: glow::ALWAYS,
            reference: 0,
            mask_read: 0,
            mask_write: 0,

            fail_op: glow::KEEP,
            zfail_op: glow::KEEP,
            zpass_op: glow::KEEP,
        }
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
struct VertexState {
    buffer: Option<glow::Buffer>,
    offset: BufferAddress,
    stride: i32,

    item_count: i32, // 1, 2, 3, 4
    data_type: u32,  // glow::Float
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
struct Geometry {
    // vertex
    vertexs: [Option<VertexState>; super::MAX_VERTEX_BUFFERS],

    // index_format: wgt::IndexFormat,
    index_buffer: Option<glow::Buffer>,
    index_size: u32,
    index_type: u32,
    index_offset: wgt::BufferAddress,
}

// ======================= convert to gl's const

#[inline]
fn map_compare_func(fun: wgt::CompareFunction) -> u32 {
    use wgt::CompareFunction as Cf;

    match fun {
        Cf::Never => glow::NEVER,
        Cf::Less => glow::LESS,
        Cf::LessEqual => glow::LEQUAL,
        Cf::Equal => glow::EQUAL,
        Cf::GreaterEqual => glow::GEQUAL,
        Cf::Greater => glow::GREATER,
        Cf::NotEqual => glow::NOTEQUAL,
        Cf::Always => glow::ALWAYS,
    }
}

#[inline]
fn map_stencil_op(operation: wgt::StencilOperation) -> u32 {
    use wgt::StencilOperation as So;

    match operation {
        So::Keep => glow::KEEP,
        So::Zero => glow::ZERO,
        So::Replace => glow::REPLACE,
        So::Invert => glow::INVERT,
        So::IncrementClamp => glow::INCR,
        So::DecrementClamp => glow::DECR,
        So::IncrementWrap => glow::INCR_WRAP,
        So::DecrementWrap => glow::DECR_WRAP,
    }
}

#[inline]
fn map_primitive_topology(topology: wgt::PrimitiveTopology) -> u32 {
    use wgt::PrimitiveTopology as Pt;
    match topology {
        Pt::PointList => glow::POINTS,
        Pt::LineList => glow::LINES,
        Pt::LineStrip => glow::LINE_STRIP,
        Pt::TriangleList => glow::TRIANGLES,
        Pt::TriangleStrip => glow::TRIANGLE_STRIP,
    }
}

#[inline]
fn map_index_format(format: wgt::IndexFormat) -> (u32, u32) {
    match format {
        wgt::IndexFormat::Uint16 => (2, glow::UNSIGNED_SHORT),
        wgt::IndexFormat::Uint32 => (4, glow::UNSIGNED_INT),
    }
}
