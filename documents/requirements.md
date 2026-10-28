# pi_wgpu 功能需求（v0.4.1 / wgpu 25 子集）

| 项 | 内容 |
|---|---|
| 适用版本 | 0.4.1（v4 分支） |
| API 基线 | wgpu 25.0.2 / wgpu-types 25.0.0 / naga 25.0.1 |
| 许可证 | MIT OR Apache-2.0 |
| 关联 | [overview](overview.md) · [module-design](module-design.md) · [design](design.md) · [api](api.md) · [migration](migration.md) |

## 1. 背景与定位

自有 3D 引擎需要一套渲染代码同时运行于 Windows、Android、微信小游戏。选定 wgpu 形接口后，底层实际可用的只有 GL 系驱动：微信小游戏仅 WebGL2（iOS ≥ 15.5），存量设备主力是 GLES 3.0 / OpenGL 3.3。因此本库对外提供 wgpu 兼容子集，对内把调用翻译成 GL，并保留 use_wgpu 开关做参考实现对照。

## 2. 目标用户与场景

- 跨平台引擎/小游戏运行时：业务层按 wgpu API 编码，底层按平台选 backend。
- 已用 wgpu-rs 的工程向 GL 栈迁移。
- 调试：同一份代码在 gl 与 use_wgpu 下 A/B 对拍。

## 3. 功能需求（FR）

优先级：M=必须，S=应该，C=可选。

| 编号 | 需求 | 说明 / 验收 |
|---|---|---|
| FR-01 | wgpu 25 兼容子集 | 公开类型与描述符同名同形（types 直接 pub use wgpu_types::* + wgpu api 层同款引用式别名）；官方例片段改依赖即可编译。 |
| FR-02 | 实现切换 | gl（默认）走自产实现；use_wgpu 整面 re-export 官方 wgpu，并提供统一 PiWgpuAdapter 双参 request_device。两者编译期互斥。 |
| FR-03 | Windows | EGL/OpenGL 3.3+；需要运行目录有 bin/ 下两个 dll。hello_triangle 可见且稳定。 |
| FR-04 | Android | GLES 3.0+；EGL 生命周期。编译链就绪，真机验收延后。 |
| FR-05 | Web / 微信小游戏 | WebGL2（HtmlCanvas；OffscreenCanvas 构造可达但建面暂拒）；iOS ≥ 15.5。wasm 编译面 + web_case 案例包。 |
| FR-06 | 渲染通道 | 单颜色目标（Some ≤1，纯深度通道合法）+ 可选深度/模板附件；Clear/Load、Store/Discard（Discard 落 noop，登记）；blend/depth/stencil/cull/front-face/polygon；viewport/scissor；MSAA 4x 中转 resolve；离屏 RTT。 |
| FR-07 | 绑定 | set 0..3；UBO 24 槽、纹理/采样器各 16 槽（GLES3 保底），三族独立紧凑编号；动态偏移按声明序消耗；组合采样器按 IR 配对改道；越限/不对表=断言。 |
| FR-08 | 着色器 | WGSL 单串 / GLSL 按阶段单源 → naga 校验 → GLSL ES 3.0；ADJUST_COORDINATE_SPACE 对齐 wgpu 坐标；反射自动定位 (set,binding)；entry_point=None 自动选唯一入口。 |
| FR-09 | 纹理 | 22 个显式格式条目（R/RG/RGBA 未压缩族、深度族、BC1-3、ETC2 四型）+ ASTC 参数族（14 块尺寸 × Unorm/UnormSrgb，Hdr 拒止）；mipmap 整链重生成；cube/array/3D；纹理视图；压缩直载。 |
| FR-10 | 对象生命周期 | 1:1 直建（Buffer/Texture/Sampler，Share 计数、末引用删）；程序/VAO/FBO/RBO 键复用；FBO/VAO 归零驻留，Program/RBO 归零删除；删除登记死名并刷失镜像。 |
| FR-11 | 状态机优化 | 三档镜像（可回读/整发/直发），差异才下发；驱动指令数不劣于参考实现同场景（抓帧对比）。 |
| FR-12 | 线程合同 | 上下文附着=全库唯一 GL 入口（重入锁 + 跨线程 make/unmake current）；single_thread feature 退化零开销；状态 epoch 防跨编码器水位误用。 |
| FR-13 | 录制与提交 | CommandEncoder 录制通道命令；Queue::submit 按提交序回放（多编码器顺序正确）；Queue::write_* 与 copy_* 调用点即时生效。finish 不再执行。 |
| FR-14 | 呈现 | configure 前全量校验（宽高>0、usage 非空、present=Fifo、alpha ∈ {Auto, PreMultiplied, Opaque}），失败不留半配置；get_current_texture 五态错误；一帧一次 present；离屏屏幕纹理 + 翻转 blit；suboptimal 按真实语义报 false。 |
| FR-15 | 可诊断 | 调用级日志；frame churn / rack 重挂 / 组合表满告警；错误短码；debug 构建 VALIDATION 标志；测试探针。 |
| FR-16 | 驱动特判 | 特判表唯一容器（caps_inner::SPECIAL_ROWS）：Qualcomm/Adreno 清 BC/ASTC 位；iOS18/Safari 清 ASTC 位。新增只加一行。 |
| FR-17 | util 扩展 | DeviceExt::create_buffer_init / create_texture_with_data；backend_bits_from_env / power_preference_from_env / parse_backends_from_comma_list；Web 压缩段直建。 |
| FR-18 | 描述符对齐 | 管线描述符全形照齐 25.0.2：layout:Option（None=违约）、compilation_options、multiview/cache 词位、push_constant_ranges；形宽法窄（给值=短码断言）。 |
| FR-19 | 拷贝与读回 | copy_texture_to_buffer / copy_buffer_to_texture / copy_texture_to_texture（录制订阅，随 submit 执行）；Buffer::get_sub_data 同步读回（替代 map_async 族，登记差异）。 |
| FR-20 | Web 专面 | write_texture_jsbuffer（位图零字节）、copy_external_image_to_texture（img/canvas/bitmap/video）、create_compress_texture_with_data_jsdata；均仅主线程，非位图/离线程=短码留痕。 |

## 4. 非功能需求（NFR）

| 编号 | 需求 | 说明 |
|---|---|---|
| NFR-01 | 工具链 | nightly（naga fork / 依赖所需）；edition 2021。 |
| NFR-02 | 依赖环境 | pi_egl / pi_glow 来自私有 registry「yn」；naga 用本地 fork（[patch.crates-io]）；发布前需自备或移除 patch。 |
| NFR-03 | 性能 | 状态差异下发 + 组合 VAO + 管线级对象复用；锁开销经 single_thread 消除。 |
| NFR-04 | 错误纪律 | 优先 Result<_, ErrCode>；不便上递处 log::error!(短码) 后降级；断言仅「绝不该发生」的编程合同违反，消息用短码（AGENTS.md 强制）。 |
| NFR-05 | 依赖面 | 核心路径不依赖官方 wgpu（仅 use_wgpu 打开）。 |
| NFR-06 | 单设备面 | 单进程单上下文；不枚举多 adapter；不承诺多 Device 并存的帧号/槽表隔离（FRAME 为进程级 static，多 Device 仅多回读、不错误）。 |
| NFR-07 | 资产边界 | 业务资产管理（URL 键、显存记账、异步加载、容量收缩）不在本库；由上游资产层在本库公开 API 之上实现。 |

## 5. 明确不做（Out of Scope）

Compute（Pipeline/Pass）、Storage Buffer/Texture、QuerySet/时间戳/遮挡查询、RenderBundle、indirect / multi-draw、多颜色目标、push constants（词汇在位、恒拒止）、SPIR-V 输入、Buffer 映射族（map_async/get_mapped_range）、多后端、验证层语义复刻、资源延迟回收队列、多 adapter、Android 真机调度（swappy 未开）。

## 6. 约束与假设

- set 仅 0..3；每管线最多 4 个 BindGroupLayout，成员不得重复。
- 所有 Device/Queue 使用须在 Surface::configure 之后（Web 无跨 Surface 上下文）。
- Web 形态必须先 create_surface（注入 GL 入口）再 request_adapter。
- 资源销毁遵守三条硬规则（见 overview §5），库不兜底。
- WebGL2 无 Storage Buffer，统一按 UBO 建模；无 polygonMode；无 base_instance（实例区间起点须 0）。
- draw_indexed 的 base_vertex 必须为 0（参数级约束）。

## 7. 验收矩阵

| 场景 | 样例 | 判据 |
|---|---|---|
| 桌面冒烟 | examples/clear、hello_triangle | 窗口可见、12s 稳定 |
| 纹理/着色 | examples/textured_quad | 渐变方块、坐标方向正确 |
| 混合 | examples/alpha_blend | 半透明可见（非覆盖） |
| 深度 | examples/d32_iso、shadow_cube | 深度正确、D32 不报 0x502 |
| 离屏 | examples/rtt_probe | RTT 内容正确、跨帧稳定 |
| 上传 | examples/upload_probe | 子写/读回/压缩路径 |
| 浏览器 | web_case/build.ps1 + index.html | 画面正确、抓帧指令量不劣于 use_wgpu |
| 单测 | cargo test --lib | 全绿（状态差异、槽表、格式门控、着色器、错误面） |
| wasm 编译 | cargo check --lib --target wasm32-unknown-unknown | 零错误 |
