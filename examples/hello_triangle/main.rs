//! hello_triangle 冒烟样例：winit 窗口 + 本库 Surface + WGSL 三角（初始化+全帧循环+资源创建+录制绘制）。
//! 验收锚（needs）：triangle 的 WGSL 渲染正确（绑定名自动定位，本样例无绑定）。
use pi_wgpu::*;

/// 最小故障报告器（Windows VEH）：捕获 ACCESS_VIOLATION 时打印出错地址 + 模块 + 偏移。
#[cfg(windows)]
mod fault {
    use core::ffi::c_void;
    use windows_sys::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};
    /// EXCEPTION_ACCESS_VIOLATION 码（windows-sys 0.52 未导出常量）。
    const ACCESS_VIOLATION: i32 = 0xC000_0005u32 as i32;
    use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleExW};
    use windows_sys::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION};

    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        let rec = unsafe { (*info).ExceptionRecord };
        if !rec.is_null() && unsafe { (*rec).ExceptionCode } == ACCESS_VIOLATION {
            let addr: *mut c_void = unsafe { (*rec).ExceptionAddress };
            let mut mbi: MEMORY_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
            let n = unsafe { VirtualQuery(addr, &mut mbi, core::mem::size_of::<MEMORY_BASIC_INFORMATION>()) };
            let base = if n != 0 { mbi.AllocationBase } else { core::ptr::null_mut() };
            let mut buf = [0u16; 260];
            let mut name = String::from("<unknown>");
            if !base.is_null() {
                let len = unsafe { GetModuleFileNameW(base as _, buf.as_mut_ptr(), buf.len() as u32) };
                if len > 0 {
                    name = String::from_utf16_lossy(&buf[..len as usize]);
                }
            }
            let off = (addr as usize).wrapping_sub(base as usize);
            let sub = unsafe { (*rec).ExceptionInformation[0] };
            let data = unsafe { (*rec).ExceptionInformation[1] };
            let kind = match sub { 0 => "read", 1 => "write", 8 => "execute", _ => "?" };
            let mut dbuf = [0u16; 260];
            let mut dname = String::from("<unknown>");
            let mut dmbi: MEMORY_BASIC_INFORMATION = unsafe { core::mem::zeroed() };
            if unsafe { VirtualQuery(data as *const c_void, &mut dmbi, core::mem::size_of::<MEMORY_BASIC_INFORMATION>()) } != 0 && !dmbi.AllocationBase.is_null() {
                let l = unsafe { GetModuleFileNameW(dmbi.AllocationBase as _, dbuf.as_mut_ptr(), dbuf.len() as u32) };
                if l > 0 { dname = String::from_utf16_lossy(&dbuf[..l as usize]); }
            }
            let mut base2: isize = 0;
            let mut m2 = [0u16; 260];
            let mut n2 = String::from("<unknown>");
            let mut off2 = 0usize;
            if unsafe { GetModuleHandleExW(0x4 | 0x2, addr as *const u16, &mut base2) } != 0 && base2 != 0 {
                let l = unsafe { GetModuleFileNameW(base2, m2.as_mut_ptr(), m2.len() as u32) };
                if l > 0 { n2 = String::from_utf16_lossy(&m2[..l as usize]); }
                off2 = (addr as usize).wrapping_sub(base2 as usize);
            }
            eprintln!("!!! AV kind={kind} insn={addr:p}(vq:{name}+0x{off:x} / gh:{n2}+0x{off2:x}) data={data:#x}({dname})");
            let ctx = unsafe { (*info).ContextRecord };
            let mut rbp = unsafe { (*ctx).Rbp } as *const usize;
            for i in 0..16 {
                if rbp.is_null() || (rbp as usize) < 0x10000 {
                    break;
                }
                let ret = unsafe { *rbp.add(1) };
                let nxt = unsafe { *rbp } as *const usize;
                eprintln!("   #{i} ret={ret:#x}");
                rbp = nxt;
            }
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }

    pub fn install() {
        unsafe { AddVectoredExceptionHandler(1, Some(handler)); }
    }
}
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
                        pass.set_vertex_buffer(0, &s.vbuf, 0, 8);
                        pass.draw(0, 3);
                    }
                    let cmd = encoder.finish();
                    s.queue.submit(cmd);
                    frame.present();
                    // s.window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    #[cfg(windows)]
    fault::install();
    let event_loop = EventLoop::new().expect("event_loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App { state: None };
    event_loop.run_app(&mut app).expect("run_app");
}
