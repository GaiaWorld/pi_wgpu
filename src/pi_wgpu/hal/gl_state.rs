//! GL 全局状态机，调用 gl 函数之前，做全状态比较，减少GL指令的设置
//! 全局 缓冲表，见 GLCache
//!

use core::hash;
use std::hash::{DefaultHasher, Hash, Hasher};

use pi_assets::allocator::Allocator;
use pi_hash::XHashMap;

use glow::HasContext;
use naga::{
    back::glsl::{self, ReflectionInfo},
    proc::BoundsCheckPolicy,
    valid::{Capabilities as Caps, ModuleInfo},
};
use pi_share::{Share, ShareCell, ShareWeak};

use super::{
    super::{hal, wgt, BufferSize, ColorWrites}, gl_cache::GLCache, gl_conv as conv, PiBindingType, PrivateCapabilities, ShaderID, VertexAttribKind
};

#[derive(Clone)]
pub(crate) struct GLState {
    imp: Share<ShareCell<GLStateImpl>>,
}

impl std::fmt::Debug for GLState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GLState").finish()
    }
}

impl GLState {
    #[inline]
    pub(crate) fn new(gl: &glow::Context) -> Self {
        let imp = GLStateImpl::new(&gl);

        Self {
            imp: Share::new(ShareCell::new(imp)),
        }
    }

    #[inline]
    pub(crate) fn reset_state(&self, gl: &glow::Context,) {
        self.imp.as_ref().borrow_mut().reset_state(gl);
    }

    #[inline]
    pub(crate) fn next_shader_id(&self) -> ShaderID {
        // log::trace!(
        //     "========== GLState::next_shader_id lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.global_shader_id += 1;
            s.global_shader_id
        };

        // log::trace!(
        //     "========== GLState::next_shader_id unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn max_attribute_slots(&self) -> usize {
        // log::trace!(
        //     "========== GLState::max_attribute_slots lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = { self.imp.as_ref().borrow().max_attribute_slots };

        // log::trace!(
        //     "========== GLState::max_attribute_slots unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn max_textures_slots(&self) -> usize {
        // log::trace!(
        //     "========== GLState::max_textures_slots lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = { self.imp.as_ref().borrow().max_textures_slots };

        // log::trace!(
        //     "========== GLState::max_textures_slots unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn max_color_attachments(&self) -> usize {
        // log::trace!(
        //     "========== GLState::max_color_attachments lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = { self.imp.as_ref().borrow().max_color_attachments };

        // log::trace!(
        //     "========== GLState::max_color_attachments unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn get_program(&self, id: &super::ProgramID) -> Option<super::Program> {
        // log::trace!(
        //     "========== GLState::get_program lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = { self.imp.as_ref().borrow().cache.get_program(id) };

        // log::trace!(
        //     "========== GLState::get_program unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn insert_program(&self, id: super::ProgramID, program: super::Program) {
        // log::trace!(
        //     "========== GLState::insert_program lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            self.imp
                .as_ref()
                .borrow_mut()
                .cache
                .insert_program(id, program);
        }

        // log::trace!(
        //     "========== GLState::insert_program unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn get_or_insert_rs(&self, rs: super::RasterStateImpl) -> Share<super::RasterState> {
        // log::trace!(
        //     "========== GLState::get_or_insert_rs lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.get_or_insert_rs(rs)
        };

        // log::trace!(
        //     "========== GLState::get_or_insert_rs unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn get_or_insert_ds(&self, ds: super::DepthStateImpl) -> Share<super::DepthState> {
        // log::trace!(
        //     "========== GLState::get_or_insert_ds lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.get_or_insert_ds(ds)
        };

        // log::trace!(
        //     "========== GLState::get_or_insert_ds unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn get_or_insert_ss(
        &self,
        rs: super::StencilStateImpl,
    ) -> Share<super::StencilState> {
        // log::trace!(
        //     "========== GLState::get_or_insert_ss lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.get_or_insert_ss(rs)
        };

        // log::trace!(
        //     "========== GLState::get_or_insert_ss unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn get_or_insert_bs(&self, bs: super::BlendStateImpl) -> Share<super::BlendState> {
        // log::trace!(
        //     "========== GLState::get_or_insert_bs lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.get_or_insert_bs(bs)
        };

        // log::trace!(
        //     "========== GLState::get_or_insert_bs unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn create_program(
        &self,
        gl: &glow::Context,
        vs_id: ShaderID,
        fs_id: ShaderID,
    ) -> Result<(glow::Program, Box<[Box<[super::PiBindEntry]>]>), super::ShaderError> {
        // log::trace!(
        //     "========== GLState::create_program lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.create_program(gl, vs_id, fs_id)
        };

        // log::trace!(
        //     "========== GLState::create_program unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn compile_shader(
        &self,
        gl: &glow::Context,
        shader: &super::ShaderModule,
        shader_stage: naga::ShaderStage,
        version: &glow::Version,
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        entry_point: String,
        multiview: Option<std::num::NonZeroU32>,
        naga_options: &naga::back::glsl::Options,
    ) -> Result<(), super::ShaderError> {
        // log::trace!(
        //     "========== GLState::compile_shader lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = {
            let mut s = self.imp.as_ref().borrow_mut();
            s.compile_shader(
                gl,
                shader,
                shader_stage,
                version,
                features,
                downlevel,
                entry_point,
                multiview,
                naga_options,
            )
        };

        // log::trace!(
        //     "========== GLState::compile_shader unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn remove_shader(&self, id: ShaderID) {
        // log::trace!(
        //     "========== GLState::remove_shader lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.remove_shader(id);
        }

        // log::trace!(
        //     "========== GLState::remove_shader unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn clear_cache(&self, gl: &glow::Context) {
        // log::trace!(
        //     "========== GLState::clear_cache lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let mut s = self.imp.as_ref().borrow_mut();
            s.cache.clear_weak_refs(gl);
        }

        // log::trace!(
        //     "========== GLState::clear_cache unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn remove_render_buffer(&self, gl: &glow::Context, rb: glow::Renderbuffer) {
        profiling::scope!("hal::GLState::remove_render_buffer");

        // log::trace!(
        //     "========== GLState::remove_render_buffer lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let cache = &mut self.imp.as_ref().borrow_mut().cache;
            cache.remove_render_buffer(gl, rb);
        }

        // log::trace!(
        //     "========== GLState::remove_render_buffer unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    pub(crate) fn remove_buffer(&self, gl: &glow::Context, bind_target: u32, buffer: glow::Buffer) {
        profiling::scope!("hal::GLState::remove_buffer");

        // log::trace!(
        //     "========== GLState::remove_buffer 2 lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        self.imp
            .as_ref()
            .borrow_mut()
            .remove_buffer(gl, bind_target, buffer);

        // log::trace!(
        //     "========== GLState::remove_buffer 2 unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn restore_current_texture(&self, gl: &glow::Context, unit: u32, target: u32) {
        self.imp
            .as_ref()
            .borrow_mut()
            .restore_current_texture(gl, unit, target);
    }

    #[inline]
    pub(crate) fn remove_texture(&self, gl: &glow::Context, texture: glow::Texture) {
        profiling::scope!("hal::GLState::remove_texture");

        // log::trace!(
        //     "========== GLState::remove_texture lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        self.imp.as_ref().borrow_mut().remove_texture(gl, texture);

        // log::trace!(
        //     "========== GLState::remove_texture unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn remove_sampler(&self, gl: &glow::Context, sampler: glow::Sampler) {
        profiling::scope!("hal::GLState::remove_sampler");

        self.imp.as_ref().borrow_mut().remove_sampler(gl, sampler);
    }

    #[inline]
    pub(crate) fn set_buffer_size(
        &self,
        gl: &glow::Context,
        buffer: &super::BufferImpl,
        size: i32,
    ) {
        profiling::scope!("hal::GLState::set_buffer_size");

        // log::trace!(
        //     "========== GLState::set_buffer_size lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow();

            imp.set_buffer_size(gl, buffer, size);
        }

        // log::trace!(
        //     "========== GLState::set_buffer_size unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_buffer_sub_data(
        &self,
        gl: &glow::Context,
        buffer: &super::BufferImpl,
        offset: i32,
        data: &[u8],
    ) {
        profiling::scope!("hal::GLState::set_buffer_sub_data");

        // log::trace!(
        //     "========== GLState::set_buffer_sub_data lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow();
            imp.set_buffer_sub_data(gl, buffer, offset, data)
        }

        // log::trace!(
        //     "========== GLState::set_buffer_sub_data unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_render_pipeline(&self, gl: &glow::Context, pipeline: &super::RenderPipeline) {
        profiling::scope!("hal::GLState::set_render_pipeline");

        // log::trace!(
        //     "========== GLState::set_render_pipeline lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();
            imp.set_render_pipeline(gl, pipeline);
        }

        // log::trace!(
        //     "========== GLState::set_render_pipeline unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_render_target(
        &self,
        gl: &glow::Context,
        desc: &super::super::RenderPassDescriptor,
    ) {
        profiling::scope!("hal::GLState::set_render_target");

        // log::trace!(
        //     "========== GLState::set_render_target lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_render_target(gl, desc);
        }

        // log::trace!(
        //     "========== GLState::set_render_target unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_bind_group(
        &self,
        index: u32,
        bind_group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        profiling::scope!("hal::GLState::set_bind_group");

        // log::trace!(
        //     "========== GLState::set_bind_group lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_bind_group(index, bind_group, dynamic_offsets);
        }

        // log::trace!(
        //     "========== GLState::set_bind_group unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_vertex_buffer(
        &self,
        index: usize,
        buffer: &super::Buffer,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        profiling::scope!("hal::GLState::set_vertex_buffer");

        // log::trace!(
        //     "========== GLState::set_vertex_buffer lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_vertex_buffer(index, buffer, offset, size)
        }

        // log::trace!(
        //     "========== GLState::set_vertex_buffer unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_index_buffer(
        &self,
        gl: &glow::Context,
        buffer: &super::Buffer,
        format: wgt::IndexFormat,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        profiling::scope!("hal::GLState::set_index_buffer");

        // log::trace!(
        //     "========== GLState::set_index_buffer lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_index_buffer(gl, buffer, format, offset, size)
        }

        // log::trace!(
        //     "========== GLState::set_index_buffer unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn draw(
        &self,
        gl: &glow::Context,
        private_caps: PrivateCapabilities,
        start_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32, 
        // first_instance_location: &Option<NativeUniformLocation>
    ) {
        profiling::scope!("hal::GLState::draw");

        // log::trace!(
        //     "========== GLState::draw lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.draw(gl, private_caps, start_vertex, vertex_count, first_instance, instance_count);
        }

        // log::trace!(
        //     "========== GLState::draw unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn draw_indexed(
        &self,
        gl: &glow::Context,
        start_index: i32,
        index_count: i32,
        first_instance: u32,
        instance_count: i32,
    ) {
        profiling::scope!("hal::GLState::draw_indexed");

        // log::trace!(
        //     "========== GLState::draw_indexed lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.draw_indexed(gl, start_index, index_count, first_instance, instance_count);
        }

        // log::trace!(
        //     "========== GLState::draw_indexed unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_viewport(&self, gl: &glow::Context, x: i32, y: i32, w: i32, h: i32) {
        profiling::scope!("hal::GLState::set_viewport");

        // log::trace!(
        //     "========== GLState::set_viewport lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_viewport(gl, x, y, w, h);
        }

        // log::trace!(
        //     "========== GLState::set_viewport unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_scissor(&self, gl: &glow::Context, x: i32, y: i32, w: i32, h: i32) {
        profiling::scope!("hal::GLState::set_scissor");

        // log::trace!(
        //     "========== GLState::set_scissor lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_scissor(gl, x, y, w, h)
        }

        // log::trace!(
        //     "========== GLState::set_scissor unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_depth_range(&self, gl: &glow::Context, min_depth: f32, max_depth: f32) {
        profiling::scope!("hal::GLState::set_depth_range");

        // log::trace!(
        //     "========== GLState::set_depth_range lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_depth_range(gl, min_depth, max_depth);
        }

        // log::trace!(
        //     "========== GLState::set_depth_range unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_blend_color(&self, gl: &glow::Context, color: &[f32; 4]) {
        profiling::scope!("hal::GLState::set_blend_color");

        // log::trace!(
        //     "========== GLState::set_blend_color lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();

            imp.set_blend_color(gl, color);
        }

        // log::trace!(
        //     "========== GLState::set_blend_color unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }

    #[inline]
    pub(crate) fn set_stencil_reference(&self, gl: &glow::Context, reference: i32) {
        profiling::scope!("hal::GLState::set_stencil_reference");

        // log::trace!(
        //     "========== GLState::set_stencil_reference lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            let imp = &mut self.imp.as_ref().borrow_mut();
            imp.set_stencil_reference(gl, reference);
        }

        // log::trace!(
        //     "========== GLState::set_stencil_reference unlock, thread_id = {:?}",
        //     thread::current().id()
        // );
    }
}

// #[derive(Debug)]
pub(crate) struct GLStateImpl {
    cache: GLCache,
    global_shader_id: ShaderID,
    last_vbs: Option<Box<[Option<VBState>]>>,

    // 各种 MAX
    max_attribute_slots: usize,         // glow::MAX_VERTEX_ATTRIBS
    max_textures_slots: usize,          // glow::MAX_TEXTURE_IMAGE_UNITS
    max_color_attachments: usize,       // glow::MAX_COLOR_ATTACHMENTS
    max_uniform_buffer_bindings: usize, // glow::MAX_UNIFORM_BUFFER_BINDINGS 同时帮到Program的UBO的最大数量

    // 全局 GL 状态
    // VAO = render_pipeline.attributes + vertex_buffers + index_buffer
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

    is_depth_test_enable: bool,
    is_stencil_test_enable: bool,

    stencil_ref: i32,
    blend_color: [f32; 4],

    // 长度 不会 超过 max_uniform_buffer_bindings
    ubos: Box<[Option<UBOState>]>,

    active_texture_unit: u32, // 当前激活的纹理单元

    // 长度 不会 超过 max_textures_slots
    textures: Box<
        [(
            Option<(u32, glow::Texture)>, // (target, glow::Texture)
            Option<glow::Sampler>,        // glow::Sampler
        )],
    >,

    group_dirty: usize,
}


#[derive(Debug, Clone, PartialEq, Eq)]
struct UBOState {
    buffer: glow::Buffer,
    offset: i32,
    size: i32,
}

impl GLStateImpl {

    fn new(gl: &glow::Context) -> Self {
        // 一个 Program 能同时接受的 UBO 绑定的个数
        // PC Chrome 浏览器 24
        // MAX_VERTEX_UNIFORM_BLOCKS / MAX_FRAGMENT_UNIFORM_BLOCKS 各 12 个
        let max_uniform_buffer_bindings =
            unsafe { gl.get_parameter_i32(glow::MAX_UNIFORM_BUFFER_BINDINGS) as usize };

        // 一个 VS 能接受的 最大 Attribute 数量
        // PC Chrome 浏览器 16
        let max_attribute_slots =
            unsafe { gl.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) as usize };

        // 一个 FS 能接受的最多 Texture 通道数
        // PC Chrome 浏览器 16
        let max_textures_slots =
            unsafe { gl.get_parameter_i32(glow::MAX_TEXTURE_IMAGE_UNITS) as usize };

        // 一个 FS 能接受的最多 颜色 Attachement 的 数量
        // PC Chrome 浏览器 8
        let max_color_attachments =
            unsafe { gl.get_parameter_i32(glow::MAX_COLOR_ATTACHMENTS) as usize };

        let cache = GLCache::new(max_uniform_buffer_bindings, max_textures_slots);

        let is_depth_test_enable = false;
        Self::apply_depth_test_enable(gl, is_depth_test_enable);

        let is_stencil_test_enable = false;
        Self::apply_stencil_test_enable(gl, is_stencil_test_enable);

        let ubos = vec![None; max_uniform_buffer_bindings];
        let textures = vec![(None, None); max_textures_slots];

        Self {
            active_texture_unit: 0,

            global_shader_id: 0,
            last_vbs: None,

            max_attribute_slots,
            max_uniform_buffer_bindings,
            max_textures_slots,
            max_color_attachments,

            cache,

            render_pipeline: None,
            vertex_buffers: vec![None; max_attribute_slots].into_boxed_slice(),

            index_buffer: None,

            bind_group_set: [None, None, None, None],

            is_depth_test_enable,
            is_stencil_test_enable,

            viewport: Default::default(),
            scissor: Default::default(),

            clear_color: super::super::Color::default(),
            clear_depth: 1.0,
            clear_stencil: 0,

            blend_color: [0.0; 4],
            stencil_ref: 0,

            ubos: ubos.into_boxed_slice(),
            textures: textures.into_boxed_slice(),
            group_dirty: 0,
        }
    }

    fn reset_state(&mut self, gl: &glow::Context) {
        self.is_depth_test_enable = false;
        Self::apply_depth_test_enable(gl, self.is_depth_test_enable);

        self.is_stencil_test_enable = false;
        Self::apply_stencil_test_enable(gl, self.is_stencil_test_enable);

        // 重置清屏颜色
        let clear_color = wgt::Color::default();
        if self.clear_color != clear_color {
            unsafe {
                gl.clear_color(
                    clear_color.r as f32,
                    clear_color.g as f32,
                    clear_color.b as f32,
                    clear_color.a as f32,
                );
            }
            self.clear_color = clear_color;
        }
        unsafe {
            gl.clear_depth_f32(1.0);
        }
        self.clear_depth = 1.0;

        unsafe {
            gl.clear_stencil(0);
        }
        self.clear_stencil = 0;

        self.viewport = Default::default();
        self.scissor = Default::default();
        unsafe { gl.viewport(self.viewport.x, self.viewport.y, self.viewport.w, self.viewport.h) };
        unsafe { gl.viewport(self.scissor.x, self.scissor.y, self.scissor.w, self.scissor.h) };
        
        for i in self.vertex_buffers.iter_mut() {
            *i = None;
        }
        self.cache.reset_vao_state(gl);

        for (i, item) in self.textures.iter_mut().enumerate() {
            if let Some((target, _)) = &mut item.0 {
                unsafe {
                    self.active_texture_unit = i as u32;
                    gl.active_texture(glow::TEXTURE0 + i as u32);
                    gl.bind_texture(*target, None);
                }
                item.0 = None;
            }

            if item.1.is_some() {
                unsafe { gl.bind_sampler(i as u32, None) };
                item.1 = None;
            }
        }
        for i in self.ubos.iter_mut() {
            *i = None;
        }
       
        
        self.render_pipeline = None;
        self.index_buffer = None;
        self.active_texture_unit = 0;
    }

    #[inline]
    fn set_buffer_size(&self, gl: &glow::Context, buffer: &super::BufferImpl, size: i32) {
        unsafe {
            gl.bind_vertex_array(None);

            gl.bind_buffer(buffer.gl_target, Some(buffer.raw));
            gl.buffer_data_size(buffer.gl_target, size, buffer.gl_usage);

            self.cache.restore_current_vao(gl);
        }
    }

    #[inline]
    fn set_buffer_sub_data(
        &self,
        gl: &glow::Context,
        buffer: &super::BufferImpl,
        offset: i32,
        data: &[u8],
    ) {
        unsafe {
            gl.bind_vertex_array(None);

            gl.bind_buffer(buffer.gl_target, Some(buffer.raw));
            gl.buffer_sub_data_u8_slice(buffer.gl_target, offset, data);

            self.cache.restore_current_vao(gl);
        }
    }

    fn set_render_pipeline(&mut self, gl: &glow::Context, pipeline: &super::RenderPipeline) {
        if self.render_pipeline.is_none() {
            // 旧的没有，全部设置
            profiling::scope!("hal::GLState::apply_render_pipeline");

            let new = pipeline.0.as_ref();

            Self::apply_alpha_to_coverage(gl, new.alpha_to_coverage_enabled);
            Self::apply_color_mask(gl, &new.color_writes);

            Self::apply_program(gl, Some(&new.program));

            Self::apply_raster(gl, &new.rs.imp);
            Self::apply_depth(gl, &new.ds.imp);
            Self::apply_stencil(gl, self.stencil_ref, &new.ss.imp);
            Self::apply_blend(gl, &new.bs.imp);
        } else {
            // 有旧的，比较 Arc 指针

            let old = self.render_pipeline.as_ref().unwrap();
            if Share::ptr_eq(&pipeline.0, &old.0) {
                return;
            }

            let new = pipeline.0.as_ref();
            let old = old.0.as_ref();

            if new.alpha_to_coverage_enabled != old.alpha_to_coverage_enabled {
                Self::apply_alpha_to_coverage(gl, new.alpha_to_coverage_enabled);
            }

            if new.color_writes != old.color_writes {
                Self::apply_color_mask(gl, &new.color_writes);
            }

            if new.program.get_raw() != old.program.get_raw() {
                Self::apply_program(gl, Some(&new.program));
            }

            if !Share::ptr_eq(&new.rs, &old.rs) {
                Self::set_raster(gl, &new.rs.imp, &old.rs.imp);
            }

            if !Share::ptr_eq(&new.ds, &old.ds) {
                Self::set_depth(gl, &new.ds.imp, &old.ds.imp);
            }

            if !Share::ptr_eq(&new.ss, &old.ss) {
                Self::set_stencil(gl, self.stencil_ref, &new.ss.imp, &old.ss.imp);
            }

            if !Share::ptr_eq(&new.bs, &old.bs) {
                Self::set_blend(gl, &new.bs.imp, &old.bs.imp);
            }
        }

        self.render_pipeline = Some(pipeline.clone());
    }

    // 设置 FBO，设置 Viewport & Scissor，清屏
    fn set_render_target(&mut self, gl: &glow::Context, desc: &super::super::RenderPassDescriptor) {
        // TODO 不支持 多目标 渲染
        assert!(desc.color_attachments.len() == 1);

        let color = desc.color_attachments[0].as_ref().unwrap();

        // TODO 不支持 多重采样
        assert!(color.resolve_target.is_none());

        let (depth_stencil, depth_ops, stencil_ops) = match &desc.depth_stencil_attachment {
            None => (None, None, None),
            Some(ds) => (
                GLTextureInfo::try_from(ds.view).ok(),
                ds.depth_ops,
                ds.stencil_ops,
            ),
        };

        let colors: GLTextureInfo = color.view.into();

        // 移除 所有 self.textures 中 含 colors 和 depth_stencil 的 纹理单元
        self.reset_unit_texture(gl, &colors);

        match depth_stencil.as_ref() {
            Some(ds) => self.reset_unit_texture(gl, ds),
            None => {}
        };

        let render_target = super::RenderTarget {
            depth_stencil,
            colors,
        };

        self.cache.bind_fbo(gl, &render_target);

        // 视口 & 裁剪
        let size = color.view.get_size();
        self.set_viewport(gl, 0, 0, size.0 as i32, size.1 as i32);
        self.set_scissor(gl, 0, 0, size.0 as i32, size.1 as i32);

        // 清屏
        self.clear_render_target(
            gl,
            &color.ops.load,
            depth_ops.as_ref().map(|d| &d.load),
            stencil_ops.as_ref().map(|s| &s.load),
        );
    }

    // 重置纹理单元，避免 渲染目标 和 纹理 同时 绑定
    fn reset_unit_texture(&mut self, gl: &glow::Context, unit: &GLTextureInfo) {
        match unit {
            GLTextureInfo::Texture(c) => {
                for (i, (t, _)) in self.textures.iter_mut().enumerate() {
                    let need_update = match t.as_ref() {
                        None => false,
                        Some((target, tex)) => {
                            if *c == *tex {
                                unsafe {
                                    self.active_texture_unit = i as u32;
                                    gl.active_texture(glow::TEXTURE0 + i as u32);

                                    gl.bind_texture(*target, None);
                                }
                            }
                            *c == *tex
                        }
                    };

                    if need_update {
                        *t = None;
                    }
                }
            }
            _ => {}
        };
    }

    #[inline]
    fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        assert!(index < super::MAX_BIND_GROUPS as u32);

        let mut contents = Vec::with_capacity(bind_group.contents.len());
        for b in bind_group.contents.iter() {
            contents.push(b.into());
        }

        let bg = BindGroupState {
            bgs: contents.into_boxed_slice(),
            dynamic_offsets: dynamic_offsets.to_vec().into_boxed_slice(),
        };

        self.bind_group_set[index as usize] = Some(bg);
        self.group_dirty |= 1 << index;
    }

    #[inline]
    fn set_vertex_buffer(
        &mut self,
        index: usize,
        buffer: &super::Buffer,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        debug_assert!(buffer.0.gl_target == glow::ARRAY_BUFFER);

        let raw = buffer.0.raw;
        let offset = offset;
        let size = match size {
            None => buffer.0.size,
            Some(size) => u64::from(size) as i32,
        };

        self.vertex_buffers[index] = Some(VBState { raw, offset, size });
    }

    fn set_index_buffer(
        &mut self,
        _gl: &glow::Context,
        buffer: &super::Buffer,
        format: wgt::IndexFormat,
        offset: i32,
        size: Option<BufferSize>,
    ) {
        debug_assert!(buffer.0.gl_target == glow::ELEMENT_ARRAY_BUFFER);

        let (item_count, item_type) = conv::map_index_format(format);

        let raw = buffer.0.raw;
        let ib_type = item_type;
        let ib_count = item_count;

        let size = match size {
            None => buffer.0.size,
            Some(size) => u64::from(size) as i32,
        };

        self.index_buffer = Some(IBState {
            raw,
            ib_type,
            ib_count,
            size,
            offset,
        });
    }

    fn draw(
        &mut self,
        gl: &glow::Context,
        private_caps: PrivateCapabilities,
        first_vertex: u32,
        vertex_count: u32,
        first_instance: u32,
        instance_count: u32,
    ) {
        self.before_draw(gl, first_instance);

        let rp = self.render_pipeline.as_ref().unwrap().0.as_ref();

        // let supports_full_instancing = private_caps.contains(PrivateCapabilities::FULLY_FEATURED_INSTANCING);
        // let topology = rp.topology;

        // if instance_count == 1 {
        //     unsafe { gl.draw_arrays(rp.topology, start_vertex as i32, vertex_count as i32) };
        // } else {
            unsafe {
                gl.draw_arrays_instanced(
                    rp.topology,
                    first_vertex as i32,
                    vertex_count as i32,
                    instance_count as i32,
                )
            };
        // }

        // if supports_full_instancing {
        //     unsafe {
        //         gl.draw_arrays_instanced_base_instance(
        //             topology,
        //             first_vertex as i32,
        //             vertex_count as i32,
        //             instance_count as i32,
        //             first_instance,
        //         )
        //     }
        // } else {

        //     // Don't use `gl.draw_arrays` for `instance_count == 1`.
        //     // Angle has a bug where it doesn't consider the instance divisor when `DYNAMIC_DRAW` is used in `draw_arrays`.
        //     // See https://github.com/gfx-rs/wgpu/issues/3578
        //     unsafe {
        //         gl.draw_arrays_instanced(
        //             topology,
        //             first_vertex as i32,
        //             vertex_count as i32,
        //             instance_count as i32,
        //         )
        //     }
        // };

        self.after_draw(gl);
    }

    fn draw_indexed(
        &mut self,
        gl: &glow::Context,
        start_index: i32,
        index_count: i32,
        first_instance: u32,
        instance_count: i32,
    ) {
        self.before_draw(gl, first_instance);

        let rp = self.render_pipeline.as_ref().unwrap().0.as_ref();

        let ib = self.index_buffer.as_ref().unwrap();

        let offset = ib.offset + start_index * ib.ib_count;

        if instance_count == 1 {
            unsafe {
                gl.draw_elements(rp.topology, index_count, ib.ib_type, offset);
            }
        } else {
            unsafe {
                gl.draw_elements_instanced(
                    rp.topology,
                    index_count,
                    ib.ib_type,
                    offset,
                    instance_count,
                )
            }
        }

        self.after_draw(gl);
    }

    #[inline]
    fn set_viewport(&mut self, gl: &glow::Context, x: i32, y: i32, w: i32, h: i32) {
        let vp = &mut self.viewport;

        if x != vp.x || y != vp.y || w != vp.w || h != vp.h {
            unsafe { gl.viewport(x, y, w, h) };

            vp.x = x;
            vp.y = y;
            vp.w = w;
            vp.h = h;
        }
    }

    #[inline]
    fn set_scissor(&mut self, gl: &glow::Context, x: i32, y: i32, w: i32, h: i32) {
        let s = &mut self.scissor;

        if !s.is_enable {
            unsafe { gl.enable(glow::SCISSOR_TEST) };
            s.is_enable = true;
        }

        if x != s.x || y != s.y || w != s.w || h != s.h {
            unsafe { gl.scissor(x, y, w, h) };

            s.x = x;
            s.y = y;
            s.w = w;
            s.h = h;
        }
    }

    #[inline]
    fn set_depth_range(&mut self, gl: &glow::Context, n: f32, f: f32) {
        let vp = &mut self.viewport;

        if n != vp.min_depth || f != vp.max_depth {
            unsafe { gl.depth_range_f32(n, f) };

            vp.min_depth = n;
            vp.max_depth = f;
        }
    }

    #[inline]
    fn set_blend_color(&mut self, gl: &glow::Context, color: &[f32; 4]) {
        if self.blend_color[0] != color[0]
            || self.blend_color[1] != color[1]
            || self.blend_color[2] != color[2]
            || self.blend_color[3] != color[3]
        {
            unsafe { gl.blend_color(color[0], color[1], color[2], color[3]) };

            self.blend_color[0] = color[0];
            self.blend_color[1] = color[1];
            self.blend_color[2] = color[2];
            self.blend_color[3] = color[3];
        }
    }

    #[inline]
    fn set_stencil_reference(&mut self, gl: &glow::Context, reference: i32) {
        if reference == self.stencil_ref {
            return;
        }

        if let Some(p) = self.render_pipeline.as_ref() {
            let ss = &p.0.ss.as_ref().imp;

            unsafe {
                gl.stencil_func_separate(glow::FRONT, ss.front.test_func, reference, ss.mask_read);

                gl.stencil_func_separate(glow::BACK, ss.back.test_func, reference, ss.mask_read);
            }
        }
        self.stencil_ref = reference;
    }

    fn compile_shader(
        &mut self,
        gl: &glow::Context,
        shader: &super::ShaderModule,
        shader_stage: naga::ShaderStage,
        version: &glow::Version,
        features: &wgt::Features,
        downlevel: &wgt::DownlevelCapabilities,
        entry_point: String,
        multiview: Option<std::num::NonZeroU32>,
        naga_options: &naga::back::glsl::Options,
    ) -> Result<(), super::ShaderError> {
        // 如果编译过了，直接返回
        if self.cache.get_shader(shader.id).is_some() {
            return Ok(());
        }

        let mut module: Option<naga::Module> = None;

        let module_ref: &naga::Module = match &shader.input {
            super::ShaderInput::Naga(module) => module,
            super::ShaderInput::Glsl {
                shader,
                stage,
                defines,
            } => {
                assert!(*stage == shader_stage);

                let options = naga::front::glsl::Options {
                    stage: *stage,
                    defines: defines.clone(),
                };
                let mut parser = naga::front::glsl::Frontend::default();

                let m = parser.parse(&options, shader).map_err(|e| {
                    super::ShaderError::Compilation(format!("naga compile shader err = {:?}", e))
                })?;

                module = Some(m);
                module.as_ref().unwrap()
            }
        };

        let entry_point_index = module_ref
            .entry_points
            .iter()
            .position(|ep| ep.name.as_str() == entry_point)
            .ok_or(super::ShaderError::Compilation(
                "Shader not find entry point".to_string(),
            ))?;

        let info = get_shader_info(module_ref, features, downlevel)?;

        let (gl_str, reflection_info) = compile_naga_shader(
            module_ref,
            version,
            &info,
            shader_stage,
            entry_point,
            naga_options,
            multiview,
        )?;

        let shader_type = match shader_stage {
            naga::ShaderStage::Vertex => glow::VERTEX_SHADER,
            naga::ShaderStage::Fragment => glow::FRAGMENT_SHADER,
            naga::ShaderStage::Compute => unreachable!(),
        };

        let raw = compile_gl_shader(gl, gl_str.as_ref(), shader_type)?;

        let bg_set_info = self.consume_naga_reflection(
            module_ref,
            &info.get_entry_point(entry_point_index),
            reflection_info,
        )?;

        self.cache.insert_shader(
            shader.id,
            super::ShaderInner {
                raw,
                shader_type,
                bg_set_info,
            },
        );

        Ok(())
    }

    fn create_program(
        &mut self,
        gl: &glow::Context,
        vs_id: ShaderID,
        fs_id: ShaderID,
    ) -> Result<(glow::Program, Box<[Box<[super::PiBindEntry]>]>), super::ShaderError> {
        let vs_inner = self.cache.get_shader(vs_id).unwrap();
        let fs_inner = self.cache.get_shader(fs_id).unwrap();

        assert!(vs_inner.shader_type == glow::VERTEX_SHADER);
        assert!(fs_inner.shader_type == glow::FRAGMENT_SHADER);

        let raw = unsafe {
            let raw = gl.create_program().unwrap();

            gl.attach_shader(raw, vs_inner.raw);
            gl.attach_shader(raw, fs_inner.raw);

            gl.link_program(raw);

            if !gl.get_program_link_status(raw) {
                let info = gl.get_program_info_log(raw);

                log::error!("program link error, info = {:?}", info);

                gl.delete_program(raw);

                return Err(super::ShaderError::LinkProgram(format!(
                    "program link error, info = {:?}",
                    info
                )));
            }

            raw
        };

        let mut us: [Vec<super::PiBindEntry>; super::MAX_BIND_GROUPS] =
            [vec![], vec![], vec![], vec![]];
        let mut max_set: i32 = -1;

        unsafe {
            gl.use_program(Some(raw));
        }

        vs_inner
            .bg_set_info
            .iter()
            .enumerate()
            .chain(fs_inner.bg_set_info.iter().enumerate())
            .for_each(|(index, bg)| {
                if max_set < index as i32 {
                    max_set = index as i32;
                }

                let us = &mut us[index];

                bg.iter().for_each(|entry| {
                    if us.iter().all(|v| v.binding != entry.binding) {
                        us.push(entry.clone());
                    }

                    match entry.ty {
                        super::PiBindingType::Buffer => unsafe {
                            let loc = gl
                                .get_uniform_block_index(raw, entry.glsl_name.as_ref())
                                .unwrap();

                            gl.uniform_block_binding(raw, loc, entry.glow_binding as u32);
                        },
                        super::PiBindingType::Sampler => unsafe {
                            let loc = gl.get_uniform_location(raw, entry.glsl_name.as_ref());

                            gl.uniform_1_i32(loc.as_ref(), entry.glow_binding as i32);
                        },
                        super::PiBindingType::Texture => {}
                    }
                });
            });

        match self.render_pipeline.as_ref() {
            Some(p) => Self::apply_program(gl, Some(&p.0.program)),
            None => Self::apply_program(gl, None),
        }

        max_set += 1;
        let max_set = max_set as usize;
        let mut uniforms: Vec<Box<[super::PiBindEntry]>> = Vec::with_capacity(max_set);

        for i in 0..max_set {
            let v: Vec<_> = us[i].drain(..).collect();
            uniforms.push(v.into_boxed_slice());
        }

        Ok((raw, uniforms.into_boxed_slice()))
    }

    fn consume_naga_reflection(
        &mut self,
        module: &naga::Module,
        ep_info: &naga::valid::FunctionInfo,
        reflection_info: naga::back::glsl::ReflectionInfo,
    ) -> Result<Box<[Box<[super::PiBindEntry]>]>, super::ShaderError> {
        let mut r = [vec![], vec![], vec![], vec![]];
        let mut max_set: i32 = -1;

        // UBO
        for (handle, name) in reflection_info.uniforms {
            let var = &module.global_variables[handle];
            let br = var.binding.as_ref().unwrap();

            let pi_br = super::PiResourceBinding {
                group: br.group,
                binding: br.binding,
            };

            let glow_binding = self.cache.update_ubo(pi_br);

            if br.group as i32 > max_set {
                max_set = br.group as i32;
            }
            let set = &mut r[br.group as usize];
            set.push(super::PiBindEntry {
                binding: br.binding as usize,
                ty: super::PiBindingType::Buffer,

                glsl_name: name,
                glow_binding,
            });
        }

        // println!("texture_mapping====");
        // for (name, mapping) in reflection_info.texture_mapping.iter() {
        //     println!("texture_mapping1===={:?}, {:?}", name, (
        //         mapping.sampler.is_some(), 
        //         mapping.sampler.map(|r| {&module.global_variables[r].binding}),
        //         &module.global_variables[mapping.texture].binding,
        //     ));
        // }
        let mut temp = XHashMap::default();

        // Sampler / Texture
        for (name, mapping) in reflection_info.texture_mapping {
            assert!(mapping.sampler.is_some());

            let tex_var = &module.global_variables[mapping.texture];
            let tex_br = tex_var.binding.as_ref().unwrap();
            if tex_br.group as i32 > max_set {
                max_set = tex_br.group as i32;
            }

            let pi_br = super::PiResourceBinding {
                group: tex_br.group,
                binding: tex_br.binding,
            };
            let glow_binding = self.cache.update_sampler(pi_br);

            let set = &mut r[tex_br.group as usize];
            if !temp.contains_key(&(tex_br.group, tex_br.binding)) {
                set.push(super::PiBindEntry {
                    binding: tex_br.binding as usize,
                    ty: PiBindingType::Texture,
    
                    glsl_name: name.clone(),
                    glow_binding,
                });
                temp.insert((tex_br.group, tex_br.binding), ());
            } 

            let sampler_handle = mapping.sampler.unwrap();
            let sampler_var = &module.global_variables[sampler_handle];
            let sampler_br = sampler_var.binding.as_ref().unwrap();

            if sampler_br.group as i32 > max_set {
                max_set = sampler_br.group as i32;
            }
            let set = &mut r[sampler_br.group as usize];
            if !temp.contains_key(&(sampler_br.group, sampler_br.binding)) {
                set.push(super::PiBindEntry {
                    binding: sampler_br.binding as usize,
                    ty: PiBindingType::Sampler,
    
                    glsl_name: name,
                    glow_binding,
                });
                temp.insert((sampler_br.group, sampler_br.binding), ());
    
            }
            
            
        }

        let max_set = max_set + 1;
        let max_set = max_set as usize;
        let mut us = Vec::with_capacity(max_set);
        for i in 0..max_set {
            let v: Vec<_> = r[i].drain(..).collect();
            us.push(v.into_boxed_slice());
        }
        Ok(us.into_boxed_slice())
    }

    #[inline]
    fn before_draw(&mut self, gl: &glow::Context, first_instance: u32) {
        self.update_vao(gl, first_instance);

        self.update_uniforms(gl);
    }

    #[inline]
    fn after_draw(&mut self, _gl: &glow::Context) {}

    // 根据 render_pipeline.attributes + vertex_buffers 更新 vao
    fn update_vao(&mut self, gl: &glow::Context, first_instance: u32) {
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

        let geometry = super::GeometryState::new(
            rp.attributes.clone(),
            vbs,
            self.index_buffer.as_ref().map(|ib| ib.raw),
            first_instance,
        );

        self.cache.bind_vao(gl, &geometry);
        

        // 回收 vbs
        self.last_vbs = Some(geometry.vbs);
    }

    fn remove_buffer(&mut self, gl: &glow::Context, bind_target: u32, buffer: glow::Buffer) {
        if bind_target == glow::UNIFORM_BUFFER {
            for ubo in self.ubos.iter_mut() {
                let need_update = if let Some(u) = ubo {
                    u.buffer == buffer
                } else {
                    false
                };

                if need_update {
                    *ubo = None;
                }
            }
        } else {
            self.cache.remove_buffer(gl, bind_target, buffer);
        }
    }

    fn restore_current_texture(&mut self, gl: &glow::Context, unit: u32, target: u32) {
        let (slot, _) = &self.textures[unit as usize];

        match slot {
            None => unsafe {
                self.active_texture_unit = unit;
                gl.active_texture(glow::TEXTURE0 + unit);
                gl.bind_texture(target, None);
            },
            Some((target, raw)) => unsafe {
                self.active_texture_unit = unit;
                gl.active_texture(glow::TEXTURE0 + unit);
                gl.bind_texture(*target, Some(*raw));
            },
        }
    }

    fn remove_texture(&mut self, gl: &glow::Context, tex: glow::Texture) {
        self.cache.remove_texture(gl, tex);

        for (slot, _) in self.textures.iter_mut() {
            let update = if let Some((_, t)) = slot {
                *t == tex
            } else {
                false
            };

            if update {
                *slot = None;
            }
        }
    }

    fn remove_sampler(&mut self, gl: &glow::Context, sampler: glow::Sampler) {
        for (_, slot) in self.textures.iter_mut() {
            let update = if let Some(s) = slot {
                *s == sampler
            } else {
                false
            };

            if update {
                *slot = None;
            }
        }
    }

    // 根据 render_pipeline.program + bind_group 更新 uniform
    fn update_uniforms(&mut self, gl: &glow::Context) {
        let program = &self.render_pipeline.as_ref().unwrap().0.program;

        let program = program.0.as_ref();

        let bg_set = &mut self.bind_group_set;

        let reorder = &self.render_pipeline.as_ref().unwrap().0.layout_reoder;

        for (i, bindings) in program.uniforms.iter().enumerate() {
            if self.group_dirty & (1 << i) == 0 {
                continue;
            }
            let bg = &bg_set[i];
            if bg.is_none() {
                // assert!(bindings.len() == 0);
                continue;
            }
            let bg = bg.as_ref().unwrap();

            let reorder = &reorder[i];
            for (j, binding) in bindings.iter().enumerate() {
                let index = reorder[j];

                match &bg.bgs[index] {
                    RawBindingState::Buffer {
                        raw,
                        dynamic_offset,
                        offset,
                        size,
                    } => unsafe {
                        assert!(binding.ty == PiBindingType::Buffer);
                        let inner = raw.upgrade().unwrap();
                        let imp = inner.as_ref();

                        let offset = if *dynamic_offset >= 0 {
                            *offset + bg.dynamic_offsets[*dynamic_offset as usize] as i32
                        } else {
                            *offset
                        };

                        let state = UBOState {
                            buffer: imp.raw,
                            offset,
                            size: *size,
                        };

                        let need_update = match self.ubos[binding.glow_binding as usize].as_ref() {
                            None => true,
                            Some(s) => s.buffer != imp.raw || s.offset != offset || s.size != *size,
                        };

                        if need_update {
                            self.ubos[binding.glow_binding as usize] = Some(state);

                            if offset == 0 && *size == imp.size {
                                gl.bind_buffer_base(
                                    glow::UNIFORM_BUFFER,
                                    binding.glow_binding,
                                    Some(imp.raw),
                                );
                            } else {
                                gl.bind_buffer_range(
                                    glow::UNIFORM_BUFFER,
                                    binding.glow_binding,
                                    Some(imp.raw),
                                    offset,
                                    *size,
                                );
                            }
                        }
                    },
                    RawBindingState::Texture { raw: raw_ref } => unsafe {
                        assert!(binding.ty == PiBindingType::Texture);
                        let inner = raw_ref.upgrade().unwrap();
                        let imp = inner.as_ref();
                        match &imp.inner {
                            hal::TextureInner::Texture { raw, target, .. } => {
                                let need_update = match self.textures[binding.glow_binding as usize]
                                {
                                    (None, _) => true,
                                    (Some((old_target, old_texture)), _) => {
                                        old_target != *target || old_texture != *raw
                                    }
                                };
                                if need_update {
                                    self.textures[binding.glow_binding as usize].0 =
                                        Some((*target, *raw));

                                    if self.active_texture_unit != binding.glow_binding {
                                        self.active_texture_unit = binding.glow_binding;
                                        gl.active_texture(glow::TEXTURE0 + binding.glow_binding);
                                    }
                                    gl.bind_texture(*target, Some(*raw));
                                }
                            }
                            _ => panic!("mis match texture size"),
                        }
                    },
                    RawBindingState::Sampler { raw } => unsafe {
                        assert!(binding.ty == PiBindingType::Sampler);

                        let sampler = raw.upgrade().unwrap();

                        let imp = sampler.as_ref();

                        let need_update = match self.textures[binding.glow_binding as usize] {
                            (_, None) => true,
                            (_, Some(old_sampler)) => old_sampler != imp.raw,
                        };
                        if need_update {
                            self.textures[binding.glow_binding as usize].1 = Some(imp.raw);

                            gl.bind_sampler(binding.glow_binding, Some(imp.raw));
                        }
                    },
                }
            }
        }
        self.group_dirty = 0;
    }

    fn clear_render_target(
        &mut self,
        gl: &glow::Context,
        color: &super::super::LoadOp<super::super::Color>,
        depth: Option<&super::super::LoadOp<f32>>,
        stencil: Option<&super::super::LoadOp<u32>>,
    ) {
        profiling::scope!("hal::GLState::clear_render_target");

        let mut clear_mask = 0;

        let state = self
            .render_pipeline
            .as_ref()
            .map(|rp| rp.0.as_ref())
            .map(|imp| (imp.ds.as_ref().imp.is_write_enable, imp.color_writes));

        if let super::super::LoadOp::Clear(color) = color {
            clear_mask |= glow::COLOR_BUFFER_BIT;

            if let Some((_, color_writes)) = state {
                if color_writes != ColorWrites::ALL {
                    // clear 受到 color_mask 的 影响
                    unsafe {
                        gl.color_mask(true, true, true, true);
                    }
                }
            }

            if self.clear_color != *color {
                unsafe {
                    gl.clear_color(
                        color.r as f32,
                        color.g as f32,
                        color.b as f32,
                        color.a as f32,
                    );
                }
                self.clear_color = *color;
            }
        }

        match depth {
            None => self.set_depth_test_enable(gl, false),
            Some(ds_ops) => {
                self.set_depth_test_enable(gl, true);

                if let super::super::LoadOp::Clear(depth) = ds_ops {
                    clear_mask |= glow::DEPTH_BUFFER_BIT;

                    // 深度 clear 受 depth-mask 的 影响
                    match state {
                        Some((mask, _)) => unsafe {
                            if !mask {
                                gl.depth_mask(true);
                            }
                        },
                        None => unsafe {
                            gl.depth_mask(true);
                        },
                    }

                    if self.clear_depth != *depth {
                        unsafe {
                            gl.clear_depth_f32(*depth);
                        }
                        self.clear_depth = *depth;
                    }
                }
            }
        }

        match stencil {
            None => self.set_stencil_test_enable(gl, false),
            Some(stencil_ops) => {
                self.set_stencil_test_enable(gl, true);

                if let super::super::LoadOp::Clear(stencil) = &stencil_ops {
                    clear_mask |= glow::STENCIL_BUFFER_BIT;
                    if self.clear_stencil != *stencil {
                        unsafe {
                            gl.clear_stencil(*stencil as i32);
                        }
                        self.clear_stencil = *stencil;
                    }
                }
            }
        }

        if clear_mask != 0 {
            if self.scissor.is_enable {
                unsafe {
                    // 清屏 受到 裁剪的影响，而 wgpu 的接口预期 清 全屏
                    gl.disable(glow::SCISSOR_TEST);
                }
            }

            unsafe {
                gl.clear(clear_mask);
            }

            if self.scissor.is_enable {
                unsafe {
                    gl.enable(glow::SCISSOR_TEST);
                }
            }
        }

        if clear_mask & glow::COLOR_BUFFER_BIT != 0 {
            if let Some((_, color_writes)) = state {
                if color_writes != ColorWrites::ALL {
                    Self::apply_color_mask(gl, &color_writes);
                }
            }
        }

        if clear_mask & glow::DEPTH_BUFFER_BIT != 0 {
            match state {
                Some((mask, _)) => unsafe {
                    if !mask {
                        gl.depth_mask(false);
                    }
                },
                None => unsafe {
                    gl.depth_mask(false);
                },
            }
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
    fn apply_program(gl: &glow::Context, program: Option<&super::Program>) {
        match program {
            Some(p) => unsafe {
                let program = p.0.as_ref();
                gl.use_program(Some(program.raw));
            },
            None => unsafe {
                gl.use_program(None);
            },
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
        Self::apply_depth_write_enable(gl, new);
        Self::apply_depth_test_function(gl, new);
        Self::apply_depth_bias(gl, &new.depth_bias);
    }

    #[inline]
    fn apply_stencil(gl: &glow::Context, stencil_ref: i32, new: &super::StencilStateImpl) {
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
    fn set_depth_test_enable(&mut self, gl: &glow::Context, is_enable: bool) {
        if self.is_depth_test_enable != is_enable {
            self.is_depth_test_enable = is_enable;
            Self::apply_depth_test_enable(gl, is_enable);
        }
    }

    #[inline]
    fn set_stencil_test_enable(&mut self, gl: &glow::Context, is_enable: bool) {
        if self.is_stencil_test_enable != is_enable {
            self.is_stencil_test_enable = is_enable;
            Self::apply_stencil_test_enable(gl, is_enable);
        }
    }

    #[inline]
    fn apply_depth_test_enable(gl: &glow::Context, is_enable: bool) {
        if is_enable {
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
    fn apply_stencil_test_enable(gl: &glow::Context, is_enable: bool) {
        if is_enable {
            unsafe {
                gl.enable(glow::STENCIL_TEST);
            }
        } else {
            unsafe {
                gl.disable(glow::STENCIL_TEST);
            }
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct AttributeState {
    pub(crate) vb_count: usize, // vertex_buffers 的 前多少个 VB 对这个 Attribute 有效
    pub(crate) info: Box<[Option<AttributeInfo>]>,
    pub(crate) hash: u64,
}

impl AttributeState {
    #[inline]
    pub(crate) fn new(max_vertex_attributes: usize, vb_count: usize) -> Self {
        let mut hash = DefaultHasher::default();
        let info = vec![None; max_vertex_attributes].into_boxed_slice();
        vb_count.hash(&mut hash);
        info.hash(&mut hash);
        Self {
            vb_count,
            info,
            hash: hash.finish(),
        }
    }
}
impl Hash for AttributeState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
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

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct RenderTarget {
    pub(crate) depth_stencil: Option<GLTextureInfo>,
    pub(crate) colors: GLTextureInfo,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum GLTextureInfo {
    NativeRenderBuffer,

    Renderbuffer(glow::Renderbuffer),

    Texture(glow::Texture),
}

impl From<&super::super::TextureView> for GLTextureInfo {
    fn from(value: &super::super::TextureView) -> Self {
        match &value.inner.inner.inner {
            super::TextureInner::NativeRenderBuffer => Self::NativeRenderBuffer,

            super::TextureInner::Renderbuffer { raw, .. } => Self::Renderbuffer(*raw),

            super::TextureInner::Texture { raw, .. } => Self::Texture(*raw),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RawBindingState {
    Buffer {
        raw: ShareWeak<super::BufferImpl>,
        dynamic_offset: i32, // 如果没有，等于 -1
        offset: i32,
        size: i32,
    },
    Texture {
        raw: ShareWeak<super::TextureImpl>,
    },
    Sampler {
        raw: ShareWeak<super::SamplerImpl>,
    },
}

impl From<&super::RawBinding> for RawBindingState {
    fn from(value: &super::RawBinding) -> Self {
        match value {
            super::RawBinding::Buffer {
                raw,
                dynamic_offset,
                offset,
                size,
            } => Self::Buffer {
                raw: Share::downgrade(&raw.0),
                dynamic_offset: *dynamic_offset,
                offset: *offset,
                size: *size,
            },
            super::RawBinding::Texture(view) => Self::Texture {
                raw: Share::downgrade(&view.inner),
            },
            super::RawBinding::Sampler(sampler) => Self::Sampler {
                raw: Share::downgrade(&sampler.0),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BindGroupState {
    bgs: Box<[RawBindingState]>,

    dynamic_offsets: Box<[wgt::DynamicOffset]>,
}

fn get_shader_info(
    module: &naga::Module,
    features: &wgt::Features,
    downlevel: &wgt::DownlevelCapabilities,
) -> Result<ModuleInfo, super::ShaderError> {
    let mut caps = Caps::empty();
    caps.set(
        Caps::PUSH_CONSTANT,
        features.contains(wgt::Features::PUSH_CONSTANTS),
    );
    caps.set(Caps::FLOAT64, features.contains(wgt::Features::SHADER_F64));
    caps.set(
        Caps::PRIMITIVE_INDEX,
        features.contains(wgt::Features::SHADER_PRIMITIVE_INDEX),
    );
    caps.set(
        Caps::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING),
    );
    caps.set(
        Caps::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING),
    );
    caps.set(
        Caps::SAMPLER_NON_UNIFORM_INDEXING,
        features
            .contains(wgt::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING),
    );
    caps.set(
        Caps::STORAGE_TEXTURE_16BIT_NORM_FORMATS,
        features.contains(wgt::Features::TEXTURE_FORMAT_16BIT_NORM),
    );
    caps.set(Caps::MULTIVIEW, features.contains(wgt::Features::MULTIVIEW));
    caps.set(
        Caps::EARLY_DEPTH_TEST,
        features.contains(wgt::Features::SHADER_EARLY_DEPTH_TEST),
    );
    caps.set(
        Caps::MULTISAMPLED_SHADING,
        downlevel
            .flags
            .contains(wgt::DownlevelFlags::MULTISAMPLED_SHADING),
    );

    naga::valid::Validator::new(naga::valid::ValidationFlags::all(), caps)
        .validate(&module)
        .map_err(|e| super::ShaderError::Compilation(e.to_string()))
}

fn compile_naga_shader(
    module: &naga::Module,
    version: &glow::Version,
    module_info: &ModuleInfo,
    shader_stage: naga::ShaderStage,
    entry_point: String,
    naga_options: &naga::back::glsl::Options,
    multiview: Option<std::num::NonZeroU32>,
) -> Result<(String, ReflectionInfo), super::ShaderError> {
    println!("version = {:?}", version);

    let image_check = if !version.is_embedded && (version.major, version.minor) >= (1, 3) {
        BoundsCheckPolicy::ReadZeroSkipWrite
    } else {
        BoundsCheckPolicy::Unchecked
    };

    // Other bounds check are either provided by glsl or not implemented yet.
    let policies = naga::proc::BoundsCheckPolicies {
        index: BoundsCheckPolicy::Unchecked,
        buffer: BoundsCheckPolicy::Unchecked,
        binding_array: BoundsCheckPolicy::Unchecked,
        image_load: image_check,
        image_store: image_check,
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage,
        entry_point,
        multiview,
    };

    let mut output = String::new();
    let mut writer = glsl::Writer::new(
        &mut output,
        &module,
        &module_info,
        naga_options,
        &pipeline_options,
        policies,
    )
    .map_err(|e| super::ShaderError::Compilation(format!("glsl::Writer::new() error = {:?}", e)))?;

    let reflection_info = writer.write().map_err(|e| {
        super::ShaderError::Compilation(format!("glsl::Writer::write() error = {:?}", e))
    })?;

    Ok((output, reflection_info))
}

fn compile_gl_shader(
    gl: &glow::Context,
    source: &str,
    shader_type: u32,
) -> Result<glow::Shader, super::ShaderError> {
    let raw = unsafe {
        gl.create_shader(shader_type)
            .map_err(|e| super::ShaderError::Compilation("gl.create_shader error".to_string()))
    }?;

    unsafe { gl.shader_source(raw, source.as_ref()) };

    unsafe { gl.compile_shader(raw) };

    if unsafe { gl.get_shader_compile_status(raw) } {
        Ok(raw)
    } else {
        let info = unsafe { gl.get_shader_info_log(raw) };

        log::error!(
            "shader compile error, type = {:?}, info = {}, source = {}",
            shader_type,
            info,
            source
        );

        unsafe { gl.delete_shader(raw) };
        Err(super::ShaderError::Compilation(format!(
            "shader compile error, info = {:?}",
            info
        )))
    }
}
