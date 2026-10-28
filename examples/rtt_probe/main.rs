//! rtt_probe 最小复现：离屏 RGBA8 画三色带 → 合成 pass 采样写窗口默认 fb。
//! 目的：判定「离屏 RT 被合成 pass 采样呈现」链路是否通（与 shadow 复杂度无关）。
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const W_H: u32 = 800;
const W_V: u32 = 600;
const QUAD_VERT: &str = include_str!("quad.vert");
const OFF_FRAG: &str = include_str!("off.frag");
const COMP_FRAG: &str = include_str!("../shadow_cube/comp.frag");

struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    rt_view: TextureView,
    _rt: Texture,
    sampler: Sampler,
    bg: BindGroup,
    off_pipe: RenderPipeline,
    comp_pipe: RenderPipeline,
    quad_buf: Buffer,
}
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("rtt_probe")
                        .with_inner_size(PhysicalSize::new(W_H, W_V))
                        .with_resizable(false),
                )
                .expect("win"),
        ));
        let instance: &'static Instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor {
            backends: Backends::GL,
            ..Default::default()
        })));
        let surface = instance.create_surface(window).expect("surf");
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            power_preference: PowerPreference::default(),
        }))
        .expect("ad");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("rp"),
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: Trace::Off,
            },
            None,
        ))
        .expect("dev");
        let format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: W_H,
            height: W_V,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let rt = device.create_texture(&TextureDescriptor {
            label: Some("rt"),
            size: Extent3d {
                width: W_H,
                height: W_V,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let rt_view = rt.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("s"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let bgl_off = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl-off"),
            entries: &[],
        });
        let bgl_comp = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl-c"),
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
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bg-c"),
            layout: &bgl_comp,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&rt_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });
        let pl_off = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pl-off"),
            bind_group_layouts: &[&bgl_off],
            push_constant_ranges: &[],
        });
        let pl_comp = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pl-c"),
            bind_group_layouts: &[&bgl_comp],
            push_constant_ranges: &[],
        });
        let mk_sm = |label: &str, src: &str, stage: ShaderStage| {
            device.create_shader_module(ShaderModuleDescriptor {
                label: Some(label),
                source: ShaderSource::Glsl {
                    shader: src.into(),
                    stage,
                    defines: &[],
                },
            })
        };
        let ov = mk_sm("ov", QUAD_VERT, ShaderStage::Vertex);
        let of_ = mk_sm("of", OFF_FRAG, ShaderStage::Fragment);
        let cv = mk_sm("cv", QUAD_VERT, ShaderStage::Vertex);
        let cf = mk_sm("cf", COMP_FRAG, ShaderStage::Fragment);
        let vb = VertexBufferLayout {
            array_stride: 8,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };
        let mkt = |vs: &ShaderModule,
                   fs: &ShaderModule,
                   fmt: TextureFormat,
                   label: &str,
                   pl: &PipelineLayout| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(pl),
                vertex: VertexState {
                    module: vs,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    buffers: &[vb.clone()],
                },
                fragment: Some(FragmentState {
                    module: fs,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    targets: &[Some(ColorTargetState {
                        format: fmt,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                primitive: PrimitiveState::default(),
                multiview: None,
                cache: None,
            })
        };
        let off_pipe = mkt(&ov, &of_, TextureFormat::Rgba8Unorm, "off", &pl_off);
        let comp_pipe = mkt(&cv, &cf, format, "comp", &pl_comp);
        let mut qb: Vec<u8> = Vec::new();
        for x in [
            -1.0f32, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
        ] {
            let xs = [x];
            qb.extend_from_slice(&xs[0].to_le_bytes());
        }
        let quad_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("q"),
            contents: &qb,
            usage: BufferUsages::VERTEX,
        });
        self.state = Some(State {
            window,
            surface,
            device,
            queue,
            config,
            rt_view,
            _rt: rt,
            sampler,
            bg,
            off_pipe,
            comp_pipe,
            quad_buf,
        });
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let Some(s) = self.state.as_mut() else { return };
                let frame = s.surface.get_current_texture().expect("frame");
                let mut enc = s
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor { label: Some("e") });
                {
                    let ca = RenderPassColorAttachment {
                        view: &s.rt_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    };
                    let desc = RenderPassDescriptor {
                        label: Some("off"),
                        color_attachments: &[Some(ca)],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut p = enc.begin_render_pass(&desc);
                    p.set_viewport(0.0, 0.0, W_H as f32, W_V as f32, 0.0, 1.0);
                    p.set_pipeline(&s.off_pipe);
                    p.set_vertex_buffer(0, s.quad_buf.slice(..));
                    p.draw(0..6, 0..1);
                }
                {
                    let ca = RenderPassColorAttachment {
                        view: frame.view(),
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    };
                    let desc = RenderPassDescriptor {
                        label: Some("comp"),
                        color_attachments: &[Some(ca)],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut p = enc.begin_render_pass(&desc);
                    p.set_viewport(0.0, 0.0, W_H as f32, W_V as f32, 0.0, 1.0);
                    p.set_pipeline(&s.comp_pipe);
                    p.set_bind_group(0, &s.bg, &[]);
                    p.set_vertex_buffer(0, s.quad_buf.slice(..));
                    p.draw(0..6, 0..1);
                }
                let cmd = enc.finish();
                s.queue.submit([cmd]);
                frame.present();
                s.window.request_redraw();
            }
            _ => {}
        }
    }
}
fn main() {
    let event_loop = EventLoop::new().expect("loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("run");
}
