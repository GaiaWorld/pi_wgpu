//! D32 depth-format regression probe.
//! Verdict = zero library error records (a missing logger once produced a false green here,
//! so this probe installs its own log sink and asserts on it).
#![allow(dead_code)]
use std::sync::atomic::{AtomicUsize, Ordering};
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

static REJECTS: AtomicUsize = AtomicUsize::new(0);

struct Sink;
impl log::Log for Sink {
    fn enabled(&self, m: &log::Metadata<'_>) -> bool {
        m.level() <= log::Level::Error
    }
    fn log(&self, r: &log::Record<'_>) {
        // Only library failures count; pi_egl leaves debug chatter at error level.
        if r.level() <= log::Level::Error && r.target().starts_with("pi_wgpu") {
            REJECTS.fetch_add(1, Ordering::SeqCst);
            eprintln!("PROBE-ERR [{}] {}", r.target(), r.args());
        }
    }
    fn flush(&self) {}
}

fn probe(device: &Device) {
    for w in [9u32, 32, 800, 1024] {
        let d = TextureDescriptor {
            label: Some("probe-d32"),
            size: Extent3d { width: w, height: w, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let _t = device.create_texture(&d);
        println!("PROBE D32 {}x{} built", w, w);
    }
    for (fmt, tag) in [(TextureFormat::Depth32Float, "D32"), (TextureFormat::Depth24Plus, "D24")] {
        let arr = TextureDescriptor {
            label: Some("probe-array"),
            size: Extent3d { width: 512, height: 512, depth_or_array_layers: 4 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: fmt,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let _a = device.create_texture(&arr);
        println!("PROBE ARRAY {} layers=4 built", tag);
    }
}

struct App {
    dev: Option<Device>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, lp: &ActiveEventLoop) {
        match self.dev.as_ref() {
            Some(d) => probe(d),
            None => println!("PROBE-NO-DEVICE"),
        }
        let n = REJECTS.load(Ordering::SeqCst);
        println!("PROBE-VERDICT={} rejects={}", if n == 0 { "PASS" } else { "FAIL" }, n);
        lp.exit();
    }
    fn window_event(&mut self, _l: &ActiveEventLoop, _i: WindowId, _e: WindowEvent) {}
}

fn main() {
    log::set_logger(&Sink).expect("logger");
    log::set_max_level(log::LevelFilter::Error);
    let el = EventLoop::new().unwrap();
    let win = Box::leak(Box::new(
        el.create_window(Window::default_attributes().with_visible(false)).expect("win"),
    ));
    let inst = Box::leak(Box::new(Instance::new(&InstanceDescriptor { backends: Backends::GL, ..Default::default() })));
    let surf = inst.create_surface(win).expect("surf");
    let ad = pollster::block_on(inst.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surf),
        force_fallback_adapter: false,
        power_preference: PowerPreference::default(),
    }))
    .expect("adapter");
    let (d, _q) = pollster::block_on(ad.request_device(
        &DeviceDescriptor {
            label: Some("probe"),
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::default(),
            trace: Trace::Off,
        },
        None,
    ))
    .expect("device");
    let mut app = App { dev: Some(d) };
    let _ = el.run_app(&mut app);
}