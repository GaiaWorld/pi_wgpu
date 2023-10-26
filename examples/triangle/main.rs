#[path = "../framework.rs"]
mod framework;

use bytemuck::{Pod, Zeroable};
use framework::Example;
use pi_wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]

fn main() {
    framework::start::<TriangleExample>();
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[wasm_bindgen_test::wasm_bindgen_test]
fn test() {
    web_sys::console::log_1(&"aaaa===========".into());
    framework::start::<TriangleExample>();
}

pub struct TriangleExample {
    temp_buf: pi_wgpu::Buffer,
    temp_data: Vec<u8>,

    vertex_buf: pi_wgpu::Buffer,
    pipeline: pi_wgpu::RenderPipeline,
}

impl Example for TriangleExample {
    fn init(device: &Device, _queue: &pi_wgpu::Queue, config: &SurfaceConfiguration) -> Self {
        let depth_format = TextureFormat::Depth24Plus;

        let vertex_buf = create_vb(&device);

        let temp_data = vec![0u8; 11 * 1024];

        let temp_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            usage: BufferUsages::VERTEX,
            contents: temp_data.as_slice(),
        });

        let pipeline: RenderPipeline = create_render_pipeline(&device, depth_format, config);

        TriangleExample {
            temp_buf,
            temp_data,

            vertex_buf,
            pipeline,
        }
    }

    fn render<'b, 'a: 'b>(
        &'a mut self,
        _device: &'a Device,
        queue: &'a pi_wgpu::Queue,
        rpass: &'b mut pi_wgpu::RenderPass<'a>,
    ) {
        let count = 1000;
        let begin = pi_time::Instant::now();
        for _ in 0..count {
            queue.write_buffer(&self.temp_buf, 0, &self.temp_data.as_slice());
        }
        let d = begin.elapsed();
        log::warn!("{} write_buffer time = {:?}", count, d);

        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        rpass.draw(0..3, 0..1);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

fn create_vb(device: &Device) -> Buffer {
    let vertices = [
        Vertex {
            position: [0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.0, 0.5],
            color: [0.0, 1.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let slice = vertices.as_slice();
    let contents = unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let desc = BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        usage: BufferUsages::VERTEX,
        contents,
    };

    device.create_buffer_init(&desc)
}

fn create_pipeline_layout(device: &Device) -> PipelineLayout {
    let desc = PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    };

    let layout = device.create_pipeline_layout(&desc);

    layout
}

fn create_render_pipeline(
    device: &Device,
    depth_format: TextureFormat,
    config: &SurfaceConfiguration,
) -> RenderPipeline {
    let vs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("VS"),
        source: ShaderSource::Glsl {
            shader: include_str!("triangle.vert").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    let fs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("FS"),
        source: ShaderSource::Glsl {
            shader: include_str!("triangle.frag").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    // let targets = [Some(ColorTargetState {
    //     format: swapchain_format,
    //     blend: None,
    //     write_mask: ColorWrites::ALL,
    // })];

    let pipeline_layout = create_pipeline_layout(&device);
    let target = [Some(config.view_formats[0].into())];

    let desc = RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &vs,
            entry_point: "main",
            buffers: &[VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                step_mode: VertexStepMode::Vertex,
                attributes: &[
                    VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    },
                    VertexAttribute {
                        format: VertexFormat::Float32x4,
                        offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                        shader_location: 1,
                    },
                ],
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Cw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: depth_format,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            module: &fs,
            entry_point: "main",
            targets: &target,
        }),
        multiview: None,
    };

    let rp = device.create_render_pipeline(&desc);

    rp
}
