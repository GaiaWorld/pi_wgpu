//! textured_quad 冒烟样例：运行时生成「左红→右绿」渐变纹理，贴到正方形上渲染。
//! 验收锚：窗口中出现带红→绿水平渐变的正方形（线性过滤），持续重绘不崩。
use pi_wgpu::*;
use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// 顶点着色器（GLSL 450 源文件）。
const VERT: &str = include_str!("quad.vert");
/// 片元着色器（GLSL 450 源文件，separate texture/sampler）。
const FRAG: &str = include_str!("quad.frag");

/// 清屏色。
const CLEAR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.08,
    a: 1.0,
};
/// 纹理宽高（像素）。
const TEX_W: u32 = 256;
const TEX_H: u32 = 256;

/// 运行时渐变（RGBA8）：x 从 0→1 红→绿插值，与 y 无关。
fn gradient_bytes() -> Vec<u8> {
    let mut d = vec![0u8; (TEX_W * TEX_H * 4) as usize];
    for y in 0..TEX_H {
        for x in 0..TEX_W {
            let t = x as f32 / (TEX_W - 1) as f32;
            let i = ((y * TEX_W + x) * 4) as usize;
            d[i] = ((1.0 - t) * 255.0) as u8;
            d[i + 1] = (t * 255.0) as u8;
            d[i + 2] = 0;
            d[i + 3] = 255;
        }
    }
    d
}

/// 正方形 6 顶点（两三角）：每顶点 = position(vec2) + uv(vec2)。
const QUAD: [[f32; 4]; 6] = [
    [-0.6, -0.6, 0.0, 0.0],
    [0.6, -0.6, 1.0, 0.0],
    [0.6, 0.6, 1.0, 1.0],
    [-0.6, -0.6, 0.0, 0.0],
    [0.6, 0.6, 1.0, 1.0],
    [-0.6, 0.6, 0.0, 1.0],
];

/// 应用态（instance/window 泄漏为 'static）。
struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vbuf: Buffer,
    texture: Texture,
    sampler: Sampler,
    bind_group: BindGroup,
}

/// 事件回调宿主。
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes().with_title("textured_quad"))
                .expect("create_window"),
        ));
        let instance: &'static Instance =
            Box::leak(Box::new(Instance::new(&InstanceDescriptor::default())));
        let size = window.inner_size();
        let surface = instance.create_surface(window).expect("create_surface");
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            power_preference: PowerPreference::default(),
        }))
        .expect("request_adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("q"),
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::default(),
            },
            None,
        ))
        .expect("request_device");
        let format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            width: size.width.max(1),
            height: size.height.max(1),
            format,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        // 运行时纹理：USAGE=绑定采样+可写入，mip 单级。
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("grad"),
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            dim: TextureDimension::D2,
            size: Extent3d {
                width: TEX_W,
                height: TEX_H,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });
        queue.write_texture(
            &ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::default(),
                aspect: TextureAspect::All,
            },
            &gradient_bytes(),
            &ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(TEX_W * 4),
                rows_per_image: NonZeroU32::new(TEX_H),
            },
            Extent3d {
                width: TEX_W,
                height: TEX_H,
                depth_or_array_layers: 1,
            },
        );
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("lin"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            anisotropy_clamp: None,
            border_color: None,
            compare: None,
        });
        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("v"),
            format: TextureFormat::Rgba8Unorm,
            dimension: TextureViewDimension::D2,
            aspect: TextureAspect::All,
            base_mip_level: 0,
            level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("qbgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
            ],
        });
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("qbg"),
            layout: &bgl,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let vshader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("q-v"),
            source: ShaderSource::Glsl {
                shader: VERT.into(),
                stage: ShaderStage::Vertex,
                defines: &[],
            },
        });
        let fshader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("q-f"),
            source: ShaderSource::Glsl {
                shader: FRAG.into(),
                stage: ShaderStage::Fragment,
                defines: &[],
            },
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ql"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("q"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &vshader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: 16,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &fshader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            multiview: None,
            cache: None,
        });
        let mut bytes = Vec::with_capacity(QUAD.len() * 16);
        for v in QUAD {
            for c in v {
                bytes.extend_from_slice(&c.to_ne_bytes());
            }
        }
        let vbuf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad"),
            contents: &bytes,
            usage: BufferUsages::VERTEX,
        });
        self.state = Some(State {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            vbuf,
            texture,
            sampler,
            bind_group,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(s) = self.state.as_mut() {
                    s.config.width = size.width.max(1);
                    s.config.height = size.height.max(1);
                    s.surface.configure(&s.device, &s.config);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(s) = self.state.as_mut() {
                    let frame = s
                        .surface
                        .get_current_texture()
                        .expect("get_current_texture");
                    let mut encoder = s
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor { label: Some("q") });
                    {
                        let ca = RenderPassColorAttachment {
                            view: frame.view(),
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(CLEAR),
                                store: StoreOp::Store,
                            },
                        };
                        let desc = RenderPassDescriptor {
                            label: Some("q"),
                            color_attachments: &[Some(ca)],
                            depth_stencil_attachment: None,
                        };
                        let mut pass = encoder.begin_render_pass(&desc);
                        pass.set_pipeline(&s.pipeline);
                        pass.set_bind_group(0, &s.bind_group, &[]);
                        pass.set_vertex_buffer(0, &s.vbuf, 0, 16);
                        pass.set_viewport(
                            0.,
                            0.,
                            s.config.width as f32,
                            s.config.height as f32,
                            0.,
                            1.,
                        );
                        pass.draw(0, 6);
                    }
                    let cmd = encoder.finish();
                    s.queue.submit(cmd);
                    frame.present();
                    s.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    /// 显式按序销毁（不依赖 Drop 顺序）：GL 资源 → device/queue → surface → 窗口。
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(s) = self.state.take() {
            let State {
                window,
                surface,
                device,
                queue,
                config,
                pipeline,
                vbuf,
                texture,
                sampler,
                bind_group,
            } = s;
            drop(pipeline);
            drop(vbuf);
            drop(bind_group);
            drop(sampler);
            drop(texture);
            drop(queue);
            drop(device);
            drop(surface);
            drop(config);
            drop(window);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(s) = self.state.as_ref() {
            s.window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event_loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { state: None };
    let _ = event_loop.run_app(&mut app);
}
