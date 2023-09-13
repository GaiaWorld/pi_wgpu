use glow::HasContext;
use naga::back::glsl;
use ordered_float::OrderedFloat;
use pi_share::Share;

use crate::pi_wgpu::hal::PiBindEntry;

use super::{super::wgt, gl_conv as conv, AdapterContext, AttributeState, GLState, ProgramID};

#[derive(Debug, Clone)]
pub(crate) struct PipelineLayout {
    pub(crate) group_infos: Box<[BindGroupLayoutInfo]>,
    pub(crate) naga_options: naga::back::glsl::Options,
}

#[derive(Debug, Clone)]
pub(crate) struct BindGroupLayoutInfo {
    entries: Share<[wgt::BindGroupLayoutEntry]>,
}

impl PipelineLayout {
    pub fn new(
        device_features: &wgt::Features,
        adapter: &AdapterContext,
        desc: &super::super::PipelineLayoutDescriptor,
    ) -> Result<Self, super::super::DeviceError> {
        let group_infos = desc
            .bind_group_layouts
            .iter()
            .map(|layout| BindGroupLayoutInfo {
                entries: layout.inner.entries.clone(),
            })
            .collect();

        let mut writer_flags = glsl::WriterFlags::ADJUST_COORDINATE_SPACE;
        writer_flags.set(
            glsl::WriterFlags::TEXTURE_SHADOW_LOD,
            adapter
                .private_caps()
                .contains(super::PrivateCapabilities::SHADER_TEXTURE_SHADOW_LOD),
        );
        // We always force point size to be written and it will be ignored by the driver if it's not a point list primitive.
        // https://github.com/gfx-rs/wgpu/pull/3440/files#r1095726950
        writer_flags.set(glsl::WriterFlags::FORCE_POINT_SIZE, true);

        let naga_options = glsl::Options {
            version: adapter.shading_language_version(),
            writer_flags,
            binding_map: Default::default(), // 自己分配槽位
            zero_initialize_workgroup_memory: true,
        };

        Ok(Self {
            group_infos,
            naga_options,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RenderPipeline(pub(crate) Share<RenderPipelineImpl>);

#[derive(Debug)]
pub(crate) struct RenderPipelineImpl {
    pub(crate) layout: PipelineLayout,
    pub(crate) layout_reoder: Box<[Box<[usize]>]>, // 根据 layout 和 program 重新映射的 binding
    pub(crate) topology: u32,
    pub(crate) alpha_to_coverage_enabled: bool,

    pub(crate) program: super::Program,

    pub(crate) color_writes: wgt::ColorWrites,
    pub(crate) attributes: super::AttributeState,

    pub(crate) rs: Share<super::RasterState>,
    pub(crate) ds: Share<super::DepthState>,
    pub(crate) bs: Share<super::BlendState>,
    pub(crate) ss: Share<super::StencilState>,
}

impl RenderPipelineImpl {
    pub fn new(
        state: &GLState,
        adapter: &AdapterContext,
        device_features: &wgt::Features,
        desc: &super::super::RenderPipelineDescriptor,
    ) -> Result<Self, super::PipelineError> {
        let topology = conv::map_primitive_topology(desc.primitive.topology);
        let alpha_to_coverage_enabled = desc.multisample.alpha_to_coverage_enabled;

        let vs = &desc.vertex;
        let fs = desc.fragment.as_ref().unwrap();

        let layout = desc.layout.as_ref().unwrap().inner.clone();

        let version = { adapter.lock().version().clone() };

        let naga_options = &layout.naga_options;

        {
            let gl = adapter.lock();
            state
                .compile_shader(
                    &gl,
                    &vs.module.inner,
                    naga::ShaderStage::Vertex,
                    &version,
                    device_features,
                    &adapter.downlevel(),
                    vs.entry_point.to_string(),
                    desc.multiview,
                    naga_options,
                )
                .map_err(|e| {
                    super::PipelineError::Linkage(wgt::ShaderStages::VERTEX, e.to_string())
                })?;

            state
                .compile_shader(
                    &gl,
                    &fs.module.inner,
                    naga::ShaderStage::Fragment,
                    &version,
                    device_features,
                    &adapter.downlevel(),
                    fs.entry_point.to_string(),
                    desc.multiview,
                    naga_options,
                )
                .map_err(|e| {
                    super::PipelineError::Linkage(wgt::ShaderStages::FRAGMENT, e.to_string())
                })?;
        }

        let program = Self::create_program(&state, adapter, &vs.module.inner, &fs.module.inner)?;

        let layout_reoder = program.reorder(&layout);

        let max_vertex_attributes = state.max_attribute_slots();

        let attributes = Self::create_attributes(desc.vertex.buffers, max_vertex_attributes);

        let rs = Self::create_rs(&state, &desc.primitive);
        let ds = Self::create_ds(&state, desc.depth_stencil.as_ref());
        let ss = Self::create_ss(&state, desc.depth_stencil.as_ref());

        let (bs, color_writes) = Self::create_bs(&state, fs);

        Ok(Self {
            topology,
            alpha_to_coverage_enabled,

            program,
            layout,
            layout_reoder,

            color_writes,
            attributes,

            rs,
            ds,
            bs,
            ss,
        })
    }
}

impl RenderPipelineImpl {
    fn create_program(
        state: &GLState,
        adapter: &AdapterContext,
        vs: &super::ShaderModule,
        fs: &super::ShaderModule,
    ) -> Result<Program, super::PipelineError> {
        let vs_id = vs.id;
        let fs_id = fs.id;

        match state.get_program(&(vs_id, fs_id)) {
            Some(program) => Ok(program),
            None => {
                let program = ProgramImpl::new(state, adapter, vs, fs).unwrap();

                let id = program.id;

                let program = Program(Share::new(program));

                state.insert_program(id, program.clone());
                Ok(program)
            }
        }
    }

    fn create_attributes<'a>(
        buffers: &'a [super::super::VertexBufferLayout<'a>],
        max_vertex_attributes: usize,
    ) -> AttributeState {
        let mut dst = AttributeState::new(max_vertex_attributes, buffers.len());

        for (i, v) in buffers.iter().enumerate() {
            debug_assert!(i < max_vertex_attributes);

            let is_buffer_instance = v.step_mode == wgt::VertexStepMode::Instance;

            for a in v.attributes.iter() {
                let desc = conv::describe_vertex_format(a.format);

                debug_assert!(a.shader_location < max_vertex_attributes as u32);

                dst.info[a.shader_location as usize] = Some(super::AttributeInfo {
                    buffer_slot: i,
                    is_buffer_instance,

                    element_count: desc.element_count,
                    element_format: desc.element_format,

                    attrib_stride: v.array_stride as i32,
                    attrib_offset: a.offset as i32,
                    attrib_kind: desc.attrib_kind,
                });
            }
        }

        dst
    }

    fn create_rs(
        state: &GLState,
        desc: &super::super::PrimitiveState,
    ) -> Share<super::RasterState> {
        let (is_cull_enable, cull_face) = match desc.cull_mode {
            Some(wgt::Face::Front) => (true, glow::FRONT),
            Some(wgt::Face::Back) => (true, glow::BACK),
            None => (false, glow::BACK),
        };

        let front_face = match desc.front_face {
            super::super::FrontFace::Ccw => glow::CCW,
            super::super::FrontFace::Cw => glow::CW,
        };

        state.get_or_insert_rs(super::RasterStateImpl {
            is_cull_enable,
            front_face,
            cull_face,
        })
    }

    fn create_ds(
        state: &GLState,
        desc: Option<&super::super::DepthStencilState>,
    ) -> Share<super::DepthState> {
        let ds = match desc {
            None => super::DepthStateImpl::default(),
            Some(d) => {
                let is_write_enable = d.depth_write_enabled;
                let function = conv::map_compare_func(d.depth_compare);
                let is_test_enable = d.depth_compare != wgt::CompareFunction::Always;

                let depth_bias = super::DepthBiasState {
                    constant: d.bias.constant,
                    slope_scale: OrderedFloat(d.bias.slope_scale),
                };

                super::DepthStateImpl {
                    is_test_enable,
                    is_write_enable,
                    function,
                    depth_bias,
                }
            }
        };

        state.get_or_insert_ds(ds)
    }

    fn create_ss(
        state: &GLState,
        desc: Option<&super::super::DepthStencilState>,
    ) -> Share<super::StencilState> {
        let ss = match desc {
            None => super::StencilStateImpl::default(),
            Some(s) => super::StencilStateImpl {
                is_enable: s.stencil.is_enabled(),
                mask_read: s.stencil.read_mask,
                mask_write: s.stencil.write_mask,
                front: Self::create_stencil_face(&s.stencil.front),
                back: Self::create_stencil_face(&s.stencil.back),
            },
        };

        state.get_or_insert_ss(ss)
    }

    fn create_bs(
        state: &GLState,
        desc: &super::super::FragmentState<'_>,
    ) -> (Share<super::BlendState>, wgt::ColorWrites) {
        assert!(desc.targets.len() < 2);

        let (bs, color_writes) = if desc.targets.len() == 0 || desc.targets[0].is_none() {
            (super::BlendStateImpl::default(), wgt::ColorWrites::ALL)
        } else {
            let b = desc.targets[0].as_ref().unwrap();

            let color_writes = b.write_mask;

            if b.blend.is_none() {
                (super::BlendStateImpl::default(), color_writes)
            } else {
                let b = b.blend.as_ref().unwrap();

                let is_enable = b.color != wgt::BlendComponent::REPLACE
                    || b.alpha != wgt::BlendComponent::REPLACE;

                let color = conv::map_blend_component(&b.color);
                let alpha = conv::map_blend_component(&b.alpha);

                (
                    super::BlendStateImpl {
                        is_enable,
                        color,
                        alpha,
                    },
                    color_writes,
                )
            }
        };

        let bs = state.get_or_insert_bs(bs);
        (bs, color_writes)
    }

    #[inline]
    fn create_stencil_face(desc: &super::super::StencilFaceState) -> super::StencilFaceState {
        super::StencilFaceState {
            test_func: conv::map_compare_func(desc.compare),
            fail_op: conv::map_stencil_op(desc.fail_op),
            zfail_op: conv::map_stencil_op(desc.depth_fail_op),
            zpass_op: conv::map_stencil_op(desc.pass_op),
        }
    }
}

#[derive(Debug)]
pub(crate) struct BlendState {
    pub(crate) imp: BlendStateImpl,
}

impl BlendState {
    #[inline]
    pub(crate) fn new(imp: BlendStateImpl) -> Self {
        Self { imp }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct BlendStateImpl {
    pub(crate) is_enable: bool,

    pub(crate) color: super::BlendComponent,
    pub(crate) alpha: super::BlendComponent,
}

impl Default for BlendStateImpl {
    #[inline]
    fn default() -> Self {
        Self {
            is_enable: true,
            color: Default::default(),
            alpha: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DepthState {
    pub(crate) imp: DepthStateImpl,
}

impl DepthState {
    #[inline]
    pub(crate) fn new(imp: DepthStateImpl) -> Self {
        Self { imp }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct DepthStateImpl {
    pub(crate) is_test_enable: bool,
    pub(crate) is_write_enable: bool,
    pub(crate) function: u32, // wgt::CompareFunction, map_compare_func

    pub(crate) depth_bias: DepthBiasState, // wgt::DepthBiasState,
}

impl Default for DepthStateImpl {
    #[inline]
    fn default() -> Self {
        Self {
            is_test_enable: false,
            is_write_enable: false,
            function: glow::ALWAYS,

            depth_bias: DepthBiasState::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub(crate) struct DepthBiasState {
    pub(crate) constant: i32,
    pub(crate) slope_scale: OrderedFloat<f32>,
}

#[derive(Debug)]
pub(crate) struct RasterState {
    pub(crate) imp: RasterStateImpl,
}

impl RasterState {
    #[inline]
    pub(crate) fn new(imp: RasterStateImpl) -> Self {
        Self { imp }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct RasterStateImpl {
    pub(crate) is_cull_enable: bool,
    pub(crate) front_face: u32, // glow::CW,  glow::CCW
    pub(crate) cull_face: u32,  // glow::FRONT, glow::BACK
}

impl Default for RasterStateImpl {
    #[inline]
    fn default() -> Self {
        Self {
            is_cull_enable: false,
            front_face: glow::CCW,
            cull_face: glow::BACK,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StencilState {
    pub(crate) imp: StencilStateImpl,
}

impl StencilState {
    #[inline]
    pub(crate) fn new(imp: StencilStateImpl) -> Self {
        Self { imp }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct StencilStateImpl {
    pub(crate) is_enable: bool,
    pub(crate) mask_read: u32,
    pub(crate) mask_write: u32,

    pub(crate) front: StencilFaceState,
    pub(crate) back: StencilFaceState,
}

impl Default for StencilStateImpl {
    #[inline]
    fn default() -> Self {
        Self {
            is_enable: false,
            mask_read: 0,
            mask_write: 0,

            front: Default::default(),
            back: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct StencilFaceState {
    pub(crate) test_func: u32, // wgt::CompareFunction, map_compare_func

    pub(crate) fail_op: u32,  // wgt::StencilOperation, map_stencil_op
    pub(crate) zfail_op: u32, // wgt::StencilOperation, map_stencil_op
    pub(crate) zpass_op: u32, // wgt::StencilOperation, map_stencil_op
}

impl Default for StencilFaceState {
    #[inline]
    fn default() -> Self {
        Self {
            test_func: glow::ALWAYS,

            fail_op: glow::KEEP,
            zfail_op: glow::KEEP,
            zpass_op: glow::KEEP,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Program(pub(crate) Share<ProgramImpl>);

impl Program {
    #[inline]
    pub(crate) fn get_id(&self) -> ProgramID {
        self.0.id
    }

    #[inline]
    pub(crate) fn get_raw(&self) -> glow::Program {
        self.0.raw
    }

    // 找 program 的 每个 binding 在 layout 的 索引
    pub(crate) fn reorder(&self, layout: &PipelineLayout) -> Box<[Box<[usize]>]> {
        let imp = &self.0.uniforms;

        let mut r = vec![];

        log::debug!("program reorder: layout = {:#?}, imp = {:#?}", layout, imp);

        for (i, info) in imp.iter().enumerate() {
            let mut v = vec![];

            let bg = &layout.group_infos[i].entries;

            for binding in info.iter() {
                let index = bg
                    .iter()
                    .position(|&x| x.binding as usize == binding.binding)
                    .unwrap();

                v.push(index);
            }

            r.push(v.into_boxed_slice());
        }

        r.into_boxed_slice()
    }
}

#[derive(Debug)]
pub(crate) struct ProgramImpl {
    pub(crate) id: ProgramID,

    pub(crate) raw: glow::Program,
    pub(crate) adapter: AdapterContext,

    // Box 中的顺序 和 RenderPipelineLayout 的 一样
    pub(crate) uniforms: Box<[Box<[PiBindEntry]>]>,
}

impl Drop for ProgramImpl {
    fn drop(&mut self) {
        let gl = self.adapter.lock();

        unsafe {
            gl.delete_program(self.raw);
        }
    }
}

impl ProgramImpl {
    fn new(
        state: &GLState,
        adapter: &AdapterContext,
        vs: &super::ShaderModule,
        fs: &super::ShaderModule,
    ) -> Result<Self, super::ShaderError> {
        let gl = adapter.lock();

        let (raw, uniforms) = state.create_program(&gl, vs.id, fs.id)?;

        Ok(Self {
            raw,
            adapter: adapter.clone(),
            id: (vs.id, fs.id),
            uniforms,
        })
    }
}
