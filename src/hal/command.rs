use std::{ops::Range, sync::Arc};

use arrayvec::ArrayVec;
use glow::HasContext;

use super::{AdapterContextLock, AdapterShared};
use crate::{wgt, Extent3d, LoadOp};

#[derive(Debug)]
pub(crate) struct CommandBuffer;

#[derive(Debug)]
pub(crate) struct CommandEncoder {
    state: State,
    gl: Arc<AdapterShared>,
}

impl CommandEncoder {
    pub(crate) unsafe fn begin_encoding(
        &mut self,
        label: super::Label,
    ) -> Result<(), super::DeviceError> {
        Ok(())
    }

    pub(crate) unsafe fn discard_encoding(&mut self) {
        unreachable!()
    }

    pub(crate) unsafe fn end_encoding(
        &mut self,
    ) -> Result<super::CommandBuffer, super::DeviceError> {
        Ok(CommandBuffer)
    }

    pub(crate) unsafe fn reset_all<I>(&mut self, _command_buffers: I) {
        unreachable!()
    }

    pub(crate) unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = super::BufferBarrier<'a, super::GL>>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = super::TextureBarrier<'a, super::GL>>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn clear_buffer(
        &mut self,
        buffer: &super::Buffer,
        range: super::MemoryRange,
    ) {
        unreachable!()
    }

    pub(crate) unsafe fn copy_buffer_to_buffer<T>(
        &mut self,
        src: &super::Buffer,
        dst: &super::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = super::BufferCopy>,
    {
        unreachable!()
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "emscripten")))]
    pub(crate) unsafe fn copy_external_image_to_texture<T>(
        &mut self,
        src: &wgt::ImageCopyExternalImage,
        dst: &super::Texture,
        dst_premultiplication: bool,
        regions: T,
    ) where
        T: Iterator<Item = super::TextureCopy>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &super::Texture,
        _src_usage: super::TextureUses,
        dst: &super::Texture,
        regions: T,
    ) where
        T: Iterator<Item = super::TextureCopy>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn copy_buffer_to_texture<T>(
        &mut self,
        src: &super::Buffer,
        dst: &super::Texture,
        regions: T,
    ) where
        T: Iterator<Item = super::BufferTextureCopy>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &super::Texture,
        _src_usage: super::TextureUses,
        dst: &super::Buffer,
        regions: T,
    ) where
        T: Iterator<Item = super::BufferTextureCopy>,
    {
        unreachable!()
    }

    pub(crate) unsafe fn begin_query(&mut self, set: &super::QuerySet, index: u32) {
        unreachable!()
    }

    pub(crate) unsafe fn end_query(&mut self, set: &super::QuerySet, _index: u32) {
        unreachable!()
    }

    pub(crate) unsafe fn write_timestamp(&mut self, _set: &super::QuerySet, _index: u32) {
        unimplemented!()
    }

    pub(crate) unsafe fn reset_queries(&mut self, _set: &super::QuerySet, _range: Range<u32>) {
        unreachable!()
    }

    pub(crate) unsafe fn copy_query_results(
        &mut self,
        set: &super::QuerySet,
        range: Range<u32>,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
        _stride: wgt::BufferSize,
    ) {
        unreachable!()
    }

    // render

    pub(crate) unsafe fn insert_debug_marker(&mut self, label: &str) {
        unreachable!()
    }

    pub(crate) unsafe fn begin_debug_marker(&mut self, group_label: &str) {
        unreachable!()
    }

    pub(crate) unsafe fn end_debug_marker(&mut self) {
        unreachable!()
    }

    // 绑定 FBO
    // 设 Viewport, Scissor
    // Clear
    unsafe fn begin_render_pass(&mut self, desc: &crate::RenderPassDescriptor) {
        let mut extent: Option<Extent3d> = None;
        desc.color_attachments
            .first()
            .filter(|at| at.is_some())
            .and_then(|at| {
                at.as_ref().map(|at| {
                    extent = Some(at.view.render_extent);
                })
            });

        let extent = extent.unwrap();
        self.state.render_size = extent;

        self.state.resolve_attachments.clear();
        self.state.invalidate_attachments.clear();

        let rendering_to_external_framebuffer = desc
            .color_attachments
            .iter()
            .filter_map(|at| at.as_ref())
            .any(|at| match at.view.inner {
                #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
                super::TextureInner::ExternalFramebuffer { .. } => true,
                _ => false,
            });

        if rendering_to_external_framebuffer && desc.color_attachments.len() != 1 {
            panic!("Multiple render attachments with external framebuffers are not supported.");
        }

        let gl = self.gl.context.lock();

        match desc
            .color_attachments
            .first()
            .filter(|at| at.is_some())
            .and_then(|at| at.as_ref().map(|at| &at.view.inner.inner))
        {
            // default framebuffer (provided externally)
            Some(&super::TextureInner::DefaultRenderbuffer) => {
                self.reset_framebuffer(&gl, true);
            }
            _ => {
                // set the framebuffer
                self.reset_framebuffer(&gl, false);

                for (i, cat) in desc.color_attachments.iter().enumerate() {
                    if let Some(cat) = cat.as_ref() {
                        let attachment = glow::COLOR_ATTACHMENT0 + i as u32;

                        self.bind_attachment(&gl, attachment, &cat.view.inner);

                        if let Some(ref rat) = cat.resolve_target {
                            self.state
                                .resolve_attachments
                                .push((attachment, rat.inner.clone()));
                        }

                        if !cat.ops.store {
                            self.state.invalidate_attachments.push(attachment);
                        }
                    }
                }
                if let Some(ref dsat) = desc.depth_stencil_attachment {
                    let aspects = dsat.view.inner.aspects;
                    let attachment = match aspects {
                        super::FormatAspects::DEPTH => glow::DEPTH_ATTACHMENT,
                        super::FormatAspects::STENCIL => glow::STENCIL_ATTACHMENT,
                        _ => glow::DEPTH_STENCIL_ATTACHMENT,
                    };

                    self.bind_attachment(&gl, attachment, &dsat.view.inner);

                    let contain_store =
                        dsat.depth_ops.is_some() && dsat.depth_ops.as_ref().unwrap().store;
                    if aspects.contains(super::FormatAspects::DEPTH) && !contain_store {
                        self.state
                            .invalidate_attachments
                            .push(glow::DEPTH_ATTACHMENT);
                    }

                    let contain_store =
                        dsat.stencil_ops.is_some() && dsat.stencil_ops.as_ref().unwrap().store;
                    if aspects.contains(super::FormatAspects::STENCIL) && !contain_store {
                        self.state
                            .invalidate_attachments
                            .push(glow::STENCIL_ATTACHMENT);
                    }
                }

                if !rendering_to_external_framebuffer {
                    // set the draw buffers and states
                    self.set_draw_color_bufers(&gl, desc.color_attachments.len() as u8);
                }
            }
        }

        let rect = super::Rect {
            x: 0,
            y: 0,
            w: extent.width as u32,
            h: extent.height as u32,
        };
        self.set_scissor_rect(&rect);

        let rect = super::Rect {
            x: 0.0,
            y: 0.0,
            w: extent.width as f32,
            h: extent.height as f32,
        };
        self.set_viewport(&rect, 0.0..1.0);

        // issue the clears
        for (i, cat) in desc
            .color_attachments
            .iter()
            .filter_map(|at| at.as_ref())
            .enumerate()
        {
            if let LoadOp::Clear(ref c) = cat.ops.load {
                match cat.view.inner.sample_type {
                    wgt::TextureSampleType::Float { .. } => {
                        self.clear_color_f(
                            &gl,
                            i as u32,
                            &[c.r as f32, c.g as f32, c.b as f32, c.a as f32],
                            cat.view.inner.format.describe().srgb,
                        );
                    }
                    wgt::TextureSampleType::Depth => {
                        unreachable!()
                    }
                    wgt::TextureSampleType::Uint => {
                        self.clear_color_u(
                            &gl,
                            i as u32,
                            &[c.r as u32, c.g as u32, c.b as u32, c.a as u32],
                        );
                    }
                    wgt::TextureSampleType::Sint => {
                        self.clear_color_i(
                            &gl,
                            i as u32,
                            &[c.r as i32, c.g as i32, c.b as i32, c.a as i32],
                        );
                    }
                }
            }
        }
        if let Some(ref dsat) = desc.depth_stencil_attachment {
            match (dsat.depth_ops, dsat.stencil_ops) {
                (Some(ref dops), Some(ref sops)) => match (dops.load, sops.load) {
                    (LoadOp::Clear(d), LoadOp::Clear(s)) => {
                        self.clear_color_depth_and_stencil(&gl, d, s);
                    }
                    (LoadOp::Clear(d), LoadOp::Load) => {
                        self.clear_depth(&gl, d);
                    }
                    (LoadOp::Load, LoadOp::Clear(s)) => {
                        self.clear_stencil(&gl, s as i32);
                    }
                    (LoadOp::Load, LoadOp::Load) => {}
                },
                (Some(ref dops), None) => {
                    if let LoadOp::Clear(d) = dops.load {
                        self.clear_depth(&gl, d);
                    }
                }
                (None, Some(ref sops)) => {
                    if let LoadOp::Clear(s) = sops.load {
                        self.clear_stencil(&gl, s as i32);
                    }
                }
                (None, None) => {}
            }
        }
    }

    pub(crate) unsafe fn end_render_pass(&mut self) {
        let gl = self.gl.context.lock();

        for (attachment, dst) in self.state.resolve_attachments.drain(..) {
            self.resolve_attachment(&gl, attachment, &dst, &self.state.render_size);
        }
        if !self.state.invalidate_attachments.is_empty() {
            let list = &self.state.invalidate_attachments;
            unsafe { gl.invalidate_framebuffer(glow::DRAW_FRAMEBUFFER, list) };

            self.state.invalidate_attachments.clear();
        }

        self.state.instance_vbuf_mask = 0;
        self.state.dirty_vbuf_mask = 0;
        self.state.active_first_instance = 0;
        self.state.color_targets.clear();

        for index in 0..self.state.vertex_attributes.len() {
            self.unset_vertex_attribute(&gl, index as u32);
        }
        self.state.vertex_attributes.clear();
        self.state.primitive = super::PrimitiveState::default();
    }

    pub(crate) unsafe fn set_bind_group(
        &mut self,
        layout: &super::PipelineLayout,
        index: u32,
        group: &super::BindGroup,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
        let mut do_index = 0;
        let mut dirty_textures = 0u32;
        let mut dirty_samplers = 0u32;
        let group_info = &layout.group_infos[index as usize];

        for (binding_layout, raw_binding) in group_info.entries.iter().zip(group.contents.iter()) {
            let slot = group_info.binding_to_slot[binding_layout.binding as usize] as u32;
            match *raw_binding {
                super::RawBinding::Buffer {
                    raw,
                    offset: base_offset,
                    size,
                } => {
                    let mut offset = base_offset;
                    let target = match binding_layout.ty {
                        wgt::BindingType::Buffer {
                            ty,
                            has_dynamic_offset,
                            min_binding_size: _,
                        } => {
                            if has_dynamic_offset {
                                offset += dynamic_offsets[do_index] as i32;
                                do_index += 1;
                            }
                            match ty {
                                wgt::BufferBindingType::Uniform => glow::UNIFORM_BUFFER,
                                wgt::BufferBindingType::Storage { .. } => {
                                    glow::SHADER_STORAGE_BUFFER
                                }
                            }
                        }
                        _ => unreachable!(),
                    };
                    self.cmd_buffer.commands.push(C::BindBuffer {
                        target,
                        slot,
                        buffer: raw,
                        offset,
                        size,
                    });
                }
                super::RawBinding::Sampler(sampler) => {
                    dirty_samplers |= 1 << slot;
                    self.state.samplers[slot as usize] = Some(sampler);
                }
                super::RawBinding::Texture { raw, target } => {
                    dirty_textures |= 1 << slot;
                    self.state.texture_slots[slot as usize].tex_target = target;
                    self.cmd_buffer.commands.push(C::BindTexture {
                        slot,
                        texture: raw,
                        target,
                    });
                }
                super::RawBinding::Image(ref binding) => {
                    self.cmd_buffer.commands.push(C::BindImage {
                        slot,
                        binding: binding.clone(),
                    });
                }
            }
        }

        self.rebind_sampler_states(dirty_textures, dirty_samplers);
    }

    pub(crate) unsafe fn set_push_constants(
        &mut self,
        _layout: &super::PipelineLayout,
        _stages: wgt::ShaderStages,
        start_offset: u32,
        data: &[u32],
    ) {
        unimplemented!("hal::CommandEncoder set_push_constants isn't impl")
    }

    pub(crate) unsafe fn set_render_pipeline(&mut self, pipeline: &super::RenderPipeline) {
        self.state.topology = conv::map_primitive_topology(pipeline.primitive.topology);

        if self
            .private_caps
            .contains(super::PrivateCapabilities::VERTEX_BUFFER_LAYOUT)
        {
            for vat in pipeline.vertex_attributes.iter() {
                let vb = &pipeline.vertex_buffers[vat.buffer_index as usize];
                // set the layout
                self.cmd_buffer.commands.push(C::SetVertexAttribute {
                    buffer: None,
                    buffer_desc: vb.clone(),
                    attribute_desc: vat.clone(),
                });
            }
        } else {
            for index in 0..self.state.vertex_attributes.len() {
                self.cmd_buffer
                    .commands
                    .push(C::UnsetVertexAttribute(index as u32));
            }
            self.state.vertex_attributes.clear();

            self.state.dirty_vbuf_mask = 0;
            // copy vertex attributes
            for vat in pipeline.vertex_attributes.iter() {
                //Note: we can invalidate more carefully here.
                self.state.dirty_vbuf_mask |= 1 << vat.buffer_index;
                self.state.vertex_attributes.push(vat.clone());
            }
        }

        self.state.instance_vbuf_mask = 0;
        // copy vertex state
        for (index, (&mut (ref mut state_desc, _), pipe_desc)) in self
            .state
            .vertex_buffers
            .iter_mut()
            .zip(pipeline.vertex_buffers.iter())
            .enumerate()
        {
            if pipe_desc.step == wgt::VertexStepMode::Instance {
                self.state.instance_vbuf_mask |= 1 << index;
            }
            if state_desc != pipe_desc {
                self.state.dirty_vbuf_mask |= 1 << index;
                *state_desc = pipe_desc.clone();
            }
        }

        self.set_pipeline_inner(&pipeline.inner);

        // set primitive state
        let prim_state = conv::map_primitive_state(&pipeline.primitive);
        if prim_state != self.state.primitive {
            self.cmd_buffer
                .commands
                .push(C::SetPrimitive(prim_state.clone()));
            self.state.primitive = prim_state;
        }

        // set depth/stencil states
        let mut aspects = super::FormatAspects::empty();
        if pipeline.depth_bias != self.state.depth_bias {
            self.state.depth_bias = pipeline.depth_bias;
            self.cmd_buffer
                .commands
                .push(C::SetDepthBias(pipeline.depth_bias));
        }
        if let Some(ref depth) = pipeline.depth {
            aspects |= super::FormatAspects::DEPTH;
            self.cmd_buffer.commands.push(C::SetDepth(depth.clone()));
        }
        if let Some(ref stencil) = pipeline.stencil {
            aspects |= super::FormatAspects::STENCIL;
            self.state.stencil = stencil.clone();
            self.rebind_stencil_func();
            if stencil.front.ops == stencil.back.ops
                && stencil.front.mask_write == stencil.back.mask_write
            {
                self.cmd_buffer.commands.push(C::SetStencilOps {
                    face: glow::FRONT_AND_BACK,
                    write_mask: stencil.front.mask_write,
                    ops: stencil.front.ops.clone(),
                });
            } else {
                self.cmd_buffer.commands.push(C::SetStencilOps {
                    face: glow::FRONT,
                    write_mask: stencil.front.mask_write,
                    ops: stencil.front.ops.clone(),
                });
                self.cmd_buffer.commands.push(C::SetStencilOps {
                    face: glow::BACK,
                    write_mask: stencil.back.mask_write,
                    ops: stencil.back.ops.clone(),
                });
            }
        }
        self.cmd_buffer
            .commands
            .push(C::ConfigureDepthStencil(aspects));

        // set multisampling state
        if pipeline.alpha_to_coverage_enabled != self.state.alpha_to_coverage_enabled {
            self.state.alpha_to_coverage_enabled = pipeline.alpha_to_coverage_enabled;
            self.cmd_buffer
                .commands
                .push(C::SetAlphaToCoverage(pipeline.alpha_to_coverage_enabled));
        }

        // set blend states
        if self.state.color_targets[..] != pipeline.color_targets[..] {
            if pipeline
                .color_targets
                .iter()
                .skip(1)
                .any(|ct| *ct != pipeline.color_targets[0])
            {
                for (index, ct) in pipeline.color_targets.iter().enumerate() {
                    self.cmd_buffer.commands.push(C::SetColorTarget {
                        draw_buffer_index: Some(index as u32),
                        desc: ct.clone(),
                    });
                }
            } else {
                self.cmd_buffer.commands.push(C::SetColorTarget {
                    draw_buffer_index: None,
                    desc: pipeline.color_targets.first().cloned().unwrap_or_default(),
                });
            }
        }
        self.state.color_targets.clear();
        for ct in pipeline.color_targets.iter() {
            self.state.color_targets.push(ct.clone());
        }
    }

    pub(crate) unsafe fn set_index_buffer<'a>(
        &mut self,
        binding: super::BufferBinding<'a, super::Api>,
        format: wgt::IndexFormat,
    ) {
        self.state.index_offset = binding.offset;
        self.state.index_format = format;
        self.cmd_buffer
            .commands
            .push(C::SetIndexBuffer(binding.buffer.raw.unwrap()));
    }

    pub(crate) unsafe fn set_vertex_buffer<'a>(
        &mut self,
        index: u32,
        binding: super::BufferBinding<'a, super::Api>,
    ) {
        self.state.dirty_vbuf_mask |= 1 << index;
        let (_, ref mut vb) = self.state.vertex_buffers[index as usize];
        *vb = Some(super::BufferBinding {
            raw: binding.buffer.raw.unwrap(),
            offset: binding.offset,
        });
    }

    pub(crate) unsafe fn set_viewport(&mut self, rect: &super::Rect<f32>, depth: Range<f32>) {
        self.cmd_buffer.commands.push(C::SetViewport {
            rect: super::Rect {
                x: rect.x as i32,
                y: rect.y as i32,
                w: rect.w as i32,
                h: rect.h as i32,
            },
            depth,
        });
    }

    pub(crate) unsafe fn set_scissor_rect(&mut self, rect: &super::Rect<u32>) {
        self.cmd_buffer.commands.push(C::SetScissor(super::Rect {
            x: rect.x as i32,
            y: rect.y as i32,
            w: rect.w as i32,
            h: rect.h as i32,
        }));
    }

    pub(crate) unsafe fn set_stencil_reference(&mut self, value: u32) {
        self.state.stencil.front.reference = value;
        self.state.stencil.back.reference = value;
        self.rebind_stencil_func();
    }

    pub(crate) unsafe fn set_blend_constants(&mut self, color: &[f32; 4]) {
        self.cmd_buffer.commands.push(C::SetBlendConstant(*color));
    }

    pub(crate) unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
        self.prepare_draw(start_instance);
        self.cmd_buffer.commands.push(C::Draw {
            topology: self.state.topology,
            start_vertex,
            vertex_count,
            instance_count,
        });
    }

    pub(crate) unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
        self.prepare_draw(start_instance);
        let (index_size, index_type) = match self.state.index_format {
            wgt::IndexFormat::Uint16 => (2, glow::UNSIGNED_SHORT),
            wgt::IndexFormat::Uint32 => (4, glow::UNSIGNED_INT),
        };
        let index_offset = self.state.index_offset + index_size * start_index as wgt::BufferAddress;
        self.cmd_buffer.commands.push(C::DrawIndexed {
            topology: self.state.topology,
            index_type,
            index_offset,
            index_count,
            base_vertex,
            instance_count,
        });
    }

    pub(crate) unsafe fn draw_indirect(
        &mut self,
        _buffer: &super::Buffer,
        _offset: wgt::BufferAddress,
        _draw_count: u32,
    ) {
        unreachable!()
    }

    pub(crate) unsafe fn draw_indexed_indirect(
        &mut self,
        _buffer: &super::Buffer,
        _offset: wgt::BufferAddress,
        _draw_count: u32,
    ) {
        unreachable!()
    }

    pub(crate) unsafe fn draw_indirect_count(
        &mut self,
        _buffer: &super::Buffer,
        _offset: wgt::BufferAddress,
        _count_buffer: &super::Buffer,
        _count_offset: wgt::BufferAddress,
        _max_count: u32,
    ) {
        unreachable!()
    }
    pub(crate) unsafe fn draw_indexed_indirect_count(
        &mut self,
        _buffer: &super::Buffer,
        _offset: wgt::BufferAddress,
        _count_buffer: &super::Buffer,
        _count_offset: wgt::BufferAddress,
        _max_count: u32,
    ) {
        unreachable!()
    }

    // compute

    pub(crate) unsafe fn begin_compute_pass(&mut self, _desc: &crate::ComputePassDescriptor) {
        unreachable!()
    }
    pub(crate) unsafe fn end_compute_pass(&mut self) {
        unreachable!()
    }

    pub(crate) unsafe fn set_compute_pipeline(&mut self, _pipeline: &crate::ComputePipeline) {
        unreachable!()
    }

    pub(crate) unsafe fn dispatch(&mut self, _count: [u32; 3]) {
        unreachable!()
    }

    pub(crate) unsafe fn dispatch_indirect(
        &mut self,
        buffer: &super::Buffer,
        offset: wgt::BufferAddress,
    ) {
        unreachable!()
    }
}

impl CommandEncoder {
    fn reset_framebuffer(&self, gl: &AdapterContextLock<'_>, is_default: bool) {
        if is_default {
            unsafe { gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, None) };
        } else {
            unsafe { gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.gl.context.draw_fbo)) };
            unsafe {
                gl.framebuffer_texture_2d(
                    glow::DRAW_FRAMEBUFFER,
                    glow::DEPTH_STENCIL_ATTACHMENT,
                    glow::TEXTURE_2D,
                    None,
                    0,
                )
            };
            for i in 0..super::MAX_COLOR_ATTACHMENTS {
                let target = glow::COLOR_ATTACHMENT0 + i as u32;
                unsafe {
                    gl.framebuffer_texture_2d(
                        glow::DRAW_FRAMEBUFFER,
                        target,
                        glow::TEXTURE_2D,
                        None,
                        0,
                    )
                };
            }
        }

        unsafe { gl.color_mask(true, true, true, true) };
        unsafe { gl.depth_mask(true) };
        unsafe { gl.stencil_mask(!0) };
        unsafe { gl.disable(glow::DEPTH_TEST) };
        unsafe { gl.disable(glow::STENCIL_TEST) };
        unsafe { gl.disable(glow::SCISSOR_TEST) };
    }

    fn bind_attachment(
        &self,
        gl: &AdapterContextLock<'_>,
        attachment: u32,
        view: &super::TextureView,
    ) {
        unsafe { self.set_attachment(gl, glow::DRAW_FRAMEBUFFER, attachment, view) };
    }

    fn resolve_attachment(
        &self,
        gl: &AdapterContextLock<'_>,
        attachment: u32,
        dst: &super::TextureView,
        size: &Extent3d,
    ) {
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.gl.context.draw_fbo)) };
        unsafe { gl.read_buffer(attachment) };
        unsafe { gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.gl.context.copy_fbo)) };
        unsafe { self.set_attachment(gl, glow::DRAW_FRAMEBUFFER, glow::COLOR_ATTACHMENT0, dst) };
        unsafe {
            gl.blit_framebuffer(
                0,
                0,
                size.width as i32,
                size.height as i32,
                0,
                0,
                size.width as i32,
                size.height as i32,
                glow::COLOR_BUFFER_BIT,
                glow::NEAREST,
            )
        };
        unsafe { gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None) };
        unsafe { gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.gl.context.draw_fbo)) };
    }

    fn set_draw_color_bufers(&mut self, gl: &AdapterContextLock<'_>, count: u8) {
        self.draw_buffer_count = count;

        let indices = (0..count as u32)
            .map(|i| glow::COLOR_ATTACHMENT0 + i)
            .collect::<ArrayVec<_, { super::MAX_COLOR_ATTACHMENTS }>>();

        unsafe { gl.draw_buffers(&indices) };
    }

    fn clear_color_f(
        &self,
        gl: &AdapterContextLock<'_>,
        draw_buffer: u32,
        color: &[f32; 4],
        is_srgb: bool,
    ) {
        unsafe { gl.clear_buffer_f32_slice(glow::COLOR, draw_buffer, color) };
    }

    fn clear_color_u(&self, gl: &AdapterContextLock<'_>, draw_buffer: u32, color: &[u32; 4]) {
        unsafe { gl.clear_buffer_u32_slice(glow::COLOR, draw_buffer, color) };
    }

    fn clear_color_i(&self, gl: &AdapterContextLock<'_>, draw_buffer: u32, color: &[i32; 4]) {
        unsafe { gl.clear_buffer_i32_slice(glow::COLOR, draw_buffer, color) };
    }

    fn clear_depth(&self, gl: &AdapterContextLock<'_>, depth: f32) {
        unsafe { gl.clear_buffer_f32_slice(glow::DEPTH, 0, &[depth]) };
    }

    fn clear_stencil(&self, gl: &AdapterContextLock<'_>, value: i32) {
        unsafe { gl.clear_buffer_i32_slice(glow::STENCIL, 0, &[value as i32]) };
    }

    fn clear_color_depth_and_stencil(
        &self,
        gl: &AdapterContextLock<'_>,
        depth: f32,
        stencil_value: u32,
    ) {
        unsafe {
            gl.clear_buffer_depth_stencil(glow::DEPTH_STENCIL, 0, depth, stencil_value as i32)
        };
    }

    unsafe fn set_attachment(
        &self,
        gl: &AdapterContextLock<'_>,
        fbo_target: u32,
        attachment: u32,
        view: &super::TextureView,
    ) {
        match view.inner {
            super::TextureInner::Renderbuffer { raw } => {
                unsafe {
                    gl.framebuffer_renderbuffer(
                        fbo_target,
                        attachment,
                        glow::RENDERBUFFER,
                        Some(raw),
                    )
                };
            }
            super::TextureInner::DefaultRenderbuffer => panic!("Unexpected default RBO"),
            super::TextureInner::Texture { raw, target } => {
                let num_layers = view.array_layers.end - view.array_layers.start;
                if num_layers > 1 {
                    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
                    unsafe {
                        gl.framebuffer_texture_multiview_ovr(
                            fbo_target,
                            attachment,
                            Some(raw),
                            view.mip_levels.start as i32,
                            view.array_layers.start as i32,
                            num_layers as i32,
                        )
                    };
                } else if super::is_layered_target(target) {
                    unsafe {
                        gl.framebuffer_texture_layer(
                            fbo_target,
                            attachment,
                            Some(raw),
                            view.mip_levels.start as i32,
                            view.array_layers.start as i32,
                        )
                    };
                } else if target == glow::TEXTURE_CUBE_MAP {
                    unsafe {
                        gl.framebuffer_texture_2d(
                            fbo_target,
                            attachment,
                            CUBEMAP_FACES[view.array_layers.start as usize],
                            Some(raw),
                            view.mip_levels.start as i32,
                        )
                    };
                } else {
                    unsafe {
                        gl.framebuffer_texture_2d(
                            fbo_target,
                            attachment,
                            target,
                            Some(raw),
                            view.mip_levels.start as i32,
                        )
                    };
                }
            }
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            super::TextureInner::ExternalFramebuffer { ref inner } => unsafe {
                gl.bind_external_framebuffer(glow::FRAMEBUFFER, inner);
            },
        }
    }

    fn unset_vertex_attribute(&self, gl: &AdapterContextLock<'_>, location: u32) {
        unsafe { gl.disable_vertex_attrib_array(location) };
    }
}

const CUBEMAP_FACES: [u32; 6] = [
    glow::TEXTURE_CUBE_MAP_POSITIVE_X,
    glow::TEXTURE_CUBE_MAP_NEGATIVE_X,
    glow::TEXTURE_CUBE_MAP_POSITIVE_Y,
    glow::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    glow::TEXTURE_CUBE_MAP_POSITIVE_Z,
    glow::TEXTURE_CUBE_MAP_NEGATIVE_Z,
];

#[derive(Default, Debug)]
struct State {
    // begin_render_pass, end_render_pass
    render_size: wgt::Extent3d,
    resolve_attachments: ArrayVec<(u32, super::TextureView), { super::MAX_COLOR_ATTACHMENTS }>,
    invalidate_attachments: ArrayVec<u32, { super::MAX_COLOR_ATTACHMENTS + 2 }>,

    // end_render_pass
    primitive: super::PrimitiveState,
    vertex_attributes: ArrayVec<super::AttributeDesc, { super::MAX_VERTEX_ATTRIBUTES }>,

    instance_vbuf_mask: usize,
    dirty_vbuf_mask: usize,
    active_first_instance: u32,
    color_targets: ArrayVec<super::ColorTargetDesc, { super::MAX_COLOR_ATTACHMENTS }>,
}
