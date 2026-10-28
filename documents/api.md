# pi_wgpu 接口文档（v0.4.1 导出面）

> 权威文本在各壳 mod.rs 注释；本文按使用顺序汇总。
> 目标：写了 wgpu 25 子集代码的人能直接查「这里有没有、形状是否一样、哪里会被拒」。

## 1. 构建与启用

~~~toml
[dependencies]
pi_wgpu = { version = "0.4", features = ["gl"] }          # 自产实现（默认）
# pi_wgpu = { version = "0.4", features = ["use_wgpu"] }  # 官方 wgpu 薄壳对照
~~~

- 默认 feature = gl；use_wgpu 与 gl 互斥（同时开启 = compile_error），切 use_wgpu 需 --no-default-features。
- 工具链：nightly（依赖 naga fork 等）；edition 2021。
- pi_egl / pi_glow 来自私有 registry「yn」；开发仓 [patch.crates-io] 指向本地 ../wgpu/naga。

## 2. Feature 开关

| feature | 作用 | 现状 |
|---|---|---|
| gl | 自产 GL 实现 | 默认开 |
| use_wgpu | 整面 re-export wgpu 25 + PiWgpuAdapter | 与 gl 互斥 |
| single_thread | 锁与上下文搬运退化零开销 | 已接线（context 芯） |
| loose_upload | 接受 bytes_per_row 非 4 倍数 | 已接线（upload 芯） |
| geterror | wasm 纹理路径 glGetError 探针 | 已声明，src 内无消费者 |
| webgl / webgl_context | 平台行为分支 | 已声明，src 内无消费者 |
| webgpu | — | 已删除（非 GL 系后端，OUT） |

## 3. 根模块导出

~~~text
pi_wgpu::
  ├─ types 模块（pub use wgpu_types::*）与引用式别名
  ├─ entry::*（Instance/Adapter/Device/Queue/Encoder/Buffer/Texture/Sampler/...
  │            BindGroup/BindGroupLayout/PipelineLayout/RenderPipeline/ShaderModule
  │            QuerySet/PipelineCache 占位、RequestDeviceError/CreateSurfaceError）
  ├─ RenderPass
  ├─ Surface / SurfaceTarget / SurfaceTexture / SurfaceError / SurfaceCapabilities
  ├─ Backends / BufferUsages / Extent3d / PredefinedColorSpace / TextureFormat
  ├─ DeviceExt / BufferInitDescriptor
  ├─ backend_bits_from_env / power_preference_from_env / parse_backends_from_comma_list
  ├─ util::{TextureDataOrder, DeviceExt, ...}
  ├─ Error / ErrorFilter / ErrorSource / UncapturedErrorHandler
  └─ PiWgpuAdapter（双参 request_device 的伞面 trait）
~~~

Web（wasm32）额外：CopyExternalImageDestInfo / ExternalImageSource / ImageCopyExternalImage（旧名形）。

## 4. 初始化顺序

~~~text
Instance::new(&InstanceDescriptor { backends: Backends::GL, .. })
  → instance.create_surface(target)?        // Web：此步注入 WebGL2 入口
  → adapter = instance.request_adapter(&RequestAdapterOptions{
        compatible_surface: Some(&surface), .. }).await.expect(...)
  → (device, queue) = adapter.request_device(&DeviceDescriptor{..}, None).await?
  → surface.configure(&device, &SurfaceConfiguration{..})
  ⇒ 此后才可使用 Device / Queue
~~~

要点：

- 单 adapter（EGL 系统默认卡）；软件 adapter 不支持。
- Web 形态必须先建 surface 再 request_adapter（无跨 surface 上下文）。
- request_device 失败返回 Err，设备未建，Adapter 可重试。
- trace 词位仅保持形状：DeviceDescriptor.trace / trace_path 均无效果、不拒止。

## 5. Instance / Adapter

| 方法 | 说明 |
|---|---|
| Instance::new(&InstanceDescriptor) | backends 必须是 GL 的子集；附着+探测在此完成（Web 推迟到 request_adapter）。 |
| create_surface(target) -> Result<Surface, CreateSurfaceError> | 目标类型不符/无附着=Err；无半附着。 |
| request_adapter(&opts).await -> Option<Adapter> | 单卡；失败 None 无副作用；Lost 重建可对同一 Instance 再调用。 |
| Adapter::request_device(desc, trace_path).await -> Result<(Device, Queue), RequestDeviceError> | 36 项 Limits + Features 全量对照。 |
| features() / limits() / get_info() / get_downlevel_capabilities() / get_texture_format_features() / is_surface_supported() | 纯读快照，不失败。 |
| unmake_current() | 让出当前线程 GL 附着（跨线程搬运底座），私有差异。 |

## 6. Surface

| 方法 | 说明 |
|---|---|
| configure(&device, &cfg) | 非 Result。校验：宽高>0、usage 非空、present_mode=Fifo、alpha_mode ∈ {Auto, PreMultiplied, Opaque}。失败 log 短码且不写配置。平台表面只附着一次。 |
| get_current_texture() -> Result<SurfaceTexture, SurfaceError> | 未 configure=Outdated；成功返回屏幕纹理租约（suboptimal=false）。 |
| get_capabilities(&adapter) -> SurfaceCapabilities | 桌面报 BGRA/RGBA 四格式；Web 报 RGBA 两格式；present=Fifo；alpha 三值。 |

SurfaceTexture：

| 成员 | 说明 |
|---|---|
| texture: Texture | 帧背板（真实屏幕纹理）。 |
| suboptimal: bool | 本层恒 false（按真实语义）。 |
| view() -> &TextureView | 颜色附件用视图。 |
| present(self) | 翻转 blit 上屏 + 换帧 + 推进帧号；租约消费。未 present 即 Drop=丢弃（debug log）。 |

SurfaceError：Timeout / Outdated / Lost / OutOfMemory / Other。

## 7. Device 创建面

共同合同：即时创建、描述符在边界一次性全量校验；违约=断言（短码 + label），断言前不发驱动调用、无半建对象。

| 方法 | 关键约束 |
|---|---|
| create_buffer(&BufferDescriptor) -> Buffer | usage 非空且在 COPY_DST/VERTEX/INDEX/UNIFORM/COPY_SRC 内；size>0 且 4 字节倍数。Web 不允许 VERTEX+INDEX 双用。 |
| create_texture(&TextureDescriptor) -> Texture | 宽高≥1、层≥1、mip≥1、samples∈{1,4}；MSAA 仅 D2 单层无 mip；格式在支持表且被 caps 放行；view_formats 同族。 |
| create_sampler(&SamplerDescriptor) -> Sampler | anisotropy_clamp≥1；非 1 时三过滤全 Linear；compare 可选。 |
| create_shader_module(desc) -> ShaderModule | WGSL/GLSL → naga 校验。源错=log 短码+降级（空产物），不 panic；真错延后到管线创建。 |
| create_bind_group_layout(desc) -> BindGroupLayout | binding 域合法、visibility 非空、数组长度>0、类型在子集内。 |
| create_bind_group(desc) -> BindGroup | 与布局逐条对表；资源活态；缓冲偏移 4 对齐且界内；动态条目数登记。 |
| create_pipeline_layout(desc) -> PipelineLayout | 成员 ≤4、set 0..3、不重复；push_constant_ranges 非空=ErrPushRange 断言。 |
| create_render_pipeline(desc) -> RenderPipeline | layout 必须 Some；multiview/cache 必须 None；compilation_options 必须默认；顶点步长/偏移 4 对齐；顶点格式在域内；双源混合因子拒止。 |
| create_command_encoder(desc) -> CommandEncoder | 仅置守卫，零驱动调用。 |
| features() / limits() | 读快照。 |
| reset_state() | 外部 GL 干扰后丢弃镜像并按驱动真态重同步（幂等）。 |
| on_uncaptured_error(handler) / push_error_scope(filter) / pop_error_scope().await | 错误通道；缺省未捕获=log 降级（上游默认 panic，差异登记）。 |

## 8. Queue

| 方法 | 说明 |
|---|---|
| write_buffer(&Buffer, offset, &[u8]) | 即时；offset 4 对齐、界内；违约 log(UploadErr)，debug 不下发。 |
| write_texture(TexelCopyTextureInfo, data, TexelCopyBufferLayout, size) | 即时；矩形/面/层按创建期快照预校验；bytes_per_row 0=紧凑，否则须 256 倍数（loose_upload 放宽 4）。 |
| submit(iter of CommandBuffer) | 按参数顺序回放各命令表；空列表=no-op。 |
| write_texture_jsbuffer(texture, JsValue) [wasm] | 位图零字节直传；仅主线程。 |
| copy_external_image_to_texture(src, dst, size) [wasm] | img/canvas/bitmap/video → 纹理；单层 2D；主线程。 |

## 9. CommandEncoder / CommandBuffer / RenderPass

CommandEncoder：

| 方法 | 说明 |
|---|---|
| begin_render_pass(&RenderPassDescriptor) -> RenderPass | 记录 Begin；通道借用编码器；至多一个开启通道。 |
| copy_texture_to_buffer(source, destination, extent) | 记录拷贝命令；submit 时执行（PBO+glFinish）。约束：活名、COPY_SRC/COPY_DST、aspect=All、非深度族/数组、mip 域内。 |
| copy_buffer_to_texture(source, destination, extent) | 同上；目标单层、mip 域内、bytes_per_row 0=256 对齐紧凑或 256 倍数。 |
| copy_texture_to_texture(source, destination, extent) | CPU 中转两程；双方单层同像素宽、非深度族。 |
| finish() -> CommandBuffer | 交出命令表，无执行。 |

RenderPass（Drop = 记录 End；回放时执行）：

| 方法 | 约束 |
|---|---|
| set_pipeline(&RenderPipeline) | 程序/VAO/深度/混合/图元/写掩码随管线差异下发；死句柄=断言。 |
| set_bind_group(set, &BindGroup, &[DynamicOffset]) | set 0..3；偏移数=声明数；槽/句柄/类型预检失败=断言且不改绑定。 |
| set_vertex_buffer(slot, BufferSlice) | 步长由管线布局决定；记录槽位供组合 VAO。 |
| set_index_buffer(BufferSlice, IndexFormat) | 记录格式与有效区间；draw_indexed 校验。 |
| set_viewport(x,y,w,h,dn,up) | 存原始值（负尺寸=断言）；Y 翻换由汇合层处理（当前版本存原始值，待目标尺寸面）。 |
| set_scissor_rect(x,y,w,h) | 存原始值。 |
| set_stencil_reference(u32) | 差异才下发。 |
| set_blend_constant(Color) | 差异才下发。 |
| set_push_constants(offset, &[u8]) | 恒拒止（断言，GL 无 push constants）。 |
| draw(vertices, instances) | 管线+属性就绪；instances.start 必须 0；glDrawArraysInstanced。 |
| draw_indexed(indices, base_vertex, instances) | base_vertex 必须 0；索引已绑；区间在界内；instances.start=0；glDrawElementsInstanced。 |

RenderPassDescriptor 约束：

- color_attachments 中 Some ≤1（0=纯深度通道）；深度槽不接受帧视图；色槽拒深度族。
- timestamp_writes / occlusion_query_set = Some 恒拒（QuerySet 无构造面）。
- color view 宿主必须活到通道结束（通道持行强引用）。
- MSAA resolve：附件 sample=4，resolve 目标 sample=1、D2、单层、同格式。

## 10. 资源对象

Buffer：slice(RangeBounds) / as_entire_buffer_binding() / as_entire_binding() / usage() / get_sub_data(offset,size)（同步读回，替代 map_async）。Clone/PartialEq（活名+size）。

Texture：create_view(&TextureViewDescriptor)（Cube/CubeArray 需宿主目标匹配；base/count 切片暂不生效）、destroy()（即刻删+死名）、size/width/height/depth_or_array_layers/mip_level_count/sample_count/dimension/format/usage。

Sampler：无方法。

TextureView：纯值；帧视图无宿主、target=0（不可用于绑定组）。

BindGroupLayout / BindGroup / PipelineLayout / ShaderModule：无公开查询方法；生命周期靠强引用。

RenderPipeline：无公开方法；Drop 释放 program/VAO。

QuerySet / PipelineCache：占位类型，无构造路径；词汇在场、给值=断言。

## 11. util

| 项 | 说明 |
|---|---|
| DeviceExt::create_buffer_init(&BufferInitDescriptor) | create_buffer + write_buffer 组合。 |
| DeviceExt::create_texture_with_data(&Queue, &TextureDescriptor, TextureDataOrder, &[u8]) | create_texture + mip0 单段写入；order 多段排布未实现（登记差异）。 |
| DeviceExt::create_compress_texture_with_data_jsdata(...) [wasm] | 压缩纹理按 mip/层逐段 ArrayBufferView 零拷贝直灌。 |
| backend_bits_from_env() | 恒 Some(Backends::GL)；WGPU_BACKEND 不生效。 |
| power_preference_from_env() | WGPU_POWER_PREF=low/high；单卡不选路。 |
| parse_backends_from_comma_list(s) | 只识别 gl，其余忽略。 |

## 12. 错误面

~~~rust
pub enum Error {
    OutOfMemory { source: ErrorSource },
    Validation { source: ErrorSource, description: String },
    Internal   { source: ErrorSource, description: String },
}
pub type ErrorSource = String;   // 差异：上游是 dyn Error 盒
pub enum ErrorFilter { OutOfMemory, Validation, Internal }
pub trait UncapturedErrorHandler: Fn(Error) + Send + 'static {}
~~~

- 作用域栈：push 开层，捕获按最内匹配层；pop 取回首条。
- 缺省未捕获 = log 降级；不 panic（上游默认 panic，差异登记）。
- 三个显式 Result 面之外的创建违约仍走断言（短码+label），尚未全部迁移为 Error 通道（登记）。

## 13. 着色器与坐标约定（产品端必读）

1. 输入：WGSL 单串，或 GLSL 按阶段单源（ShaderStage::Vertex/Fragment），#version 由源码自承载。
2. 输出：GLSL ES 3.0；ADJUST_COORDINATE_SPACE 生效 → 顶点 y 取反、z 映射到 [-1,1]。
3. 正面绕序：库内已按镜像反向映射；业务侧按 wgpu 约定设置 FrontFace 即可。
4. 上屏：present 自动翻转；不要在业务 shader 里再翻 y。
5. 纹理坐标：与 wgpu 一致（行 0=顶）；上传数据不要预翻。
6. entry_point：建议显式给；None 时按该 stage 唯一入口选，多入口会断言。
7. bind group：set 仅 0..3；WGSL @group(n) 与 Rust set 参数必须一致；每 stage 的 binding 由 naga 反射自动定位，无需手写 location。
8. 组合采样器：WGSL 的 texture+sampler 会合并为 GLSL sampler2D；库按 IR 配对把采样器绑到纹理单元，业务无需干预。

## 14. 已知差异速查（详见 migration.md）

| 项 | 现状 |
|---|---|
| submit 语义 | v4 起按提交序回放；write_*/copy_* 仍即时。 |
| Buffer 映射 | 不移植 map_async/get_mapped_range；用 get_sub_data 同步读。 |
| polygonMode | Web 恒填充；非 Fill 只 warn 一次。 |
| sample_mask | 仅镜像记录，无驱动下发面。 |
| Clear | CLEAR_SCISSORED 默认开，按裁剪框延后清（非标准）。 |
| 默认帧缓冲通道 | 不复位 viewport/scissor（无尺寸来源）。 |
| 多颜色目标 | 不支持（Some ≤1）。 |
| 多采样 resolve | CPU readPixels 中转（滤波采样），非精确 blit。 |
| 存储缓冲/纹理 | 不支持；WebGL2 统一 UBO 建模。 |
| 间接绘制/多 draw | 不支持；未合并 WEBGL_multi_draw。 |
| OffscreenCanvas | 构造可达，create_surface 暂拒。 |
| Android | 编译链就绪，真机未验。 |
| 错误通道 | 仅三处 Result；其余断言（短码+label）。 |
