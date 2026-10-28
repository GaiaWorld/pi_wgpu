# pi_wgpu

![Rust](https://img.shields.io/badge/Rust-nightly-orange)
![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue)
![Platform](https://img.shields.io/badge/Platform-Windows%7CAndroid%7CWeb-green)

**跨平台图形渲染库**：在 OpenGL 3.3 / GLES 3.0 / WebGL2 之上实现 wgpu 25 兼容子集。

> 运行前需把 bin/ 下两个 dll 拷贝到运行目录或 exe 所在目录（Windows）。
> 版本 0.4.1；API 基线 wgpu 25.0.2 / naga 25.0.1。

## 定位

自有引擎只维护一套 wgpu 风格渲染代码，同时跑 Windows、Android、微信小游戏。微信小游戏只提供 WebGL2（iOS ≥ 15.5），存量设备主力仍是 GL 驱动栈，因此需要「对外 wgpu 形接口、对内 GL 直替」的一层。

~~~text
上层引擎（wgpu 风格）
   ├─ feature = use_wgpu → 整面 re-export 官方 wgpu（A/B 对照）
   └─ feature = gl（默认）→ pi_wgpu 自产实现 → GL / GLES / WebGL2
~~~

## 特性

- wgpu 25 子集：Instance / Adapter / Device / Queue / Encoder / RenderPass / Buffer / Texture / Sampler / BindGroup(Layout) / Pipeline(Layout) / ShaderModule / Surface。
- 单颜色目标 + 可选深度模板；blend / depth / stencil / cull；离屏 RTT；MSAA 4x 中转 resolve。
- 录制订阅：CommandEncoder 记录，Queue::submit 按提交序回放；写路径即时生效。
- 三档状态镜像（可回读 / 整发水位 / 直发回写）+ 死名刷失 + 帧门 + 世代失效。
- 1:1 直建（Buffer/Texture/Sampler）+ 四类键复用（Program/VAO/FBO/RBO）；FBO/VAO 归零驻留。
- 绑定三族独立紧凑槽（UBO / 纹理 / 采样器，各 24），组合采样器自动改道，动态偏移按声明序。
- WGSL / GLSL → GLSL ES 3.0；ADJUST_COORDINATE_SPACE 对齐 wgpu 坐标；反射自动定位绑定。
- 格式表：22 显式条目 + ASTC 参数族；mipmap 整链重生成；BC/ETC2/ASTC 直载；cube/array/3D。
- 离屏屏幕纹理 + present 翻转 blit；suboptimal 按真实语义报 false。
- Web 专面：位图零字节直传、外部图源、压缩段直建。
- 错误纪律：三处显式 Result，其余短码断言 / log 降级，不 panic 让运行崩溃。

## 平台

| 平台 | GL 版本 | 状态 |
|---|---|---|
| Windows | OpenGL 3.3+ | 已验证 |
| Android | GLES 3.0+ | 编译链就绪，真机待验 |
| Web / 小程序 | WebGL2（iOS ≥ 15.5） | 已验证（web_case） |

## 依赖集成

~~~toml
[dependencies]
pi_wgpu = { version = "0.4", features = ["gl"] }
# pi_wgpu = { version = "0.4", features = ["use_wgpu"] }  # 官方 wgpu 对照
~~~

- 需要 nightly 工具链。
- pi_egl / pi_glow 来自私有 registry「yn」；开发仓 [patch.crates-io] 指向本地 ../wgpu/naga，作为外部依赖或发布前需自备该 fork 或移除 patch。

## 快速开始

~~~bash
cargo run --example hello_triangle --features gl
cargo run --example textured_quad --features gl
cargo run --example alpha_blend --features gl
cargo run --example rtt_probe --features gl
~~~

浏览器案例包：

~~~bash
powershell -ExecutionPolicy Bypass -File .\web_case\build.ps1
python -m http.server 8731 -d web_case
# 打开 http://localhost:8731
~~~

Android：

~~~bash
cargo apk run --example hello_triangle --lib
adb install target/debug/apk/examples/hello_triangle.apk
~~~

## 初始化顺序

~~~text
Instance::new → create_surface → request_adapter → request_device → Surface::configure
~~~

Web 必须先 create_surface（注入 WebGL2 入口）再 request_adapter；所有 Device/Queue 使用须在 configure 之后。

## 验证

~~~bash
cargo test --lib
cargo check --all-targets
cargo check --lib --target wasm32-unknown-unknown
~~~

## 文档

| 文档 | 内容 |
|---|---|
| [总览](documents/overview.md) | 定位、架构、执行模型、机制速览 |
| [需求](documents/requirements.md) | FR/NFR、边界、验收矩阵 |
| [模块设计](documents/module-design.md) | 模块清单、依赖图、数据结构、生命周期、线程 |
| [关键机制](documents/design.md) | 状态缓存、提交回放、RTT、VAO、上传、上屏、踩坑表 |
| [接口](documents/api.md) | 公开面、约束、着色器与坐标约定 |
| [迁移](documents/migration.md) | 与 wgpu 25 的行为/形状差异、迁移清单 |
| [导出索引](documents/exported-api.md) | 公开符号清单 |

设计册（实现契约来源）：

| 册 | 主题 |
|---|---|
| [design/pi-wgpu-rebuild](design/pi-wgpu-rebuild/00-index.md) | 从零重建：模块/依赖/数据结构/调用链 |
| [design/desc-align-wgpu25](design/desc-align-wgpu25/00-index.md) | 描述符逐字段对齐 wgpu 25 |
| [design/sampler-unit-assign](design/sampler-unit-assign/00-index.md) | 采样器/UBO 指位与槽位独立编号 |

## 已知缺口

默认帧缓冲通道不复位 viewport/scissor；CLEAR_SCISSORED 非标准清屏语义（默认开）；Web 无 polygonMode；sample_mask 仅镜像；多颜色目标/存储缓冲/间接绘制不支持；Buffer 映射族不移植（用 get_sub_data）；OffscreenCanvas 建面暂拒；Android 真机待验；Limits 为静态保底表。

## 许可证

MIT OR Apache-2.0。
