# pi_wgpu 导出接口清单（v0.3.7）

> 自动扫描全仓 `pub` 顶层声明生成（explorer 读取 + 声明提取，说明取声明上方 rustdoc 首行）。
> 使用方式/约束见 `documents/api.md`；「支持矩阵」真相源为 README §2.0x（注意 §2.07 对 `Surface::get_capabilities` 记录已滞后）。
> 启用 `use_wgpu` 时整份导出面由 `pub use wgpu::*` 替换（原生 wgpu），仅 `PiWgpuAdapter` 来自本 crate。

## 1. crate 根 `pi_wgpu`

| 符号 | 类别 | 来源文件 | 说明 |
|---|---|---|---|
| `PiWgpuAdapter` | trait | lib.rs | — |
| `Label` | type | pi_wgpu/mod.rs | Object debugging label. |
| `Operations` | struct | pi_wgpu/mod.rs | Pair of load and store operations for an attachment aspect. |
| `StoreOp` | enum | pi_wgpu/mod.rs | Operation to perform to the output attachment at the end of a render pass. |
| `LoadOp` | enum | pi_wgpu/mod.rs | Operation to perform to the output attachment at the start of a render pass. |
| `Error` | enum | pi_wgpu/mod.rs | Error type |
| *re-exports* | wgc::* / hal api::* / ExposedAdapter / util::TextureDataOrder | | 见对应小节 |
| *wasm 追加* | `ImageCopyExternalImage`, `ExternalImageSource` | | `#[cfg(target_arch = "wasm32")] pub use wgt::{…}` |

## 2. WebGPU API 对象层（wgc —— 随 `pi_wgpu::*` 并入根）

| 符号 | 类别 | 定义模块 | 说明 |
|---|---|---|---|
| `Instance` | struct | wgc/instance | Context for all other wgpu objects. Instance of wgpu. |
| `RawHandle` | struct | wgc/instance | The window/canvas/surface/swap-chain/etc. a surface is attached to, for use with unsafe surface creation. |
| `RequestAdapterOptions` | type | wgc/adapter | Additional information required when requesting an adapter. |
| `Adapter` | struct | wgc/adapter | Handle to a physical graphics and/or compute device. |
| `InvalidAdapter` | struct | wgc/adapter | — |
| `RequestAdapterError` | enum | wgc/adapter | — |
| `SurfaceTarget` | enum | wgc/surface | — |
| `WindowHandle` | trait | wgc/surface | — |
| `Surface` | struct | wgc/surface | Handle to a presentable surface. |
| `SurfaceTexture` | struct | wgc/surface | Surface texture that can be rendered to. |
| `SurfaceError` | enum | wgc/surface | Result of an unsuccessful call to [`Surface::get_current_texture`]. |
| `CreateSurfaceError` | struct | wgc/surface | [`Instance::create_surface()`] or a related function failed. |
| `SurfaceConfiguration` | type | wgc/surface | Describes a [`Surface`]. |
| `Device` | struct | wgc/device | Open connection to a graphics and/or compute device. |
| `DeviceDescriptor` | type | wgc/device | Describes a [`Device`]. |
| `MemoryHints` | type | wgc/device | — |
| `Trace` | type | wgc/device | — |
| `RequestDeviceError` | struct | wgc/device | Requesting a device failed. |
| `DeviceError` | enum | wgc/device | — |
| `Maintain` | type | wgc/device | Passed to [`Device::poll`] to control how and if it should block. |
| `ShaderModuleDescriptorImpl` | struct | wgc/device | — |
| `Queue` | struct | wgc/queue | Handle to a command queue on a device. |
| `SubmissionIndex` | struct | wgc/queue | Identifier for a particular call to [`Queue::submit`]. Can be used |
| `BufferDescriptor` | type | wgc/buffer | Describes a [`Buffer`]. |
| `Buffer` | struct | wgc/buffer | Handle to a GPU-accessible buffer. |
| `BufferSlice` | struct | wgc/buffer | Slice into a [`Buffer`]. |
| `BufferBinding` | struct | wgc/buffer | Describes the segment of a buffer to bind. |
| `Texture` | struct | wgc/texture | Handle to a texture on the GPU. |
| `TextureView` | struct | wgc/texture | Handle to a texture view. |
| `TextureViewDescriptor` | struct | wgc/texture | Describes a [`TextureView`]. |
| `TextureDescriptor` | type | wgc/texture | Describes a [`Texture`]. |
| `ImageCopyTexture` | type | wgc/texture | View of a texture which can be used to copy to/from a buffer/texture. |
| `ImageCopyTextureTagged` | type | wgc/texture | — |
| `Sampler` | struct | wgc/sampler | Handle to a sampler. |
| `SamplerDescriptor` | struct | wgc/sampler | Describes a [`Sampler`]. |
| `ShaderModule` | struct | wgc/shader_module | Handle to a compiled shader module. |
| `ShaderSource` | enum | wgc/shader_module | Source of a shader module. |
| `ShaderModuleDescriptor` | struct | wgc/shader_module | Descriptor for use with [`Device::create_shader_module`]. |
| `ShaderModuleDescriptorSpirV` | struct | wgc/shader_module | Descriptor for a shader module given by SPIR-V binary, for use with |
| `BindGroup` | struct | wgc/bind_group | Handle to a binding group. |
| `BindGroupDescriptor` | struct | wgc/bind_group | Describes a group of bindings and the resources to be bound. |
| `BindGroupEntry` | struct | wgc/bind_group | An element of a [`BindGroupDescriptor`], consisting of a bindable resource |
| `BindingResource` | enum | wgc/bind_group_layout | Resource that can be bound to a pipeline. |
| `BindGroupLayout` | struct | wgc/bind_group_layout | Handle to a binding group layout. |
| `BindGroupLayoutDescriptor` | struct | wgc/bind_group_layout | Describes a [`BindGroupLayout`]. |
| `PipelineLayout` | struct | wgc/pipeline_layout | Handle to a pipeline layout. |
| `PipelineLayoutDescriptor` | struct | wgc/pipeline_layout | Describes a [`PipelineLayout`]. |
| `RenderPipeline` | struct | wgc/render_pipeline | Handle to a rendering (graphics) pipeline. |
| `RenderPipelineDescriptor` | struct | wgc/render_pipeline | Describes a render (graphics) pipeline. |
| `VertexState` | struct | wgc/render_pipeline | Describes the vertex processing in a render pipeline. |
| `VertexBufferLayout` | struct | wgc/render_pipeline | Describes how the vertex buffer is interpreted. |
| `FragmentState` | struct | wgc/render_pipeline | Describes the fragment processing in a render pipeline. |
| `PipelineCompilationOptions` | struct | wgc/common_pipeline | Advanced options for use when a pipeline is compiled |
| `PipelineCacheDescriptor` | struct | wgc/common_pipeline | Describes a pipeline cache, which allows reusing compilation work |

## 3. `pi_wgpu::util`（显式公开模块）

| 符号 | 类别 | 文件 | 说明 |
|---|---|---|---|
| `BufferInitDescriptor` | struct | device.rs | Describes a [Buffer](super::super::Buffer) when allocating. |
| `TextureDataOrder` | enum | device.rs | Order in which TextureData is laid out in memory. |
| `DeviceExt` | trait | device.rs | Utility methods not meant to be in the main API. |
| `parse_backends_from_comma_list` | fn | init.rs | Always returns WEBGPU on wasm over webgpu. |
| `backend_bits_from_env` | fn | init.rs | Get a set of backend bits from the environment variable WGPU_BACKEND. |
| `power_preference_from_env` | fn | init.rs | Get a power preference from the environment variable WGPU_POWER_PREF |
| `dx12_shader_compiler_from_env` | fn | init.rs | Choose which DX12 shader compiler to use from the environment variable `WGPU_DX12_COMPILER`. |

> `util/mutex.rs`（ReentrantMutexWrap 等）为 pub(crate)，不属导出面。

## 4. wgt 类型白名单再导出（共 85 个项，均自 `pi_wgpu/wgt/mod.rs`，语义对齐 wgpu-types）

**枚举/类型（77）**：

`AdapterInfo`, `AddressMode`, `AstcBlock`, `AstcChannel`, `Backend`, `Backends`, `BindGroupLayoutEntry`, `BindingType`, `BlendComponent`, `BlendFactor`, `BlendOperation`, `BlendState`, `BufferAddress`, `BufferBindingType`, `BufferSize`, `BufferUsages`, `Color`, `ColorTargetState`, `ColorWrites`, `CommandBufferDescriptor`, `CompareFunction`, `CompositeAlphaMode`, `DepthBiasState`, `DepthStencilState`, `DeviceType`, `DownlevelCapabilities`, `DownlevelFlags`, `Dx12Compiler`, `DynamicOffset`, `Extent3d`, `Face`, `Features`, `FilterMode`, `FrontFace`, `ImageDataLayout`, `ImageSubresourceRange`, `IndexFormat`, `InstanceDescriptor`, `Limits`, `MultisampleState`, `Origin2d`, `Origin3d`, `PipelineStatisticsTypes`, `PolygonMode`, `PowerPreference`, `PredefinedColorSpace`, `PresentMode`, `PresentationTimestamp`, `PrimitiveState`, `PrimitiveTopology`, `PushConstantRange`, `QueryType`, `RenderBundleDepthStencil`, `SamplerBindingType`, `SamplerBorderColor`, `ShaderLocation`, `ShaderModel`, `ShaderStages`, `StencilFaceState`, `StencilOperation`, `StencilState`, `StorageTextureAccess`, `SurfaceCapabilities`, `SurfaceStatus`, `TextureAspect`, `TextureDimension`, `TextureFormat`, `TextureFormatFeatureFlags`, `TextureFormatFeatures`, `TextureSampleType`, `TextureUsages`, `TextureViewDimension`, `VertexAttribute`, `VertexFormat`, `VertexStepMode`, `InstanceFlags`, `Gles3MinorVersion`

**常量（8）** —— 上传/绑定地址对齐约束：

`COPY_BUFFER_ALIGNMENT`, `COPY_BYTES_PER_ROW_ALIGNMENT`, `MAP_ALIGNMENT`, `PUSH_CONSTANT_ALIGNMENT`, `QUERY_RESOLVE_BUFFER_ALIGNMENT`, `QUERY_SET_MAX_QUERIES`, `QUERY_SIZE`, `VERTEX_STRIDE_ALIGNMENT`

## 5. 内部再导出声明明细（pub use，影响导出面的透传）

```text
src/lib.rs: pub use wgpu::*;
src/lib.rs: pub use pi_wgpu::*;
pi_wgpu/mod.rs: pub mod util;
pi_wgpu/mod.rs: pub use hal:: api::*, ExposedAdapter ;
pi_wgpu/mod.rs: pub use wgc::*;
pi_wgpu/mod.rs: pub use wgt:: ImageCopyExternalImage, ExternalImageSource ;
pi_wgpu/mod.rs: pub use util::TextureDataOrder;
wgc/device.rs: pub use wgt::Maintain as MaintainBase;
wgc/texture.rs: pub use super::super::wgt::ImageCopyTexture as ImageCopyTextureBase;
wgc/texture.rs: pub use super::super::wgt::ImageCopyTextureTagged as ImageCopyTextureTaggedBase;
```

## 6. 维护说明

- 新增顶层 `pub` 项或调整 `mod.rs` / `lib.rs` re-export 时，以本文件为 diff 基线；语义变更须同步 `documents/api.md` 与 README 矩阵。
- 建议核对脚本化：重跑「读取上述文件 → 提取列 0 的 `pub (struct|enum|trait|type|const|fn|use)`」即可再生成本清单（扫描器未入库，可在会话内再生）。
