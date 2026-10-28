# pi_wgpu 导出面索引（v0.4.1）

> 人工整理的公开符号索引；契约细节与约束见 api.md，源码权威在各 mod.rs 壳注释。
> 只列公开面；*_inner.rs 中的 pub(crate) 不计。

## 1. 根导出（lib.rs）

| 类别 | 符号 |
|---|---|
| 模块 | types、entry、surface、draw、upload、utils、error、util |
| 对象 | Instance、Adapter、Device、Queue、CommandEncoder、CommandBuffer、RenderPass |
| 资源 | Buffer、BufferSlice、Texture、TextureView、Sampler、BindGroup、BindGroupLayout、PipelineLayout、RenderPipeline、ShaderModule |
| 占位 | QuerySet、PipelineCache（无构造路径） |
| 表面 | Surface、SurfaceTarget、SurfaceTexture、SurfaceError、SurfaceCapabilities |
| 错误 | RequestDeviceError、CreateSurfaceError、Error、ErrorFilter、ErrorSource、UncapturedErrorHandler |
| 词汇 | Backends、BufferUsages、Extent3d、PredefinedColorSpace、TextureFormat、AstcBlock、AstcChannel |
| 便捷 | DeviceExt、BufferInitDescriptor、backend_bits_from_env、power_preference_from_env、parse_backends_from_comma_list |
| trait | PiWgpuAdapter、WindowHandle |
| Web 专面 | CopyExternalImageDestInfo、ExternalImageSource、ImageCopyExternalImage（wasm32） |

## 2. types（词汇面）

### 2.1 直接再导出

pub use wgpu_types::*（wgpu-types 25.0.0 全量）。

### 2.2 别名（api 层引用式）

| 别名 | 定义 |
|---|---|
| Label<'a> | Option<&'a str> |
| BufferDescriptor<'a> | wgpu_types::BufferDescriptor<Label<'a>> |
| TextureDescriptor<'a> | wgpu_types::TextureDescriptor<Label<'a>, &'a [TextureFormat]> |
| SamplerDescriptor<'a> | wgpu_types::SamplerDescriptor<Label<'a>> |
| DeviceDescriptor<'a> | wgpu_types::DeviceDescriptor<Label<'a>> |
| CommandEncoderDescriptor<'a> | wgpu_types::CommandEncoderDescriptor<Label<'a>> |
| SurfaceConfiguration | wgpu_types::SurfaceConfiguration<Vec<TextureFormat>> |
| TextureViewDescriptor<'a> | wgpu_types::TextureViewDescriptor<Label<'a>> |
| TexelCopyTextureInfo<'a> | wgpu_types::TexelCopyTextureInfo<&'a Texture> |
| ImageCopyTexture<'a> | 同上（旧名，deprecated） |

### 2.3 自建结构 / 别名

| 符号 | 说明 |
|---|---|
| TexelCopyBufferInfo<'a> | { buffer: &'a Buffer, layout: TexelCopyBufferLayout } |
| BindGroupLayoutDescriptor<'a> | { label, entries } |
| BufferBinding<'a> | { buffer, offset, size } |
| BindingResource<'a> | Buffer(BufferBinding) / Sampler(&Sampler) / TextureView(&TextureView) |
| BindGroupEntry<'a> | { binding, resource } |
| BindGroupDescriptor<'a> | { label, layout, entries } |
| RenderPassColorAttachment<'tex> | { view, resolve_target, ops } |
| RenderPassDepthStencilAttachment<'tex> | { view, depth_ops, stencil_ops } |
| RenderPassTimestampWrites<'a> | { query_set, beginning_of_pass_write_index, end_of_pass_write_index } |
| RenderPassDescriptor<'a> | 五字段（label/color/depth/timestamp/occlusion） |
| PipelineLayoutDescriptor<'a> | { label, bind_group_layouts, push_constant_ranges } |
| VertexBufferLayout<'a> | { array_stride, step_mode, attributes } |
| PipelineCompilationOptions<'a> | { constants, zero_initialize_workgroup_memory } |
| VertexState<'a> | { module, entry_point, compilation_options, buffers } |
| FragmentState<'a> | { module, entry_point, compilation_options, targets } |
| RenderPipelineDescriptor<'a> | { label, layout, vertex, primitive, depth_stencil, multisample, fragment, multiview, cache } |
| ShaderModuleDescriptor<'a> | { label, source } |
| ShaderSource<'a> | Wgsl(Cow) / Glsl { shader, stage, defines } |
| CullMode | Option<Face> |
| ShaderStage | pub use naga::ShaderStage |

## 3. entry

### 3.1 Instance

| 方法 | 签名 |
|---|---|
| new | fn new(&InstanceDescriptor) -> Self |
| create_surface | fn create_surface(impl Into<SurfaceTarget>) -> Result<Surface, CreateSurfaceError> |
| request_adapter | async fn request_adapter(&RequestAdapterOptions) -> Option<Adapter> |

### 3.2 Adapter

request_device（双参，返回 impl Future<Result<(Device, Queue), RequestDeviceError>>）、features、limits、get_info、get_downlevel_capabilities、get_texture_format_features、is_surface_supported、unmake_current。

### 3.3 Device

create_buffer、create_texture、create_sampler、create_shader_module、create_bind_group_layout、create_bind_group、create_pipeline_layout、create_render_pipeline、create_command_encoder、features、limits、reset_state、on_uncaptured_error、push_error_scope、pop_error_scope。

### 3.4 Queue

submit、write_buffer、write_texture；wasm32：write_texture_jsbuffer、copy_external_image_to_texture。

### 3.5 CommandEncoder

begin_render_pass、copy_texture_to_buffer、copy_buffer_to_texture、copy_texture_to_texture、finish。

### 3.6 资源对象

Buffer：slice、as_entire_buffer_binding、as_entire_binding、usage、get_sub_data。

Texture：as_image_copy、create_view、destroy、size、width、height、depth_or_array_layers、mip_level_count、sample_count、dimension、format、usage。

BufferSlice：get_buffer、get_offset、get_size、get_range、slice。

SurfaceTexture：view、present。

## 4. draw::RenderPass

set_pipeline、set_bind_group、set_vertex_buffer、set_index_buffer、set_viewport、set_scissor_rect、set_stencil_reference、set_blend_constant、set_push_constants（恒拒）、draw、draw_indexed；Drop=End。

## 5. surface

Surface：configure、get_current_texture、get_capabilities。

SurfaceTexture：texture、suboptimal、view、present。

SurfaceError：Timeout / Outdated / Lost / OutOfMemory / Other。

SurfaceTarget：Window / Canvas(wasm) / OffscreenCanvas(wasm)；SurfaceTarget::canvas / offscreen_canvas（wasm）。

## 6. utils

DeviceExt：create_buffer_init、create_texture_with_data、（wasm）create_compress_texture_with_data_jsdata。

自由函数：backend_bits_from_env、power_preference_from_env、parse_backends_from_comma_list。

## 7. error

Error { OutOfMemory / Validation / Internal }、ErrorSource = String、ErrorFilter { OutOfMemory / Validation / Internal }、UncapturedErrorHandler。

## 8. 未导出（内部）

context、caps、state、objects、formats、binds、shader、record、gid 全部为 pub(crate)；SurfaceErr、UploadErr、ShaderErr、SurfaceOpErr 等短码枚举不公开。
