//! 全局 GL 状态缓冲表
//!
//! 作用：状态机 分配资源时，先到这里找，找到了就返回，有利于设置时候，全局比较指针
//!

use derive_deref_rs::Deref;
use pi_time::Instant;
use std::{hash::{Hash, Hasher}, time::Duration};

use pi_hash::{DefaultHasher, XHashMap, XHashSet};
use glow::HasContext;
use pi_share::{cell::TrustCell, Share, ShareWeak};
use pi_assets::asset::{Asset, Garbageer, Size};

use super::{
    super::hal, AttributeState, BlendState, BlendStateImpl, DepthState, DepthStateImpl,
    RasterState, RasterStateImpl, RenderTarget, ShaderID, StencilState, StencilStateImpl, VBState,
};

pub(crate) type ProgramID = (ShaderID, ShaderID);

#[derive(Debug, PartialEq, Eq, Hash, Deref)]
pub struct VertexArrayAsset(pub glow::VertexArray);

impl Asset for VertexArrayAsset {
    type Key = u64;
}

impl Size for VertexArrayAsset {
    fn size(&self) -> usize {
        std::mem::align_of::<Self>()
    }
}

pub struct VaoGarbage(pub Share<TrustCell<Vec<glow::VertexArray>>>);

impl Garbageer<VertexArrayAsset> for VaoGarbage {
    fn garbage(&self, _k: <VertexArrayAsset as Asset>::Key, v: VertexArrayAsset, _timeout: u64) {
        self.0.borrow_mut().push(v.0);
    }
}
// #[derive(Debug)]
pub(crate) struct GLCache {
    last_clear_time: Instant,

    // vao: Option<Handle<VertexArrayAsset>>,
    vao: Option<glow::VertexArray>,

    shader_binding_map: super::ShaderBindingMap,
    // vao_map: Share<AssetMgr<VertexArrayAsset, VaoGarbage>>,
    vao_map: XHashMap<u64, glow::VertexArray>,
    garbage_vao: Share<TrustCell<Vec<glow::VertexArray>>>,
    buffer_vao_map: XHashMap<glow::Buffer, Vec<u64>>, // buffer 与vao资源key的对应关系
    fbo_map: XHashMap<RenderTarget, glow::Framebuffer>,
    shader_map: XHashMap<ShaderID, ShaderInner>,

    program_map: XHashMap<ProgramID, ShareWeak<super::ProgramImpl>>,
    bs_map: XHashMap<BlendStateImpl, ShareWeak<BlendState>>,
    rs_map: XHashMap<RasterStateImpl, ShareWeak<RasterState>>,
    ds_map: XHashMap<DepthStateImpl, ShareWeak<DepthState>>,
    ss_map: XHashMap<StencilStateImpl, ShareWeak<StencilState>>,
}


impl GLCache {
    #[inline]
    pub(crate) fn new(max_uniform_buffer_bindings: usize, max_textures_slots: usize) -> Self {
        let garbage_vao = Share::new(TrustCell::new(Vec::new()));
        // let vao_map = AssetMgr::new(VaoGarbage(garbage_vao.clone()), false, 20 * 1024, 10 * 1000); // 过期时间：10分钟
        // alloter.register(vao_map.clone(), 20 * 1024, 30 * 1024);
        let vao_map = XHashMap::default();
        Self {
            vao: None,

            last_clear_time: Instant::now(),
            vao_map,
            garbage_vao,
            buffer_vao_map: Default::default(),
            fbo_map: Default::default(),
            shader_map: Default::default(),

            program_map: Default::default(),

            bs_map: Default::default(),
            rs_map: Default::default(),
            ds_map: Default::default(),
            ss_map: Default::default(),

            shader_binding_map: super::ShaderBindingMap::new(
                max_uniform_buffer_bindings,
                max_textures_slots,
            ),
        }
    }

    pub(crate) fn reset_vao_state(&mut self, gl: &glow::Context) {
        if let Some(_) = self.vao {
            unsafe { gl.bind_vertex_array(None)};
        }
        self.vao = None;

    }

    pub(crate) fn clear_weak_refs(&mut self, gl: &glow::Context) {
        let now = Instant::now();
        if now - self.last_clear_time < Duration::from_secs(CLEAR_DURATION) {
            return;
        }
        self.last_clear_time = now;

        self.bs_map.retain(|_, v| v.upgrade().is_some());
        self.rs_map.retain(|_, v| v.upgrade().is_some());
        self.ds_map.retain(|_, v| v.upgrade().is_some());
        self.ss_map.retain(|_, v| v.upgrade().is_some());
        self.program_map.retain(|_, v| v.upgrade().is_some());

        // 回收vao
        let mut garbage_vao = self.garbage_vao.borrow_mut();
        if garbage_vao.len() > 0 {
            for vao in garbage_vao.drain(..) {
                unsafe {
                    gl.delete_vertex_array(vao);
                    if let Some(old) = &self.vao {
                        if *old == vao {
                            self.vao = None;
                            gl.bind_vertex_array(None);
                        }
                    }
                }
            }
        }
        
    }

    #[inline]
    pub(crate) fn get_shader(&self, id: ShaderID) -> Option<&ShaderInner> {
        self.shader_map.get(&id)
    }

    #[inline]
    pub(crate) fn insert_shader(&mut self, id: ShaderID, inner: ShaderInner) {
        self.shader_map.insert(id, inner);
    }

    #[inline]
    pub(crate) fn remove_shader(&mut self, id: ShaderID) {
        self.shader_map.remove(&id);
    }

    #[inline]
    pub(crate) fn update_ubo(&mut self, binding: super::PiResourceBinding) -> u32 {
        self.shader_binding_map.get_or_insert_ubo(binding)
    }

    #[inline]
    pub(crate) fn update_sampler(&mut self, binding: super::PiResourceBinding) -> u32 {
        self.shader_binding_map.get_or_insert_sampler(binding)
    }

    pub(crate) fn get_or_insert_rs(&mut self, rs: RasterStateImpl) -> Share<RasterState> {
        profiling::scope!("hal::GLCache::get_or_insert_rs");
        // 尝试获取一个存在的Weak引用并升级
        if let Some(weak) = self.rs_map.get(&rs) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        // 如果没有找到或者无法升级，创建一个新的Share（Arc）并插入
        let new: RasterState = super::RasterState::new(rs.clone());
        let new_strong = Share::new(new);

        self.rs_map.insert(rs, Share::downgrade(&new_strong));

        new_strong
    }

    pub(crate) fn get_or_insert_bs(
        &mut self,
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
        let new = super::BlendState::new(bs.clone());
        let new_strong = Share::new(new);

        self.bs_map.insert(bs, Share::downgrade(&new_strong));

        new_strong
    }

    pub(crate) fn get_or_insert_ds(
        &mut self,
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
        let new = super::DepthState::new(ds.clone());
        let new_strong = Share::new(new);

        self.ds_map.insert(ds, Share::downgrade(&new_strong));

        new_strong
    }

    pub(crate) fn get_or_insert_ss(
        &mut self,
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
        let new = super::StencilState::new(ss.clone());
        let new_strong = Share::new(new);

        self.ss_map.insert(ss, Share::downgrade(&new_strong));

        new_strong
    }

    #[inline]
    pub(crate) fn get_program(&self, id: &super::ProgramID) -> Option<super::Program> {
        self.program_map.get(id).and_then(|p| {
            let p = p.upgrade();
            p.map(|p| super::Program(p))
        })
    }

    #[inline]
    pub(crate) fn insert_program(&mut self, id: super::ProgramID, program: super::Program) {
        self.program_map.insert(id, Share::downgrade(&program.0));
    }

    pub(crate) fn bind_fbo(&mut self, gl: &glow::Context, render_target: &RenderTarget) {
        profiling::scope!("hal::GLCache::bind_fbo");

        match self.fbo_map.get(render_target) {
            Some(fbo) => unsafe {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(*fbo));
            },
            None => unsafe {
                if let hal::GLTextureInfo::NativeRenderBuffer = &render_target.colors {
                    gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                } else {
                    let fbo = gl.create_framebuffer().unwrap();

                    gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

                    match &render_target.colors {
                        hal::GLTextureInfo::NativeRenderBuffer => unreachable!(),
                        hal::GLTextureInfo::Renderbuffer(raw) => {
                            gl.framebuffer_renderbuffer(
                                glow::FRAMEBUFFER,
                                glow::COLOR_ATTACHMENT0,
                                glow::RENDERBUFFER,
                                Some(*raw),
                            );
                        }
                        hal::GLTextureInfo::Texture(raw) => {
                            gl.framebuffer_texture_2d(
                                glow::FRAMEBUFFER,
                                glow::COLOR_ATTACHMENT0,
                                glow::TEXTURE_2D,
                                Some(*raw),
                                0,
                            );
                        }
                    }

                    if let Some(depth_stencil) = &render_target.depth_stencil {
                        match depth_stencil {
                            hal::GLTextureInfo::NativeRenderBuffer => unreachable!(),
                            hal::GLTextureInfo::Renderbuffer(raw) => {
                                gl.framebuffer_renderbuffer(
                                    glow::FRAMEBUFFER,
                                    glow::DEPTH_ATTACHMENT, // GL_STENCIL_ATTACHMENT 会 自动绑定
                                    glow::RENDERBUFFER,
                                    Some(*raw),
                                );
                            }
                            hal::GLTextureInfo::Texture(raw) => {
                                gl.framebuffer_texture_2d(
                                    glow::FRAMEBUFFER,
                                    glow::DEPTH_ATTACHMENT, // GL_STENCIL_ATTACHMENT 会 自动绑定
                                    glow::TEXTURE_2D,
                                    Some(*raw),
                                    0,
                                );
                            }
                        }
                    }
                    let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
                    if status != glow::FRAMEBUFFER_COMPLETE {
                        panic!("bind_fbo error, reason = {}, colors: {:?}, depth: {:?}", status, &render_target.colors, &render_target.depth_stencil);
                    }

                    self.fbo_map.insert(render_target.clone(), fbo);
                }
            },
        }
    }

    #[inline]
    pub(crate) fn restore_current_vao(&self, gl: &glow::Context) {
        match &self.vao {
            Some(v) => unsafe {
                gl.bind_vertex_array(Some((*v).clone()));
            },
            None => unsafe {
                gl.bind_vertex_array(None);
            },
        };
    }

    pub(crate) fn bind_vao(&mut self, gl: &glow::Context, geometry: &super::GeometryState) {
        profiling::scope!("hal::GLCache::bind_vao");

        let hash = geometry.hash;

        match self.vao_map.get(&hash) {
            Some(vao) => unsafe {
                let need_update = match &self.vao {
                    Some(v) => *v != *vao,
                    None => true,
                };

                if need_update {
                    gl.bind_vertex_array(Some(*vao));
                    self.vao = Some(*vao);
                }
            },
            None => unsafe {
                let vao = gl.create_vertex_array().unwrap();
                gl.bind_vertex_array(Some(vao));

                // log::warn!("Creating new VAO: {:?}", &vao);

                // let vao = self.vao_map.insert(hash, VertexArrayAsset(vao)).unwrap();
                self.vao_map.insert(hash, vao.clone());
                self.vao = Some(vao);

                

                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, geometry.ib);

                // 创建ib与vao之间的关系
                if let Some(ib) = geometry.ib {
                    match self.buffer_vao_map.entry(ib) {
                        std::collections::hash_map::Entry::Occupied(mut r) => {r.get_mut().push(hash);},
                        std::collections::hash_map::Entry::Vacant(r) => {r.insert(vec![hash]);},
                    };
                }
                

                for (i, attrib) in geometry.attributes.info.iter().enumerate() {
                    let i = i as u32;

                    match attrib {
                        None => {
                            // TODO 有些低端的Android机子，可能需要显示设置
                            // gl.disable_vertex_attrib_array(i);
                        }
                        Some(attrib) => {
                            gl.enable_vertex_attrib_array(i);
                            let vb = geometry.vbs[attrib.buffer_slot].as_ref().unwrap();

                            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vb.raw));

                            // 创建vb与vao之间的关系
                            match self.buffer_vao_map.entry(vb.raw) {
                                std::collections::hash_map::Entry::Occupied(mut r) => {r.get_mut().push(hash);},
                                std::collections::hash_map::Entry::Vacant(r) => {r.insert(vec![hash]);},
                            };

                            let mut offset = attrib.attrib_offset + vb.offset;
                            if attrib.is_buffer_instance {
                                offset +=  geometry.first_instance as i32 * attrib.attrib_stride;
                            }

                            // log::warn!("bind buffer: {:?}", (offset, i, attrib.attrib_stride, attrib.element_count, &attrib.attrib_kind, vb.raw));

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
            },
        }
    }

    pub(crate) fn remove_render_buffer(&mut self, gl: &glow::Context, rb: glow::Renderbuffer) {
        profiling::scope!("hal::GLCache::remove_render_buffer");

        let set = self
            .fbo_map
            .extract_if(|k, _fbo| {
                if let super::GLTextureInfo::Renderbuffer(b) = &k.colors {
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
            .map(|(_, v)| v)
            .collect::<XHashSet<_>>();

        for fbo in set {
            unsafe {
                gl.delete_framebuffer(fbo);
            }
        }
    }

    pub(crate) fn remove_texture(&mut self, gl: &glow::Context, texture: glow::Texture) {
        profiling::scope!("hal::GLCache::remove_texture");

        let set = self
            .fbo_map
            .extract_if(|k, _| {
                if let super::GLTextureInfo::Texture(t) = &k.colors {
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
            .map(|(_, v)| v)
            .collect::<XHashSet<_>>();

        for fbo in set {
            unsafe {
                gl.delete_framebuffer(fbo);
            }
        }
    }

    pub(crate) fn remove_buffer(
        &mut self,
        gl: &glow::Context,
        bind_target: u32,
        buffer: glow::Buffer,
    ) {
        profiling::scope!("hal::GLCache::remove_buffer");
        if bind_target == glow::ARRAY_BUFFER || bind_target == glow::ELEMENT_ARRAY_BUFFER {
            if let Some(r) = self.buffer_vao_map.remove(&buffer) {
                for hash in r.into_iter() {
                    if let Some(vao) = self.vao_map.get(&hash) {
                        unsafe {
                            gl.delete_vertex_array(*vao);
                            if let Some(old) = &self.vao {
                                if *old == *vao {
                                    self.vao = None;
                                    gl.bind_vertex_array(None);
                                }
                            }
                        }

                        self.vao_map.remove(&hash);
                    }
                }
            }
        } else {
            unreachable!();
        }

        // let set: XHashSet<glow::VertexArray> = if bind_target == glow::ARRAY_BUFFER {
        //     self.vao_map
        //         .extract_if(|k, vao| {
        //             let mut r = false;
        //             for v in k.vbs.iter() {
        //                 if let Some(vb) = v.as_ref() {
        //                     if vb.raw == buffer {
        //                         r = true;

        //                         if let Some(old) = self.vao {
        //                             if old == *vao {
        //                                 self.vao = None;
        //                                 unsafe { gl.bind_vertex_array(None) };
        //                             }
        //                         }

        //                         break;
        //                     }
        //                 }
        //             }
        //             return r;
        //         })
        //         .map(|(_, v)| v)
        //         .collect::<XHashSet<_>>()
        // } else if bind_target == glow::ELEMENT_ARRAY_BUFFER {
        //     self.vao_map
        //         .extract_if(|k, vao| {
        //             let mut r = false;
        //             if let Some(ib) = &k.ib {
        //                 r = *ib == buffer;

        //                 if r {
        //                     if let Some(old) = self.vao {
        //                         if old == *vao {
        //                             self.vao = None;
        //                             unsafe { gl.bind_vertex_array(None) };
        //                         }
        //                     }
        //                 }
        //             }
        //             return r;
        //         })
        //         .map(|(_, v)| v)
        //         .collect::<XHashSet<_>>()
        // } else {
        //     unreachable!();
        // };
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct GeometryState {
    pub(crate) attributes: AttributeState,
    pub(crate) vbs: Box<[Option<VBState>]>, // 长度 为 attributes.vb_count
    pub(crate) ib: Option<glow::Buffer>,
    pub(crate) first_instance: u32,
    pub(crate) hash: u64,
}
impl GeometryState {
    pub fn new(attributes: AttributeState,
        vbs: Box<[Option<VBState>]>,
        ib: Option<glow::Buffer>,
        first_instance: u32,
    ) -> Self {
        let mut state = DefaultHasher::default();
        attributes.hash(&mut state);
        vbs.hash(&mut state);
        ib.hash(&mut state);
        first_instance.hash(&mut state);

        Self {
            attributes,
            vbs,
            ib,
            first_instance,
            hash: state.finish()
        }
    }
}
impl Hash for GeometryState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Debug)]
pub(crate) struct ShaderInner {
    pub(crate) raw: glow::Shader,
    pub(crate) shader_type: u32, // glow::VERTEX_SHADER,
    pub(crate) bg_set_info: Box<[Box<[super::PiBindEntry]>]>,
}

const CLEAR_DURATION: u64 = 20;
