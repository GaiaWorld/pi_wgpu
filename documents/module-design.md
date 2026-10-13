# pi_wgpu 模块设计文档

> 依据 v0.3.7 源码结构与 design.md 设计笔记整理；描述「当前实现是什么」，论证「为什么」以 [design.md](design.md) 为准，接口细节见 [api.md](api.md)。

## 1. 分层架构

```text
应用代码（wgpu-rs 风格 API）
┌─────────────────────────────────────────────────────────┐
│ src/lib.rs  ── use_wgpu? 转发真实 wgpu : 内建 pi_wgpu    │
├─────────────────────────────────────────────────────────┤
│ wgc（WebGPU Core）：对外 API 对象层                      │
│   Instance/Adapter/Device/Queue/CommandEncoder/RenderPass│
│   Buffer/Texture/Sampler/BindGroup( Layout)/Pipeline…    │
├─────────────────────────────────────────────────────────┤
│ wgt（WebGPU Types）：纯类型层（位标志/描述符/限制）      │
├─────────────────────────────────────────────────────────┤
│ hal（GL Hardware Abstraction）：                         │
│   AdapterContext 锁与 make_current ─▶ GLState 全状态机   │
│   GLCache 资源缓存表 ─▶ pi_glow（OpenGL/GLES/WebGL 入口）│
├─────────────────────────────────────────────────────────┤
│ 驱动：OpenGL 3.3+ │ GLES 3.0+ │ WebGL2                  │
└─────────────────────────────────────────────────────────┘
   纵向支撑：util（DeviceExt/env 助手/锁包装）· pi_egl（EGL 绑定）
```

要点：**wgc 不再保留 wgpu-core 的 Context Hub / 共享句柄表，公开对象是对 hal 结构的直接薄封装，调用即执行**（录制即提交），与 WebGPU 语义的差别集中记录在 api.md。

## 2. 模块与文件职责

### 2.1 crate 根

| 文件 | 职责 |
|---|---|
| src/lib.rs | feature 分叉：`use_wgpu` → `pub use wgpu::*`；否则 `pub use pi_wgpu::*`。定义统一 trait `PiWgpuAdapter`（request_device 双参），声明 nightly gate。 |
| src/pi_wgpu/mod.rs | 内部 crate root：re-export `hal::api::*`、`ExposedAdapter`、`wgc::*`、`wgt` 白名单类型/常量；定义 Label、Operations/LoadOp/StoreOp、crate 级 `Error`。 |

### 2.2 wgt（类型层）

- `wgt/mod.rs`：单文件（~7400 行）fork 自 wgpu-types：Backend/Backends、Features/Limits、TextureFormat 及能力位、各 `*Descriptor`、ShaderStages、IndexFormat、SurfaceCapabilities、`Gles3MinorVersion`、常量族（COPY_BUFFER_ALIGNMENT 等）。上层 wgc/hal 均以其为唯一词汇表。

### 2.3 wgc（API 层）

| 文件 | 职责 |
|---|---|
| instance.rs | `Instance`（Share<hal::Instance>）；`request_adapter` 按 DeviceType + PowerPreference 排序挑选（不支持软件 adapter，断言 force_fallback 为 false）；`create_surface`（Window/Canvas/OffscreenCanvas → RawHandle，内部 unsafe 延拓生命周期）。 |
| surface.rs | `SurfaceTarget`、辅助 trait `WindowHandle`、`Surface`.configure/get_current_texture/get_capabilities、`SurfaceTexture`+present、`SurfaceError`、`CreateSurfaceError`。 |
| adapter.rs | `Adapter` 封装 hal::Adapter：`request_device`（同步包装成 ready future）、capabilities 查询族、特有 `unmake_current`。实现 crate 根 `PiWgpuAdapter`。 |
| device.rs | `Device`：create_* 全族 + `reset_state`；single_thread 分支。 |
| queue.rs | `Queue`：write_buffer / write_texture（实现为**直接 GL 调用**，非文档所述「调度到下次 submit」）；submit 空；`write_texture_jsbuffer`（wasm 专用）。 |
| command.rs | `CommandEncoder.finish`（空）、`begin_render_pass`、`RenderPass`（Drop → end_render_pass；set_*/draw* 转发并携带 `AdapterContextLock`）、RenderPassDescriptor/Attachment、QuerySet 仅占位。 |
| buffer / texture / sampler / shader_module / bind_group(_layout) / pipeline_layout / render_pipeline / common_pipeline | 对象句柄封装 + `from_hal/into_hal`；持有 Share 强引用形成 §3 的生命周期链。 |

### 2.4 hal（GL 后端层）

| 文件 | 职责 |
|---|---|
| api.rs | `Api`/`HalApi` 关联类型 trait + 零尺寸标记 `GL`：对齐 wgpu-hal 形态，使上层（含未来多后端可能）以类型参数而非动态分发绑定实现。 |
| mod.rs | hal 根：`ExposedAdapter`、公共对象 re-export、`PrivateCapabilities`/`ShaderID`/`PiBindingType` 等胶水类型。 |
| instance.rs | `InstanceFlags`（DEBUG/VALIDATION/DISCARD_HAL_LABELS…）；Instance 仅持有一个共享 `AdapterContext`；EGL 枚举 = 系统默认显卡（只返回 1 个 adapter）。 |
| egl_impl.rs | **`AdapterContext`（核心）**：`lock: ReentrantMutexWrap + reentrant_count`、egl(EGL Display/Surface)、glow Context；能力探测（PrivateCapabilities）→ features/limits/downlevel；IS_QUALCOMM 特判；`make_current_with_surface / unmake_current`（见 design.md §4）与 try_lock_for 包装。 |
| adapter.rs | expose（读 info/caps）、open（产出 hal (Device,Queue)）、surface_capabilities。 |
| device.rs | 资源创建编排：buffer/texture/sampler/shader/bind group/layout/pipeline。 |
| gl_state.rs | **全局状态机**：GLStateImpl 缓存 blend/depth/stencil/raster/vb/attributes/program/rt/UBO/sampler 槽等；每次 set/draw 前做全量比较（gl_state.rs 头注释）；is_ios18 特判；驱动 `gl_cache`。 |
| gl_cache.rs | 全局**状态缓冲表**：以哈希 key 缓存 VBO/VAO/RBO/纹理/program 等 GL 资产（pi_assets `Asset/Size/Garbageer`），分配先查表、命中比较指针，超时回收（含 VaoGarbage）。 |
| command.rs / queue.rs | pass 录制（begin/end、FBO 绑定、blit resolve）、写缓冲/纹理。 |
| buffer / texture / sampler / bind_group / pipeline / shader_module | 对象到 GL handle/描述的转换；pipeline 维护 set/binding↔GL location 映射（`PiBindingType` 扩展 reflection）；shader_module 调 naga → GLSL300es 编译与链接。 |
| gl_conv.rs | wgt ↔ glow 枚举转换器。 |
| gles.rs | GL 侧常量与上限：**MAX_SAMPLERS/MAX_TEXTURE_SLOTS/MAX_VERTEX_ATTRIBUTES = 16**；blend/raster/depth 等 GL 语义描述结构。 |
| shaders/blit.{vert,frag} | 内部 blit/resolve 着色器源码（离屏 RT → 呈现）。 |
| （egl 绑定 | 外部 crate `pi_egl` + `raw-window-handle` 转接） |

### 2.5 util

| 文件 | 职责 |
|---|---|
| device.rs | `BufferInitDescriptor`、`TextureDataOrder`（LayerMajor=DDS 密度序 / MipMajor=KTX 密度序）、`DeviceExt`（create_buffer_init / create_texture_with_data）。 |
| init.rs | env 助手：backend_bits_from_env（恒 GL）、power_preference_from_env（WGPU_POWER_PREF）、dx12_shader_compiler_from_env 占位、parse_backends_from_comma_list(wasm→GL)。 |
| mutex.rs | `ReentrantMutexWrap/GuardWrap`：`single_thread` 时整体替换为空实现（零开销）。 |

## 3. 对象关系与生命周期

```text
Share 跨线程共享所有权；引用方向（强）：
BindGroup ─▶ {Buffer, TextureView(→Texture), Sampler}
RenderPipeline ─▶ ShaderModule(program) ；BindGroupLayout / PipelineLayout 持有映射表
RenderPass ─▶ (借用) CommandEncoder + AdapterContextLock ⇒ Drop 前资源不可销毁
Queue ─▶ AdapterContext（锁） ；Surface ─▶ AdapterContext + 窗口句柄
```

销毁时序（README §2.02 的设计落点）：
1. wgc 对象 drop → 对应 GL 资源先入 `GLCache` 超时队列，而非立即 delete；
2. `RenderPass` Drop 之后其引用资源方可进入回收；
3. 被 BindGroup 握住的资源必须与 BindGroup 一同 drop（防止悬空槽位重绑）。

## 4. 关键机制设计

### 4.1 Set/Binding → GL 槽位映射（design.md §1/§2 落地）

- pipeline_layout.bind_group_layouts 下标 = shader `set`（硬件约定 0..3）。
- 全局映射表以 `(set, binding) 类型` 为键在 AdapterContext 存活期内分配 UBO 槽 / Sampler 槽（上限 16/16）。
- 同程序同 buffer 尽量重合槽位；每个 RenderPipeline 维护 reorder 数组，把 program 实际布局重排到 layout 声明（允许不同 pipeline 复用同一 program 而拥有不同 layout）。
- 动态偏移：`set_bind_group(set, bg, &dyn_offsets)` 按 layout 中 `has_dynamic_offset` 条目顺序消耗数组元素。

### 4.2 状态机 + 全局缓冲表（FR-11）

调用侧 → GLState 全量字段比较（仅差异值下发）；资源侧 → GLCache 哈希表（同配置命中同一 handle，指针对比），配合 pi_assets 延迟回收；目标是驱动调用数显著低于 wgpu-rs。

### 4.3 着色器编译链（FR-08）

`create_shader_module` → naga parse（WGSL/GLSL）+ validate → `back::glsl`（Version 300 es + std140 + 无 #define）→ program/link → 反射出 (set,binding)↔uniform 名 / attrib location 登记表（`ReflectionInfo`/`PiBindingType`）。错误经 codespan-reporting 格式化输出。

### 4.4 录制-提交流（FR-12/13）

- 所有 wgc 方法 = 取 AdapterContext 锁后的即时 GL 调用；submit/finish 空；`single_thread` 省锁。
- 跨线程：`AdapterContext::unmake_current()` 把 context 从他线程解绑；lock 支持超时获取。

### 4.5 Surface 与交换链（FR-14/NFR）

- configure 触发 EGL Surface 绑定 swapchain 语义（Fifo/Opaque 唯一合法组合），Web 平台无跨 surface context ⇒ 一切 device 使用后置。
- get_current_texture 直接返回可写纹理（suboptimal 恒 true），present = swapBuffers。

## 5. 线程与数据流

```text
创建期（多线程允许）：
  thread A ─ device.create_texture ─▶ GLCache 分配 ─▶ queue.write_texture（即时）
录制/提交期（单线程约定）：
  thread Main ─ encoder.begin_render_pass ─ set_*/draw* ─▶ GLState 差异 ─▶ draw ─▶ present
回收期：
  drop(资源) ─▶ GLGarbage 队列 ─(超时/GC)─▶ 回收线程安全 delete（锁内）
```

## 6. 依赖关系

| 外部 crate | 用途 | 使用者 |
|---|---|---|
| pi_glow（glow） | GL/GLES/WebGL 统一函数入口 | hal 全局 |
| pi_egl | EGL Display/Context/Surface | egl_impl |
| naga（本地 fork patch） | WGSL/GLSL→GLSL300es 编译与反射 | shader_module、gl_state、pipeline |
| pi_share | Share/ShareCell/Weak 跨线程共享所有权 | 各层对象 |
| pi_assets | GPU 资源记账分配 + 超时 GC（Asset/Size/Garbageer） | buffer/texture/gl_cache |
| pi_hash / twox-hash | XHashMap/XHashSet（高速哈希表） | gl_cache/gl_state/env |
| pi_time | Instant 超时回收计时 | gl_cache |
| thiserror/bitflags/lazy_static/ordered-float/derive_more | 基础支撑 | 各层 |
| raw-window-handle 0.6 | 三平台窗口句柄统一 | surface |
| log + profiling + codespan-reporting | 诊断 | 各层 |
| wasm-bindgen/web-sys/js-sys（wasm） | Canvas/OffscreenCanvas/ImageBitmap | surface/queue |
| wgpu（可选） | 对照实现（use_wgpu） | lib.rs |
| ndk-glue（android） | 应用生命周期 | examples |

## 7. 扩展与维护注意点

- 新增纹理格式：wgt::TextureFormat → gl_conv → GLCache 尺寸/mipmap 表 → Adapter::texture_format_features。
- 新增平台特判：统一放 AdapterContext 能力位（PrivateCapabilities / IS_* 原子），避免在算法文件散落 cfg。
- 对齐新 wgpu 版本：以 `wgt/mod.rs` 逐字段 diff 移植；公开层优先保持方法签名不变，语义缩减必须同步更新 api.md 与 README 矩阵。
- 与 README §2.0x 矩阵保持同步：该矩阵是功能需求验收的真相源（本 crate 无独立 CI 测试套件时尤其重要）。
