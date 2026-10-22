# pi_wgpu 设计总览 — 目标 · 架构 · 运行流程 · 特性

> 版本基线 v0.3.7。本文为项目设计内容的单篇总纲，提炼自 README 与 documents/{requirements, module-design, design, api, exported-api} 及 design/pi-wgpu-rebuild/plan/；各单篇文档仍为真相源。

## 1. 项目定位与设计目标

**一句话定位**：在 OpenGL 3.3 / GLES 3.0 / WebGL2 驱动之上，以「单线程、录制即执行」模型实现 WebGPU（wgpu-rs 25.x 大版本兼容子集）API 的渲染实现，让上层引擎只维护一套 wgpu 风格代码，同时编译运行于 Windows / Android / Web(小程序) 三大平台。

**为什么不直接用原生 wgpu-rs**：Vulkan / 原生 WebGPU 短期内无法广泛覆盖目标平台 —— 微信小游戏仅提供 WebGL2，iPhone 上 WebGL2 要求 iOS ≥ 15.5，存量 Android / Windows 设备主力仍是 GL 驱动栈。

设计目标（对应需求文档 FR 编号）：
1. **API 兼容子集**（FR-01）：Instance / Adapter / Device / Queue / CommandEncoder / RenderPass / Buffer / Texture / Sampler / BindGroup(Layout) / Pipeline(Layout) / ShaderModule / Surface 及描述符结构同名同形。
2. **无缝切换**（FR-02）：`use_wgpu` feature 下本 crate 直接 re-export 原生 wgpu，同一份上层代码可在内建实现与参考实现间 A/B 对照。
3. **三平台贯通**（FR-03/04/05）：Windows 走 EGL+OpenGL 3.3+；Android 走 GLES 3.0+（ndk-glue 生命周期）；Web 走 WebGL2（HtmlCanvas / OffscreenCanvas）。
4. **性能**（FR-11）：GLState 全状态比较 + GLCache 资源表，同场景下发 GL 指令数量 ≤ wgpu-rs。
5. **极简语义**（FR-13）：submit / finish 为空壳，录制即执行；无 wgpu-core 的 Context Hub / 验证层 / 命令队列。
6. **渲染特性子集**（FR-06/07/08/09）：单颜色目标 + 深度模板附件、blend / depth / stencil / cull、viewport（wgpu Y 向下坐标，内部翻 Y）、VAO 批处理、UBO-纹理-sampler 绑定与动态偏移、WGSL/GLSL → GLSL 300 es、mipmap 自动生成、DDS/ASTC 压缩纹理。
7. **资源治理**（FR-10/12）：pi_assets 记账分配 + 超时延迟回收（含 VAO 垃圾）；资源创建/销毁允许跨线程。

**明确不做（Out of Scope）**：Compute（Pipeline/Pass）、QuerySet / 时间戳、RenderBundle、copy_* 传输族、Buffer map/unmap（无 GPU→CPU 回读）、indirect 绘制、多渲染目标、push constants、SPIR-V 输入、poll 与捕获调试面。

## 2. 架构分层

```text
应用代码（wgpu-rs 风格 API）
┌────────────────────────────────────────────────┐
│ src/lib.rs ── use_wgpu? 转发真实 wgpu : 内建实现    │
├────────────────────────────────────────────────┤
│ wgc ── 对外 API 对象层                             │
│   Instance/Adapter/Device/Queue/Encoder/Pass/资源  │
│   hal 结构的直接薄封装：调用即执行                  │
├────────────────────────────────────────────────┤
│ wgt ── 纯类型层（fork 自 wgpu-types）              │
│   描述符/位标志/Limits/TextureFormat/常量，唯一词汇表│
├────────────────────────────────────────────────┤
│ hal ── GL 后端层                                   │
│   AdapterContext：EGL/Context + 重入锁 + make_current│
│   GLState：全状态机镜像，差异才下发                 │
│   GLCache：哈希资源表，handle 复用 + 超时回收        │
│   pi_glow：GL / GLES / WebGL 统一函数入口            │
├────────────────────────────────────────────────┤
│ 驱动：OpenGL 3.3+ │ GLES 3.0+ │ WebGL2             │
└────────────────────────────────────────────────┘
 纵向支撑：util（DeviceExt/env 助手/锁包装）· pi_egl（EGL 绑定）· naga（着色器编译）
```

Feature 开关：

| feature | 作用 |
|---|---|
| gl（默认） | 选择内建 GL 实现 |
| use_wgpu | lib 直接 re-export 原生 wgpu（参考实现 / A/B 对照） |
| single_thread | 去除重入锁与 make_current 搬运，要求全部调用同线程（零开销） |
| geterror | wasm 纹理路径插入 glGetError（排障） |
| webgl / webgpu | 驱动 Canvas/OffscreenCanvas 等 cfg 分支；按 Cargo 定义不会自动开启，需 --cfg 注入 |
| webgl_context | WebGL 上下文下的 gl_state / pipeline 行为分支 |

## 3. 运行流程

### 3.1 初始化（必须顺序）

```text
Instance::new(Backends::GL)
 → create_surface(window / canvas)      # SurfaceTarget: raw-window-handle 0.6 / Canvas / OffscreenCanvas
 → request_adapter(compatible_surface)  # EGL 枚举 = 系统默认显卡，仅 1 个 adapter；软件 adapter assert 拒绝
 → adapter.request_device(desc, None)   # 双参签名经 PiWgpuAdapter trait 与原生 wgpu 统一
 → Surface::configure(device, cfg)      # present_mode 仅 Fifo、alpha_mode 仅 Opaque
 ⇒ 此后才允许一切 Device / Queue 使用（Web 平台无跨 Surface Context）
```

### 3.2 每帧流程（录制即执行，无队列层）

```text
surface.get_current_texture()            # acquire 后缓冲；当前实现 suboptimal 恒 true
encoder = device.create_command_encoder  # 轻量对象
pass = encoder.begin_render_pass(desc)   # 绑定 FBO、执行 clear/load
  pass.set_pipeline / set_bind_group / set_vertex_buffer / set_index_buffer / set_viewport ··
      # 每次调用：取 AdapterContext 锁 → GLState 全量比较 → 仅差异下发 GL
  pass.draw / draw_indexed               # VAO 批处理属性，立即绘制；first_instance=0, base_vertex=0
Drop(pass)                             # 离开作用域 = end_render_pass（新版 wgpu 无 end() 一致）
encoder.finish() / queue.submit()      # 空壳，无实际动作
frame.present()                        # eglSwapBuffers + GLFence 记位；重复 present = panic
```

### 3.3 资源生命周期

```text
创建   device.create_* → GLCache 哈希查表（同配置命中同一 handle）→ 绑定槽分配
       （(set,binding) → UBO/Sampler 全局槽，上限 16/16/16attributed，满了 label panic）
上传   queue.write_buffer / write_texture → 即时生效（直接下发 GL；非「下次 submit 起效」）
编译   ShaderSource(WGSL/GLSL) → naga parse+validate → GLSL 300 es（std140，无 #define）
       → program 编译链接 → 反射登记 (set,binding)↔uniform 名 / attrib location
回收   drop(资源) → GLCache 超时队列（pi_assets Garbageer）→ 锁内延迟 delete
硬规则×3：资源须 draw 之后方可销毁；RenderPass 绑定资源须 pass Drop 之后；被 BindGroup 握住的资源与 BindGroup 同弃
```

### 3.4 线程与上下文（单线程内核 + 可选跨线程资源）

```text
录制/执行（单线程合同）：main 线程 lock → make_current → GL 调用 → 解锁
创建/销毁（允许多线程）：AdapterContext 重入锁 ReentrantMutexWrap + egl make/unmake_current 搬运
                       Adapter::unmake_current() 主动把 context 让给其他线程
single_thread feature：锁与搬运整体退化为空实现（零开销）
锁内纪律：禁止 await / 回调再入锁（死锁）
```

## 4. 核心特性与机制要点

| 特性 | 机制 | 关联 |
|---|---|---|
| GL 指令量优化 | GLState 全字段镜像比较，差异才下发；GLCache 以哈希 key 缓存 VBO/VAO/RBO/纹理/program，指针对比命中复用 | FR-11 |
| 绑定模型 | pipeline_layout 下标 = shader set（硬件约定 0..3）；全局 (set,binding)→槽 映射；每个 RenderPipeline 持 reorder 数组，同一 program 可供不同 layout 的 pipeline 复用 | design.md §1/§2 |
| 动态偏移 | set_bind_group(set, bg, offsets) 按 layout 中 has_dynamic_offset 条目声明顺序消耗 | design.md §3 |
| 离屏与合成 | render_target 支持；MSAA 经内部 blit resolve（shaders/blit.vert/frag）离屏落地 | examples |
| 格式与压缩 | RGBA8 / RGBA16F / RG / BGRA / Depth24·32+stencil8 / BC / ASTC；mipmap 自动生成；DDS/ASTC 直载 | FR-09 |
| 驱动特判 | iOS18（GLState::is_ios18）、Qualcomm（IS_QUALCOMM）、小米 9 debug flag 规避；iOS 14~15.3 VB/IB 共享偏移性能告警 | FR-16 |
| 可诊断性 | trace 级日志按伪代码记录每一步调用；geterror 检查；debug 构建默认 VALIDATION 标志 | FR-15 |
| 外部 GL 干扰恢复 | Device::reset_state() 重建状态机缓存 | api.md §5.3 |
| 显存记账 | pi_assets Asset/Size/Garbageer 统一分配与超时回收（含 VaoGarbage） | FR-10 |

## 5. 平台支持与限制

| 平台 | 图形基线 | 关键约束与注意事项 |
|---|---|---|
| Windows | OpenGL 3.3+ | 不支持 Geometry Shader；运行目录需携带 bin/ 下两个 dll（EGL/GL 运行时）；pi_egl 创建 Display/Context |
| Android | GLES 3.0+ | ndk-glue 生命周期；cargo apk 构建；部分纹理格式需扩展支持 |
| Web / 微信小游戏 | WebGL2 | iPhone/iPad 需 iOS ≥ 15.5；wasm-pack test --chrome 验证；私有扩展 write_texture_jsbuffer；无跨 surface context ⇒ configure 必须前置 |
| 通用 | — | set 仅 0..3；UBO/Sampler 槽 ≤16；WebGL2 无 Storage Buffer 统一按 UBO 建模；viewport 为 wgpu 坐标内部翻 Y |

## 6. 错误与失败语义

- Result 面仅三处：request_device → RequestDeviceError、create_surface → CreateSurfaceError、get_current_texture → SurfaceError{Timeout/Outdated/Lost/OutOfMemory}。
- 其余不支持/越界路径 = assert!/panic + 清晰 label（与「调用即生效」的错误呈现方式一致）；写越界不报错，靠 GL 兜底 + log。
- 根 Error::{OutOfMemory, Validation}（thiserror/Display）。

## 7. 工程要点与依赖

- 工具链：内建实现使用 `#![feature(hash_extract_if)]`，当前需 **nightly**（README 的 Rust-1.65+ 徽章与之不符，待确认）。
- 私有环境：pi_egl / pi_glow 来自私有 registry `yn`；开发仓库 `[patch.crates-io] naga = { path = "../wgpu/naga" }`，发布或外部使用前需自备该 fork 或移除 patch。
- 关键依赖：naga（WGSL/GLSL→GLSL300es 编译与反射）、pi_share（跨线程共享所有权）、pi_assets（GPU 资源记账+GC）、pi_hash/twox-hash（高速哈希）、raw-window-handle 0.6（三平台窗口句柄）、wgpu（仅 use_wgpu 对照）。
- 版本对齐策略：以 wgt/mod.rs 为唯一词汇表逐字段 diff 移植 wgpu-types 新版；公开层优先保持方法签名不变，语义缩减同步更新 api.md 与 README §2.0x 矩阵（验收真相源）。

## 8. 验证与验收矩阵

| 场景 | 样例 | 平台/方式 |
|---|---|---|
| 平台冒烟 | triangle / hello / clear | Win: cargo run · Android: cargo apk run · Web: wasm-pack test --chrome |
| 离屏 RT / 深度 | render_target | 全平台 |
| 纹理全格式 / 压缩 | image · compress_texture（Cargo example test=true） | 全平台 |
| UI 高频动态 UBO | gui_demo | 全平台 |
| GL 指令量对照 | webgl_spector 抓帧 vs use_wgpu 同样例 | Web / Win（FR-11） |

## 9. 文档导航

- [requirements.md](requirements.md) — 功能需求 FR-N/NFR/验收矩阵（真相源）
- [module-design.md](module-design.md) — 源码分层、模块职责、对象生命周期与依赖
- [design.md](design.md) — 设计论证笔记（绑定模型 / 槽复用 / 多线程）
- [api.md](api.md) — 导出接口全表 + 与 wgpu-rs 差异汇总（迁移必读）+ §8 着色器与坐标约定（产品端必读）
- [exported-api.md](exported-api.md) — 全部 pub 导出符号清单（v0.3.7 自动扫描）
- ../README.md — 快速开始、平台支持矩阵、§2.0x 支持度
