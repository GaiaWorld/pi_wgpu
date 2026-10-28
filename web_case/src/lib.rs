//! pi_wgpu Web/WebGL2 案例：离屏三点数值对账（自动）+ 混合场景画布目检（90 帧自止）。
//! 用法：build.ps1（wasm-pack build --target web）后静态服务本目录，开 index.html；F12 看 BLEND-VERIFY。
use web_sys::console;
use pi_wgpu::*;
use wasm_bindgen::prelude::*;
#[allow(unused_imports)] // prelude 版差异兜底：dyn_into/unchecked_ref 的 trait 来源
use wasm_bindgen::JsCast as _;
use web_sys::{HtmlCanvasElement, Window};

/// 直通顶点（坐标系处理归出码侧 ADJUST，见 documents/api.md §8.2-5）。
const VS: &str = concat!("#version 450\n", "layout(location = 0) in vec2 a_pos;\n",
    "void main() { gl_Position = vec4(a_pos, 0.0, 1.0); }\n");
/// 片元红（直线 alpha 0.5）。
const FS_RED: &str = concat!("#version 450\n",
    "layout(location = 0) out vec4 o_color;\n",
    "void main() { o_color = vec4(1.0, 0.0, 0.0, 0.5); }\n");
/// 片元蓝。
const FS_BLUE: &str = concat!("#version 450\n",
    "layout(location = 0) out vec4 o_color;\n",
    "void main() { o_color = vec4(0.0, 0.0, 1.0, 0.5); }\n");
/// 离屏 RT 边长。
const RT: u32 = 64;
/// 画布尺寸。
const CW: u32 = 640;
const CH: u32 = 400;
/// 检查行/底色。
const ROW: u32 = 32;
const CLEARBG: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

/// JS console 承 log 面（库内注入日志/对账输出都走它）。
struct JsLogger;
impl log::Log for JsLogger {
    fn enabled(&self, _m: &log::Metadata) -> bool {
        true
    }
    fn log(&self, r: &log::Record) {
        console::log_1(&format!("[{}] {}", r.level(), r.args()).as_str().into());
    }
    fn flush(&self) {}
}
fn err(m: &str) -> JsValue {
    JsValue::from_str(m)
}
/// 经典直线 alpha 混合（与 examples/alpha_blend 同参——两形态共用一套数分期望）。
fn blend() -> BlendState {
    BlendState {
        color: BlendComponent { src_factor: BlendFactor::SrcAlpha, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
        alpha: BlendComponent { src_factor: BlendFactor::One, dst_factor: BlendFactor::OneMinusSrcAlpha, operation: BlendOperation::Add },
    }
}
fn quad(cx: f32, half: f32, h: f32, b: &mut Vec<u8>) {
    let push = |v: [f32; 2], b: &mut Vec<u8>| { for c in v { b.extend_from_slice(&c.to_ne_bytes()); } };
    push([cx - half, -h], b); push([cx + half, -h], b); push([cx + half, h], b);
    push([cx - half, -h], b); push([cx + half, h], b); push([cx - half, h], b);
}
fn px(d: &[u8], col: u32) -> [u8; 3] {
    let i = ((ROW * RT + col) * 4) as usize;
    [d[i], d[i + 1], d[i + 2]]
}
fn near(a: [u8; 3], b: [i16; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (*x as i16 - y).abs() <= 3)
}

/// 演示常驻态（泄漏至 'static，rAF 回调引用；90 帧后循环自然停）.
struct Demo {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
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
    frames: u32,
}
const FRAMES: u32 = 90;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(|p| {
        console::error_1(&format!("panic: {p}").as_str().into());
    }));
    let _ = log::set_boxed_logger(Box::new(JsLogger));
    log::set_max_level(log::LevelFilter::Info);
    let win: &'static Window = Box::leak(Box::new(web_sys::window().ok_or_else(|| err("no window"))?));
    let doc = win.document().ok_or_else(|| err("no document"))?;
    let canvas: HtmlCanvasElement = doc
        .get_element_by_id("screen")
        .ok_or_else(|| err("no canvas#screen"))?
        .dyn_into()
        .map_err(|_| err("#screen 非 canvas"))?;
    canvas.set_width(CW);
    canvas.set_height(CH);
    let instance: &'static Instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor {
        backends: Backends::GL,
        ..Default::default()
    })));
    let surface: Surface<'static> = instance
        .create_surface(SurfaceTarget::canvas(canvas.clone()))
        .map_err(|_| err("create_surface"))?;
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        power_preference: PowerPreference::default(),
    }))
    .ok_or_else(|| err("request_adapter"))?;
    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: Some("web_case"),
            required_features: Features::default(),
            required_limits: Limits::default(),
            memory_hints: MemoryHints::default(),
            trace: Trace::Off,
        },
        None,
    ))
    .map_err(|_| err("request_device"))?;
    // Web=ES 域：sRGB×BGRA 无核心组合（formats 门拒路），画布面取 RGBA_SRGB 族。
    let format = TextureFormat::Rgba8UnormSrgb;
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: CW,
        height: CH,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: CompositeAlphaMode::PreMultiplied,
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let vs = device.create_shader_module(ShaderModuleDescriptor { label: Some("v"), source: ShaderSource::Glsl { shader: VS.into(), stage: ShaderStage::Vertex, defines: &[] } });
    let fsr = device.create_shader_module(ShaderModuleDescriptor { label: Some("vr"), source: ShaderSource::Glsl { shader: FS_RED.into(), stage: ShaderStage::Fragment, defines: &[] } });
    let fsb = device.create_shader_module(ShaderModuleDescriptor { label: Some("vb"), source: ShaderSource::Glsl { shader: FS_BLUE.into(), stage: ShaderStage::Fragment, defines: &[] } });
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("l"), bind_group_layouts: &[], push_constant_ranges: &[] });
    let mk = |fs: &ShaderModule, fmt: TextureFormat, label: &str| -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&layout),
            vertex: VertexState { module: &vs, entry_point: Some("main"), compilation_options: Default::default(), buffers: &[VertexBufferLayout { array_stride: 8, step_mode: VertexStepMode::Vertex, attributes: &[VertexAttribute { format: VertexFormat::Float32x2, offset: 0, shader_location: 0 }] }] },
            fragment: Some(FragmentState { module: fs, entry_point: Some("main"), compilation_options: Default::default(), targets: &[Some(ColorTargetState { format: fmt, blend: Some(blend()), write_mask: ColorWrites::ALL })] }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
            multiview: None,
            cache: None,
        })
    };
    let pl_rt_red = mk(&fsr, TextureFormat::Rgba8Unorm, "rt-red");
    let pl_rt_blue = mk(&fsb, TextureFormat::Rgba8Unorm, "rt-blue");
    let pl_w_red = mk(&fsr, format, "w-red");
    let pl_w_blue = mk(&fsb, format, "w-blue");
    let mut bytes: Vec<u8> = Vec::new();
    quad(-0.55, 0.35, 0.6, &mut bytes);
    quad(-0.2, 0.4, 0.6, &mut bytes);
    let vbuf = device.create_buffer_init(&BufferInitDescriptor { label: Some("quads"), contents: &bytes, usage: BufferUsages::VERTEX });
    let rt = device.create_texture(&TextureDescriptor { label: Some("rt"), usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC, dimension: TextureDimension::D2, size: Extent3d { width: RT, height: RT, depth_or_array_layers: 1 }, format: TextureFormat::Rgba8Unorm, mip_level_count: 1, sample_count: 1, view_formats: &[] });
    let rt_view = rt.create_view(&TextureViewDescriptor { label: Some("rtv"), ..Default::default() });
    let rbuf = device.create_buffer(&BufferDescriptor { label: Some("rb"), size: (RT * RT * 4) as u64, usage: BufferUsages::COPY_DST, mapped_at_creation: false });
    let demo: &'static mut Demo = Box::leak(Box::new(Demo {
        device, queue, surface, config, pl_rt_red, pl_rt_blue, pl_w_red, pl_w_blue,
        vbuf, rt, rt_view, rbuf, verified: false, frames: 0,
    }));
    console::log_1(&"pi_web_case: 场景就绪（左红条、中叠紫、右白底=正确）".into());
    tick(demo, win);
    Ok(())
}
fn tick(s: &'static mut Demo, win: &'static Window) {
    draw(s);
    s.frames += 1;
    if s.frames >= FRAMES {
        console::log_1(&format!("pi_web_case: {} 帧结束（画布保留末帧）", FRAMES).as_str().into());
        return;
    }
    let sraw: *mut Demo = s;
    let cb = Closure::wrap(Box::new(move || tick(unsafe { &mut *sraw }, win)) as Box<dyn FnMut()>);
    win.request_animation_frame(cb.as_ref().unchecked_ref()).expect("raf");
    cb.forget();
}
fn draw(s: &mut Demo) {
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
        let pass3: bool = near(px(&d, 8), [255, 128, 128])
            && near(px(&d, 19), [128, 64, 192])
            && near(px(&d, 48), [255, 255, 255]);
        if pass3 {
            console::log_1(&"BLEND-VERIFY ok: red/overlap/blank 三点数值全对（WebGL2 半透明已真混合）".into());
        } else {
            console::error_1(&format!("BLEND-VERIFY FAIL red={:?} overlap={:?} blank={:?}", px(&d, 8), px(&d, 19), px(&d, 48)).as_str().into());
        }
        s.verified = true;
    }
    let frame = match s.surface.get_current_texture() {
        Ok(f) => f,
        Err(e) => {
            log::error!("web_case surface frame: {e:?}");
            return;
        }
    };
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
