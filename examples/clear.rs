use std::mem::transmute;

use pi_wgpu::{
    Color, CommandEncoderDescriptor, DeviceDescriptor, Features, Instance, Limits, LoadOp, Operations, PowerPreference, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureUsages, TextureViewDescriptor
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

    let surface = instance.create_surface(&window).unwrap();
    let surface: Surface<'static> = unsafe { transmute(surface) };

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
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let mut color = 0.0;

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

                color += 0.01;
                if color > 1.0 {
                    color = 0.0;
                }

                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    let _rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: color,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: pi_wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
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
