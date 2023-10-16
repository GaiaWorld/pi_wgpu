#[path = "../framework.rs"]
mod framework;

use bytemuck::{Pod, Zeroable};
use framework::Example;
use pi_wgpu::{util::DeviceExt, *};


fn main() {
	framework::start::<ImageExample>();
}
pub struct ImageExample {
    vertex_buf: pi_wgpu::Buffer,
    index_buf: pi_wgpu::Buffer,
    index_count: usize,
    bind_group: pi_wgpu::BindGroup,
    pipeline: pi_wgpu::RenderPipeline,
}


impl Example for ImageExample {
    fn init(device: &Device, queue: &pi_wgpu::Queue, config: &SurfaceConfiguration) -> Self {
        // let vertex_size = mem::size_of::<Vertex>();
		let depth_format = TextureFormat::Depth24Plus;
	
		let (vertex_data, vertex_attributes, index_data) = create_vb();
		let vertex_buf = device.create_buffer_init(&pi_wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(&vertex_data),
			usage: pi_wgpu::BufferUsages::VERTEX,
		});
		let index_buf = device.create_buffer_init(&pi_wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(&index_data),
			usage: pi_wgpu::BufferUsages::INDEX,
		});

		// Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&pi_wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                pi_wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: pi_wgpu::ShaderStages::FRAGMENT,
					ty: pi_wgpu::BindingType::Sampler(pi_wgpu::SamplerBindingType::Filtering),
					count: None,
				},
                pi_wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: pi_wgpu::ShaderStages::FRAGMENT,
                    ty: pi_wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: pi_wgpu::TextureSampleType::Uint,
                        view_dimension: pi_wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
	 
        // Create the texture
		let sampler = device.create_sampler(&pi_wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: pi_wgpu::AddressMode::ClampToEdge,
            address_mode_v: pi_wgpu::AddressMode::ClampToEdge,
            address_mode_w: pi_wgpu::AddressMode::ClampToEdge,
            mag_filter: pi_wgpu::FilterMode::Linear,
            min_filter: pi_wgpu::FilterMode::Nearest,
            mipmap_filter: pi_wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_size = 256u32;
        let texels = create_texels(texture_size as usize);
        let texture_extent = pi_wgpu::Extent3d {
            width: texture_size,
            height: texture_size,
            depth_or_array_layers: 1,
        };
		println!("mip_level_count====================");
        let texture = device.create_texture(&pi_wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: pi_wgpu::TextureDimension::D2,
            format: pi_wgpu::TextureFormat::Rgba8Unorm,
            usage: pi_wgpu::TextureUsages::TEXTURE_BINDING | pi_wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
		println!("mip_level_count end====================");
        let texture_view = texture.create_view(&pi_wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            texture.as_image_copy(),
            &texels,
            pi_wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture_size),
                rows_per_image: None,
            },
            texture_extent,
        );
		let bind_group = device.create_bind_group(&pi_wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
				pi_wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pi_wgpu::BindingResource::Sampler(&sampler),
                },
                pi_wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pi_wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

		let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});
		let pipeline: RenderPipeline = create_render_pipeline(&device, depth_format, config, &pipeline_layout, vertex_attributes.as_slice());



        // Done
        ImageExample {
            vertex_buf,
            index_buf,
			// index_count: 0,
            index_count: index_data.len(),
            bind_group,
            pipeline,
        }
    }

    fn render<'b, 'a: 'b>(&'a mut self, _device: &'a Device, _queue: &'a pi_wgpu::Queue, rpass: &'b mut pi_wgpu::RenderPass<'a>) {
		rpass.set_pipeline(&self.pipeline);
		rpass.set_bind_group(0, &self.bind_group, &[]);
		rpass.set_index_buffer(self.index_buf.slice(..), pi_wgpu::IndexFormat::Uint16);
		rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
		rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
		// rpass.draw(0..3, 0..1);
    }
}

fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size * 4)
        .map(|mut id| {
			if id as f32 % 4.0 != 0.0 {
				return 0;
			}

			id = id / 4;
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}


fn create_vb() -> (Vec<Vertex>, Vec<VertexAttribute>, Vec<u16>) {
    let vertex_data = [
        Vertex {
            position: [0.0, 0.0],
            tex_coord: [0.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.5],
            tex_coord: [0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5],
            tex_coord: [1.0, 1.0],
        },
		Vertex {
            position: [0.5, 0.0],
            tex_coord: [1.0, 0.0],
        },
    ];
	let index_data: &[u16] = &[
        0, 1, 2, 
		0, 2, 3,
    ];

	(
		vertex_data.to_vec(), 
		vec![
			VertexAttribute {
				format: VertexFormat::Float32x2,
				offset: 0,
				shader_location: 0,
			},
			VertexAttribute {
				format: VertexFormat::Float32x2,
				offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
				shader_location: 1,
			},
		],
		index_data.to_vec()
	)
}


fn create_render_pipeline(
    device: &Device,
    depth_format: TextureFormat,
	config: &SurfaceConfiguration,
	pipeline_layout: &PipelineLayout,
	vertex_attributes: &[VertexAttribute],
) -> RenderPipeline {
    let vs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("VS"),
        source: ShaderSource::Glsl {
            shader: include_str!("image.vert").into(),
            stage: naga::ShaderStage::Vertex,
            defines: Default::default(),
        },
    });

    let fs = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("FS"),
        source: ShaderSource::Glsl {
            shader: include_str!("image.frag").into(),
            stage: naga::ShaderStage::Fragment,
            defines: Default::default(),
        },
    });

    // let targets = [Some(ColorTargetState {
    //     format: swapchain_format,
    //     blend: None,
    //     write_mask: ColorWrites::ALL,
    // })];

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
                attributes: vertex_attributes,
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





