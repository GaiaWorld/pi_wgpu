use std::{mem, sync::Arc};

use arrayvec::ArrayVec;
use glow::HasContext;
use parking_lot::Mutex;

use super::AdapterShared;
use crate::hal;

#[derive(Debug)]
pub(crate) struct Device {
    pub(crate) shared: Arc<AdapterShared>,

    pub(crate) main_vao: glow::VertexArray,
}

#[derive(Debug)]
pub(crate) struct OpenDevice<A: hal::Api> {
    pub device: A::Device,
    pub queue: A::Queue,
}

impl Device {
    pub(crate) unsafe fn create_buffer(
        &self,
        desc: &crate::BufferDescriptor,
    ) -> Result<super::Buffer, super::DeviceError> {
        let target = if desc.usage.contains(super::BufferUses::INDEX) {
            glow::ELEMENT_ARRAY_BUFFER
        } else {
            glow::ARRAY_BUFFER
        };

        let emulate_map = self
            .shared
            .workarounds
            .contains(super::Workarounds::EMULATE_BUFFER_MAP)
            || !self
                .shared
                .private_caps
                .contains(super::PrivateCapabilities::BUFFER_ALLOCATION);

        if emulate_map && desc.usage.intersects(super::BufferUses::MAP_WRITE) {
            return Ok(super::Buffer {
                raw: None,
                target,
                size: desc.size,
                map_flags: 0,
                data: Some(Arc::new(Mutex::new(vec![0; desc.size as usize]))),
            });
        }

        let gl = &self.shared.context.lock();

        let target = if desc.usage.contains(super::BufferUses::INDEX) {
            glow::ELEMENT_ARRAY_BUFFER
        } else {
            glow::ARRAY_BUFFER
        };

        let is_host_visible = desc
            .usage
            .intersects(super::BufferUses::MAP_READ | super::BufferUses::MAP_WRITE);
        let is_coherent = desc
            .memory_flags
            .contains(super::MemoryFlags::PREFER_COHERENT);

        let mut map_flags = 0;
        if desc.usage.contains(super::BufferUses::MAP_READ) {
            map_flags |= glow::MAP_READ_BIT;
        }
        if desc.usage.contains(super::BufferUses::MAP_WRITE) {
            map_flags |= glow::MAP_WRITE_BIT;
        }

        let raw = Some(unsafe { gl.create_buffer() }.map_err(|_| super::DeviceError::OutOfMemory)?);
        unsafe { gl.bind_buffer(target, raw) };
        let raw_size = desc
            .size
            .try_into()
            .map_err(|_| super::DeviceError::OutOfMemory)?;

        if self
            .shared
            .private_caps
            .contains(super::PrivateCapabilities::BUFFER_ALLOCATION)
        {
            if is_host_visible {
                map_flags |= glow::MAP_PERSISTENT_BIT;
                if is_coherent {
                    map_flags |= glow::MAP_COHERENT_BIT;
                }
            }
            unsafe { gl.buffer_storage(target, raw_size, None, map_flags) };
        } else {
            assert!(!is_coherent);
            let usage = if is_host_visible {
                if desc.usage.contains(super::BufferUses::MAP_READ) {
                    glow::STREAM_READ
                } else {
                    glow::DYNAMIC_DRAW
                }
            } else {
                // Even if the usage doesn't contain SRC_READ, we update it internally at least once
                // Some vendors take usage very literally and STATIC_DRAW will freeze us with an empty buffer
                // https://github.com/gfx-rs/wgpu/issues/3371
                glow::DYNAMIC_DRAW
            };
            unsafe { gl.buffer_data_size(target, raw_size, usage) };
        }

        unsafe { gl.bind_buffer(target, None) };

        if !is_coherent && desc.usage.contains(super::BufferUses::MAP_WRITE) {
            map_flags |= glow::MAP_FLUSH_EXPLICIT_BIT;
        }
        //TODO: do we need `glow::MAP_UNSYNCHRONIZED_BIT`?

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(label) = desc.label {
            if gl.supports_debug() {
                let name = unsafe { mem::transmute(raw) };
                unsafe { gl.object_label(glow::BUFFER, name, Some(label)) };
            }
        }

        let data = if emulate_map && desc.usage.contains(super::BufferUses::MAP_READ) {
            Some(Arc::new(Mutex::new(vec![0; desc.size as usize])))
        } else {
            None
        };

        Ok(super::Buffer {
            raw,
            target,
            size: desc.size,
            map_flags,
            data,
        })
    }

    pub(crate) unsafe fn create_texture(
        &self,
        desc: &TextureDescriptor,
    ) -> Result<super::Texture, super::DeviceError> {
        let gl = &self.shared.context.lock();

        let render_usage = TextureUses::COLOR_TARGET
            | TextureUses::DEPTH_STENCIL_WRITE
            | TextureUses::DEPTH_STENCIL_READ;
        let format_desc = self.shared.describe_texture_format(desc.format);

        let mut copy_size = CopyExtent {
            width: desc.size.width,
            height: desc.size.height,
            depth: 1,
        };

        let (inner, is_cubemap) = if render_usage.contains(desc.usage)
            && desc.dimension == wgt::TextureDimension::D2
            && desc.size.depth_or_array_layers == 1
        {
            let raw = unsafe { gl.create_renderbuffer().unwrap() };
            unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, Some(raw)) };
            if desc.sample_count > 1 {
                unsafe {
                    gl.renderbuffer_storage_multisample(
                        glow::RENDERBUFFER,
                        desc.sample_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            } else {
                unsafe {
                    gl.renderbuffer_storage(
                        glow::RENDERBUFFER,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(label) = desc.label {
                if gl.supports_debug() {
                    let name = unsafe { mem::transmute(raw) };
                    unsafe { gl.object_label(glow::RENDERBUFFER, name, Some(label)) };
                }
            }

            unsafe { gl.bind_renderbuffer(glow::RENDERBUFFER, None) };
            (super::TextureInner::Renderbuffer { raw }, false)
        } else {
            let raw = unsafe { gl.create_texture().unwrap() };
            let (target, is_3d, is_cubemap) =
                super::Texture::get_info_from_desc(&mut copy_size, desc);

            unsafe { gl.bind_texture(target, Some(raw)) };
            //Note: this has to be done before defining the storage!
            match desc.format.describe().sample_type {
                wgt::TextureSampleType::Float { filterable: false }
                | wgt::TextureSampleType::Uint
                | wgt::TextureSampleType::Sint => {
                    // reset default filtering mode
                    unsafe {
                        gl.tex_parameter_i32(target, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32)
                    };
                    unsafe {
                        gl.tex_parameter_i32(target, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32)
                    };
                }
                wgt::TextureSampleType::Float { filterable: true }
                | wgt::TextureSampleType::Depth => {}
            }

            if is_3d {
                unsafe {
                    gl.tex_storage_3d(
                        target,
                        desc.mip_level_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                        desc.size.depth_or_array_layers as i32,
                    )
                };
            } else if desc.sample_count > 1 {
                unsafe {
                    gl.tex_storage_2d_multisample(
                        target,
                        desc.sample_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                        true,
                    )
                };
            } else {
                unsafe {
                    gl.tex_storage_2d(
                        target,
                        desc.mip_level_count as i32,
                        format_desc.internal,
                        desc.size.width as i32,
                        desc.size.height as i32,
                    )
                };
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(label) = desc.label {
                if gl.supports_debug() {
                    let name = unsafe { mem::transmute(raw) };
                    unsafe { gl.object_label(glow::TEXTURE, name, Some(label)) };
                }
            }

            unsafe { gl.bind_texture(target, None) };
            (super::TextureInner::Texture { raw, target }, is_cubemap)
        };

        Ok(super::Texture {
            inner,
            drop_guard: None,
            mip_level_count: desc.mip_level_count,
            array_layer_count: if desc.dimension == wgt::TextureDimension::D2 {
                desc.size.depth_or_array_layers
            } else {
                1
            },
            format: desc.format,
            format_desc,
            copy_size,
            is_cubemap,
        })
    }

    pub(crate) unsafe fn create_texture_view(
        &self,
        texture: &super::Texture,
        desc: &crate::TextureViewDescriptor,
    ) -> Result<super::TextureView, super::DeviceError> {
        let end_array_layer = match desc.range.array_layer_count {
            Some(count) => desc.range.base_array_layer + count.get(),
            None => texture.array_layer_count,
        };
        let end_mip_level = match desc.range.mip_level_count {
            Some(count) => desc.range.base_mip_level + count.get(),
            None => texture.mip_level_count,
        };
        Ok(super::TextureView {
            //TODO: use `conv::map_view_dimension(desc.dimension)`?
            inner: texture.inner.clone(),
            sample_type: texture.format.describe().sample_type,
            aspects: FormatAspects::from(texture.format) & FormatAspects::from(desc.range.aspect),
            mip_levels: desc.range.base_mip_level..end_mip_level,
            array_layers: desc.range.base_array_layer..end_array_layer,
            format: texture.format,
        })
    }

    pub(crate) unsafe fn create_sampler(
        &self,
        desc: &SamplerDescriptor,
    ) -> Result<super::Sampler, super::DeviceError> {
        let gl = &self.shared.context.lock();

        let raw = unsafe { gl.create_sampler().unwrap() };

        let (min, mag) =
            conv::map_filter_modes(desc.min_filter, desc.mag_filter, desc.mipmap_filter);

        unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MIN_FILTER, min as i32) };
        unsafe { gl.sampler_parameter_i32(raw, glow::TEXTURE_MAG_FILTER, mag as i32) };

        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_S,
                conv::map_address_mode(desc.address_modes[0]) as i32,
            )
        };
        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_T,
                conv::map_address_mode(desc.address_modes[1]) as i32,
            )
        };
        unsafe {
            gl.sampler_parameter_i32(
                raw,
                glow::TEXTURE_WRAP_R,
                conv::map_address_mode(desc.address_modes[2]) as i32,
            )
        };

        if let Some(border_color) = desc.border_color {
            let border = match border_color {
                wgt::SamplerBorderColor::TransparentBlack | wgt::SamplerBorderColor::Zero => {
                    [0.0; 4]
                }
                wgt::SamplerBorderColor::OpaqueBlack => [0.0, 0.0, 0.0, 1.0],
                wgt::SamplerBorderColor::OpaqueWhite => [1.0; 4],
            };
            unsafe { gl.sampler_parameter_f32_slice(raw, glow::TEXTURE_BORDER_COLOR, &border) };
        }

        if let Some(ref range) = desc.lod_clamp {
            unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MIN_LOD, range.start) };
            unsafe { gl.sampler_parameter_f32(raw, glow::TEXTURE_MAX_LOD, range.end) };
        }

        if let Some(anisotropy) = desc.anisotropy_clamp {
            unsafe {
                gl.sampler_parameter_i32(raw, glow::TEXTURE_MAX_ANISOTROPY, anisotropy.get() as i32)
            };
        }

        //set_param_float(glow::TEXTURE_LOD_BIAS, info.lod_bias.0);

        if let Some(compare) = desc.compare {
            unsafe {
                gl.sampler_parameter_i32(
                    raw,
                    glow::TEXTURE_COMPARE_MODE,
                    glow::COMPARE_REF_TO_TEXTURE as i32,
                )
            };
            unsafe {
                gl.sampler_parameter_i32(
                    raw,
                    glow::TEXTURE_COMPARE_FUNC,
                    conv::map_compare_func(compare) as i32,
                )
            };
        }

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(label) = desc.label {
            if gl.supports_debug() {
                let name = unsafe { mem::transmute(raw) };
                unsafe { gl.object_label(glow::SAMPLER, name, Some(label)) };
            }
        }

        Ok(super::Sampler { raw })
    }

    pub(crate) unsafe fn create_command_encoder(
        &self,
        _desc: &crate::CommandEncoderDescriptor,
    ) -> Result<super::CommandEncoder, super::DeviceError> {
        Ok(super::CommandEncoder {
            cmd_buffer: super::CommandBuffer::default(),
            state: Default::default(),
            private_caps: self.shared.private_caps,
        })
    }

    pub(crate) unsafe fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor,
    ) -> Result<super::BindGroupLayout, super::DeviceError> {
        Ok(super::BindGroupLayout {
            entries: Arc::from(desc.entries),
        })
    }

    pub(crate) unsafe fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor,
    ) -> Result<super::PipelineLayout, super::DeviceError> {
        use naga::back::glsl;

        let mut group_infos = Vec::with_capacity(desc.bind_group_layouts.len());
        let mut num_samplers = 0u8;
        let mut num_textures = 0u8;
        let mut num_images = 0u8;
        let mut num_uniform_buffers = 0u8;
        let mut num_storage_buffers = 0u8;

        let mut writer_flags = glsl::WriterFlags::ADJUST_COORDINATE_SPACE;
        writer_flags.set(
            glsl::WriterFlags::TEXTURE_SHADOW_LOD,
            self.shared
                .private_caps
                .contains(super::PrivateCapabilities::SHADER_TEXTURE_SHADOW_LOD),
        );
        let mut binding_map = glsl::BindingMap::default();

        for (group_index, bg_layout) in desc.bind_group_layouts.iter().enumerate() {
            // create a vector with the size enough to hold all the bindings, filled with `!0`
            let mut binding_to_slot = vec![
                !0;
                bg_layout
                    .entries
                    .last()
                    .map_or(0, |b| b.binding as usize + 1)
            ]
            .into_boxed_slice();

            for entry in bg_layout.entries.iter() {
                let counter = match entry.ty {
                    wgt::BindingType::Sampler { .. } => &mut num_samplers,
                    wgt::BindingType::Texture { .. } => &mut num_textures,
                    wgt::BindingType::StorageTexture { .. } => &mut num_images,
                    wgt::BindingType::Buffer {
                        ty: wgt::BufferBindingType::Uniform,
                        ..
                    } => &mut num_uniform_buffers,
                    wgt::BindingType::Buffer {
                        ty: wgt::BufferBindingType::Storage { .. },
                        ..
                    } => &mut num_storage_buffers,
                };

                binding_to_slot[entry.binding as usize] = *counter;
                let br = naga::ResourceBinding {
                    group: group_index as u32,
                    binding: entry.binding,
                };
                binding_map.insert(br, *counter);
                *counter += entry.count.map_or(1, |c| c.get() as u8);
            }

            group_infos.push(super::BindGroupLayoutInfo {
                entries: Arc::clone(&bg_layout.entries),
                binding_to_slot,
            });
        }

        Ok(super::PipelineLayout {
            group_infos: group_infos.into_boxed_slice(),
            naga_options: glsl::Options {
                version: self.shared.shading_language_version,
                writer_flags,
                binding_map,
                zero_initialize_workgroup_memory: true,
            },
        })
    }

    pub(crate) unsafe fn create_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor,
    ) -> Result<super::BindGroup, crate::DeviceError> {
        let mut contents = Vec::new();

        for (entry, layout) in desc.entries.iter().zip(desc.layout.entries.iter()) {
            let binding = match layout.ty {
                wgt::BindingType::Buffer { .. } => {
                    let bb = &desc.buffers[entry.resource_index as usize];
                    super::RawBinding::Buffer {
                        raw: bb.buffer.raw.unwrap(),
                        offset: bb.offset as i32,
                        size: match bb.size {
                            Some(s) => s.get() as i32,
                            None => (bb.buffer.size - bb.offset) as i32,
                        },
                    }
                }
                wgt::BindingType::Sampler { .. } => {
                    let sampler = desc.samplers[entry.resource_index as usize];
                    super::RawBinding::Sampler(sampler.raw)
                }
                wgt::BindingType::Texture { .. } => {
                    let view = desc.textures[entry.resource_index as usize].view;
                    if view.mip_levels.start != 0 || view.array_layers.start != 0 {
                        log::error!("Unable to create a sampled texture binding for non-zero mipmap level or array layer.\n{}",
                            "This is an implementation problem of wgpu-hal/gles backend.")
                    }
                    let (raw, target) = view.inner.as_native();
                    super::RawBinding::Texture { raw, target }
                }
                wgt::BindingType::StorageTexture {
                    access,
                    format,
                    view_dimension,
                } => {
                    let view = desc.textures[entry.resource_index as usize].view;
                    let format_desc = self.shared.describe_texture_format(format);
                    let (raw, _target) = view.inner.as_native();
                    super::RawBinding::Image(super::ImageBinding {
                        raw,
                        mip_level: view.mip_levels.start,
                        array_layer: match view_dimension {
                            wgt::TextureViewDimension::D2Array
                            | wgt::TextureViewDimension::CubeArray => None,
                            _ => Some(view.array_layers.start),
                        },
                        access: conv::map_storage_access(access),
                        format: format_desc.internal,
                    })
                }
            };
            contents.push(binding);
        }

        Ok(super::BindGroup {
            contents: contents.into_boxed_slice(),
        })
    }

    pub(crate) unsafe fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDescriptor,
    ) -> Result<super::ShaderModule, super::ShaderError> {
        Ok(super::ShaderModule {
            naga: match shader {
                ShaderInput::SpirV(_) => {
                    panic!("`Features::SPIRV_SHADER_PASSTHROUGH` is not enabled")
                }
                ShaderInput::Naga(naga) => naga,
            },
            label: desc.label.map(|str| str.to_string()),
            id: self.shared.next_shader_id.fetch_add(1, Ordering::Relaxed),
        })
    }

    pub(crate) unsafe fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor,
    ) -> Result<super::RenderPipeline, super::PipelineError> {
        let gl = &self.shared.context.lock();
        let mut shaders = ArrayVec::new();
        shaders.push((naga::ShaderStage::Vertex, &desc.vertex_stage));
        if let Some(ref fs) = desc.fragment_stage {
            shaders.push((naga::ShaderStage::Fragment, fs));
        }
        let inner =
            unsafe { self.create_pipeline(gl, shaders, desc.layout, desc.label, desc.multiview) }?;

        let (vertex_buffers, vertex_attributes) = {
            let mut buffers = Vec::new();
            let mut attributes = Vec::new();
            for (index, vb_layout) in desc.vertex_buffers.iter().enumerate() {
                buffers.push(super::VertexBufferDesc {
                    step: vb_layout.step_mode,
                    stride: vb_layout.array_stride as u32,
                });
                for vat in vb_layout.attributes.iter() {
                    let format_desc = conv::describe_vertex_format(vat.format);
                    attributes.push(super::AttributeDesc {
                        location: vat.shader_location,
                        offset: vat.offset as u32,
                        buffer_index: index as u32,
                        format_desc,
                    });
                }
            }
            (buffers.into_boxed_slice(), attributes.into_boxed_slice())
        };

        let color_targets = {
            let mut targets = Vec::new();
            for ct in desc.color_targets.iter().filter_map(|at| at.as_ref()) {
                targets.push(super::ColorTargetDesc {
                    mask: ct.write_mask,
                    blend: ct.blend.as_ref().map(conv::map_blend),
                });
            }
            //Note: if any of the states are different, and `INDEPENDENT_BLEND` flag
            // is not exposed, then this pipeline will not bind correctly.
            targets.into_boxed_slice()
        };

        Ok(super::RenderPipeline {
            inner,
            primitive: desc.primitive,
            vertex_buffers,
            vertex_attributes,
            color_targets,
            depth: desc.depth_stencil.as_ref().map(|ds| super::DepthState {
                function: conv::map_compare_func(ds.depth_compare),
                mask: ds.depth_write_enabled,
            }),
            depth_bias: desc
                .depth_stencil
                .as_ref()
                .map(|ds| ds.bias)
                .unwrap_or_default(),
            stencil: desc
                .depth_stencil
                .as_ref()
                .map(|ds| conv::map_stencil(&ds.stencil)),
            alpha_to_coverage_enabled: desc.multisample.alpha_to_coverage_enabled,
        })
    }

    pub(crate) unsafe fn destroy_command_encoder(&self, _encoder: super::CommandEncoder) {}

    pub(crate) unsafe fn destroy_buffer(&self, _buffer: super::Buffer) {}

    pub(crate) unsafe fn destroy_texture(&self, _texture: super::Texture) {}

    pub(crate) unsafe fn destroy_texture_view(&self, _view: super::TextureView) {}

    pub(crate) unsafe fn destroy_sampler(&self, _sampler: super::Sampler) {}

    pub(crate) unsafe fn destroy_bind_group_layout(&self, _bg_layout: super::BindGroupLayout) {}

    pub(crate) unsafe fn destroy_bind_group(&self, _group: super::BindGroup) {}

    pub(crate) unsafe fn destroy_shader_module(&self, _module: super::ShaderModule) {}

    pub(crate) unsafe fn destroy_pipeline_layout(&self, _pipeline_layout: super::PipelineLayout) {}

    pub(crate) unsafe fn destroy_render_pipeline(&self, _pipeline: super::RenderPipeline) {}
}
