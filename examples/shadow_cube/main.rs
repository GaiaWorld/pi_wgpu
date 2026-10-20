//! shadow_cube 3D 样例：深度纹理全流程——离屏阴影 pass（ortho 光视→D24 深度纹理）
//! → 主 pass（离屏 RGBA8 场景色 + D24 深度，平面+双立方体+2x2 PCF 阴影+斜向平行光）
//! → 合成 pass（全屏 quad 采样场景色写窗口默认帧缓冲——本机窗口 pf 无深度=实测定论，
//! 主渲染走离屏 FBO，present 由采样 blit 完成）。
use pi_wgpu::*;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// 阴影图边长（像素）。
const SHADOW_S: u32 = 1024;
/// 窗口/场景 RT 固定尺寸。
const WIN_W: u32 = 800;
const WIN_H: u32 = 600;

const DEPTH_VERT: &str = include_str!("depth.vert");
const DEPTH_FRAG: &str = include_str!("depth.frag");
const BOX_VERT: &str = include_str!("box.vert");
const BOX_FRAG: &str = include_str!("box.frag");
const GROUND_VERT: &str = include_str!("ground.vert");
const GROUND_FRAG: &str = include_str!("ground.frag");
const COMP_VERT: &str = include_str!("comp.vert");
const COMP_FRAG: &str = include_str!("comp.frag");

type M4 = [f32; 16];

fn mul(a: &M4, b: &M4) -> M4 {
    let mut o = [0.0f32; 16];
    for c in 0..4 {
        for r in 0..4 {
            let mut s = 0.0;
            for k in 0..4 {
                s += a[k * 4 + r] * b[c * 4 + k];
            }
            o[c * 4 + r] = s;
        }
    }
    o
}

fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> M4 {
    let f = 1.0 / (fovy / 2.0).tan();
    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) / (near - far),
        -1.0,
        0.0,
        0.0,
        2.0 * far * near / (near - far),
        0.0,
    ]
}

fn ortho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> M4 {
    [
        2.0 / (r - l),
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / (t - b),
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 / (f - n),
        0.0,
        -(r + l) / (r - l),
        -(t + b) / (t - b),
        -(f + n) / (f - n),
        1.0,
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = dot3(v, v).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

fn look_at(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> M4 {
    let z = norm3([eye[0] - center[0], eye[1] - center[1], eye[2] - center[2]]);
    let x = norm3(cross3(up, z));
    let y = cross3(z, x);
    [
        x[0],
        y[0],
        z[0],
        0.0,
        x[1],
        y[1],
        z[1],
        0.0,
        x[2],
        y[2],
        z[2],
        0.0,
        -dot3(x, eye),
        -dot3(y, eye),
        -dot3(z, eye),
        1.0,
    ]
}

fn cube_verts(ox: f32, oy: f32, oz: f32, h: f32, out: &mut Vec<f32>) {
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        (
            [0.0, 0.0, 1.0],
            [
                [-1.0, -1.0, 1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [-1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0],
                [1.0, 1.0, 1.0],
                [-1.0, 1.0, 1.0],
            ],
        ),
        (
            [1.0, 0.0, 0.0],
            [
                [1.0, -1.0, -1.0],
                [1.0, -1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, -1.0],
            ],
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [1.0, -1.0, -1.0],
                [-1.0, -1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [1.0, 1.0, -1.0],
            ],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-1.0, -1.0, 1.0],
                [-1.0, -1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, 1.0, 1.0],
            ],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-1.0, -1.0, -1.0],
                [1.0, -1.0, -1.0],
                [1.0, -1.0, 1.0],
                [-1.0, -1.0, 1.0],
            ],
        ),
    ];
    for (n, quad) in faces {
        let q = [0, 1, 2, 0, 2, 3];
        for i0 in q {
            let c = quad[i0];
            out.push(ox + c[0] * h);
            out.push(oy + c[1] * h);
            out.push(oz + c[2] * h);
            out.extend_from_slice(&n);
        }
    }
}

fn ground_verts(half: f32) -> Vec<f32> {
    // 俯视+Y 系 CW（法线朝下）：Back 剔除下光视不写地面深度。
    let c = [
        [-half, 0.0, -half],
        [half, 0.0, half],
        [half, 0.0, -half],
        [-half, 0.0, -half],
        [-half, 0.0, half],
        [half, 0.0, half],
    ];
    let mut v = Vec::new();
    for p in c {
        v.extend_from_slice(&p);
    }
    v
}

fn quad_verts() -> Vec<f32> {
    vec![
        -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
    ]
}

fn f32_bytes(xs: &[f32]) -> Vec<u8> {
    let mut b = Vec::with_capacity(xs.len() * 4);
    for x in xs {
        b.extend_from_slice(&x.to_le_bytes());
    }
    b
}

struct State {
    window: &'static Window,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    shadow_view: TextureView,
    scene_view: TextureView,
    main_depth_view: TextureView,
    shadow_tex: Texture,
    _shadow_depth: Texture,
    shadow_depth_view: TextureView,
    scene_tex: Texture,
    _main_depth: Texture,
    _sampler: Sampler,
    _comp_sampler: Sampler,
    bg_u: BindGroup,
    bg_full: BindGroup,
    bg_comp: BindGroup,
    sh_pipe: RenderPipeline,
    sh_box_pipe: RenderPipeline,
    box_pipe: RenderPipeline,
    ground_pipe: RenderPipeline,
    comp_pipe: RenderPipeline,
    cube_buf: Buffer,
    ground_buf: Buffer,
    quad_buf: Buffer,
    uniform: Buffer,
}

struct App {
    state: Option<State>,
}

impl App {
    fn build(&mut self, event_loop: &ActiveEventLoop) {
        let window: &'static Window = Box::leak(Box::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("shadow_cube")
                        .with_inner_size(PhysicalSize::new(WIN_W, WIN_H))
                        .with_resizable(false),
                )
                .expect("create_window"),
        ));
        let instance: &'static Instance = Box::leak(Box::new(Instance::new(&InstanceDescriptor {
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
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("sc"),
                required_features: Features::default(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::default(),
                trace: Trace::Off,
            },
            None,
        ))
        .expect("request_device");
        let format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: WIN_W,
            height: WIN_H,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::PreMultiplied,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // —— 离屏三纹理：阴影深度、场景色、主深度 ——
        let mk_tex = |label: &str, w: u32, h: u32, fmt: TextureFormat, usage: TextureUsages| {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: fmt,
                usage,
                view_formats: &[],
            })
        };
        let shadow_tex = mk_tex(
            "shadow",
            SHADOW_S,
            SHADOW_S,
            TextureFormat::Rgba16Float,
            TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
        );
        // 深度 pass 的测试深度附件（D24，仅 shadow pass 内部遮挡用）。
        let shadow_depth = mk_tex(
            "shadow-depth",
            SHADOW_S,
            SHADOW_S,
            TextureFormat::Depth24Plus,
            TextureUsages::RENDER_ATTACHMENT,
        );
        let shadow_depth_view = shadow_depth.create_view(&TextureViewDescriptor::default());
        let scene_tex = mk_tex(
            "scene",
            WIN_W,
            WIN_H,
            TextureFormat::Rgba8Unorm,
            TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
        );
        let main_depth = mk_tex(
            "main-depth",
            WIN_W,
            WIN_H,
            TextureFormat::Depth24Plus,
            TextureUsages::RENDER_ATTACHMENT,
        );
        let shadow_view = shadow_tex.create_view(&TextureViewDescriptor::default());
        let scene_view = scene_tex.create_view(&TextureViewDescriptor::default());
        let main_depth_view = main_depth.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("shadow-smp"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..Default::default()
        });
        let comp_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("scene-smp"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        // —— 矩阵 UBO ——
        let light_dir = norm3([0.45, 0.8, 0.35]);
        let eye_l = [
            light_dir[0] * 30.0,
            light_dir[1] * 30.0,
            light_dir[2] * 30.0,
        ];
        let light_view = look_at(eye_l, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let light_proj = ortho(-13.0, 13.0, -13.0, 13.0, -60.0, 60.0);
        let light_vp = mul(&light_proj, &light_view);
        let view = look_at([6.5, 4.5, 9.0], [0.0, 0.9, 0.0], [0.0, 1.0, 0.0]);
        let proj = perspective(
            std::f32::consts::FRAC_PI_4,
            WIN_W as f32 / WIN_H as f32,
            0.1,
            100.0,
        );
        let view_proj = mul(&proj, &view);
        let texel = 1.0 / SHADOW_S as f32;
        let mut ub: Vec<f32> = Vec::new();
        ub.extend_from_slice(&view_proj);
        ub.extend_from_slice(&light_vp);
        ub.extend_from_slice(&[light_dir[0], light_dir[1], light_dir[2], 0.0]);
        ub.extend_from_slice(&[texel, 0.0028, 0.0, 0.0]);
        let uniform = device.create_buffer(&BufferDescriptor {
            label: Some("mats"),
            size: (ub.len() as u64) * 4,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform, 0, &f32_bytes(&ub));

        // —— 几何 ——
        let mut verts: Vec<f32> = Vec::new();
        cube_verts(-0.6, 0.8, 0.6, 0.8, &mut verts);
        cube_verts(3.4, 0.45, -2.2, 0.45, &mut verts);
        let cube_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("cubes"),
            contents: &f32_bytes(&verts),
            usage: BufferUsages::VERTEX,
        });
        let ground_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("ground"),
            contents: &f32_bytes(&ground_verts(8.6)),
            usage: BufferUsages::VERTEX,
        });
        let quad_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("quad"),
            contents: &f32_bytes(&quad_verts()),
            usage: BufferUsages::VERTEX,
        });

        // —— 着色器 ——
        let mk_sm = |label: &str, src: &str, stage: ShaderStage| {
            device.create_shader_module(ShaderModuleDescriptor {
                label: Some(label),
                source: ShaderSource::Glsl {
                    shader: src.into(),
                    stage,
                    defines: &[],
                },
            })
        };
        let sh_vs = mk_sm("sh-v", DEPTH_VERT, ShaderStage::Vertex);
        let sh_fs = mk_sm("sh-f", DEPTH_FRAG, ShaderStage::Fragment);
        let box_vs = mk_sm("box-v", BOX_VERT, ShaderStage::Vertex);
        let box_fs = mk_sm("box-f", BOX_FRAG, ShaderStage::Fragment);
        let gr_vs = mk_sm("gr-v", GROUND_VERT, ShaderStage::Vertex);
        let gr_fs = mk_sm("gr-f", GROUND_FRAG, ShaderStage::Fragment);
        let cp_vs = mk_sm("cp-v", COMP_VERT, ShaderStage::Vertex);
        let cp_fs = mk_sm("cp-f", COMP_FRAG, ShaderStage::Fragment);

        // —— 布局 ——
        let ubo_entry = || BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            count: None,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        };
        let bgl_u = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl-u"),
            entries: &[BindGroupLayoutEntry {
                visibility: ShaderStages::VERTEX,
                ..ubo_entry()
            }],
        });
        let bgl_full = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl-full"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
                ubo_entry(),
            ],
        });
        let bgl_comp = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl-comp"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    count: None,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                },
            ],
        });
        let bg_u = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bg-u"),
            layout: &bgl_u,
            entries: &[BindGroupEntry {
                binding: 2,
                resource: BindingResource::Buffer(uniform.as_entire_buffer_binding()),
            }],
        });
        let bg_full = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bg-full"),
            layout: &bgl_full,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&shadow_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(uniform.as_entire_buffer_binding()),
                },
            ],
        });
        let bg_comp = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bg-comp"),
            layout: &bgl_comp,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&scene_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&comp_sampler),
                },
            ],
        });
        let pl_u = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pl-u"),
            bind_group_layouts: &[&bgl_u],
            push_constant_ranges: &[],
        });
        let pl_full = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pl-full"),
            bind_group_layouts: &[&bgl_full],
            push_constant_ranges: &[],
        });
        let pl_comp = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pl-comp"),
            bind_group_layouts: &[&bgl_comp],
            push_constant_ranges: &[],
        });

        let dss = DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };
        let prim = PrimitiveState {
            cull_mode: Some(Face::Back),
            ..Default::default()
        };
        // 光视与主视上下反置→同几何视图绕序相反：shadow 管线单独 Cw+关剔除（光视须见
        // 双面）；主三通道仍 prim（Back+Ccw，仰视地面 CW 假象已消）。
        let prim_shadow = PrimitiveState {
            front_face: FrontFace::Cw,
            cull_mode: None,
            ..prim.clone()
        };

        fn vpos3() -> VertexBufferLayout<'static> {
            VertexBufferLayout {
                array_stride: 12,
                step_mode: VertexStepMode::Vertex,
                attributes: &[VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            }
        }
        let sh_pipe = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("shadow-p"),
            layout: Some(&pl_u),
            vertex: VertexState {
                module: &sh_vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[vpos3()],
            },
            fragment: Some(FragmentState {
                module: &sh_fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[],
            }),
            depth_stencil: Some(dss.clone()),
            multisample: MultisampleState::default(),
            primitive: prim_shadow.clone(),
            multiview: None,
            cache: None,
        });
        fn vcube() -> VertexBufferLayout<'static> {
            VertexBufferLayout {
                array_stride: 24,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                ],
            }
        }
        let sh_box_pipe = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("shadow-box-p"),
            layout: Some(&pl_u),
            vertex: VertexState {
                module: &sh_vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[vcube()],
            },
            fragment: Some(FragmentState {
                module: &sh_fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[],
            }),
            depth_stencil: Some(dss.clone()),
            multisample: MultisampleState::default(),
            primitive: prim.clone(),
            multiview: None,
            cache: None,
        });
        let box_pipe = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("box-p"),
            layout: Some(&pl_full),
            vertex: VertexState {
                module: &box_vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[vcube()],
            },
            fragment: Some(FragmentState {
                module: &box_fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(dss.clone()),
            multisample: MultisampleState::default(),
            primitive: prim.clone(),
            multiview: None,
            cache: None,
        });
        let ground_pipe = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("gr-p"),
            layout: Some(&pl_full),
            vertex: VertexState {
                module: &gr_vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[vpos3()],
            },
            fragment: Some(FragmentState {
                module: &gr_fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(dss),
            multisample: MultisampleState::default(),
            primitive: prim,
            multiview: None,
            cache: None,
        });
        let comp_pipe = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("comp-p"),
            layout: Some(&pl_comp),
            vertex: VertexState {
                module: &cp_vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(FragmentState {
                module: &cp_fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            primitive: prim.clone(),
            multiview: None,
            cache: None,
        });
        self.state = Some(State {
            window,
            surface,
            device,
            queue,
            config,
            shadow_view,
            scene_view,
            main_depth_view,
            shadow_tex,
            _shadow_depth: shadow_depth,
            shadow_depth_view,
            scene_tex,
            _main_depth: main_depth,
            _sampler: sampler,
            _comp_sampler: comp_sampler,
            bg_u,
            bg_full,
            bg_comp,
            sh_pipe,
            sh_box_pipe,
            box_pipe,
            ground_pipe,
            comp_pipe,
            cube_buf,
            ground_buf,
            quad_buf,
            uniform,
        });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.build(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let Some(s) = self.state.as_mut() else { return };
                let frame = s
                    .surface
                    .get_current_texture()
                    .expect("get_current_texture");
                let mut enc = s.device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("sc-frame"),
                });
                {
                    let sca = RenderPassColorAttachment {
                        view: &s.shadow_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    };
                    let dsa = RenderPassDepthStencilAttachment {
                        view: &s.shadow_depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    };
                    let desc = RenderPassDescriptor {
                        label: Some("shadow"),
                        color_attachments: &[Some(sca)],
                        depth_stencil_attachment: Some(dsa),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut pass = enc.begin_render_pass(&desc);
                    pass.set_viewport(0.0, 0.0, SHADOW_S as f32, SHADOW_S as f32, 0.0, 1.0);
                    pass.set_pipeline(&s.sh_box_pipe);
                    pass.set_bind_group(0, &s.bg_u, &[]);
                    pass.set_vertex_buffer(0, s.cube_buf.slice(..));
                    pass.draw(0..36, 0..1);
                    pass.draw(36..72, 0..1);
                    pass.set_pipeline(&s.sh_pipe);
                    pass.set_vertex_buffer(0, s.ground_buf.slice(..));
                    pass.draw(0..6, 0..1);
                }
                {
                    let ca = RenderPassColorAttachment {
                        view: &s.scene_view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.62,
                                g: 0.74,
                                b: 0.86,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    };
                    let dsa = RenderPassDepthStencilAttachment {
                        view: &s.main_depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    };
                    let desc = RenderPassDescriptor {
                        label: Some("main"),
                        color_attachments: &[Some(ca)],
                        depth_stencil_attachment: Some(dsa),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut pass = enc.begin_render_pass(&desc);
                    pass.set_viewport(0.0, 0.0, WIN_W as f32, WIN_H as f32, 0.0, 1.0);
                    pass.set_pipeline(&s.ground_pipe);
                    pass.set_bind_group(0, &s.bg_full, &[]);
                    pass.set_vertex_buffer(0, s.ground_buf.slice(..));
                    pass.draw(0..6, 0..1);
                    pass.set_pipeline(&s.box_pipe);
                    pass.set_vertex_buffer(0, s.cube_buf.slice(..));
                    pass.draw(0..36, 0..1);
                    pass.draw(36..72, 0..1);
                }
                {
                    let ca = RenderPassColorAttachment {
                        view: frame.view(),
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    };
                    let desc = RenderPassDescriptor {
                        label: Some("comp"),
                        color_attachments: &[Some(ca)],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    };
                    let mut pass = enc.begin_render_pass(&desc);
                    pass.set_viewport(0.0, 0.0, WIN_W as f32, WIN_H as f32, 0.0, 1.0);
                    pass.set_pipeline(&s.comp_pipe);
                    pass.set_bind_group(0, &s.bg_comp, &[]);
                    pass.set_vertex_buffer(0, s.quad_buf.slice(..));
                    pass.draw(0..6, 0..1);
                }
                let cmd = enc.finish();
                s.queue.submit([cmd]);
                frame.present();
                s.window.request_redraw();
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
