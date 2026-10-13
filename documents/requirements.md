# pi_wgpu 功能需求文档

| 项 | 内容 |
|---|---|
| 适用版本 | pi_wgpu 0.3.7 |
| API 基线 | wgpu-rs 25.x 大版本兼容子集（类型命名保留少量早期版本痕迹，见《接口文档》） |
| 许可证 | MIT OR Apache-2.0 |
| 关联文档 | [module-design.md](module-design.md)（模块设计） · [api.md](api.md)（接口文档） · [design.md](design.md)（设计笔记/论证） |

## 1. 背景与定位

`Vulkan` / 原生 `WebGPU` 在相当长时间内无法覆盖全部目标平台：微信小游戏只提供 WebGL（2）、iPhone 上 WebGL2 要求 iOS ≥ 15.5，存量 Android / Windows 设备的主力仍是 GL 驱动栈。

**pi_wgpu 的定位：在 GL-3.3 / GLES-3.0 / WebGL2 上，以单线程录制模型实现 WebGPU API 规范的渲染子集**，让上层引擎只维护一套 wgpu 风格代码，在三大平台同时编译运行；并保留 `use_wgpu` 开关，可无缝切回原生 wgpu-rs 作为参考实现进行 A/B 对照。

## 2. 目标用户与使用场景

- 跨平台 3D 引擎 / 小游戏运行时：业务层按 WebGPU API 编码，底层按平台选择 backend。
- 已有 wgpu-rs 代码工程：向 GL 栈移植（含微信小游戏目标）。
- 调试与验证：同一份调用代码在 `use_wgpu` 与内建实现之间对比行为差异。

## 3. 功能需求（FR）

优先级：M = 必须，S = 应该，C = 可选增强。

| 编号 | 需求 | 说明 | 优先级 | 主要落点 / 验收 |
|---|---|---|---|---|
| FR-01 | wgpu 兼容 API 子集 | Instance/Adapter/Device/Queue/CommandEncoder/RenderPass/Buffer/Texture/TextureView/Sampler/BindGroup(Layout)/PipelineLayout/ShaderModule/RenderPipeline/Surface，描述符结构同名同形 | M | `api.md` 全表；`examples/triangle` 在 use_wgpu 与 gl 下行为一致 |
| FR-02 | API 切换开关 | `use_wgpu` 时 lib 直接 re-export 真实 wgpu，并为本机 Adapter 提供统一 trait；否则编译内建实现 | M | `src/lib.rs` |
| FR-03 | Windows 桌面 | OpenGL 3.3+，经 pi_egl 创建 Display/Context 与换帧；运行目录需携带 bin/ 下两个 dll | M | `examples/triangle`、`gui_demo` |
| FR-04 | Android | GLES 3.0+，ndk-glue 生命周期 | M | `cargo apk run --example triangle` |
| FR-05 | Web / 微信小游戏 | WebGL2（HtmlCanvas / OffscreenCanvas）；iOS 需 ≥ 15.5 | M | `wasm-pack test --chrome --example triangle`；`webgl_spector` 抓帧 |
| FR-06 | 渲染通道 | 单个颜色目标 + 可选深度模板附件；Clear/Load、Store/Discard；blend / depth / stencil / cull / front-face / polygon-mode；viewport（wgpu 坐标，内部翻 Y）/scissor；VAO 批处理属性；draw / draw_indexed | M | `examples/clear`、`render_target`；README §2.13 |
| FR-07 | 绑定资源 | BindGroupLayout 的 set(0..3)/binding → GL 全局槽位映射；UBO 动态偏移；texture+sampler 合并映射；上限 16 sampler / 16 texture / 16 attributed（gles.rs） | M | design.md §1–3；image 场景 |
| FR-08 | 着色器 | WGSL / GLSL 源码 → naga 校验 → GLSL 300 es（std140、无 #define）输出；(set,binding)↔uniform 名 / location 映射登记 | M | README §2.19；`examples/test` |
| FR-09 | 纹理 | RGBA8/RGBA16F/RG/BGRA/Depth24·32·stencil8/BC·ASTC 格式表；mipmap 自动生成；DDS/ASTC 压缩纹理 | S | `examples/image`、`compress_texture`（Cargo example test=true） |
| FR-10 | 显存管理 | GPU 资源经 pi_assets 统一记账分配、延迟回收（含 VAO 垃圾清理）；支持创建/销毁与录制分属不同线程 | S | hal/gl_cache.rs；多资源反复装卸回归 |
| FR-11 | 状态机优化 | 调用 GL 前对全局状态做全量比较，相同则跳过；全局缓冲表指针对比；GL 指令数量 ≤ wgpu-rs 同场景 | M | hal/gl_state.rs、gl_cache.rs；对比抓帧 |
| FR-12 | 线程模型 | 指令录制单线程且录制即执行；资源创建/销毁可跨线程：内部互斥锁 + egl make_current / unmake_current（可释放他绑）；`single_thread` feature 去掉锁与上下文搬运 | M | design.md §4；util/mutex.rs；Adapter::unmake_current |
| FR-13 | 提交语义空实现 | `Queue::submit` / `CommandEncoder::finish` / `CommandBuffer` 为空壳；write_* 直接生效 | M | README §2.10–2.12 |
| FR-14 | 呈现 | configure（present_mode 仅 Fifo、alpha 仅 Opaque）、get_current_texture（含 Lost/Outdated 错误态）、SurfaceTexture::present | M | README §2.07–2.08 |
| FR-15 | 可诊断性 | trace 级日志以近乎伪代码形式记录每一步调用；`geterror` 在 wasm 纹理路径插入 glGetError；debug 构建默认开启 VALIDATION 标志 | C | hal/*、texture.rs geterror 路径 |
| FR-16 | 驱动兼容性处理 | iOS18 特判（GLState::is_ios18）、Qualcomm 特判（IS_QUALCOMM）、小米9 debug flag 规避；iOS 14~15.3 VB/IB 共享偏移的性能告警 | M | gl_state.rs、egl_impl.rs、README §3.2 |
| FR-17 | util 扩展 | DeviceExt::create_buffer_init / create_texture_with_data、TextureDataOrder、WGPU_* 环境变量助手（与 wgpu util 同名） | S | util/；README §2.04 |

## 4. 非功能需求（NFR）

| 编号 | 需求 | 说明 |
|---|---|---|
| NFR-01 | 工具链 | `src/lib.rs` 使用 `#![feature(hash_extract_if)]`，当前仅 nightly 可编（README 的 Rust-1.65+ 徽章与之不符，待确认）。 |
| NFR-02 | 依赖环境 | pi_egl / pi_glow 来自私有 registry `yn`；开发用仓库 Cargo.toml 带 `[patch.crates-io] naga = { path = "../wgpu/naga" }`，作为依赖或发布前需自备该 fork 或移除 patch。 |
| NFR-03 | 性能 | 录制路径零拷贝转发、最小化 GL 状态重设；锁开销经 single_thread 消除。 |
| NFR-04 | 错误语义 | thiserror 错误类型；超限/不支持路径以 panic + 清晰 label（与内建实现「调用即生效」的错误呈现方式一致）。 |
| NFR-05 | 体积/依赖面 | 核心路径不依赖 wgpu / ash 等大依赖，`use_wgpu` 为可选。 |

## 5. 明确不做（Out of Scope）

Compute（Pipeline/Pass）、QuerySet / time-stamp、RenderBundle、资源间 copy_* 传输族、Buffer map/unmap、indirect 与多重采样目标、多渲染目标、push constants、SPIR-V 直接输入、poll / 捕获调试面（README §2.03、§2.09–2.17 矩阵列 × 项）。

## 6. 约束与假设

- set 仅 0..3 四个槽；动态偏移仅作用于 `has_dynamic_offset=true` 的条目（按声明顺序）。
- 所有 Device / Queue 使用必须在 `Surface::configure`（窗口 Resumed 后）之后。
- WebGL2 无 Storage Buffer，统一按 Uniform 建模。
- 资源销毁遵循「draw 之后」「RenderPass Drop 之后」「BindGroup 同弃」三条规则（README §2.02）。
- 微信小游戏 WebGL 路径为跨平台首要约束（无跨 Surface Context）。

## 7. 验收与测试矩阵

| 场景 | 样例 | 备注 |
|---|---|---|
| 平台冒烟 | examples/triangle、hello、clear | Win/Apk/wasm-pack 各跑通 |
| RT / 离屏 | examples/render_target | 深度模板与 MSAA resolve 回归 |
| 纹理全格式 | examples/image（Cargo example test = true） | 压缩纹理格式表 |
| 压缩纹理 | examples/compress_texture（test = true） | DDS/ASTC |
| UI 链路 | examples/gui_demo | 动态偏移 UBO 高频更新 |
| 浏览器诊断 | examples/webgl_spector、`test`（test = true） | Spector 抓帧校对 GL 指令量（FR-11） |
| 引擎对照 | use_wgpu 下跑同一样例 | FR-01/FR-02 |
