use glow::HasContext;
use pi_share::{Share, ShareCell};

use super::{gl_cache::GLCache, gl_conv as conv, RawBinding, ShaderID, VertexAttribKind};
use super::super::{hal, hal::GLUniformType, wgt, BufferSize};

#[derive(Debug, Clone)]
pub(crate) struct GLState(pub Share<ShareCell<GLStateImpl>>);

impl GLState {
    #[inline]
    pub fn new(gl: glow::Context) -> Self {
        let imp = GLStateImpl::new(gl);
        Self(Share::new(ShareCell::new(imp)))
    }

    #[inline]
    pub fn next_shader_id(&self) -> ShaderID {
        let mut s = self.0.borrow_mut();

        s.global_shader_id += 1;

        s.global_shader_id
    }

    #[inline]
    pub fn max_attribute_slots(&self) -> usize {
        self.0.borrow().max_attribute_slots
    }
    #[inline]
    pub fn max_textures_slots(&self) -> usize {
        self.0.borrow().max_textures_slots
    }

    #[inline]
    pub fn max_color_attachments(&self) -> usize {
        self.0.borrow().max_color_attachments
    }

    #[inline]
    pub fn get_program(&self, id: &super::ProgramID) -> Option<super::Program> {
        self.0.borrow().cache.get_program(id)
    }

    #[inline]
    pub fn insert_program(&self, id: super::ProgramID, program: super::Program) {
        self.0.borrow_mut().cache.insert_program(id, program)
    }

    #[inline]
    pub fn get_or_insert_rs(&self, rs: super::RasterStateImpl) -> Share<super::RasterState> {
        let mut s = self.0.borrow_mut();
        s.cache.get_or_insert_rs(self.clone(), rs)
    }

    #[inline]
    pub fn get_or_insert_ds(&self, ds: super::DepthStateImpl) -> Share<super::DepthState> {
        let mut s = self.0.borrow_mut();
        s.cache.get_or_insert_ds(self.clone(), ds)
    }

    #[inline]
    pub fn get_or_insert_ss(&self, rs: super::StencilStateImpl) -> Share<super::StencilState> {
        let mut s = self.0.borrow_mut();
        s.cache.get_or_insert_ss(self.clone(), rs)
    }

    #[inline]
    pub fn get_or_insert_bs(&self, bs: super::BlendStateImpl) -> Share<super::BlendState> {
        let mut s = self.0.borrow_mut();
        s.cache.get_or_insert_bs(self.clone(), bs)
    }

    #[inline]
    pub fn remove_bs(&self, bs: &super::BlendStateImpl) {
        let mut s = self.0.borrow_mut();
        s.cache.remove_bs(bs);
    }

    #[inline]
    pub fn remove_rs(&self, rs: &super::RasterStateImpl) {
        let mut s = self.0.borrow_mut();
        s.cache.remove_rs(rs);
    }

    #[inline]
    pub fn remove_ds(&self, ds: &super::DepthStateImpl) {
        let mut s = self.0.borrow_mut();
        s.cache.remove_ds(ds);
    }

    #[inline]
    pub fn remove_ss(&self, ss: &super::StencilStateImpl) {
        let mut s = self.0.borrow_mut();
        s.cache.remove_ss(ss);
    }

    #[inline]
    pub fn remove_program(&self, id: &super::ProgramID) {
        let mut s = self.0.borrow_mut();
        s.cache.remove_program(id);
    }

    #[inline]
    pub fn remove_render_buffer(&self, rb: glow::Renderbuffer) {
        let mut s = self.0.borrow_mut();
        let s: &mut _ = &mut *s;

        let gl = &s.gl;
        let cache = &mut s.cache;

        cache.remove_render_buffer(gl, rb);
    }

    pub fn remove_buffer(&mut self, bind_target: u32, buffer: glow::Buffer) {
        profiling::scope!("hal::GLState::remove_buffer");

        if bind_target == glow::UNIFORM_BUFFER {
            return;
        }

        let mut imp = self.0.borrow_mut();
        let imp: &mut _ = &mut *imp;

        if bind_target == glow::ELEMENT_ARRAY_BUFFER {
            if let Some(ib) = imp.index_buffer.as_ref() {
                if ib.raw == buffer {
                    imp.index_buffer = None;
                }
            }
            return;
        }

        imp.cache.remove_buffer(&imp.gl, bind_target, buffer);
    }

    #[inline]
    pub fn remove_texture(&self, texture: glow::Texture) {
        let mut s = self.0.borrow_mut();
        let s = &mut *s;

        s.cache.remove_texture(&s.gl, texture);

        // TODO 到 TextureCache 移除 对应的 槽位
    }

    #[inline]
    pub fn remove_sampler(&self, sampler: glow::Sampler) {
        // TODO 到 TextureCache 移除 对应的 槽位
    }
}

#[derive(Debug)]
pub(crate) struct GLStateImpl {
    pub(crate) gl: glow::Context,

    cache: GLCache,
    global_shader_id: ShaderID,
    last_vbs: Option<Box<[Option<VBState>]>>,

    // 各种 MAX
    max_attribute_slots: usize,   // glow::MAX_VERTEX_ATTRIBS
    max_textures_slots: usize,    // glow::MAX_TEXTURE_IMAGE_UNITS
    max_color_attachments: usize, // glow::MAX_COLOR_ATTACHMENTS

    // 全局 GL 状态
    // VAO = render_pipeline.attributes + vertex_buffers
    render_pipeline: Option<super::RenderPipeline>,
    vertex_buffers: Box<[Option<VBState>]>, // 长度 不会 超过 max_attribute_slots
    index_buffer: Option<IBState>,

    bind_group_set: [Option<BindGroupState>; super::MAX_BIND_GROUPS],

    clear_color: wgt::Color,
    clear_depth: f32,
    clear_stencil: u32,

    // begin_pass 时，会自动设置为渲染目标的 宽 / 高
    viewport: Viewport,
    scissor: Scissor,

    stencil_ref: i32,
    blend_color: [f32; 4],

    textures: Box<[(Option<super::Texture>, Option<super::Sampler>)]>, // 长度 不会 超过 max_textures_slots
}

impl GLStateImpl {
    pub fn new(gl: glow::Context) -> Self {
        let max_attribute_slots =
            unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) as usize };
        let max_textures_slots =
            unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS) as usize };
        let max_color_attachments =
            unsafe { gl.get_parameter_i32(glow::MAX_COLOR_ATTACHMENTS) as usize };

        Self {
            gl,
            global_shader_id: 0,
            last_vbs: None,

            max_attribute_slots,
            max_textures_slots,
            max_color_attachments,

            cache: Default::default(),

            render_pipeline: None,
            vertex_buffers: vec![None; max_attribute_slots].into_boxed_slice(),
            index_buffer: None,

            bind_group_set: [None, None, None, None, None, None, None, None],

            viewport: Default::default(),
            scissor: Default::default(),

            clear_color: super::super::Color::default(),
            clear_depth: 1.0,
            clear_stencil: 0,

            blend_color: [0.0; 4],
            stencil_ref: 0,

            textures: Default::default(),
        }
    }

    #[inline]
    pub fn set_buffer_size(&self, buffer: &super::BufferImpl, size: i32) {
        profiling::scope!("hal::GLState::set_buffer_size");

        let gl = &self.gl;

        unsafe {
            gl.bind_buffer(buffer.gl_target, Some(buffer.raw));

            gl.buffer_data_size(buffer.gl_target, size, buffer.gl_usage);

            if buffer.gl_target == glow::ELEMENT_ARRAY_BUFFER {
                // 还原回 当前 状态机的状态

                let curr = self.index_buffer.as_ref().map(|v| v.raw);

                gl.bind_buffer(buffer.gl_target, curr);
            }
        }
    }

    #[inline]
    pub fn set_buffer_sub_data(&self, buffer: &super::BufferImpl, offset: i32, data: &[u8]) {
        profiling::scope!("hal::GLState::set_buffer_sub_data");

        let gl = &self.gl;

        unsafe {
            gl.bind_buffer(buffer.gl_target, Some(buffer.raw));
            gl.buffer_sub_data_u8_slice(buffer.gl_target, offset, data);

            if buffer.gl_target == glow::ELEMENT_ARRAY_BUFFER {
                // 还原回 当前 状态机的状态
                let curr = self.index_buffer.as_ref().map(|v| v.raw);

                gl.bind_buffer(buffer.gl_target, curr);
            }
        }
    }

    pub fn set_render_pipeline(&mut self, pipeline: &super::RenderPipeline) {
        profiling::scope!("hal::GLState::set_render_pipeline");

        if self.render_pipeline.is_none() {
            // 旧的没有，全部设置
            profiling::scope!("hal::GLState::apply_render_pipeline");

            let new = pipeline.0.as_ref();

            Self::apply_alpha_to_coverage(&self.gl, new.alpha_to_coverage_enabled);
            Self::apply_color_mask(&self.gl, &new.color_writes);
            Self::apply_program(&self.gl, &new.program);

            Self::apply_raster(&self.gl, &new.rs.imp);
            Self::apply_depth(&self.gl, &new.ds.imp);
            Self::apply_stencil(&self.gl, self.stencil_ref, &new.ss.imp);
            Self::apply_blend(&self.gl, &new.bs.imp);
        } else {
            // 有旧的，比较 Arc 指针

            let old = self.render_pipeline.as_ref().unwrap();
            if Share::ptr_eq(&pipeline.0, &old.0) {
                return;
            }

            let new = pipeline.0.as_ref();
            let old = old.0.as_ref();

            if new.alpha_to_coverage_enabled != old.alpha_to_coverage_enabled {
                Self::apply_alpha_to_coverage(&self.gl, new.alpha_to_coverage_enabled);
            }

            if new.color_writes != old.color_writes {
                Self::apply_color_mask(&self.gl, &new.color_writes);
            }

            if new.program.get_raw() != old.program.get_raw() {
                Self::apply_program(&self.gl, &new.program);
            }

            if !Share::ptr_eq(&new.rs, &old.rs) {
                Self::set_raster(&self.gl, &new.rs.imp, &old.rs.imp);
            }

            if !Share::ptr_eq(&new.ds, &old.ds) {
                Self::set_depth(&self.gl, &new.ds.imp, &old.ds.imp);
            }

            if !Share::ptr_eq(&new.ss, &old.ss) {
                Self::set_stencil(&self.gl, self.stencil_ref, &new.ss.imp, &old.ss.imp);
            }

            if !Share::ptr_eq(&new.bs, &old.bs) {
                Self::set_blend(&self.gl, &new.bs.imp, &old.bs.imp);
            }
        }

        self.render_pipeline = Some(pipeline.clone());
    }

    // 设置 FBO，设置 Viewport & Scissor，清屏
    pub fn set_render_target(&mut self, desc: &super::super::RenderPassDescriptor) {
        profiling::scope!("hal::GLState::set_render_target");

        // TODO 不支持 多目标 渲染
        assert!(desc.color_attachments.len() == 1);

        let color = desc.color_attachments[0].as_ref().unwrap();

        // TODO 不支持 多重采样
        assert!(color.resolve_target.is_none());

        let colors = GLTextureInfo::try_from(color.view).ok();

        let (depth_stencil, depth_ops, stencil_ops) = match &desc.depth_stencil_attachment {
            None => (None, None, None),
            Some(ds) => (
                GLTextureInfo::try_from(ds.view).ok(),
                ds.depth_ops,
                ds.stencil_ops,
            ),
        };

        if colors.is_none() {
            unsafe {
                self.gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None);
            }
        } else {
            let render_target = super::RenderTarget {
                depth_stencil,
                colors,
            };
            self.cache.bind_fbo(&self.gl, &render_target);
        }

        // 视口 & 裁剪
        let size = color.view.get_size();
        self.set_viewport(0, 0, size.0 as i32, size.1 as i32);
        self.set_scissor(0, 0, size.0 as i32, size.1 as i32);

        // 清屏
        self.clear_render_target(
            &color.ops.load,
            depth_ops.as_ref().map(|d| &d.load),
            stencil_ops.as_ref().map(|s| &s.load),
        );
    }

    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        profiling::scope!("hal::GLState::set_bind_group");

        assert!(index < super::MAX_BIND_GROUPS as u32);

        self.bind_group_set[index as usize] = Some(BindGroupState {
            bind_group: bind_group.contents.clone(),
            dynamic_offsets: dynamic_offsets.to_vec().into_boxed_slice(),
        });
    }

    pub fn set_vertex_buffer(
        &mut self,
        index: usize,
        buffer: &super::Buffer,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        profiling::scope!("hal::GLState::set_vertex_buffer");

        debug_assert!(buffer.0.gl_target == glow::ARRAY_BUFFER);

        let raw = buffer.0.raw;
        let offset = offset;
        let size = match size {
            None => buffer.0.size,
            Some(size) => u64::from(size) as i32,
        };

        self.vertex_buffers[index] = Some(VBState { raw, offset, size });
    }

    pub fn set_index_buffer(
        &mut self,
        buffer: &super::Buffer,
        format: wgt::IndexFormat,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        profiling::scope!("hal::GLState::set_index_buffer");

        debug_assert!(buffer.0.gl_target == glow::ELEMENT_ARRAY_BUFFER);

        let (item_count, item_type) = conv::map_index_format(format);

        let raw = buffer.0.raw;
        let ib_type = item_type;
        let ib_count = item_count;

        let size = match size {
            None => buffer.0.size,
            Some(size) => u64::from(size) as i32,
        };

        let need_update = match self.index_buffer.as_ref() {
            None => true,
            Some(ib) => {
                ib.raw != raw || ib.size != size || ib.offset != offset || ib.ib_type != ib_type
            }
        };

        if need_update {
            Self::apply_ib(&self.gl, Some(raw));

            self.index_buffer = Some(IBState {
                raw,
                ib_type,
                ib_count,
                size,
                offset,
            });
        }
    }

    pub fn draw(&mut self, start_vertex: u32, vertex_count: u32, instance_count: u32) {
        profiling::scope!("hal::GLState::draw");

        self.before_draw();

        let rp = self.render_pipeline.as_ref().unwrap().0.as_ref();

        if instance_count == 1 {
            unsafe {
                self.gl
                    .draw_arrays(rp.topology, start_vertex as i32, vertex_count as i32)
            };
        } else {
            unsafe {
                self.gl.draw_arrays_instanced(
                    rp.topology,
                    start_vertex as i32,
                    vertex_count as i32,
                    instance_count as i32,
                )
            };
        }

        self.after_draw();
    }

    pub fn draw_indexed(&mut self, start_index: i32, index_count: i32, instance_count: i32) {
        profiling::scope!("hal::GLState::draw_indexed");

        self.before_draw();

        let rp = self.render_pipeline.as_ref().unwrap().0.as_ref();

        let ib = self.index_buffer.as_ref().unwrap();

        let offset = ib.offset + start_index * ib.ib_count;

        if instance_count == 1 {
            unsafe {
                self.gl
                    .draw_elements(rp.topology, index_count, ib.ib_type, offset);
            }
        } else {
            unsafe {
                self.gl.draw_elements_instanced(
                    rp.topology,
                    index_count,
                    ib.ib_type,
                    offset,
                    instance_count,
                )
            }
        }

        self.after_draw();
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn set_depth_range(&mut self, min_depth: f32, max_depth: f32) {
        profiling::scope!("hal::GLState::set_depth_range");

        let vp = &mut self.viewport;

        if min_depth != vp.min_depth || max_depth != vp.max_depth {
            unsafe { self.gl.depth_range_f32(min_depth, max_depth) };

            vp.min_depth = min_depth;
            vp.max_depth = max_depth;
        }
    }

    #[inline]
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

    #[inline]
    pub fn set_stencil_reference(&mut self, reference: i32) {
        profiling::scope!("hal::GLState::set_stencil_reference");

        if reference == self.stencil_ref {
            return;
        }

        if let Some(p) = self.render_pipeline.as_ref() {
            let ss = &p.0.ss.as_ref().imp;

            unsafe {
                self.gl.stencil_func_separate(
                    glow::FRONT,
                    ss.front.test_func,
                    reference,
                    ss.mask_read,
                );

                self.gl.stencil_func_separate(
                    glow::BACK,
                    ss.back.test_func,
                    reference,
                    ss.mask_read,
                );
            }
        }
        self.stencil_ref = reference;
    }
}

impl GLStateImpl {
    #[inline]
    fn before_draw(&mut self) {
        self.update_vao();

        self.update_uniforms();
    }

    fn after_draw(&mut self) {
        // 必须 清空 VAO 绑定，否则 之后 如果 bind_buffer 修改 vb / ib 的话 就会 误操作了
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }

    // 根据 render_pipeline.attributes + vertex_buffers 更新 vao
    fn update_vao(&mut self) {
        profiling::scope!("hal::GLState::update_vao");

        let rp = self.render_pipeline.as_ref().unwrap().0.as_ref();

        let mut vbs = match self.last_vbs.take() {
            None => vec![None; rp.attributes.vb_count].into_boxed_slice(),
            Some(mut vbs) => {
                if vbs.len() != rp.attributes.vb_count {
                    vec![None; rp.attributes.vb_count].into_boxed_slice()
                } else {
                    for vb in vbs.iter_mut() {
                        *vb = None
                    }
                    vbs
                }
            }
        };

        for attrib in rp.attributes.info.iter() {
            if let Some(a) = attrib {
                vbs[a.buffer_slot] = self.vertex_buffers[a.buffer_slot].clone();
            }
        }

        let mut geometry = super::GeometryState {
            attributes: rp.attributes.clone(),
            vbs,
        };

        self.cache.bind_vao(&self.gl, &geometry);

        // 回收 vbs
        self.last_vbs = Some(geometry.vbs);
    }

    // 根据 render_pipeline.program + bind_group 更新 uniform
    fn update_uniforms(&mut self) {
        let program = &self.render_pipeline.as_ref().unwrap().0.program;
        let program = program.0.borrow_mut();

        let bg_set = &mut self.bind_group_set;

        let reorder = &self.render_pipeline.as_ref().unwrap().0.layout_reoder;

        let gl = &self.gl;

        for (i, bindings) in program.uniforms.iter().enumerate() {
            let bg = &bg_set[i];
            if bg.is_none() {
                assert!(bindings.len() == 0);
                continue;
            }
            let bg = bg.as_ref().unwrap();

            let reorder = &reorder[i];
            for (j, binding) in bindings.iter().enumerate() {
                let index = reorder[j];

                match &bg.bind_group[index] {
                    RawBinding::Buffer {
                        raw,
                        dynamic_offset,
                        offset,
                        size,
                    } => unsafe {
                        assert!(binding.u_type == GLUniformType::Buffer);
                        let imp = raw.0.as_ref();

                        let mut offset = if *dynamic_offset >= 0 {
                            *offset + bg.dynamic_offsets[*dynamic_offset as usize] as i32
                        } else {
                            *offset
                        };

                        // TODO 加 比较
                        if offset == 0 && *size == imp.size {
                            gl.bind_buffer_base(
                                glow::UNIFORM_BUFFER,
                                binding.glsl_binding,
                                Some(imp.raw),
                            );
                        } else {
                            gl.bind_buffer_range(
                                glow::UNIFORM_BUFFER,
                                binding.glsl_binding,
                                Some(imp.raw),
                                offset,
                                *size,
                            );
                        }
                    },
                    RawBinding::Texture(view) => unsafe {
                        assert!(binding.u_type == GLUniformType::Texture);
                        let imp = view.inner.as_ref();
                        match &imp.inner {
                            hal::TextureInner::Texture { state, raw, target } => {
                                // TODO 加 比较
                                gl.active_texture(glow::TEXTURE0 + binding.glsl_binding);
                                gl.bind_texture(*target, Some(*raw));
                            }
                            _ => panic!("mis match texture size"),
                        }
                    },
                    RawBinding::Sampler(sampler) => unsafe {
                        // TODO 加 比较
                        assert!(binding.u_type == GLUniformType::Sampler);
                        let imp = sampler.0.as_ref();
                        gl.bind_sampler(binding.glsl_binding, Some(imp.raw));
                    },
                }
            }
        }
    }

    fn clear_render_target(
        &mut self,
        color: &super::super::LoadOp<super::super::Color>,
        depth: Option<&super::super::LoadOp<f32>>,
        stencil: Option<&super::super::LoadOp<u32>>,
    ) {
        profiling::scope!("hal::GLState::clear_render_target");

        let mut clear_mask = 0;

        if let super::super::LoadOp::Clear(color) = color {
            clear_mask |= glow::COLOR_BUFFER_BIT;
            if self.clear_color != *color {
                unsafe {
                    self.gl.clear_color(
                        color.r as f32,
                        color.g as f32,
                        color.b as f32,
                        color.a as f32,
                    );
                }
                self.clear_color = *color;
            }
        }

        if let Some(ds_ops) = depth {
            if let super::super::LoadOp::Clear(depth) = ds_ops {
                clear_mask |= glow::DEPTH_BUFFER_BIT;
                if self.clear_depth != *depth {
                    unsafe {
                        self.gl.clear_depth_f32(*depth);
                    }
                    self.clear_depth = *depth;
                }
            }
        }

        if let Some(stencil_ops) = stencil {
            if let super::super::LoadOp::Clear(stencil) = &stencil_ops {
                clear_mask |= glow::STENCIL_BUFFER_BIT;
                if self.clear_stencil != *stencil {
                    unsafe {
                        self.gl.clear_stencil(*stencil as i32);
                    }
                    self.clear_stencil = *stencil;
                }
            }
        }

        if clear_mask != 0 {
            unsafe {
                self.gl.clear(clear_mask);
            }
        }
    }
}

impl GLStateImpl {
    #[inline]
    fn apply_ib(gl: &glow::Context, ib: Option<glow::Buffer>) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, ib);
        }
    }

    #[inline]
    fn apply_alpha_to_coverage(gl: &glow::Context, alpha_to_coverage_enabled: bool) {
        if alpha_to_coverage_enabled {
            unsafe { gl.enable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
        } else {
            unsafe { gl.disable(glow::SAMPLE_ALPHA_TO_COVERAGE) };
        }
    }

    #[inline]
    fn apply_color_mask(gl: &glow::Context, mask: &wgt::ColorWrites) {
        use wgt::ColorWrites as Cw;
        unsafe {
            gl.color_mask(
                mask.contains(Cw::RED),
                mask.contains(Cw::GREEN),
                mask.contains(Cw::BLUE),
                mask.contains(Cw::ALPHA),
            )
        };
    }

    #[inline]
    fn apply_program(gl: &glow::Context, program: &super::Program) {
        let program = program.0.borrow();
        unsafe {
            gl.use_program(Some(program.raw));
        }
    }

    #[inline]
    fn apply_raster(gl: &glow::Context, new: &super::RasterStateImpl) {
        Self::apply_cull_enable(gl, new);
        Self::apply_front_face(gl, new);
        Self::apply_cull_face(gl, new);
    }

    #[inline]
    fn apply_depth(gl: &glow::Context, new: &super::DepthStateImpl) {
        Self::apply_depth_test_enable(gl, new);
        Self::apply_depth_write_enable(gl, new);
        Self::apply_depth_test_function(gl, new);
        Self::apply_depth_bias(gl, &new.depth_bias);
    }

    #[inline]
    fn apply_stencil(gl: &glow::Context, stencil_ref: i32, new: &super::StencilStateImpl) {
        Self::apply_stencil_test(&gl, new);

        Self::apply_stencil_face(&gl, glow::FRONT, stencil_ref, &new, &new.front);

        Self::apply_stencil_face(&gl, glow::BACK, stencil_ref, &new, &new.back);
    }

    #[inline]
    fn apply_blend(gl: &glow::Context, new: &super::BlendStateImpl) {
        Self::apply_blend_enable(gl, new);
        Self::apply_blend_equation(gl, new);
        Self::apply_blend_factor(gl, new);
    }

    fn set_raster(gl: &glow::Context, new: &super::RasterStateImpl, old: &super::RasterStateImpl) {
        profiling::scope!("hal::GLState::set_raster");

        if new.is_cull_enable != old.is_cull_enable {
            Self::apply_cull_enable(gl, new);
        }

        if new.front_face != old.front_face {
            Self::apply_front_face(gl, new);
        }

        if new.cull_face != old.cull_face {
            Self::apply_cull_face(gl, new);
        }
    }

    fn set_depth(gl: &glow::Context, new: &super::DepthStateImpl, old: &super::DepthStateImpl) {
        profiling::scope!("hal::GLState::set_depth");

        if new.is_test_enable != old.is_test_enable {
            Self::apply_depth_test_enable(gl, new);
        }

        if new.is_write_enable != old.is_write_enable {
            Self::apply_depth_write_enable(gl, new);
        }

        if new.function != old.function {
            Self::apply_depth_test_function(gl, new);
        }

        let new = &new.depth_bias;
        let old = &old.depth_bias;

        if new.slope_scale != old.slope_scale || new.constant != old.constant {
            Self::apply_depth_bias(gl, new);
        }
    }

    fn set_stencil(
        gl: &glow::Context,
        stencil_ref: i32,
        new: &super::StencilStateImpl,
        old: &super::StencilStateImpl,
    ) {
        profiling::scope!("hal::GLState::set_stencil");

        Self::set_stencil_test(&gl, new, old);

        Self::set_stencil_face(
            &gl,
            glow::FRONT,
            stencil_ref,
            &new,
            &new.front,
            &old,
            &old.front,
        );

        Self::set_stencil_face(
            &gl,
            glow::BACK,
            stencil_ref,
            &new,
            &new.back,
            &old,
            &old.back,
        );
    }

    fn set_blend(gl: &glow::Context, new: &super::BlendStateImpl, old: &super::BlendStateImpl) {
        profiling::scope!("hal::GLState::set_blend");

        if new.is_enable != old.is_enable {
            Self::apply_blend_enable(gl, new);
        }

        if new.color.equation != old.color.equation || new.alpha.equation != old.alpha.equation {
            Self::apply_blend_equation(gl, new);
        }

        if new.color.src_factor != old.color.src_factor
            || new.color.dst_factor != old.color.dst_factor
            || new.alpha.src_factor != old.alpha.src_factor
            || new.alpha.dst_factor != old.alpha.dst_factor
        {
            Self::apply_blend_factor(gl, new);
        }
    }

    #[inline]
    fn apply_cull_enable(gl: &glow::Context, new: &super::RasterStateImpl) {
        if new.is_cull_enable {
            unsafe { gl.enable(glow::CULL_FACE) };
        } else {
            unsafe { gl.disable(glow::CULL_FACE) };
        }
    }

    #[inline]
    fn apply_front_face(gl: &glow::Context, new: &super::RasterStateImpl) {
        unsafe { gl.front_face(new.front_face) };
    }

    #[inline]
    fn apply_cull_face(gl: &glow::Context, new: &super::RasterStateImpl) {
        unsafe { gl.cull_face(new.cull_face) };
    }

    #[inline]
    fn apply_depth_test_enable(gl: &glow::Context, new: &super::DepthStateImpl) {
        if new.is_test_enable {
            unsafe {
                gl.enable(glow::DEPTH_TEST);
            }
        } else {
            unsafe {
                gl.disable(glow::DEPTH_TEST);
            }
        }
    }

    #[inline]
    fn apply_depth_write_enable(gl: &glow::Context, new: &super::DepthStateImpl) {
        unsafe {
            gl.depth_mask(new.is_write_enable);
        }
    }

    #[inline]
    fn apply_depth_test_function(gl: &glow::Context, new: &super::DepthStateImpl) {
        unsafe {
            gl.depth_func(new.function);
        }
    }

    #[inline]
    fn apply_depth_bias(gl: &glow::Context, new: &super::DepthBiasState) {
        if new.constant == 0 && new.slope_scale == 0.0 {
            unsafe { gl.disable(glow::POLYGON_OFFSET_FILL) };
        } else {
            unsafe { gl.enable(glow::POLYGON_OFFSET_FILL) };

            unsafe { gl.polygon_offset(new.constant as f32, new.slope_scale.0) };
        }
    }

    #[inline]
    fn apply_stencil_test(gl: &glow::Context, new: &super::StencilStateImpl) {
        if new.is_enable {
            unsafe {
                gl.enable(glow::STENCIL_TEST);
            }
        } else {
            unsafe {
                gl.disable(glow::STENCIL_TEST);
            }
        }
    }

    fn set_stencil_test(
        gl: &glow::Context,
        new: &super::StencilStateImpl,
        old: &super::StencilStateImpl,
    ) {
        if new.is_enable != old.is_enable {
            Self::apply_stencil_test(gl, new);
        }
    }

    #[inline]
    fn apply_stencil_func(
        gl: &glow::Context,
        face: u32,
        stencil_ref: i32,
        test_func: u32,
        mask_read: u32,
    ) {
        unsafe { gl.stencil_func_separate(face, test_func, stencil_ref, mask_read) };
    }

    #[inline]
    fn apply_stencil_mask(gl: &glow::Context, face: u32, mask_write: u32) {
        unsafe { gl.stencil_mask_separate(face, mask_write) };
    }

    #[inline]
    fn apply_stencil_op(gl: &glow::Context, face: u32, fail_op: u32, zfail_op: u32, zpass_op: u32) {
        unsafe {
            gl.stencil_op_separate(face, fail_op, zfail_op, zpass_op);
        };
    }

    #[inline]
    fn apply_stencil_face(
        gl: &glow::Context,
        face: u32,
        stencil_ref: i32,
        new: &super::StencilStateImpl,
        new_face: &super::StencilFaceState,
    ) {
        Self::apply_stencil_func(gl, face, stencil_ref, new_face.test_func, new.mask_read);

        Self::apply_stencil_mask(gl, face, new.mask_write);

        Self::apply_stencil_op(
            gl,
            face,
            new_face.fail_op,
            new_face.zfail_op,
            new_face.zpass_op,
        );
    }

    fn set_stencil_face(
        gl: &glow::Context,
        face: u32,
        stencil_ref: i32,
        new: &super::StencilStateImpl,
        new_face: &super::StencilFaceState,
        old: &super::StencilStateImpl,
        old_face: &super::StencilFaceState,
    ) {
        if new_face.test_func != old_face.test_func || new.mask_read != old.mask_read {
            Self::apply_stencil_func(gl, face, stencil_ref, new_face.test_func, new.mask_read);
        }

        if new.mask_write != old.mask_write {
            Self::apply_stencil_mask(gl, face, new.mask_write);
        }

        if new_face.zpass_op != old_face.zpass_op
            || new_face.zfail_op != old_face.zfail_op
            || new_face.fail_op != old_face.fail_op
        {
            Self::apply_stencil_op(
                gl,
                face,
                new_face.fail_op,
                new_face.zfail_op,
                new_face.zpass_op,
            );
        }
    }

    #[inline]
    fn apply_blend_enable(gl: &glow::Context, new: &super::BlendStateImpl) {
        if new.is_enable {
            unsafe { gl.enable(glow::BLEND) };
        } else {
            unsafe { gl.disable(glow::BLEND) };
        }
    }

    #[inline]
    fn apply_blend_equation(gl: &glow::Context, new: &super::BlendStateImpl) {
        unsafe { gl.blend_equation_separate(new.color.equation, new.alpha.equation) };
    }

    #[inline]
    fn apply_blend_factor(gl: &glow::Context, new: &super::BlendStateImpl) {
        unsafe {
            gl.blend_func_separate(
                new.color.src_factor,
                new.color.dst_factor,
                new.alpha.src_factor,
                new.alpha.dst_factor,
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct VBState {
    pub(crate) raw: glow::Buffer,
    pub(crate) offset: i32,
    pub(crate) size: i32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct IBState {
    pub(crate) raw: glow::Buffer,

    pub(crate) ib_type: u32,  // glow::UNSIGNED_INT, glow::UNSIGNED_SHORT
    pub(crate) ib_count: i32, // 2, 4

    pub(crate) size: i32,
    pub(crate) offset: i32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct AttributeState {
    pub(crate) vb_count: usize, // vertex_buffers 的 前多少个 VB 对这个 Attribute 有效
    pub(crate) info: Box<[Option<AttributeInfo>]>,
}

impl AttributeState {
    #[inline]
    pub(crate) fn new(max_vertex_attributes: usize, vb_count: usize) -> Self {
        Self {
            vb_count,
            info: vec![None; max_vertex_attributes].into_boxed_slice(),
        }
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub(crate) struct AttributeInfo {
    pub(crate) buffer_slot: usize, // 对应 vertex_buffers 的 槽位
    pub(crate) is_buffer_instance: bool,

    // struct VertexFormatDesc
    pub(crate) element_count: i32,  // 1, 2, 3, 4
    pub(crate) element_format: u32, // glow::FLOAT

    pub(crate) attrib_stride: i32,
    pub(crate) attrib_offset: i32, // 相对于 vertex_buffer 片段 的 偏移
    pub(crate) attrib_kind: VertexAttribKind,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub(crate) struct RenderTarget {
    pub(crate) depth_stencil: Option<GLTextureInfo>,
    pub(crate) colors: Option<GLTextureInfo>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum GLTextureInfo {
    Renderbuffer(glow::Renderbuffer),

    Texture(glow::Texture),
}

impl TryFrom<&super::super::TextureView> for GLTextureInfo {
    type Error = ();

    fn try_from(value: &super::super::TextureView) -> Result<Self, Self::Error> {
        match &value.inner.inner.inner {
            super::TextureInner::DefaultRenderbuffer => Err(()),
            super::TextureInner::Renderbuffer { raw, .. } => Ok(GLTextureInfo::Renderbuffer(*raw)),
            super::TextureInner::Texture { raw, .. } => Ok(GLTextureInfo::Texture(*raw)),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BindGroupState {
    bind_group: Box<[super::RawBinding]>,

    dynamic_offsets: Box<[wgt::DynamicOffset]>,
}
