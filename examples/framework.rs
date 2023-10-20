use pi_wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features,
    Instance, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, Texture, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[allow(dead_code)]
fn main() {}

pub trait Example: 'static + Sized {
    // fn setting(&mut self, device: &Device, device: &Device,) {}
    fn init(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self;
    fn render<'b, 'a: 'b>(
        &'a mut self,
        device: &'a Device,
        queue: &'a Queue,
        rpass: &'b mut RenderPass<'a>,
    );

    fn get_init_size() -> Option<(u32, u32)> {
        // None表示使用默认值
        None
    }
}

pub fn start<T: Example + Sync + Send + 'static>() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter(Some("glow=trace"), log::LevelFilter::Trace)
            .init();

        pollster::block_on(run::<T>(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        // console_log::init().expect("could not initialize logger");
        console_log::init_with_level(log::Level::Trace).expect("could not initialize logger");

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
        wasm_bindgen_futures::spawn_local(run::<T>(event_loop, window));
    }
}

async fn run<T: Example + Sync + Send + 'static>(event_loop: EventLoop<()>, window: Window) {
    window.set_inner_size(PhysicalSize {
        width: 450,
        height: 720,
    });
    let size = window.inner_size();

    let instance = Instance::default();

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

    let depth_format = TextureFormat::Depth24Plus;

    let mut surface: Option<Surface> = None;
    let mut config: Option<SurfaceConfiguration> = None;
    let mut depth_view = None;
    let mut example: Option<T> = None;
    let mut tex: Option<Texture> = None;

    let mut already_resume = false;
    let mut already_resize = false;
    let mut is_init = false;

    let mut last_size: PhysicalSize<u32> = Default::default();

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter);

        *control_flow = ControlFlow::Poll;

        // log::error!("======= event : {:?}", event);
        match event {
            Event::WindowEvent {
                // 注: Wasm 收不到 Resize，所以全部写到 MainEventCleard 去
                event: WindowEvent::Resized(_),
                ..
            } => {}
            Event::MainEventsCleared => {
                let size = window.inner_size();
                // log::error!("======= size : {:?}", size);

                if last_size.width != size.width || last_size.height != size.height {
                    last_size = size;

                    already_resize = size.width > 0 && size.height > 0;
                    if already_resize {
                        // Reconfigure the surface with the new size
                        if let Some((config, surface)) = config.as_mut().zip(surface.as_ref()) {
                            config.width = size.width;
                            config.height = size.height;

                            surface.configure(&device, &config);

                            let depth_texture = create_depth_texture(
                                &device,
                                config.width,
                                config.height,
                                depth_format,
                            );

                            depth_view =
                                Some(depth_texture.create_view(&TextureViewDescriptor::default()));

                            window.request_redraw();
                        }
                    }
                }
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                if !already_resize || !already_resume {
                    return;
                }

                if !is_init {
                    let e = T::init(&device, &queue, config.as_ref().unwrap());
                    example.replace(e);

                    is_init = true;
                }

                if let Some(((surface, example), depth_view)) = surface
                    .as_ref()
                    .zip(example.as_mut())
                    .zip(depth_view.as_ref())
                {
                    let frame = surface.get_current_texture().unwrap();

                    let view: pi_wgpu::TextureView =
                        frame.texture.create_view(&TextureViewDescriptor::default());

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

                        example.render(&device, &queue, &mut rpass);
                    }

                    queue.submit(Some(encoder.finish()));

                    frame.present();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::Resumed => {
                already_resume = true;

                let s = unsafe { instance.create_surface(&window) }.unwrap();
                let size = window.inner_size();

                log::info!("======= Resumed : {:?}", size);

                let (cfg, view, t) =
                    create_depth_view(&adapter, &device, &s, depth_format, size.width, size.height);

                surface.replace(s);
                config.replace(cfg);
                depth_view.replace(view);
                tex.replace(t);
                // can_draw = true;
            }
            Event::Suspended => {
                println!("event: Event::Suspended");
                already_resume = false
            }
            _ => {}
        }
    });
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

fn create_depth_view(
    adapter: &Adapter,
    device: &Device,
    surface: &Surface,
    depth_format: TextureFormat,
    width: u32,
    height: u32,
) -> (SurfaceConfiguration, TextureView, Texture) {
    let swapchain_capabilities = surface.get_capabilities(&adapter);

    let swapchain_format = swapchain_capabilities.formats[0];

    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: width,
        height: height,
        present_mode: PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    let surface_view_format = config.format.add_srgb_suffix();
    config.view_formats.push(surface_view_format);

    let depth_texture = create_depth_texture(&device, config.width, config.height, depth_format);
    let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());
    (config, depth_view, depth_texture)
}
