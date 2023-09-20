use pi_wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferAddress, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Device,
    DeviceDescriptor, Extent3d, Face, Features, FragmentState, FrontFace, Instance, Limits, LoadOp,
    MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
    PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    StencilState, SurfaceConfiguration, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter(None, log::LevelFilter::Info)
            .init();

        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    window.set_inner_size(PhysicalSize {
        width: 450,
        height: 720,
    });

    let size = window.inner_size();

    let instance = Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            // Request an adapter which can render to our surface
            compatible_surface: None,
        })
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let depth_format = TextureFormat::Depth24Plus;
    let mut depth_texture =
        create_depth_texture(&device, config.width, config.height, depth_format);
    let mut depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

    let vb = create_vb(&device);

    let rp = create_render_pipeline(&device, swapchain_format, depth_format);

    let mut can_draw = false;
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter);

        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                can_draw = size.width > 0 && size.height > 0;
                if can_draw {
                    // Reconfigure the surface with the new size
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&device, &config);

                    depth_texture =
                        create_depth_texture(&device, config.width, config.height, depth_format);

                    depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

                    // On macos the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if !can_draw {
                    return;
                }
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let view = frame.texture.create_view(&TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: &depth_view,
                            depth_ops: Some(Operations {
                                load: LoadOp::Clear(1.0),
                                store: false,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    rpass.set_pipeline(&rp);
                    rpass.set_vertex_buffer(0, vb.slice(..));
                    rpass.draw(0..3, 0..1);
                }

                queue.submit(Some(encoder.finish()));

                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

fn create_vb(device: &Device) -> Buffer {
    let vertices = [
        Vertex {
            position: [0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, 0.5],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let slice = vertices.as_slice();
    let contents = unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let desc = BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        usage: BufferUsages::VERTEX,
        contents,
    };

    device.create_buffer_init(&desc)
}

fn create_pipeline_layout(device: &Device) -> PipelineLayout {
    let desc = PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    };

    let layout = device.create_pipeline_layout(&desc);

    layout
}

fn create_depth_texture(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Texture {
    let desc = TextureDescriptor {
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT,
        label: Some("Texture"),
        view_formats: &[format],
    };

    let texture = device.create_texture(&desc);

    texture
}

fn create_render_pipeline(
    device: &Device,
    swapchain_format: TextureFormat,
    depth_format: TextureFormat,
) -> RenderPipeline {
    let vs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("VS"),
        source: ShaderSource::Glsl {
            shader: include_str!("shader/triangle.vert").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    let fs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("FS"),
        source: ShaderSource::Glsl {
            shader: include_str!("shader/triangle.frag").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    let targets = [Some(ColorTargetState {
        format: swapchain_format,
        blend: None,
        write_mask: ColorWrites::ALL,
    })];

    let pipeline_layout = create_pipeline_layout(&device);

    let desc = RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &vs,
            entry_point: "main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                        shader_location: 1,
                    },
                ],
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: depth_format,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &fs,
            entry_point: "main",
            targets: &targets,
        }),
        multiview: None,
    };

    let rp = device.create_render_pipeline(&desc);

    rp
}
