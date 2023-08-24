use super::EglContext;
use crate::wgt;

use glow::HasContext;

#[derive(Debug)]
pub(crate) struct GLState {
    pub(crate) copy_fbo: glow::Framebuffer,
    pub(crate) draw_fbo: glow::Framebuffer,

    gl: glow::Context,
    egl: EglContext,

    topology: u32,

    // index_format: wgt::IndexFormat,
    index_size: u32,
    index_type: u32,
    index_offset: wgt::BufferAddress,

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

            index_size: 2,
            index_offset: 0,
            index_type: glow::UNSIGNED_SHORT,

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

    pub fn draw(&mut self, start_vertex: u32, vertex_count: u32, instance_count: u32) {
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
    }

    pub fn draw_indexed(&mut self, start_index: u32, index_count: u32, instance_count: u32) {
        let index_offset = self.index_offset + self.index_size as u64 * start_index as u64;

        if instance_count == 1 {
            unsafe {
                self.gl.draw_elements(
                    self.topology,
                    index_count as i32,
                    self.index_type,
                    index_offset as i32,
                )
            }
        } else {
            unsafe {
                self.gl.draw_elements_instanced(
                    self.topology,
                    index_count as i32,
                    self.index_type,
                    index_offset as i32,
                    instance_count as i32,
                )
            }
        }
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
