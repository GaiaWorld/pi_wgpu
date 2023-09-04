use std::collections::{HashMap, HashSet};

use glow::HasContext;
use pi_share::{Share, ShareCell, ShareWeak};
use twox_hash::RandomXxHashBuilder64;

use super::{
    AttributeState, BlendState, BlendStateImpl, DepthState, DepthStateImpl, GLState, RasterState,
    RasterStateImpl, RenderTarget, ShaderID, StencilState, StencilStateImpl, VBState,
};

pub(crate) type ProgramID = (ShaderID, ShaderID);

#[derive(Debug)]
pub(crate) struct GLCache {
    vao_map: HashMap<GeometryState, glow::VertexArray, RandomXxHashBuilder64>,
    fbo_map: HashMap<RenderTarget, glow::Framebuffer, RandomXxHashBuilder64>,

    program_map:
        HashMap<ProgramID, ShareWeak<ShareCell<super::ProgramImpl>>, RandomXxHashBuilder64>,
    bs_map: HashMap<BlendStateImpl, ShareWeak<BlendState>, RandomXxHashBuilder64>,
    rs_map: HashMap<RasterStateImpl, ShareWeak<RasterState>, RandomXxHashBuilder64>,
    ds_map: HashMap<DepthStateImpl, ShareWeak<DepthState>, RandomXxHashBuilder64>,
    ss_map: HashMap<StencilStateImpl, ShareWeak<StencilState>, RandomXxHashBuilder64>,
}

impl Default for GLCache {
    fn default() -> Self {
        Self {
            vao_map: Default::default(),
            fbo_map: Default::default(),
            program_map: Default::default(),

            bs_map: Default::default(),
            rs_map: Default::default(),
            ds_map: Default::default(),
            ss_map: Default::default(),
        }
    }
}

impl GLCache {
    pub fn get_or_insert_rs(&mut self, state: GLState, rs: RasterStateImpl) -> Share<RasterState> {
        profiling::scope!("hal::GLCache::get_or_insert_rs");
        // 尝试获取一个存在的Weak引用并升级
        if let Some(weak) = self.rs_map.get(&rs) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        // 如果没有找到或者无法升级，创建一个新的Share（Arc）并插入
        let new: RasterState = super::RasterState::new(state, rs.clone());
        let new_strong = Share::new(new);

        self.rs_map.insert(rs, Share::downgrade(&new_strong));

        new_strong
    }

    pub fn get_or_insert_bs(
        &mut self,
        state: GLState,
        bs: super::BlendStateImpl,
    ) -> Share<super::BlendState> {
        profiling::scope!("hal::GLCache::get_or_insert_bs");
        // 尝试获取一个存在的Weak引用并升级
        if let Some(weak) = self.bs_map.get(&bs) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        // 如果没有找到或者无法升级，创建一个新的Share（Arc）并插入
        let new = super::BlendState::new(state, bs.clone());
        let new_strong = Share::new(new);

        self.bs_map.insert(bs, Share::downgrade(&new_strong));

        new_strong
    }

    pub fn get_or_insert_ds(
        &mut self,
        state: GLState,
        ds: super::DepthStateImpl,
    ) -> Share<super::DepthState> {
        profiling::scope!("hal::GLCache::get_or_insert_ds");
        // 尝试获取一个存在的Weak引用并升级
        if let Some(weak) = self.ds_map.get(&ds) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        // 如果没有找到或者无法升级，创建一个新的Share（Arc）并插入
        let new = super::DepthState::new(state, ds.clone());
        let new_strong = Share::new(new);

        self.ds_map.insert(ds, Share::downgrade(&new_strong));

        new_strong
    }

    pub fn get_or_insert_ss(
        &mut self,
        state: GLState,
        ss: super::StencilStateImpl,
    ) -> Share<super::StencilState> {
        profiling::scope!("hal::GLCache::get_or_insert_ss");

        // 尝试获取一个存在的Weak引用并升级
        if let Some(weak) = self.ss_map.get(&ss) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        // 如果没有找到或者无法升级，创建一个新的Share（Arc）并插入
        let new = super::StencilState::new(state, ss.clone());
        let new_strong = Share::new(new);

        self.ss_map.insert(ss, Share::downgrade(&new_strong));

        new_strong
    }

    #[inline]
    pub fn remove_bs(&mut self, bs: &super::BlendStateImpl) {
        self.bs_map.remove(bs);
    }

    #[inline]
    pub fn remove_rs(&mut self, rs: &super::RasterStateImpl) {
        self.rs_map.remove(rs);
    }

    #[inline]
    pub fn remove_ds(&mut self, ds: &super::DepthStateImpl) {
        self.ds_map.remove(ds);
    }

    #[inline]
    pub fn remove_ss(&mut self, ss: &super::StencilStateImpl) {
        self.ss_map.remove(ss);
    }

    #[inline]
    pub fn get_program(&self, id: &super::ProgramID) -> Option<super::Program> {
        self.program_map.get(id).and_then(|p| {
            let p = p.upgrade();
            p.map(|p| super::Program(p))
        })
    }

    #[inline]
    pub fn insert_program(&mut self, id: super::ProgramID, program: super::Program) {
        self.program_map.insert(id, Share::downgrade(&program.0));
    }

    #[inline]
    pub fn remove_program(&mut self, id: &ProgramID) {
        self.program_map.remove(id);
    }

    pub fn bind_fbo(&mut self, gl: &glow::Context, render_target: &RenderTarget) {
        profiling::scope!("hal::GLCache::bind_fbo");

        match self.fbo_map.get(render_target) {
            Some(fbo) => unsafe {
                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(*fbo));
            },
            None => unsafe {
                let fbo = gl.create_framebuffer().unwrap();

                gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(fbo));

                match render_target.colors.as_ref().unwrap() {
                    crate::hal::GLTextureInfo::Renderbuffer(raw) => {
                        gl.framebuffer_renderbuffer(
                            glow::DRAW_FRAMEBUFFER,
                            glow::COLOR_ATTACHMENT0,
                            glow::RENDERBUFFER,
                            Some(*raw),
                        );
                    }
                    crate::hal::GLTextureInfo::Texture(raw) => {
                        gl.framebuffer_texture_2d(
                            glow::DRAW_FRAMEBUFFER,
                            glow::COLOR_ATTACHMENT0,
                            glow::TEXTURE_2D,
                            Some(*raw),
                            0,
                        );
                    }
                }

                if let Some(depth_stencil) = &render_target.depth_stencil {
                    match depth_stencil {
                        crate::hal::GLTextureInfo::Renderbuffer(raw) => {
                            gl.framebuffer_renderbuffer(
                                glow::DRAW_FRAMEBUFFER,
                                glow::DEPTH_ATTACHMENT, // GL_STENCIL_ATTACHMENT 会 自动绑定
                                glow::RENDERBUFFER,
                                Some(*raw),
                            );
                        }
                        crate::hal::GLTextureInfo::Texture(raw) => {
                            gl.framebuffer_texture_2d(
                                glow::DRAW_FRAMEBUFFER,
                                glow::DEPTH_ATTACHMENT, // GL_STENCIL_ATTACHMENT 会 自动绑定
                                glow::TEXTURE_2D,
                                Some(*raw),
                                0,
                            );
                        }
                    }
                }

                let status = gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER);
                if status != glow::FRAMEBUFFER_COMPLETE {
                    panic!("bind_fbo error, reason = {}", status);
                }

                self.fbo_map.insert(render_target.clone(), fbo);
            },
        }
    }

    pub fn bind_vao(&mut self, gl: &glow::Context, geometry: &super::GeometryState) {
        profiling::scope!("hal::GLCache::bind_vao");

        match self.vao_map.get(geometry) {
            Some(vao) => unsafe {
                gl.bind_vertex_array(Some(*vao));
            },
            None => unsafe {
                let vao = gl.create_vertex_array().unwrap();

                gl.bind_vertex_array(Some(vao));

                for (i, attrib) in geometry.attributes.info.iter().enumerate() {
                    let i = i as u32;

                    match attrib {
                        None => {
                            gl.disable_vertex_attrib_array(i);
                        }
                        Some(attrib) => {
                            gl.enable_vertex_attrib_array(i);
                            let vb = geometry.vbs[attrib.buffer_slot].as_ref().unwrap();

                            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vb.raw));

                            let offset = attrib.attrib_offset + vb.offset;

                            match attrib.attrib_kind {
                                super::VertexAttribKind::Float => {
                                    gl.vertex_attrib_pointer_f32(
                                        i,
                                        attrib.element_count,
                                        attrib.element_format,
                                        true, // always normalized
                                        attrib.attrib_stride,
                                        offset,
                                    );
                                }
                                super::VertexAttribKind::Integer => {
                                    gl.vertex_attrib_pointer_i32(
                                        i,
                                        attrib.element_count,
                                        attrib.element_format,
                                        attrib.attrib_stride,
                                        offset,
                                    );
                                }
                            }

                            // 实例化
                            let step = if attrib.is_buffer_instance { 1 } else { 0 };
                            gl.vertex_attrib_divisor(i, step);
                        }
                    }
                }

                self.vao_map.insert(geometry.clone(), vao);
            },
        }
    }

    pub fn remove_render_buffer(&self, gl: &glow::Context, rb: glow::Renderbuffer) {
        profiling::scope!("hal::GLCache::remove_render_buffer");

        let set = self
            .fbo_map
            .drain_filter(|k, fbo| {
                if let Some(super::GLTextureInfo::Renderbuffer(b)) = k.colors.as_ref() {
                    if *b == rb {
                        return true;
                    }
                }

                if let Some(super::GLTextureInfo::Renderbuffer(b)) = k.depth_stencil.as_ref() {
                    if *b == rb {
                        return true;
                    }
                }

                return false;
            })
            .map(|(k, v)| v)
            .collect::<HashSet<_>>();

        for fbo in set {
            unsafe {
                gl.delete_framebuffer(fbo);
            }
        }
    }

    pub fn remove_texture(&self, gl: &glow::Context, texture: glow::Texture) {
        profiling::scope!("hal::GLCache::remove_texture");

        let set = self
            .fbo_map
            .drain_filter(|k, _| {
                if let Some(super::GLTextureInfo::Texture(t)) = k.colors.as_ref() {
                    if *t == texture {
                        return true;
                    }
                }

                if let Some(super::GLTextureInfo::Texture(t)) = k.depth_stencil.as_ref() {
                    if *t == texture {
                        return true;
                    }
                }

                return false;
            })
            .map(|(k, v)| v)
            .collect::<HashSet<_>>();

        for fbo in set {
            unsafe {
                gl.delete_framebuffer(fbo);
            }
        }
    }

    pub fn remove_buffer(&self, gl: &glow::Context, bind_target: u32, buffer: glow::Buffer) {
        profiling::scope!("hal::GLCache::remove_buffer");

        assert!(bind_target == glow::ARRAY_BUFFER);

        let set = self
            .vao_map
            .drain_filter(|k, vao| {
                let mut r = false;
                for v in k.vbs.iter() {
                    if let Some(vb) = v.as_ref() {
                        if vb.raw == buffer {
                            r = true;
                            break;
                        }
                    }
                }
                return r;
            })
            .map(|(k, v)| v)
            .collect::<HashSet<_>>();

        for vao in set {
            unsafe {
                gl.delete_vertex_array(vao);
            }
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct GeometryState {
    pub(crate) attributes: AttributeState,
    pub(crate) vbs: Box<[Option<VBState>]>, // 长度 为 attributes.vb_count
}
