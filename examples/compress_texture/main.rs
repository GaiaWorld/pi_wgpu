#[path = "../framework.rs"]
mod framework;

use bytemuck::{Pod, Zeroable};
use framework::Example;
use pi_wgpu::{util::DeviceExt, *};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "full"))]
fn main() {
	framework::start::<ImageExample>();
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[wasm_bindgen_test::wasm_bindgen_test]
fn test() {
	web_sys::console::log_1(&"aaaa===========".into());
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

        let texture_view = create_texture_from_ktx(include_bytes!("./bx_lanseguanbi.s3tc.ktx"), device, queue).unwrap();
        
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



pub fn create_texture_from_ktx(
	buffer: &[u8], 
	device: &Device, 
	queue: &Queue,
) -> std::io::Result<TextureView> {

	use ktx::KtxInfo;

	let decoder = ktx::Decoder::new(buffer)?;
	let format = convert_format(decoder.gl_internal_format());

	let texture_extent = pi_wgpu::Extent3d {
		width: decoder.pixel_width(),
		height: decoder.pixel_height(),
		depth_or_array_layers: 1,
	}.physical_size(format);
	log::warn!("width====={:?}, height==={:?}", texture_extent.width, texture_extent.height);

	// let byte_size = buffer.len();
	let mut textures = decoder.read_textures();
	let data = textures.next().unwrap(); // TODO

	let texture = device.create_texture_with_data(queue, &pi_wgpu::TextureDescriptor {
		label: Some("first depth buffer"),
		size: texture_extent,
		mip_level_count: 1, // TODO
		sample_count: 1,
		dimension: pi_wgpu::TextureDimension::D2,
		format,
		usage: pi_wgpu::TextureUsages::TEXTURE_BINDING | pi_wgpu::TextureUsages::COPY_DST,
		view_formats: &[],
	}, data.as_slice());
	let texture_view = texture.create_view(&pi_wgpu::TextureViewDescriptor::default());

	Ok(texture_view)
}

fn convert_format(v: u32) -> pi_wgpu::TextureFormat {
	match v {
		// 0x83f0 => pi_wgpu::TextureFormat::Bc1RgbUnorm,// GL_COMPRESSED_RGB_S3TC_DXT1_EXT	0x83f0     GL_COMPRESSED_RGB_S3TC_DXT1_EXT	Bc1RgbUnorm
		0x83f1 => pi_wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT1_EXT	0x83f1     GL_COMPRESSED_RGBA_S3TC_DXT1_EXT	Bc1RgbaUnorm
		0x83f2 => pi_wgpu::TextureFormat::Bc2RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT3_EXT	0x83f2     GL_COMPRESSED_RGBA_S3TC_DXT3_EXT	Bc2RgbaUnorm
		0x83f3 => pi_wgpu::TextureFormat::Bc3RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT5_EXT	0x83f3     GL_COMPRESSED_RGBA_S3TC_DXT5_EXT	Bc3RgbaUnorm
		0x9274 => pi_wgpu::TextureFormat::Etc2Rgb8Unorm,// GL_COMPRESSED_RGB8_ETC2	0x9274             GL_COMPRESSED_RGB8_ETC2	Etc2Rgb8Unorm
		0x9278 => pi_wgpu::TextureFormat::Etc2Rgba8Unorm,// GL_COMPRESSED_RGBA8_ETC2_EAC	0x9278         GL_COMPRESSED_RGBA8_ETC2_EAC	Etc2Rgba8Unorm

		// 0x8c00 => pi_wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG	0x8c00  GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG	PvrtcRgb4bppUnorm 
		// 0x8c01 => pi_wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG	0x8c01 GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG	PvrtcRgb2bppUnorm 
		// 0x8c02 => pi_wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG	0x8c02 UnormGL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG	PvrtcRgba4bppUnorm
		// 0x8c03 => pi_wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG	0x8c03 GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG	PvrtcRgba2bppUnorm 

		0x93b0 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B4x4, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_4x4_KHR	0x93b0     GL_COMPRESSED_RGBA_ASTC_4x4_KHR	Astc4x4Unorm 
		0x93b1 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B5x4, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_5x4_KHR	0x93b1     GL_COMPRESSED_RGBA_ASTC_5x4_KHR	Astc5x4Unorm 
		0x93b2 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B5x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_5x5_KHR	0x93b2     GL_COMPRESSED_RGBA_ASTC_5x5_KHR	Astc5x5Unorm
		0x93b3 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B6x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_6x5_KHR	0x93b3     GL_COMPRESSED_RGBA_ASTC_6x5_KHR	Astc6x5Unorm 
		0x93b4 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B6x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_6x6_KHR	0x93b4     GL_COMPRESSED_RGBA_ASTC_6x6_KHR	Astc6x6Unorm 
		0x93b5 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B8x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x5_KHR	0x93b5     GL_COMPRESSED_RGBA_ASTC_8x5_KHR	Astc8x5Unorm 
		0x93b6 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B8x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x6_KHR	0x93b6     GL_COMPRESSED_RGBA_ASTC_8x6_KHR	Astc8x6Unorm 
		0x93b7 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B8x8, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x8_KHR	0x93b7     GL_COMPRESSED_RGBA_ASTC_8x8_KHR	Astc8x8Unorm 
		0x93b8 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B10x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x5_KHR	0x93b8     GL_COMPRESSED_RGBA_ASTC_10x5_KHR	Astc10x5Unorm 
		0x93b9 => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B10x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x6_KHR	0x93b9     GL_COMPRESSED_RGBA_ASTC_10x6_KHR	Astc10x6Unorm 
		0x93ba => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B10x8, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x8_KHR	0x93ba GL_COMPRESSED_RGBA_ASTC_10x8_KHR	Astc10x8Unorm  
		0x93bb => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B10x10, channel: AstcChannel::Unorm },//  GL_COMPRESSED_RGBA_ASTC_10x10_KHR	0x93bb     GL_COMPRESSED_RGBA_ASTC_10x10_KHR	Astc10x10Unorm 
		0x93bc => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B12x10, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_12x10_KHR	0x93bc     GL_COMPRESSED_RGBA_ASTC_12x10_KHR	Astc12x10 
		0x93bd => pi_wgpu::TextureFormat::Astc { block: AstcBlock::B12x12, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_12x12_KHR	0x93bd     GL_COMPRESSED_RGBA_ASTC_12x12_KHR	Astc12x12Unorm
		_ => panic!("not suport fomat： {}", v),
	}
}





