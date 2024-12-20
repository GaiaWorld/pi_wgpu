use std::mem::transmute;

use pi_wgpu::{
    Adapter, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits, LoadOp, Operations, PowerPreference, PresentMode, Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor
};
#[cfg(target_arch = "wasm32")]
use web_sys::Element;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, Window},
};

#[allow(dead_code)]
fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(module = "/examples/webgl_spector/webgl_spector.js")]
extern "C" {
    fn initSpector(port: u32, canvas: &Element);
}

pub trait Example: 'static + Sized {
    // fn setting(&mut self, device: &Device, device: &Device,) {}
    fn init(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self;
    fn render<'b, 'a: 'b>(
        &'a mut self,
        device: &'a Device,
        queue: &'a Queue,
        rpass: &'b mut RenderPass<'a>,
    );
    fn render1<'b, 'a: 'b>(
        &'a mut self,
        _device: &'a Device,
        _queue: &'a Queue,
        texture_surface: SurfaceTexture,
    ) {

    }

    fn get_init_size() -> Option<(u32, u32)> {
        // None表示使用默认值
        None
    }

    fn auto() -> bool {
        // None表示使用默认值
        true
    }
}

pub fn start<T: Example + Sync + Send + 'static>() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::new()
            .filter(Some("glow=trace"), log::LevelFilter::Info)
            .filter(None, log::LevelFilter::Info)
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
                window.canvas().set_width(450);
                window.canvas().set_height(720);
                // web_sys::window().unwrap().alert_with_message(web_sys::window().unwrap().navigator().user_agent().unwrap().as_str());
                // log::error!("navigator.userAgent========={:?}", );
                let el = web_sys::Element::from(window.canvas());
                initSpector(0, &el);
                body.append_child(&el)
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

    let mut already_resume = false;
    let mut already_resize = false;

    let mut engine: Option<Engine> = None;
    let mut example: Option<T> = None;
    let is_auto = T::auto();
    let mut count = 0.0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                // 注: Wasm 收不到 Resize，所以全部写到 MainEventCleard 去
                event: WindowEvent::Resized(_),
                ..
            } => {}
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::Suspended => already_resume = false,
            Event::Resumed => {
                already_resume = true;
                if already_resize {
                    window.request_redraw();
                }
            }
            Event::MainEventsCleared => {
                already_resize = true;
                if already_resume {
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if !already_resize || !already_resume {
                    return;
                }

                if engine.is_none() {
                    let e = pollster::block_on(Engine::new(&window,));
                    engine = Some(e);
                } else {
                    engine.as_mut().unwrap().configure(&window);
                }

              
                if example.is_none() {
                    let engine = engine.as_ref().unwrap();
                    let e = T::init(&engine.device, &engine.queue, &engine.config);
                    example = Some(e);
                }

                let e = engine.as_ref().unwrap();
                let surface = &e.surface;
                let depth_view = &e.depth_view;
                let example = example.as_mut().unwrap();

                let texture_surface = surface.get_current_texture().unwrap();
               

                if is_auto {
                    let view: pi_wgpu::TextureView =
                    texture_surface.texture.create_view(&TextureViewDescriptor::default());
                    let mut encoder = engine
                        .as_ref()
                        .unwrap()
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor { label: None });
                    {
                        count += 0.5;
                        if count > 1.0 {
                            count = 0.0;
                        }
                        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                            label: None,
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(Color {
                                        r: count,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: pi_wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                                view: &depth_view,
                                depth_ops: Some(Operations {
                                    load: LoadOp::Clear(1.0),
                                    store: pi_wgpu::StoreOp::Discard,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        example.render(
                            &engine.as_ref().unwrap().device,
                            &engine.as_ref().unwrap().queue,
                            &mut rpass,
                        );
                    }

                    engine
                        .as_ref()
                        .unwrap()
                        .queue
                        .submit(Some(encoder.finish()));

                    texture_surface.present();
                } else {
                    example.render1(
                        &engine.as_ref().unwrap().device,
                        &engine.as_ref().unwrap().queue,
                        texture_surface,
                    );
                }
                
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
        desired_maximum_frame_latency: 2,
    };

    let surface_view_format = config.format.add_srgb_suffix();
    config.view_formats.push(surface_view_format);

    let depth_texture = create_depth_texture(&device, config.width, config.height, depth_format);
    let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

    (config, depth_view, depth_texture)
}

struct Engine {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,

    surface: Surface<'static>,
    config: SurfaceConfiguration,

    tex: Texture,
    depth_view: TextureView,
    depth_format: TextureFormat,
}

impl Engine {
    async fn new(window: &Window) -> Self {
        let depth_format = TextureFormat::Depth24Plus;

        let instance = Instance::default();

        let surface = instance.create_surface(&window).unwrap();
        let surface = unsafe { transmute(surface) };

        // println!("surface = {:?}", surface);

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
                    required_features: Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();

        let (config, depth_view, tex) = create_depth_view(
            &adapter,
            &device,
            &surface,
            depth_format,
            size.width,
            size.height,
        );

        surface.configure(&device, &config);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            config,
            tex,
            depth_view,
            depth_format,
        }
    }

    fn configure(&mut self, window: &Window) {
        let size = window.inner_size();

        if size.width == self.config.width && size.height == self.config.height {
            return;
        }

        if size.width == 0 || size.height == 0 {
            return;
        }

        let surface = self.instance.create_surface(&window).unwrap();
        let surface: Surface<'static> = unsafe { transmute(surface) };
        let (config, depth_view, tex) = create_depth_view(
            &self.adapter,
            &self.device,
            &self.surface,
            self.depth_format,
            size.width,
            size.height,
        );

        surface.configure(&self.device, &config);

        self.surface = surface;
        self.config = config;
        self.tex = tex;
        self.depth_view = depth_view;
    }
}
