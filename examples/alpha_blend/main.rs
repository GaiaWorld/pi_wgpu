//! alpha_blend 验证样例：半透明混合接线（管线携带 blend，2026-10-22 断线修复验收件）。
//! 首帧跑离屏 64x64 读回数值对账（白底=clear；红 0.5 叠白=[255,128,128]；再叠蓝 0.5=[128,64,192]），
//! 再上窗呈现同一场景供目检：左红条、中部红蓝重叠变暗紫、右侧留白底；90 帧自动退出。
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// 顶点：直通位置（Y 翻与深度映射击出码侧 ADJUST，产品端不手翻，见 documents/api.md 8.2-5）。
const VS: &str = concat!("#version 450\n", "layout(location = 0) in vec2 a_pos;\n",
    "void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }\n");
/// 片元红：直线 alpha（非预乘，配 SrcAlpha/OneMinusSrcAlpha 经典 alpha）。
const FS_RED: &str = concat!("#version 450\n",
    "layout(location = 0) out vec4 o_color;\n",
    "void main() { o_color = vec4(1.0, 0.0, 0.0, 0.5); }\n");
/// 片元蓝。
const FS_BLUE: &str = concat!("#version 450\n",
    "layout(location = 0) out vec4 o_color;\n",
    "void main() { o_color = vec4(0.0, 0.0, 1.0, 0.5); }\n");

/// 离屏 RT 边长（64*4=256=行距天然对齐）。
const RT: u32 = 64;
/// 检查行（NDC y=0 带→行 32，红蓝矩形均跨该行）；列按 NDC 中点取样。
const ROW: u32 = 32;
/// 窗口清屏底色（与离屏 clear 同白）。
const CLEARBG: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// 经典直线 alpha 混合：色 SrcAlpha/OneMinusSrcAlpha，alpha ONE/OneMinusSrcAlpha。
fn blend() -> BlendState {
    BlendState {
        color: BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
        alpha: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
    }
}
/// 生成一个矩形（两三角共 6 顶点 float2）追加进字节表。
fn quad(cx: f32, half: f32, h: f32, b: &mut Vec<u8>) {
    let push = |v: [f32; 2], b: &mut Vec<u8>| { for c in v { b.extend_from_slice(&c.to_ne_bytes()); } };
    push([cx - half, -h], b); push([cx + half, -h], b); push([cx + half, h], b);
    push([cx - half, -h], b); push([cx + half, h], b); push([cx - half, h], b);
}
/// 读回缓冲取指定列像素 RGB（行=ROW）。
fn px(d: &[u8], col: u32) -> [u8; 3] {
    let i = ((ROW * RT + col) * 4) as usize;
    [d[i], d[i + 1], d[i + 2]]
}
struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pl_rt_red: RenderPipeline,
    pl_rt_blue: RenderPipeline,
    pl_w_red: RenderPipeline,
    pl_w_blue: RenderPipeline,
    vbuf: Buffer,
    rt: Texture,
    rt_view: TextureView,
    rbuf: Buffer,
    verified: bool,
}
struct App {
    state: Option<State>,
    frames: u32,
}
impl App {
    /// 组一条带混合管线（fmt 定附件形态：离屏 Rgba8Unorm 数值精确 / 窗面 Bgra8UnormSrgb 呈视觉）。
    fn pl(d: &Device, l: &PipelineLayout, vs: &ShaderModule, fs: &ShaderModule, fmt: TextureFormat, label: &str) -> RenderPipeline {
        d.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(l),
            vertex: VertexState { module: vs, entry_point: Some("main"), compilation_options: Default::default(), buffers: &[VertexBufferLayout { array_stride: 8, step_mode: VertexStepMode::Vertex, attributes: &[VertexAttribute { format: VertexFormat::Float32x2, offset: 0, shader_location: 0 }] }] },
            fragment: Some(FragmentState { module: fs, entry_point: Some("main"), compilation_options: Default::default(), targets: &[Some(ColorTargetState { format: fmt, blend: Some(blend()), write_mask: ColorWrites::ALL })] }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            multiview: None,
            cache: None,
        })
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(event_loop.create_window(Window::default_attributes().with_title("alpha_blend")).expect("create_window")));
        let instance: &'static Instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor { backends: Backends::GL, ..Default::default() })));
        let size = window.inner_size();
        let surface = instance.create_surface(window).expect("create_surface");
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions { compatible_surface: Some(&surface), force_fallback_adapter: false, power_preference: PowerPreference::default() })).expect("request_adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor { label: Some("ab"), required_features: Features::default(), required_limits: Limits::default(), memory_hints: MemoryHints::default(), trace: Trace::Off }, None)).expect("request_device");
        let format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration { usage: TextureUsages::RENDER_ATTACHMENT, format, width: size.width.max(1), height: size.height.max(1), present_mode: PresentMode::Fifo, desired_maximum_frame_latency: 2, alpha_mode: CompositeAlphaMode::PreMultiplied, view_formats: vec![] };
        surface.configure(&device, &config);
        let vs = device.create_shader_module(ShaderModuleDescriptor { label: Some("v"), source: ShaderSource::Glsl { shader: VS.into(), stage: ShaderStage::Vertex, defines: &[] } });
        let fsr = device.create_shader_module(ShaderModuleDescriptor { label: Some("vr"), source: ShaderSource::Glsl { shader: FS_RED.into(), stage: ShaderStage::Fragment, defines: &[] } });
        let fsb = device.create_shader_module(ShaderModuleDescriptor { label: Some("vb"), source: ShaderSource::Glsl { shader: FS_BLUE.into(), stage: ShaderStage::Fragment, defines: &[] } });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("l"), bind_group_layouts: &[], push_constant_ranges: &[] });
        let pl_rt_red = App::pl(&device, &layout, &vs, &fsr, TextureFormat::Rgba8Unorm, "rt-red");
        let pl_rt_blue = App::pl(&device, &layout, &vs, &fsb, TextureFormat::Rgba8Unorm, "rt-blue");
        let pl_w_red = App::pl(&device, &layout, &vs, &fsr, format, "w-red");
        let pl_w_blue = App::pl(&device, &layout, &vs, &fsb, format, "w-blue");
        let mut bytes: Vec<u8> = Vec::new();
        quad(-0.55, 0.35, 0.6, &mut bytes);
        quad(-0.2, 0.4, 0.6, &mut bytes);
        let vbuf = device.create_buffer_init(&BufferInitDescriptor { label: Some("quads"), contents: &bytes, usage: BufferUsages::VERTEX });
        let rt = device.create_texture(&TextureDescriptor { label: Some("rt"), usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC, dimension: TextureDimension::D2, size: Extent3d { width: RT, height: RT, depth_or_array_layers: 1 }, format: TextureFormat::Rgba8Unorm, mip_level_count: 1, sample_count: 1, view_formats: &[] });
        let rt_view = rt.create_view(&TextureViewDescriptor { label: Some("rtv"), ..Default::default() });
        let rbuf = device.create_buffer(&BufferDescriptor { label: Some("rb"), size: (RT * RT * 4) as u64, usage: BufferUsages::COPY_DST, mapped_at_creation: false });
        self.state = Some(State { window, surface, device, queue, config, pl_rt_red, pl_rt_blue, pl_w_red, pl_w_blue, vbuf, rt, rt_view, rbuf, verified: false });
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => { if let Some(s) = self.state.as_mut() { s.config.width = size.width.max(1); s.config.height = size.height.max(1); s.surface.configure(&s.device, &s.config); } }
            WindowEvent::RedrawRequested => {
                if let Some(s) = self.state.as_mut() {
                    if !s.verified {
                        let mut enc = s.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("verify") });
                        {
                            let ca = RenderPassColorAttachment { view: &s.rt_view, resolve_target: None, ops: Operations { load: LoadOp::Clear(CLEARBG), store: StoreOp::Store } };
                            let mut pass = enc.begin_render_pass(&RenderPassDescriptor { label: Some("rt"), color_attachments: &[Some(ca)], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None });
                            pass.set_pipeline(&s.pl_rt_red);
                            pass.set_vertex_buffer(0, s.vbuf.slice(0..48));
                            pass.set_viewport(0., 0., RT as f32, RT as f32, 0., 1.);
                            pass.draw(0..6, 0..1);
                            pass.set_pipeline(&s.pl_rt_blue);
                            pass.set_vertex_buffer(0, s.vbuf.slice(48..96));
                            pass.draw(0..6, 0..1);
                        }
                        enc.copy_texture_to_buffer(TexelCopyTextureInfo { texture: &s.rt, mip_level: 0, origin: Origin3d { x: 0, y: 0, z: 0 }, aspect: TextureAspect::All }, TexelCopyBufferInfo { buffer: &s.rbuf, layout: TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(RT * 4), rows_per_image: Some(RT) } }, Extent3d { width: RT, height: RT, depth_or_array_layers: 1 });
                        s.queue.submit([enc.finish()]);
                        let d = s.rbuf.get_sub_data(0, std::num::NonZeroU64::new((RT * RT * 4) as u64).expect("rb-size"));
                        let near = |a: [u8; 3], b: [i16; 3]| -> bool { a.iter().zip(b).all(|(x, y)| (*x as i16 - y).abs() <= 3) };
                        assert!(near(px(&d, 8), [255, 128, 128]), "BLEND-VERIFY red 期望约 [255,128,128] 实得 {:?}", px(&d, 8));
                        assert!(near(px(&d, 19), [128, 64, 192]), "BLEND-VERIFY overlap 期望约 [128,64,192] 实得 {:?}", px(&d, 19));
                        assert!(near(px(&d, 48), [255, 255, 255]), "BLEND-VERIFY blank 期望 [255,255,255] 实得 {:?}", px(&d, 48));
                        println!("BLEND-VERIFY ok: red/overlap/blank 三点数值全对（GL_BLEND 已随管线开启）");
                        s.verified = true;
                    }
                    let frame = s.surface.get_current_texture().expect("get_current_texture");
                    let mut enc2 = s.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("q") });
                    {
                        let ca = RenderPassColorAttachment { view: frame.view(), resolve_target: None, ops: Operations { load: LoadOp::Clear(CLEARBG), store: StoreOp::Store } };
                        let mut pass = enc2.begin_render_pass(&RenderPassDescriptor { label: Some("w"), color_attachments: &[Some(ca)], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None });
                        pass.set_pipeline(&s.pl_w_red);
                        pass.set_vertex_buffer(0, s.vbuf.slice(0..48));
                        pass.set_viewport(0., 0., s.config.width as f32, s.config.height as f32, 0., 1.);
                        pass.draw(0..6, 0..1);
                        pass.set_pipeline(&s.pl_w_blue);
                        pass.set_vertex_buffer(0, s.vbuf.slice(48..96));
                        pass.draw(0..6, 0..1);
                    }
                    s.queue.submit([enc2.finish()]);
                    frame.present();
                }
                self.frames += 1;
                if self.frames >= 90 { event_loop.exit(); } else { if let Some(s) = self.state.as_ref() { s.window.request_redraw(); } }
            }
            _ => {}
        }
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) { let _ = self.state.take(); }
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { state: None, frames: 0 };
    event_loop.run_app(&mut app).expect("run_app");
}
