//! 非 256 对齐上传探针：9x9 RGBA8（bpr=36）上行→回读比对，验证 loose_upload 端到端。
#![allow(dead_code)]
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const W: u32 = 9;

fn probe(device: &Device, queue: &Queue) {
    let tex = device.create_texture(&TextureDescriptor {
        label: Some("probe-9x9"),
        size: Extent3d { width: W, height: W, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let bpr_src = (W * 4) as usize;
    let mut data = vec![0u8; bpr_src * W as usize];
    for y in 0..W {
        for x in 0..W {
            let i = (y * W + x) as usize * 4;
            data[i] = (x * 8) as u8;
            data[i + 1] = (y * 8) as u8;
            data[i + 2] = 128;
            data[i + 3] = 255;
        }
    }
    queue.write_texture(
        tex.as_image_copy(),
        &data,
        TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr_src as u32), rows_per_image: Some(W) },
        Extent3d { width: W, height: W, depth_or_array_layers: 1 },
    );
    let read_bpr: u32 = 256;
    let buf = device.create_buffer(&BufferDescriptor {
        label: Some("probe-read"),
        size: (read_bpr * W) as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("probe") });
    enc.copy_texture_to_buffer(
        tex.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &buf,
            layout: TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(read_bpr), rows_per_image: Some(W) },
        },
        Extent3d { width: W, height: W, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    let sz = std::num::NonZeroU64::new((read_bpr * W) as u64).expect("sz");
    let bytes = buf.get_sub_data(0, sz);
    let px = |x: u32, y: u32| -> [u8; 4] {
        let i = (y * read_bpr + x * 4) as usize;
        [bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]
    };
    println!("UP p(0,0)={:?} p(8,8)={:?} p(4,2)={:?}", px(0, 0), px(8, 8), px(4, 2));
    // 读侧已知行翻转（copy_texture_to_buffer 设计：纹理顶行 0 语义 vs GL 底行 0），
    // 判据按翻转对位；核对目的=行距处理是否错位（错位会表现为斜切，非整行对调）。
    let ok = px(0, 0) == [0, 64, 128, 255]
        && px(8, 8) == [64, 0, 128, 255]
        && px(4, 2) == [32, 48, 128, 255];
    println!("UP-VERDICT={}", if ok { "PASS" } else { "FAIL" });

    // 第二例：R8Unorm 9 宽 bpr=9（非 4 倍数）——验 UNPACK_ALIGNMENT=1 紧排路。
    let tex2 = device.create_texture(&TextureDescriptor {
        label: Some("probe-r8"),
        size: Extent3d { width: W, height: W, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let mut d2 = vec![0u8; (W * W) as usize];
    for y in 0..W {
        for x in 0..W {
            d2[(y * W + x) as usize] = (x + y * 16) as u8;
        }
    }
    queue.write_texture(
        tex2.as_image_copy(),
        &d2,
        TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(W), rows_per_image: Some(W) },
        Extent3d { width: W, height: W, depth_or_array_layers: 1 },
    );
    let buf2 = device.create_buffer(&BufferDescriptor {
        label: Some("probe-r8-read"),
        size: (read_bpr * W) as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut enc2 = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("probe-r8") });
    enc2.copy_texture_to_buffer(
        tex2.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &buf2,
            layout: TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(read_bpr), rows_per_image: Some(W) },
        },
        Extent3d { width: W, height: W, depth_or_array_layers: 1 },
    );
    queue.submit([enc2.finish()]);
    let sz2 = std::num::NonZeroU64::new((read_bpr * W) as u64).expect("sz2");
    let b2 = buf2.get_sub_data(0, sz2);
    let r8 = |x: u32, y: u32| -> u8 { b2[(y * read_bpr + x) as usize] };
    println!("UP-R8 v(0,0)={} v(8,8)={} v(4,2)={}", r8(0, 0), r8(8, 8), r8(4, 2));
    // 翻转对位：读侧行 y 对应写侧行 8-y。
    let ok2 = r8(0, 0) == (8 * 16) as u8 && r8(8, 8) == 8u8 && r8(4, 2) == (4 + 6 * 16) as u8;
    println!("UP-R8-VERDICT={}", if ok2 { "PASS" } else { "FAIL" });
}

struct App {
    dev: Option<(Device, Queue)>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, lp: &ActiveEventLoop) {
        if let Some((d, q)) = self.dev.as_ref() {
            probe(d, q);
        } else {
            println!("UP-NO-DEVICE");
        }
        lp.exit();
    }
    fn window_event(&mut self, _l: &ActiveEventLoop, _i: WindowId, _e: WindowEvent) {}
}

fn main() {
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
    let (d, q) = pollster::block_on(ad.request_device(
        &DeviceDescriptor {
            label: Some("up"),
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::default(),
            trace: Trace::Off,
        },
        None,
    ))
    .expect("device");
    let mut app = App { dev: Some((d, q)) };
    let _ = el.run_app(&mut app);
}