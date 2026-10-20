//! D32 隔离探针：GL current（resumed 内）测深度/对照格式，结果 stdout ISO_* 行。
#![allow(dead_code)]
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

fn probe(device: &Device) {
    for w in [9u32, 32, 1024] {
        let d = TextureDescriptor {
            label: Some("iso-d32"),
            size: Extent3d { width: w, height: w, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let _t = device.create_texture(&d);
        println!("ISO_D32 w={} built", w);

        // 新鲜错误：create_texture 内部 with_gl 已清队列后测=ISO 行即终判据
        let c = TextureDescriptor {
            label: Some("iso-rgba"),
            size: Extent3d { width: w, height: w, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let _t2 = device.create_texture(&c);
        println!("ISO_RGBA w={} built", w);
    }
    println!("ISO-DONE");
}

struct App {
    device: Option<Device>,
    ev: Option<ActiveEventLoopW>,
}
struct ActiveEventLoopW {
    _exit: fn(&ActiveEventLoop),
}
impl ApplicationHandler for App {
    fn resumed(&mut self, _loop: &ActiveEventLoop) {
        if let Some(d) = self.device.as_ref() {
            probe(d);
        } else {
            println!("ISO-NO-DEVICE");
        }
        _loop.exit();
    }
    fn window_event(&mut self, _loop: &ActiveEventLoop, _id: WindowId, _e: WindowEvent) {}
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Box::leak(Box::new(
        event_loop
            .create_window(Window::default_attributes().with_visible(false))
            .expect("create_window"),
    ));
    let instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor {
        backends: Backends::GL,
        ..Default::default()
    })));
    let surface = instance.create_surface(window).expect("create_surface");
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        power_preference: PowerPreference::default(),
    }))
    .expect("request_adapter");
    let (device, _queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("iso"),
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::default(),
            trace: Trace::Off,
        },
        None,
    ))
    .expect("request_device");
    let mut app = App { device: Some(device), ev: None };
    let _ = event_loop.run_app(&mut app);
}