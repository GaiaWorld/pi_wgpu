use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use glow::HasContext;
use ordered_float::OrderedFloat;
use pi_share::{Share, ShareWeak};
use twox_hash::RandomXxHashBuilder64;           

use super::{EglContext, PipelineID, ShaderID, TextureID, VertexAttribKind};
use crate::{wgt, BufferAddress, BufferSize};

const RESET_INTERVAL_SECS: u64 = 5;

pub(crate) struct CreateRenderPipelineError {

}

#[derive(Debug)]
pub(crate) struct GLState {
    gl: glow::Context,
    egl: EglContext,

    last_reset_time: Instant,

    global_render_pipeline_id: PipelineID,

    vao_map: HashMap<Geometry, glow::VertexArray, RandomXxHashBuilder64>,
    fbo_map: HashMap<RenderTarget, glow::Framebuffer, RandomXxHashBuilder64>,
    program_map: HashMap<Program, ShareWeak<glow::Program>, RandomXxHashBuilder64>,

    bs_map: HashMap<BlendState, ShareWeak<BlendState>, RandomXxHashBuilder64>,
    rs_map: HashMap<RasterState, ShareWeak<RasterState>, RandomXxHashBuilder64>,
    ds_map: HashMap<DepthState, ShareWeak<DepthState>, RandomXxHashBuilder64>,
    ss_map: HashMap<StencilState, ShareWeak<StencilState>, RandomXxHashBuilder64>,

    // 每次Draw 之后，都会 重置 为 Default
    geometry: Geometry,
    vao: Option<glow::VertexArray>,

    topology: u32,

    clear_color: [f32; 4],
    clear_depth: f32,
    clear_stencil: u32,

    viewport: Viewport,
    scissor: Scissor,

    blend_color: [f32; 4],
    bs: Share<super::BlendState>,
    
    alpha_to_coverage_enabled: bool,
    color_writes: wgt::ColorWrites,

    stencil_ref: i32,
    ss: Share<StencilState>,
    rs: Share<RasterState>,
    ds: Share<DepthState>,

    render_pipeline_id: PipelineID,

    textures: [Option<(super::Texture, super::Sampler)>; super::MAX_TEXTURE_SLOTS],
}

impl GLState {
    pub fn new(gl: glow::Context, egl: EglContext) -> Self {
        let copy_fbo = unsafe { gl.create_framebuffer().unwrap() };

        let draw_fbo = unsafe { gl.create_framebuffer().unwrap() };

        Self {
            gl,
            egl,

            last_reset_time: Instant::now(),
            global_render_pipeline_id: 1,

            vao_map: Default::default(),
            fbo_map: Default::default(),
            program_map: Default::default(),

            bs_map: Default::default(),
            rs_map: Default::default(),
            ds_map: Default::default(),
            ss_map: Default::default(),

            topology: glow::TRIANGLES,

            alpha_to_coverage_enabled: false,
            color_writes: Default::default(),

            vao: Default::default(),
            geometry: Default::default(),

            viewport: Default::default(),
            scissor: Default::default(),

            clear_color: [0.0; 4],
            clear_depth: 1.0,
            clear_stencil: 0,

            render_pipeline_id: 0,

            blend_color: [0.0; 4],
            bs: Share::new(Default::default()),

            stencil_ref: 0,
            ss: Share::new(Default::default()),

            rs: Share::new(Default::default()),
            ds: Share::new(Default::default()),

            textures: Default::default(),
        }
    }

    // ==================== 创建 && 销毁

    pub fn create_render_pipeline(
        &mut self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> Result<super::RenderPipeline, CreateRenderPipelineError> {
        let pipeline_id = self.global_render_pipeline_id;
        self.global_render_pipeline_id += 1;

        let topology = map_primitive_topology(desc.primitive.topology);

        let primitive: wgt::PrimitiveState,
        let vertex_attributes: [Option<super::AttributeDesc>; super::MAX_VERTEX_ATTRIBUTES],
    
        let depth_bias: super::DepthBiasState,
    
        let alpha_to_coverage_enabled: bool,
        let color_writes: [Option<Share<super::BlendState>>; super::MAX_COLOR_ATTACHMENTS],
    
        pub(crate) program: Share<super::Program>,
    
       let ds: Share<super::DepthState>,
       let bs: [Option<Share<super::BlendState>>; super::MAX_COLOR_ATTACHMENTS],
       let rs: Share<super::RasterState>,
       let ss: Share<super::StencilState>,
    

        Ok(super::RenderPipeline {
            pipeline_id,
            topology,

            vertex_attributes,
        
            alpha_to_coverage_enabled,
            color_writes,
        
            program,
        
            ds,
            bs,
            rs,
            ss,
        })
    }

    // ==================== 渲染相关

    pub fn begin_encoding(&mut self) {}

    // 每过 几秒 就 清空一次 vao
    pub fn end_encoding(&mut self) {
        profiling::scope!("hal::GLState::end_encoding");

        let now = Instant::now();
        if now - self.last_reset_time < Duration::from_secs(RESET_INTERVAL_SECS) {
            return;
        }
        self.last_reset_time = now;

        self.reset_vao();
        self.reset_fbo();
        self.reset_program();
    }

    pub fn set_bind_group(&mut self, pipeline: &super::BindGroup) {}

    pub fn set_render_pipeline(&mut self, pipeline: &super::RenderPipeline) {
        profiling::scope!("hal::GLState::SetRenderPipeline");

        if pipeline.pipeline_id == self.render_pipeline_id {
            return;
        }

        self.topology = pipeline.topology;
        
        self.set_alpha_to_coverage(pipeline.alpha_to_coverage_enabled);

        // TODO Program

        self.set_attribute(&pipeline.vertex_attributes);

        if !Share::eq(&self.rs, &pipeline.rs) {
            self.set_raster(pipeline.rs.as_ref(), self.rs.as_ref());
            self.rs = pipeline.rs.clone();
        }

        if !Share::eq(&self.ds, &pipeline.ds) {
            self.set_depth(pipeline.ds.as_ref(), self.ds.as_ref());
            self.ds = pipeline.ds.clone();
        }

        if !Share::eq(&self.ss, &pipeline.ss) {
            self.set_stencil_test(pipeline.ss.as_ref(), self.ss.as_ref());
            self.set_stencil_face(glow::FRONT, &pipeline.ss.front, &self.ss.front);
            self.set_stencil_face(glow::BACK, &pipeline.ss.back, &self.ss.back);
            self.ss = pipeline.ss.clone();
        }

        if !Share::eq(&self.bs, &pipeline.bs) {
            self.set_blend(pipeline.bs.as_ref(), self.bs.as_ref());
            self.bs = pipeline.bs.clone();
        }

        if self.color_writes != pipeline.color_writes {
            self.set_color_mask(&pipeline.color_writes);
            self.color_writes = pipeline.color_writes;
        }

    }

    pub fn set_vertex_buffer(&mut self, index: usize, buffer: &super::Buffer, offset: i32) {
        
        let g = &mut self.geometry;
        if g.vertexs[index].is_none() {
            g.vertexs[index] = Some(Default::default());
        }

        let v = g.vertexs[index].as_mut().unwrap();
        
        v.buffer = Some(buffer.raw);
        v.offset += offset;
        v.stride = buffer.stride;
        v.divisor_step = buffer.divisor_step;
    }

    pub fn set_index_buffer(
        &mut self,
        buffer: glow::Buffer,
        offset: BufferAddress,
        format: wgt::IndexFormat,
    ) {
        let g = &mut self.geometry;

        g.index_buffer = Some(buffer);
        g.index_offset = offset;

        let (index_size, index_type) = map_index_format(format);
        g.index_size = index_size;
        g.index_type = index_type;
    }

    pub fn draw(&mut self, start_vertex: u32, vertex_count: u32, instance_count: u32) {
        profiling::scope!("super::CommandEncoder::draw");

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
        profiling::scope!("hal::GLState::draw_indexed");
        self.before_draw();

        let g = &self.geometry;

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
        profiling::scope!("hal::GLState::set_viewport");
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
        profiling::scope!("hal::GLState::set_scissor");
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
        profiling::scope!("hal::GLState::set_depth_range");
        let vp = &mut self.viewport;

        if min_depth != vp.min_depth || max_depth != vp.max_depth {
            unsafe { self.gl.depth_range_f32(min_depth, max_depth) };

            vp.min_depth = min_depth;
            vp.max_depth = max_depth;
        }
    }

    pub fn set_blend_color(&mut self, color: &[f32; 4]) {
        profiling::scope!("hal::GLState::set_blend_color");
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
        profiling::scope!("hal::GLState::set_stencil_reference");
        
        let s = self.ss.as_ref();

        if front == back
            && s.front.test_func == s.back.test_func
            && s.front.mask_read == s.back.mask_read
        {
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
        profiling::scope!("hal::GLState::after_draw");
        self.update_and_bind_vao();
    }

    fn after_draw(&mut self) {
        profiling::scope!("hal::GLState::after_draw");
        self.geometry = Default::default();

        // 必须 清空 VAO 绑定，否则 之后 如果 bind_buffer 修改 vb / ib 的话 就会 误操作了
        unsafe {
            self.gl.bind_vertex_array(None);
            self.vao = None;
        }
    }

    fn reset_vao(&mut self) {
        profiling::scope!("hal::GLState::reset_vao");

        for (_, vao) in &self.vao_map {
            unsafe {
                self.gl.delete_vertex_array(vao.clone());
            }
        }
        self.vao_map.clear();
    }

    fn reset_fbo(&mut self) {
        profiling::scope!("hal::GLState::reset_fbo");

        for (_, handle) in &self.fbo_map {
            unsafe {
                self.gl.delete_framebuffer(handle.clone());
            }
        }
        self.fbo_map.clear();
    }

    fn reset_program(&mut self) {
        profiling::scope!("hal::GLState::reset_program");

        // 仅移除 handle 为 空的项
        let mut to_remove = vec![];
        for (program, weak_program) in &self.program_map {
            if weak_program.upgrade().is_none() {
                to_remove.push(program.clone());
            }
        }

        for p in to_remove {
            self.program_map.remove(&p);
        }
    }

    // 根据 geometry 更新 vao
    fn update_and_bind_vao(&mut self) {
        profiling::scope!("hal::GLState::update_and_bind_vao");
        self.vao = match self.vao_map.get(&self.geometry) {
            Some(vao) => {
                unsafe {
                    self.gl.bind_vertex_array(Some(vao.clone()));
                }

                Some(vao.clone())
            }
            None => unsafe {
                let g = &self.geometry;

                let vao = self.gl.create_vertex_array().unwrap();

                self.gl.bind_vertex_array(Some(vao));

                if let Some(ib) = &g.index_buffer {
                    self.gl
                        .bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ib.clone()));
                }

                for (i, vb) in g.vertexs.iter().enumerate() {
                    let i = i as u32;

                    if vb.is_none() {
                        self.gl.disable_vertex_attrib_array(i);
                    } else {
                        let vb = vb.as_ref().unwrap();

                        self.gl.enable_vertex_attrib_array(i);

                        self.gl.bind_buffer(glow::ARRAY_BUFFER, vb.buffer);

                        match vb.data_kind {
                            VertexAttribKind::Float => {
                                self.gl.vertex_attrib_pointer_f32(
                                    i,
                                    vb.item_count,
                                    vb.data_type,
                                    true, // always normalized
                                    vb.stride,
                                    vb.offset,
                                );
                            }
                            VertexAttribKind::Integer => {
                                self.gl.vertex_attrib_pointer_i32(
                                    i,
                                    vb.item_count,
                                    vb.data_type,
                                    vb.stride,
                                    vb.offset,
                                );
                            }
                        }

                        self.gl.vertex_attrib_divisor(i, vb.divisor_step);
                    }
                }

                self.vao_map.insert(self.geometry.clone(), vao.clone());

                Some(vao)
            },
        };
    }
}

impl GLState {

    fn set_alpha_to_coverage(&mut self, is_enable: bool) {
        profiling::scope!("hal::GLState::set_alpha_to_coverage");

        if self.alpha_to_coverage_enabled != is_enable{
            if is_enable {
                unsafe { self.gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
            } else {
                unsafe { self.gl.disable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
            }

            self.alpha_to_coverage_enabled = is_enable;
        }
    }

    fn set_attribute(&self, attributes: &[Option<super::AttributeDesc>]) {
        profiling::scope!("hal::GLState::set_attribute");

        for (i, attrib) in attributes.iter().enumerate() {
            profiling::scope!("hal::GLState::set_attribute, VertexAttribute");
            
            let g = &mut self.geometry;
        
            if let Some(a) = attrib {
                if g.vertexs[i].is_none() {
                    g.vertexs[i] = Some(Default::default());
                }

                let v = g.vertexs[i].as_mut().unwrap();

                v.offset += a.offset as i32;
                v.item_count = a.format_desc.element_count;
                v.data_type = a.format_desc.element_format;
                v.data_kind = a.format_desc.attrib_kind;   
            }
        }
    }

    fn set_raster(&self, new: &RasterState, old: &RasterState) {
        profiling::scope!("hal::GLState::set_raster");

        if new.is_cull_enable != old.is_cull_enable {
             if new.is_cull_enable {
                unsafe { self.gl.enable(glow::CULL_FACE) };
            } else {
                unsafe { self.gl.disable(glow::CULL_FACE) };
            }
        }

        if new.front_face != old.front_face {
            unsafe { self.gl.front_face(new.front_face) };
        }

        if new.cull_face != old.cull_face {
            unsafe {self.gl.cull_face(new.cull_face)};
        }
    }

    fn set_depth(&self, new: &DepthState, old: &DepthState) {
        profiling::scope!("hal::GLState::set_depth");

        if new.is_test_enable != old.is_test_enable {
            if new.is_test_enable {
                unsafe {
                    self.gl.enable(glow::DEPTH_TEST);
                }
            } else {
                unsafe {
                    self.gl.disable(glow::DEPTH_TEST);
                }
            }
        }

        if new.is_write_enable != old.is_write_enable {
            unsafe {
                self.gl.depth_mask(new.is_write_enable);
            }
        }

        if new.function != old.function {
            unsafe {
                self.gl.depth_func(new.function);
            }
        }
        
        let new = &new.depth_bias;
        let old = &old.depth_bias;

        if new.slope_scale != old.slope_scale || new.constant != old.constant {
            if new.constant == 0 && new.slope_scale == 0.0 {
                unsafe { self.gl.disable(glow::POLYGON_OFFSET_FILL) };
            } else {
                unsafe { self.gl.enable(glow::POLYGON_OFFSET_FILL) };

                unsafe { self.gl.polygon_offset(new.constant as f32, new.slope_scale.0) };
            }
        }
    }

    fn set_stencil_test(&self, new: &StencilState, old: &StencilState) {
        profiling::scope!("hal::GLState::set_stencil_test");

        if new.is_enable != old.is_enable {
            if new.is_enable {
                unsafe {
                    self.gl.enable(glow::STENCIL_TEST);
                }
            } else {
                unsafe {
                    self.gl.disable(glow::STENCIL_TEST);
                }
            }
        }
    }

    fn set_stencil_face(&self, face: u32, new: &StencilFaceState, old: &StencilFaceState) {
        profiling::scope!("hal::GLState::set_stencil_face");
        
        if new.test_func != old.test_func 
        || new.mask_read != old.mask_read {
            unsafe { self.gl.stencil_func_separate(face, new.test_func, self.stencil_ref, new.mask_read) };
        }
    
        if new.mask_write != old.mask_write {
            unsafe { self.gl.stencil_mask_separate(face, new.mask_write) };
        }
        
        if new.zpass_op != old.zpass_op || new.zfail_op != old.zfail_op || new.fail_op != old.fail_op {
            unsafe { self.gl.stencil_op_separate(face, new.fail_op, new.zfail_op, new.zpass_op) };
        }
    }

    fn set_blend(&self, new: &BlendState, old: &BlendState) {
        profiling::scope!("hal::GLState::set_blend");

        if new.is_enable != old.is_enable {
            if new.is_enable {
                unsafe { self.gl.enable(glow::BLEND) };
            } else {
                unsafe { self.gl.disable(glow::BLEND) };
            }
        }

        if new.color.equation != old.color.equation || new.alpha.equation != old.alpha.equation{
            unsafe {
                self.gl.blend_equation_separate(
                    new.color.equation,
                    new.alpha.equation,
                )
            };
        }
        
        if new.color.src_factor != old.color.src_factor 
        || new.color.dst_factor != old.color.dst_factor 
        || new.alpha.src_factor != old.alpha.src_factor 
        || new.alpha.dst_factor != old.alpha.dst_factor {
            unsafe {
                self.gl.blend_func_separate(
                    new.color.src_factor,
                    new.color.dst_factor,
                    new.alpha.src_factor,
                    new.alpha.dst_factor,
                )
            };
        }
    }

    fn set_color_mask(&self, mask: &wgt::ColorWrites) {
        profiling::scope!("hal::GLState::set_color_mask");
        use wgt::ColorWrites as Cw;
        unsafe {
            self.gl.color_mask(
                mask.contains(Cw::RED),
                mask.contains(Cw::GREEN),
                mask.contains(Cw::BLUE),
                mask.contains(Cw::ALPHA),
            )
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct BlendState {
    is_enable: bool,

    color: BlendComponent,
    alpha: BlendComponent,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            is_enable: true,
            color: Default::default(),
            alpha: Default::default(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct BlendComponent {
    equation: u32, // glow::FUNC_ADD,

    src_factor: u32, // glow::SRC_ALPHA,
    dst_factor: u32, // glow::ONE_MINUS_SRC_ALPHA,
}

impl Default for BlendComponent {
    fn default() -> Self {
        Self {
            equation: glow::FUNC_ADD,
            src_factor: glow::ONE,
            dst_factor: glow::ZERO,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct DepthState {
    is_test_enable: bool,
    is_write_enable: bool,
    function: u32, // wgt::CompareFunction, map_compare_func

    depth_bias: DepthBiasState, // wgt::DepthBiasState,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            is_test_enable: false,
            is_write_enable: false,
            function: glow::ALWAYS,

            depth_bias: DepthBiasState::default(),
        }
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub(crate) struct DepthBiasState {
    constant: i32,
    slope_scale: OrderedFloat<f32>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct RasterState {
    is_cull_enable: bool,
    front_face: u32, // glow::CW,  glow::CCW
    cull_face: u32,  // glow::FRONT, glow::BACK
}

impl Default for RasterState {
    fn default() -> Self {
        Self {
            is_cull_enable: false,
            front_face: glow::CCW,
            cull_face: glow::BACK,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct StencilState {
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

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct StencilFaceState {
    pub test_func: u32, // wgt::CompareFunction, map_compare_func

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
    offset: i32,
    stride: i32,

    divisor_step: u32,
    
    item_count: i32, // 1, 2, 3, 4
    data_type: u32,  // glow::Float
    
    data_kind: VertexAttribKind,
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

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
struct RenderTarget {
    depth: Option<TextureID>,
    colors: [TextureID; super::MAX_COLOR_ATTACHMENTS],
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub(crate) struct Program {
    vs_id: ShaderID,
    fs_id: ShaderID,
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

#[inline]
fn map_blend_factor(factor: wgt::BlendFactor) -> u32 {
    use wgt::BlendFactor as Bf;
    match factor {
        Bf::Zero => glow::ZERO,
        Bf::One => glow::ONE,
        Bf::Src => glow::SRC_COLOR,
        Bf::OneMinusSrc => glow::ONE_MINUS_SRC_COLOR,
        Bf::Dst => glow::DST_COLOR,
        Bf::OneMinusDst => glow::ONE_MINUS_DST_COLOR,
        Bf::SrcAlpha => glow::SRC_ALPHA,
        Bf::OneMinusSrcAlpha => glow::ONE_MINUS_SRC_ALPHA,
        Bf::DstAlpha => glow::DST_ALPHA,
        Bf::OneMinusDstAlpha => glow::ONE_MINUS_DST_ALPHA,
        Bf::Constant => glow::CONSTANT_COLOR,
        Bf::OneMinusConstant => glow::ONE_MINUS_CONSTANT_COLOR,
        Bf::SrcAlphaSaturated => glow::SRC_ALPHA_SATURATE,
    }
}

#[inline]
fn map_blend_component(component: &wgt::BlendComponent) -> BlendComponent {
    BlendComponent {
        src_factor: map_blend_factor(component.src_factor),
        dst_factor: map_blend_factor(component.dst_factor),
        equation: match component.operation {
            wgt::BlendOperation::Add => glow::FUNC_ADD,
            wgt::BlendOperation::Subtract => glow::FUNC_SUBTRACT,
            wgt::BlendOperation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
            wgt::BlendOperation::Min => glow::MIN,
            wgt::BlendOperation::Max => glow::MAX,
        },
    }
}

pub(super) fn describe_vertex_format(vertex_format: wgt::VertexFormat) -> super::VertexFormatDesc {
    use super::VertexAttribKind as Vak;
    use wgt::VertexFormat as Vf;

    let (element_count, element_format, attrib_kind) = match vertex_format {
        Vf::Unorm8x2 => (2, glow::UNSIGNED_BYTE, Vak::Float),
        Vf::Snorm8x2 => (2, glow::BYTE, Vak::Float),
        Vf::Uint8x2 => (2, glow::UNSIGNED_BYTE, Vak::Integer),
        Vf::Sint8x2 => (2, glow::BYTE, Vak::Integer),
        Vf::Unorm8x4 => (4, glow::UNSIGNED_BYTE, Vak::Float),
        Vf::Snorm8x4 => (4, glow::BYTE, Vak::Float),
        Vf::Uint8x4 => (4, glow::UNSIGNED_BYTE, Vak::Integer),
        Vf::Sint8x4 => (4, glow::BYTE, Vak::Integer),
        Vf::Unorm16x2 => (2, glow::UNSIGNED_SHORT, Vak::Float),
        Vf::Snorm16x2 => (2, glow::SHORT, Vak::Float),
        Vf::Uint16x2 => (2, glow::UNSIGNED_SHORT, Vak::Integer),
        Vf::Sint16x2 => (2, glow::SHORT, Vak::Integer),
        Vf::Float16x2 => (2, glow::HALF_FLOAT, Vak::Float),
        Vf::Unorm16x4 => (4, glow::UNSIGNED_SHORT, Vak::Float),
        Vf::Snorm16x4 => (4, glow::SHORT, Vak::Float),
        Vf::Uint16x4 => (4, glow::UNSIGNED_SHORT, Vak::Integer),
        Vf::Sint16x4 => (4, glow::SHORT, Vak::Integer),
        Vf::Float16x4 => (4, glow::HALF_FLOAT, Vak::Float),
        Vf::Uint32 => (1, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32 => (1, glow::INT, Vak::Integer),
        Vf::Float32 => (1, glow::FLOAT, Vak::Float),
        Vf::Uint32x2 => (2, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x2 => (2, glow::INT, Vak::Integer),
        Vf::Float32x2 => (2, glow::FLOAT, Vak::Float),
        Vf::Uint32x3 => (3, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x3 => (3, glow::INT, Vak::Integer),
        Vf::Float32x3 => (3, glow::FLOAT, Vak::Float),
        Vf::Uint32x4 => (4, glow::UNSIGNED_INT, Vak::Integer),
        Vf::Sint32x4 => (4, glow::INT, Vak::Integer),
        Vf::Float32x4 => (4, glow::FLOAT, Vak::Float),
        Vf::Float64 | Vf::Float64x2 | Vf::Float64x3 | Vf::Float64x4 => unimplemented!(),
    };

    super::VertexFormatDesc {
        element_count,
        element_format,
        attrib_kind,
    }
}