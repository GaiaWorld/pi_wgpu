use super::{gl_conv as conv, AttributeState, GLState, ProgramID};
use crate::wgt;
use glow::HasContext;
use ordered_float::OrderedFloat;
use pi_share::Share;

#[derive(Debug)]
pub(crate) struct PipelineLayout {
    group_infos: Box<[BindGroupLayoutInfo]>,
}

#[derive(Debug)]
pub(crate) struct BindGroupLayoutInfo {
    entries: Share<[wgt::BindGroupLayoutEntry]>,

    /// Mapping of resources, indexed by `binding`, into the whole layout space.
    /// For texture resources, the value is the texture slot index.
    /// For sampler resources, the value is the index of the sampler in the whole layout.
    /// For buffers, the value is the uniform or storage slot index.
    /// For unused bindings, the value is `!0`
    binding_to_slot: Box<[u8]>,
}

impl PipelineLayout {
    pub fn new(
        state: GLState,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> Result<Self, crate::DeviceError> {
        let group_infos = desc
            .bind_group_layouts
            .iter()
            .map(|layout| BindGroupLayoutInfo {
                entries: layout.inner.entries.clone(),
                // TODO
                binding_to_slot: vec![].into(),
            })
            .collect();

        Ok(Self { group_infos })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RenderPipeline(pub(crate) Share<RenderPipelineImpl>);

#[derive(Debug)]
pub(crate) struct RenderPipelineImpl {
    pub(crate) topology: u32,
    pub(crate) alpha_to_coverage_enabled: bool,

    pub(crate) program: Share<super::Program>,

    pub(crate) color_writes: wgt::ColorWrites,
    pub(crate) attributes: super::AttributeState,

    pub(crate) rs: Share<super::RasterState>,
    pub(crate) ds: Share<super::DepthState>,
    pub(crate) bs: Share<super::BlendState>,
    pub(crate) ss: Share<super::StencilState>,
}

impl RenderPipelineImpl {
    pub fn new(
        state: GLState,
        desc: &crate::RenderPipelineDescriptor,
    ) -> Result<Self, super::PipelineError> {
        let topology = conv::map_primitive_topology(desc.primitive.topology);
        let alpha_to_coverage_enabled = desc.multisample.alpha_to_coverage_enabled;

        let fs = desc.fragment.as_ref().unwrap();
        let layout = desc.layout.map(|v| &v.inner);

        let program =
            Self::create_program(&state, &desc.vertex.module.inner, &fs.module.inner, layout)?;

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
        vs: &super::ShaderModule,
        fs: &super::ShaderModule,
        layout: Option<&super::PipelineLayout>,
    ) -> Result<Share<Program>, super::PipelineError> {
        match state.get_program(&(vs.id, fs.id)) {
            Some(program) => Ok(program),
            None => {
                let program = Program::new(state.clone(), vs, fs).unwrap();
                let program = Share::new(program);

                state.insert_program(program.id, program.clone());
                Ok(program)
            }
        }
    }

    fn create_attributes<'a>(
        buffers: &'a [crate::VertexBufferLayout<'a>],
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

    fn create_rs(state: &GLState, desc: &crate::PrimitiveState) -> Share<super::RasterState> {
        let (is_cull_enable, cull_face) = match desc.cull_mode {
            Some(wgt::Face::Front) => (true, glow::FRONT),
            Some(wgt::Face::Back) => (true, glow::BACK),
            None => (false, glow::BACK),
        };

        let front_face = match desc.front_face {
            crate::FrontFace::Ccw => glow::CCW,
            crate::FrontFace::Cw => glow::CW,
        };

        state.get_or_insert_rs(super::RasterStateImpl {
            is_cull_enable,
            front_face,
            cull_face,
        })
    }

    fn create_ds(
        state: &GLState,
        desc: Option<&crate::DepthStencilState>,
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
        desc: Option<&crate::DepthStencilState>,
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
        desc: &crate::FragmentState<'_>,
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
    fn create_stencil_face(desc: &crate::StencilFaceState) -> super::StencilFaceState {
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
    pub(crate) state: GLState,
    pub(crate) imp: BlendStateImpl,
}

impl BlendState {
    pub(crate) fn new(state: GLState, imp: BlendStateImpl) -> Self {
        Self { state, imp }
    }
}

impl Drop for BlendState {
    fn drop(&mut self) {
        self.state.remove_bs(&self.imp)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct BlendStateImpl {
    pub(crate) is_enable: bool,

    pub(crate) color: super::BlendComponent,
    pub(crate) alpha: super::BlendComponent,
}

impl Default for BlendStateImpl {
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
    pub(crate) state: GLState,
}

impl Drop for DepthState {
    fn drop(&mut self) {
        self.state.remove_ds(&self.imp);
    }
}

impl DepthState {
    pub(crate) fn new(state: GLState, imp: DepthStateImpl) -> Self {
        Self { state, imp }
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
    pub(crate) state: GLState,
}

impl Drop for RasterState {
    fn drop(&mut self) {
        self.state.remove_rs(&self.imp);
    }
}

impl RasterState {
    pub(crate) fn new(state: GLState, imp: RasterStateImpl) -> Self {
        Self { state, imp }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct RasterStateImpl {
    pub(crate) is_cull_enable: bool,
    pub(crate) front_face: u32, // glow::CW,  glow::CCW
    pub(crate) cull_face: u32,  // glow::FRONT, glow::BACK
}

impl Default for RasterStateImpl {
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
    pub(crate) state: GLState,
    pub(crate) imp: StencilStateImpl,
}

impl Drop for StencilState {
    fn drop(&mut self) {
        self.state.remove_ss(&self.imp);
    }
}

impl StencilState {
    pub(crate) fn new(state: GLState, imp: StencilStateImpl) -> Self {
        Self { state, imp }
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
    fn default() -> Self {
        Self {
            test_func: glow::ALWAYS,

            fail_op: glow::KEEP,
            zfail_op: glow::KEEP,
            zpass_op: glow::KEEP,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) id: ProgramID,

    pub(crate) raw: glow::Program,
    pub(crate) state: GLState,
}

impl Drop for Program {
    fn drop(&mut self) {
        let gl = self.state.get_gl();

        unsafe {
            gl.delete_program(self.raw);
        }
        self.state.remove_program(&self.id);
    }
}

impl Program {
    fn new(
        state: GLState,
        vs: &super::ShaderModule,
        fs: &super::ShaderModule,
    ) -> Result<Self, super::ShaderError> {
        assert!(vs.shader_type == glow::VERTEX_SHADER);
        assert!(fs.shader_type == glow::FRAGMENT_SHADER);

        let gl = state.get_gl();

        let raw = unsafe {
            let raw = gl.create_program().unwrap();

            gl.attach_shader(raw, vs.raw);
            gl.attach_shader(raw, fs.raw);

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

        Ok(Self {
            raw,
            state,
            id: (vs.id, fs.id),
        })
    }
}
