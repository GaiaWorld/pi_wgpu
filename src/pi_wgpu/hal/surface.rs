use pi_share::{Share, ShareCell};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

use super::{
    super::{util::DeviceExt, wgt, DeviceError, MissingDownlevelFlags},
    AdapterContext,
};

#[derive(Debug, Clone)]
pub(crate) struct Surface {
    pub(crate) imp: Share<ShareCell<SurfaceImpl>>,
}

impl Surface {
    #[inline]
    pub(crate) fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        adapter: AdapterContext,
        handle: &W,
    ) -> Result<Self, super::InstanceError> {
        SurfaceImpl::new(adapter, handle).map(|mut imp| {
            let s = imp.raw.clone();
            imp.adapter.set_surface(s);

            Self {
                imp: Share::new(ShareCell::new(imp)),
            }
        })
    }

    #[inline]
    pub(crate) fn present(&self) -> Result<(), String> {
        // log::trace!(
        //     "========== Surface::present lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        {
            self.imp.as_ref().borrow_mut().present();
        }

        // log::trace!(
        //     "========== Surface::present unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        Ok(())
    }

    #[inline]
    pub(crate) fn configure(
        &self,
        device: &crate::Device,
        config: &crate::SurfaceConfiguration,
    ) -> Result<(), super::SurfaceError> {
        // log::trace!(
        //     "========== Surface::configure lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        if config.width == 0 || config.height == 0 {
            log::warn!(
                "hal::Surface::configure() has 0 dimensions, size = ({}, {})",
                config.width,
                config.height
            );

            return Ok(());
        }

        let clone = self.imp.as_ref().borrow().raw.clone();
        let r = {
            self.imp
                .as_ref()
                .borrow_mut()
                .configure(device, config, clone)
        };

        // log::trace!(
        //     "========== Surface::configure unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }

    #[inline]
    pub(crate) fn acquire_texture(&self) -> Option<super::Texture> {
        // log::trace!(
        //     "========== Surface::acquire_texture lock, thread_id = {:?}",
        //     thread::current().id()
        // );

        let r = { self.imp.as_ref().borrow_mut().acquire_texture() };

        // log::trace!(
        //     "========== Surface::acquire_texture unlock, thread_id = {:?}",
        //     thread::current().id()
        // );

        r
    }
}

#[derive(Debug)]
pub(crate) struct SurfaceImpl {
    pub(crate) raw: pi_egl::Surface,

    adapter: AdapterContext,

    sc: Option<SwapChain>,
}

impl SurfaceImpl {
    fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        adapter: AdapterContext,
        handle: &W,
    ) -> Result<Self, super::InstanceError> {
        let raw = adapter.create_surface(&handle)?;

        Ok(Self {
            raw,
            adapter,
            sc: None,
        })
    }

    fn configure(
        &mut self,
        device: &crate::Device,
        config: &crate::SurfaceConfiguration,
        clone: pi_egl::Surface,
    ) -> Result<(), super::SurfaceError> {
        log::info!(
            "hal::Surface::config, width = {}, height = {}",
            config.width,
            config.height
        );

        self.adapter.set_surface(clone);

        if self.sc.is_none() {
            self.sc = Some(SwapChain::new(device, config));
        }

        self.sc.as_mut().unwrap().configure(device, config);

        Ok(())
    }

    #[inline]
    fn acquire_texture(&mut self) -> Option<super::Texture> {
        self.sc.as_mut().and_then(|sc| sc.current_texture.take())
    }

    #[inline]
    fn present(&mut self) {
        self.sc.as_mut().map(|sc| {
            sc.draw_y_flip();
        });

        self.adapter.present(&self.raw);

        self.sc.as_mut().map(|sc| {
            sc.update_current_texture();
        });
    }
}

#[derive(Clone, Debug, Error)]
pub(crate) enum ConfigureSurfaceError {
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error("invalid surface")]
    InvalidSurface,
    #[error("The view format {0:?} is not compatible with texture format {1:?}, only changing srgb-ness is allowed.")]
    InvalidViewFormat(wgt::TextureFormat, wgt::TextureFormat),
    #[error(transparent)]
    MissingDownlevelFlags(#[from] MissingDownlevelFlags),
    #[error("`SurfaceOutput` must be dropped before a new `Surface` is made")]
    PreviousOutputExists,
    #[error("Both `Surface` width and height must be non-zero. Wait to recreate the `Surface` until the window has non-zero area.")]
    ZeroArea,
    #[error("surface does not support the adapter's queue family")]
    UnsupportedQueueFamily,
    #[error("requested format {requested:?} is not in list of supported formats: {available:?}")]
    UnsupportedFormat {
        requested: wgt::TextureFormat,
        available: Vec<wgt::TextureFormat>,
    },
    #[error("requested present mode {requested:?} is not in the list of supported present modes: {available:?}")]
    UnsupportedPresentMode {
        requested: wgt::PresentMode,
        available: Vec<wgt::PresentMode>,
    },
    #[error("requested alpha mode {requested:?} is not in the list of supported alpha modes: {available:?}")]
    UnsupportedAlphaMode {
        requested: wgt::CompositeAlphaMode,
        available: Vec<wgt::CompositeAlphaMode>,
    },
    #[error("requested usage is not supported")]
    UnsupportedUsage,
}

#[repr(C)]
struct BlitVertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

#[derive(Debug)]
struct SwapChain {
    encoder: crate::CommandEncoder,

    rp: crate::RenderPipeline,
    vb: crate::Buffer,
    sampler: crate::Sampler,
    bg_layout: crate::BindGroupLayout,

    texture_size: (u32, u32),
    texture: crate::Texture,
    bg: crate::BindGroup,

    native_texture: crate::Texture,
    // 初始化 有值
    // 每次 acquire_texture 就为 None
    // present 后 会重新 有值
    current_texture: Option<super::Texture>,
}

impl SwapChain {
    fn new(device: &crate::Device, config: &crate::SurfaceConfiguration) -> Self {
        let encoder = device.create_command_encoder(&super::super::CommandEncoderDescriptor {
            label: Some("Flip-Y Command Encoder"),
        });

        let vertices = [
            BlitVertex {
                pos: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            BlitVertex {
                pos: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            BlitVertex {
                pos: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            BlitVertex {
                pos: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            BlitVertex {
                pos: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            BlitVertex {
                pos: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
        ];

        let slice = vertices.as_slice();
        let contents = unsafe {
            std::slice::from_raw_parts(
                slice.as_ptr() as *const u8,
                slice.len() * std::mem::size_of::<BlitVertex>(),
            )
        };

        let vb = device.create_buffer_init(&super::super::util::BufferInitDescriptor {
            label: Some("Flip-Y VB"),
            contents,
            usage: super::super::BufferUsages::VERTEX,
        });

        let bg_layout = device.create_bind_group_layout(&super::super::BindGroupLayoutDescriptor {
            label: Some("Flip-Y BG Layout"),
            entries: &[
                super::super::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: super::super::ShaderStages::FRAGMENT,
                    ty: super::super::BindingType::Sampler(
                        super::super::SamplerBindingType::Filtering,
                    ),
                    count: None,
                },
                super::super::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: super::super::ShaderStages::FRAGMENT,
                    ty: super::super::BindingType::Texture {
                        sample_type: super::super::TextureSampleType::Float { filterable: true },
                        view_dimension: super::super::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout =
            device.create_pipeline_layout(&super::super::PipelineLayoutDescriptor {
                label: Some("Flip-Y Pipeline Layout"),
                bind_group_layouts: &[&bg_layout],
                push_constant_ranges: &[],
            });

        let vs = device.create_shader_module(super::super::ShaderModuleDescriptor {
            label: Some("Flip-Y VS"),
            source: super::super::ShaderSource::Glsl {
                shader: include_str!("shaders/blit.vert").into(),
                stage: naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        });

        let fs = device.create_shader_module(super::super::ShaderModuleDescriptor {
            label: Some("Flip-Y FS"),
            source: super::super::ShaderSource::Glsl {
                shader: include_str!("shaders/blit.frag").into(),
                stage: naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        // TODO 处理 wgt::TextureFormat::Bgra8UnormSrgb
        let format = match config.format {
            wgt::TextureFormat::Rgba8Unorm => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8Unorm => wgt::TextureFormat::Bgra8Unorm,
            wgt::TextureFormat::Rgba8UnormSrgb => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8UnormSrgb => wgt::TextureFormat::Bgra8Unorm,
            _ => unreachable!(),
        };

        let rp = device.create_render_pipeline(&super::super::RenderPipelineDescriptor {
            label: Some("Flip-Y Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: super::super::VertexState {
                module: &vs,
                entry_point: "main",
                buffers: &[super::super::VertexBufferLayout {
                    array_stride: std::mem::size_of::<BlitVertex>() as super::super::BufferAddress,
                    step_mode: super::super::VertexStepMode::Vertex,
                    attributes: &[
                        super::super::VertexAttribute {
                            format: super::super::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        super::super::VertexAttribute {
                            format: super::super::VertexFormat::Float32x2,
                            offset: std::mem::size_of::<[f32; 2]>() as super::super::BufferAddress,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: super::super::PrimitiveState {
                topology: super::super::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: super::super::FrontFace::Cw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: super::super::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: super::super::MultisampleState::default(),
            fragment: Some(super::super::FragmentState {
                module: &fs,
                entry_point: "main",
                targets: &[Some(super::super::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: super::super::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let sampler = device.create_sampler(&super::super::SamplerDescriptor {
            label: Some("Flip-Y Sampler"),
            address_mode_u: wgt::AddressMode::ClampToEdge,
            address_mode_v: wgt::AddressMode::ClampToEdge,
            address_mode_w: wgt::AddressMode::ClampToEdge,
            mag_filter: wgt::FilterMode::Nearest,
            min_filter: wgt::FilterMode::Nearest,
            mipmap_filter: wgt::FilterMode::Nearest,
            ..Default::default()
        });

        let texture =
            Self::create_surface_texture(device, config.width, config.height, config.format);

        let native_texture =
            device.create_texture_from_surface(config.width, config.height, config.format);

        let texture_view = texture.create_view(&Default::default());

        let bg = device.create_bind_group(&super::super::BindGroupDescriptor {
            label: Some("Flip-Y BindGroup"),
            layout: &bg_layout,
            entries: &[
                super::super::BindGroupEntry {
                    binding: 0,
                    resource: super::super::BindingResource::Sampler(&sampler),
                },
                super::super::BindGroupEntry {
                    binding: 1,
                    resource: super::super::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        let current_texture = Some(texture.inner.clone());

        Self {
            encoder,

            rp,
            vb,
            bg,
            sampler,
            bg_layout,

            texture_size: (config.width, config.height),
            texture,
            current_texture,

            native_texture,
        }
    }

    fn configure(&mut self, device: &crate::Device, config: &crate::SurfaceConfiguration) {
        let size = self.texture.inner.0.as_ref().copy_size;
        let need_update_texture = size.width != config.width || size.height != config.height;

        if need_update_texture {
            self.texture =
                Self::create_surface_texture(device, config.width, config.height, config.format);
            self.native_texture =
                device.create_texture_from_surface(config.width, config.height, config.format);
            let texture_view = self.texture.create_view(&Default::default());
            self.bg = device.create_bind_group(&super::super::BindGroupDescriptor {
                label: Some("Flip-Y BindGroup"),
                layout: &self.bg_layout,
                entries: &[
                    super::super::BindGroupEntry {
                        binding: 0,
                        resource: super::super::BindingResource::Sampler(&self.sampler),
                    },
                    super::super::BindGroupEntry {
                        binding: 1,
                        resource: super::super::BindingResource::TextureView(&texture_view),
                    },
                ],
            });
            self.current_texture = None;
            self.update_current_texture();
        }
    }

    fn draw_y_flip(&mut self) {
        let view = self.native_texture.create_view(&Default::default());
        let mut rp = self
            .encoder
            .begin_render_pass(&super::super::RenderPassDescriptor {
                label: Some("Flip-Y RenderPass"),
                color_attachments: &[Some(super::super::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: super::super::Operations {
                        load: super::super::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        rp.set_pipeline(&self.rp);
        rp.set_bind_group(0, &self.bg, &[]);
        rp.set_vertex_buffer(0, self.vb.slice(..));
        rp.draw(0..6, 0..1);
    }

    #[inline]
    fn update_current_texture(&mut self) {
        assert!(self.current_texture.is_none());

        self.current_texture = Some(self.texture.inner.clone());
    }

    fn create_surface_texture(
        device: &crate::Device,
        width: u32,
        height: u32,
        format: wgt::TextureFormat,
    ) -> crate::Texture {
        // TODO 处理 wgt::TextureFormat::Bgra8UnormSrgb
        let format = match format {
            wgt::TextureFormat::Rgba8Unorm => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8Unorm => wgt::TextureFormat::Bgra8Unorm,
            wgt::TextureFormat::Rgba8UnormSrgb => wgt::TextureFormat::Rgba8Unorm,
            wgt::TextureFormat::Bgra8UnormSrgb => wgt::TextureFormat::Bgra8Unorm,
            _ => unreachable!(),
        };

        let desc = super::super::TextureDescriptor {
            label: None,
            size: super::super::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgt::TextureDimension::D2,
            format,
            usage: wgt::TextureUsages::RENDER_ATTACHMENT | wgt::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        device.create_texture(&desc)
    }
}
