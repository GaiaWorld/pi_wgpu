//! hello_triangle 冒烟样例：winit 窗口 + 本库 Surface + WGSL 三角（初始化+全帧循环+资源创建+录制绘制）。
//! 验收锚（needs）：triangle 的 WGSL 渲染正确（绑定名自动定位，本样例无绑定）。
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// WGSL：顶点位置 + 常量片元色。
const SHADER: &str = r#"
@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(pos, 0.0, 1.0);
}
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.9, 0.4, 0.2, 1.0);
}
"#;

/// 清屏色。
const CLEAR: Color = Color { r: 0.05, g: 0.05, b: 0.08, a: 1.0 };
/// 三角形三顶点（裁剪空间）。
const VERTS: [[f32; 2]; 3] = [[-0.6, -0.5], [0.6, -0.5], [0.0, 0.6]];

/// 应用态（instance/window 泄漏为 'static）。
struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    vbuf: Buffer,
}

/// 事件回调宿主。
struct App { state: Option<State> }

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(
            event_loop.create_window(Window::default_attributes()).expect("create_window"),
        ));
        let instance: &'static Instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor::default())));
        let size = window.inner_size();
        let surface = instance
            .create_surface(window)
            .expect("create_surface");
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            power_preference: PowerPreference::default(),
        }))
        .expect("request_adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor { label: Some("tri"), required_features: Features::default(), required_limits: Limits::default(), memory_hints: MemoryHints::default() },
            None,
        ))
        .expect("request_device");
        let format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            width: size.width.max(1),
            height: size.height.max(1),
            format,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("tri"),
            source: ShaderSource::Wgsl(WgslSource { code: SHADER }),
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor { label: Some("pl"), bind_group_layouts: &[] });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("tri"),
            layout: &layout,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                layouts: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute { format: VertexFormat::Float32x2, offset: 0, shader_location: 0 }],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState { format, blend: None, write_mask: ColorWrites(0xF) })],
            }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            primitive: PrimitiveState::default(),
        });
        let mut bytes = Vec::with_capacity(VERTS.len() * 8);
        for v in VERTS { for c in v { bytes.extend_from_slice(&c.to_ne_bytes()); } }
        let vbuf = device.create_buffer_init(&BufferInitDescriptor { label: Some("verts"), contents: &bytes, usage: BufferUsages::VERTEX });
        self.state = Some(State { window, surface, device, queue, config, pipeline, vbuf });
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
                if let Some(s) = self.state.as_mut() {
                    let frame = s.surface.get_current_texture().expect("get_current_texture");
                    let mut encoder = s.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("tri") });
                    {
                        let ca = RenderPassColorAttachment {
                            view: frame.view(),
                            resolve_target: None,
                            ops: Operations { load: LoadOp::Clear(CLEAR), store: StoreOp::Store },
                        };
                        let desc = RenderPassDescriptor { label: Some("tri"), color_attachments: &[Some(ca)], depth_stencil_attachment: None };
                        let mut pass = encoder.begin_render_pass(&desc);
                        pass.set_pipeline(&s.pipeline);
                        pass.set_vertex_buffer(0, &s.vbuf, 0, 0);
                        pass.draw(0, 3);
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
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("run_app");
}
