//! clear 冒烟样例：winit 窗口 + 本库 Surface + 帧循环（清屏）。
//! 验收锚（needs）：桌面链路出第一个可运行程序（初始化 + 全帧循环）。
use std::time::Instant;

use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// 清屏色。
// const CLEAR: Color = Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };

/// 应用态：instance/window 泄漏为 'static，规避 Instance↔Surface 自引用（样例简化）。
struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
}

/// 事件回调宿主。
struct App {
    state: Option<State>,
    start: Instant,
    frames: u64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(Window::default_attributes())
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
                label: Some("clear"),
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: Trace::Off,
            },
            None,
        ))
        .expect("request_device");
        let config = SurfaceConfiguration {
            width: size.width.max(1),
            height: size.height.max(1),
            format: TextureFormat::Bgra8UnormSrgb,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);
        self.state = Some(State {
            window,
            surface,
            device,
            queue,
            config,
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
                self.frames += 1;
                let t = self.start.elapsed().as_secs_f32();
                if self.frames % 60 == 0 {
                    eprintln!("frame={} t={:.2}s", self.frames, t);
                }
                if let Some(s) = self.state.as_mut() {
                    let frame = s
                        .surface
                        .get_current_texture()
                        .expect("get_current_texture");
                    let mut encoder = s.device.create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("clear"),
                    });
                    {
                        let ca = RenderPassColorAttachment {
                            view: frame.view(),
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: (t % 2.0) / 2.0,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: StoreOp::Store,
                            },
                        };
                        let desc = RenderPassDescriptor {
                            label: Some("clear"),
                            color_attachments: &[Some(ca)],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        };
                        let _pass = encoder.begin_render_pass(&desc);
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
}

fn main() {
    let event_loop = EventLoop::new().expect("event_loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        state: None,
        start: Instant::now(),
        frames: 0,
    };
    event_loop.run_app(&mut app).expect("run_app");
}
