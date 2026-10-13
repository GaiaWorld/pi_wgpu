# pi_wgpu 接口文档（导出面）

> v0.3.7。除注明外，所有类型名/字段名与 wgpu-rs 25.x 同名（wgt 层个别类型名保留了早期版本命名，见 §7）；本文与 README §2.0x 矩阵、[requirements.md](requirements.md)、[module-design.md](module-design.md) 配套。

## 1. 构建与启用

| 事项 | 说明 |
|---|---|
| 工具链 | 内建实现使用 `#![feature(hash_extract_if)]` → **nightly** 编译。 |
| Windows 运行 | exe 运行目录需放置仓库 `bin/` 的两个 dll（EGL/GL 运行时，见 README 顶部提示）。 |
| 依赖源 | `pi_egl`、`pi_glow` 取自私有 registry `yn`；开发仓库带 `[patch.crates-io] naga = { path = "../wgpu/naga" }`，作为依赖使用时需自备 naga 改动或移除 patch。 |
| feature | 见 §2。 |

## 2. Feature 开关

| feature | 效果（据源码可考项） |
|---|---|
| `gl`（默认） | 选择内建 GL 实现的清单开关；src 内未检索到 `cfg(feature="gl")` 分支。 |
| `use_wgpu` | lib 直接 re-export 真实 wgpu，`PiWgpuAdapter` 转为对原生 `Adapter::request_device(desc)` 的适配（trace_path 忽略）。 |
| `single_thread` | 去掉 AdapterContext 重入锁与 make_current 搬运（`ReentrantMutexWrap` 空实现、device/egl 路径），要求全部调用同线程。 |
| `geterror` | wasm 路径纹理操作追加 glGetError 检查（hal/texture.rs 三处）。 |
| `webgl` / `webgpu` | 供源码中裸 `cfg(any(webgl, webgpu))`（Canvas/OffscreenCanvas、部分 texture 接口）判定；按当前 Cargo 定义不会自动开启该 cfg，需要 `--cfg webgl/--cfg webgpu` 注入。 |
| `webgl_context` | 源码存在 `cfg(feature="webgl_context")` 分支（gl_state/pipeline，WebGL 上下文行为）。 |
| `serde`（未启用项） | `Operations/LoadOp/StoreOp` 上有 `trace/replay` 占位属性，对应字段当前不存在，预留。 |

## 3. 根模块导出

```rust
// src/lib.rs
pub use wgpu::*;            // cfg(feature="use_wgpu")
pub use pi_wgpu::*;         // 否则：内建实现根导出
pub trait PiWgpuAdapter { /* 统一两参 request_device */ }
```

`pi_wgpu::\*` 再导出面：`hal::api::{Api,HalApi,GL}` + `ExposedAdapter` + `wgc::*`（下列对象）+ `wgt` 白名单：适配器/限制/格式/位标志/描述符（完整列表见 src/pi_wgpu/mod.rs）+ 常量（COPY_BUFFER_ALIGNMENT、VERTEX_STRIDE_ALIGNMENT…）+`util::TextureDataOrder`、Label、`Operations/LoadOp/StoreOp`、`Error`。

## 4. 初始化流程（必须顺序）

```text
Instance::new → create_surface(window) → request_adapter(...{compatible_surface})
  → adapter.request_device → Surface::configure(device, cfg)
    ⇒ 此后才允许 Device/Queue 的一切操作（Web 平台无跨 Surface Context）
```

```rust
use pi_wgpu::{Instance, InstanceDescriptor, RequestAdapterOptions,
            Backends, PowerPreference, PiWgpuAdapter};

let instance = Instance::new(&InstanceDescriptor { backends: Backends::GL, ..Default::default() });
// 可统一用 trait 名（原生 wgpu 同形）：Instance::default()
let surface = instance.create_surface(&window)?;          // impl Into<SurfaceTarget>
let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
    compatible_surface: Some(&surface),
    power_preference: PowerPreference::HighPerformance,
    force_fallback_adapter: false,                        // 软件 adapter 直接 assert 拒绝
})).expect("no adapter");
let (device, queue) = pollster::block_on(adapter.request_device(&device_desc, None)).unwrap();
surface.configure(&device, &config);
```

## 5. 对象接口

### 5.1 Instance / Adapter / PiWgpuAdapter

- `Instance::new(&InstanceDescriptor) -> Self`；`Default`（backends=GL）。
- `request_adapter(&RequestAdapterOptions) -> impl Future<Output = Option<Adapter>> + Send`（内部同步实现；按设备类型与 power 偏好排序挑选第一个可呈现 surface 的 adapter）。
- `create_surface(impl Into<SurfaceTarget>) -> Result<Surface, CreateSurfaceError>`。
- `enumerate_adapters`：**未提供**（EGL 枚举恒为系统默认显卡）。
- `Adapter::request_device(&self, desc: &DeviceDescriptor, trace_path: Option<&Path>) -> impl Future<Output=Result<(Device, Queue), RequestDeviceError>> + Send`：与原生 wgpu 25 的一参签名不同 —— 通过 trait `PiWgpuAdapter` 统一；两种 feature 下均可写 `adapter.request_device(&desc, None)`。
- Adapter 其余：`features/limits/get_info/get_downlevel_capabilities/is_surface_supported/get_texture_format_features`；特有 `unmake_current()`（将 GL context 释放给其他线程）。

### 5.2 Surface / SurfaceTarget / 呈现

- `SurfaceTarget`（non_exhaustive）：`Window(Box<dyn WindowHandle>)`；Canvas/OffscreenCanvas 分支受 `cfg(any(webgl,webgpu))` + wasm 门控；任意实现 `HasWindowHandle+HasDisplayHandle` 的类型 `Into` 之。
- `Surface::configure(&Device, &SurfaceConfiguration)`；config 约束：**present_mode 仅 Fifo，alpha_mode 仅 Opaque**；`SurfaceConfiguration = wgt::SurfaceConfiguration<Vec<TextureFormat>>`。
- `get_current_texture() -> Result<SurfaceTexture, SurfaceError>`（**suboptimal 恒为 true** 的当前实现事实）；`SurfaceError::{Timeout,Outdated,Lost,OutOfMemory}`。
- `SurfaceTexture::present(self)`：在 submit 记录后执行 swap。
- `get_capabilities(&Adapter) -> SurfaceCapabilities`：源码已提供（README §2.07 的 × 记录已过时，见 §7）。
- `get_default_config`：无。

### 5.3 Device

签名与 wgpu 对齐：`features/limits`；create_*：`create_shader_module(desc) → ShaderModule`、`create_command_encoder(&desc) → CommandEncoder`、`create_bind_group_layout &create_bind_group`、`create_pipeline_layout`、`create_render_pipeline(&RenderPipelineDescriptor)`（**layout 字段必须 Some**；单颜色目标；无 bundle）、`create_buffer &create_texture &create_sampler`；特有 `reset_state()`（重置驱动状态机缓存，处理外部 GL 干扰）；不支持矩阵见 README §2.09。

### 5.4 Queue

- `write_buffer(&Buffer, offset, &[u8])`、`write_texture(ImageCopyTexture, &[u8], ImageDataLayout, Extent3d)`：**实现即时生效**（直接下发 GL，不是「下次 submit 起效」；与 wgpu 文档描述的语义差异，无错误路径 —— 越界即 log+行为未定义，由 GL 兜底）。
- `submit`（空）、`write_texture_jsbuffer`（wasm 私有扩展，接收 js 纹理对象）。
- 不支持：write_buffer_with / on_submitted_work_done / 时间戳 / copy_external_image。

### 5.5 CommandEncoder / CommandBuffer / RenderPass

- `begin_render_pass(&RenderPassDescriptor) -> RenderPass`：descriptor 字段与 wgpu 同形；color_attachments 仅支持一个 Some 项；occlusion/timestamp 字段存在但无能力（QuerySet 为占位类型）。
- `finish(self) -> CommandBuffer`：空；CommandBuffer 空类型。
- `RenderPass<'pass>`：`set_pipeline/set_vertex_buffer/set_index_buffer/set_bind_group(set,&bg,&[DynamicOffset])/set_viewport（wgpu Y 向下坐标，内部翻转换算）/set_scissor_rect/set_stencil_reference/set_blend_constant/draw/draw_indexed`；`draw*_indirect、multi_draw*、execute_bundles、debug 系列、write_timestamp = ×`。**drop（离开作用域）即结束 pass**，与新版 wgpu 无 end() 一致。
- 参数级约束：`first_instance == 0`、`base_vertex == 0`。

### 5.6 资源与管线对象

| 对象 | 支持方法 | 约束 |
|---|---|---|
| Buffer | size/usage/as_entire_binding/as_entire_buffer_binding/slice | 无 map；生命周期受 BindGroup；创建初始化用 util::DeviceExt::create_buffer_init |
| Texture / TextureView | create_view、as_image_copy、尺寸/格式/usage 查询族 | 无 destroy/as_hal；mipmap 自动生成；DDS/ASTC 压缩纹理 |
| Sampler |（仅描述符构造） | 不可用 sampler compare/reduction 类型于 WebGL2；被 BindGroup 握住 |
| BindGroup(Layout) |（描述符构造） | layout 条目受 16-slot 上限；BindingType::Buffer 仅 Uniform |
| ShaderModule | 构造自 `ShaderModuleDescriptor`（WGSL/GLSL Source） | 经 naga→GLSL300es；无 define 宏（§2.19） |
| RenderPipeline | 构造 + set_pipeline | layout Some；单目标；无 bundle/depth-clip |

### 5.7 util 扩展

```rust
use pi_wgpu::util::{DeviceExt, BufferInitDescriptor, TextureDataOrder};
device.create_buffer_init(&BufferInitDescriptor { label, contents, usage });
device.create_texture_with_data(&queue, &tex_desc, TextureDataOrder::LayerMajor, &data);
// env 助手（与 wgpu util 同名，含义按 pi 版简化）：
util::backend_bits_from_env();   // 恒 Some(Backends::GL)，WGPU_BACKEND 不生效
util::power_preference_from_env();// WGPU_POWER_PREF=low|high
util::parse_backends_from_comma_list(s); // wasm→GL
```

## 6. 错误与诊断

- 根 `Error::{OutOfMemory{source}, Validation{source,description}}`：thiserror/Display；用于 request_device/创建失败路径。
- `RequestDeviceError`、`CreateSurfaceError`（当前无字段）、`SurfaceError`（§5.2）。
- 多数不支持路径是 assert!/panic（录制即执行），错误经 `log::trace!` 全量伪代码留痕；`geterror` 用于 WebGL 排错。

## 7. 与 wgpu-rs 差异汇总（迁移必读）

| 差异 | 详情 |
|---|---|
| Adapter::request_device | 双参（trace_path）→ 统一经 `PiWgpuAdapter`，见 §5.1。 |
| Queue::submit | 空实现：录制即执行，无法在 submit 前校验/回滚。 |
| write_buffer 注释 | 「下次 submit 前排队」的表述在本实现中不成立（即时下发）。 |
| Buffer map 族 | 整体缺失 → GPU→CPU 数据无法读回。 |
| copy_* 传输族 | 整体缺失 → 资源转换须自行经 CPU 或 RT 采样。 |
| Compute/QuerySet/Indirect/Bundle/Spirv/push constants | 缺失（需求方需降级或放弃对应特性）。 |
| 多颜色目标 / 多重采样 | begin_render_pass 仅单目标（MSAA 内部经 blit resolve 支持离屏）。 |
| surface.configure 时序 | **强制前置条件**（Web 平台无跨 surface context；见 §4）。 |
| 类型命名遗留 | `ImageCopyTexture/ImageDataLayout` 等保留早期命名；个别常量/描述符字段有增删（编译期即暴露，逐项查 `wgt/mod.rs`）。 |
| Surface::get_capabilities | README §2.07 标记 ×，当前代码已有实现（返回格式/模式能力；configure 校验依据）—— 属文档滞后项。 |
| 生命周期 | §5.6「被 BindGroup 握住」「draw 之后才可销毁」两条硬规则。 |
| 线程 | 录制非多线程安全；资源多线程创建需要锁（single_thread 省略）。 |
