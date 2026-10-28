# pi_wgpu ![Rust](https://img.shields.io/badge/Rust-1.65%2B-orange) ![License](https://img.shields.io/badge/License-MIT-blue) ![Platform](https://img.shields.io/badge/Platform-Windows%7CAndroid%7CWeb-green)

**注意：** 运行前，需要将 bin/两个dll 拷贝 到 运行目录，或者 exe所在目录

**跨平台图形渲染库** - 基于WebGPU接口规范的OpenGL/GLES后端实现

## 项目概述
pi_wgpu 是针对WebGPU API规范的轻量级OpenGL/GLES后端实现，主要特性包括：
- 🚀 **跨平台支持**：Windows/Android/Web三大平台
- ⚡ **性能优化**：通过GL状态机优化减少驱动调用次数
- 🛠️ **兼容性**：支持WebGL2标准及扩展（DDS/ASTC压缩纹理）
- 📦 **模块化设计**：与wgpu-rs生态无缝集成

## 功能特性
- ✅ **核心功能**
  - WebGL2标准兼容（[规范文档](https://www.khronos.org/files/webgl20-reference-guide.pdf)）
  - 压缩纹理扩展支持（DDS/ASTC）
  - 多线程资源管理
- 🖥️ **平台支持**
  | 平台 | GL版本 | 测试状态 |
  |------|--------|----------|
  | Windows | OpenGL 3.3+ | ✔️ 已验证 |
  | Android | GLES 3.0+ | ✔️ 已验证 |
  | Web/小程序 | WebGL2 | ✔️ 已验证 |

## 🧠 实现要点（GL / WebGL2 直替路线）

本库把 wgpu 25 子集的调用翻译成 GL（桌面 GL 3.3 / WebGL2）。核心不是「逐条翻译」，而是**用状态镜像换掉多余驱动调用**。以下要点均由实机问题倒逼成型。

### 1. 身份层：u64 柄唯一闸口

- 所有 GL 对象对外只暴露 `GlHandle(u64)`，内部经 gid 映射到驱动名；Web 端含 version 位，杜绝「句柄复用=别名」。
- 桌面端须知：GL 对象名**可被驱动复用**，所以「上次下发的名字」不能作为「驱动仍持有该对象」的证据——删除要登记死名，镜像遇死名**丢弃期望**，而不是假装已下发。

### 2. 三档状态缓存

| 档 | 覆盖组 | 机制 |
|---|---|---|
| 可回读档 | 活动单元、开关、比较函数、视口、裁剪框、深度掩码 | 每帧首次 flush 回读驱动实值，差异才重下发 |
| 整发档 | 程序、VAO、FBO、纹理/sampler/UBO 槽 | 按「上次实际下发」水位比对，差异才发 |
| 直发路 | `set_*` 立即下发的组 | 下发后**必须回写水位**，否则下一轮 flush 判失配重发 |

三条纪律：能力开关只能 `isEnabled`（`getParameter` + cap pname = INVALID_ENUM）；名称类只能比水位不能比驱动；死名表非空时整组强发一轮。

### 3. WebGPU 每通道默认态

每个 render pass 起始：`viewport=(0,0,w,h)`、`depthRange=0..1`、`scissor=全附件且使能`。GL 不随 FBO 切换复位——继承上通道的小视口会表现成「只渲染一角、其余是清屏色」（产品实测闪黑）。

### 4. 清除语义与守卫带

`LoadOp::Clear` 清整张附件、不受裁剪影响：临时关裁剪、写掩码全开、桌面关 sRGB，清毕按镜像归还。

> 现留诊断口 `CLEAR_SCISSORED`（默认开，暂留）：清屏推迟到本通道首个 draw，按当时裁剪框清子区域。用于上层「每元素一 pass + Clear」的结构（否则同一目标反复全清、内容互相覆盖 → 闪）。此为非标准语义，仅此口子使用。

### 5. RTT 复用：形状键 + 驻留 + 附件差分

- FBO 键 = 形状（格式哈希 × samples × 目标），纹理名不入键；release 计数归零**不删**，ensure 沿名复用（行驻留）。
- `ROW_ATT` 维护附件签名（色名/目标、深名/目标、深度挂点），begin 时差分：签名不变只 bind，变才重挂。GL 在纹理删除时会**自动把附件摘下**，故所有纹理删除路径都必须同步摘签名。
- 交错占用：`pick_rtt_row` 同签架优先 → 空架 → 备用架线性探测；读回路只认同签架。

### 6. 顶点：组合 VAO 一次整备

- 键 = 布局哈希 × 按槽 `(缓冲, 偏移 + 实例 base × stride)`；首次整备 pointer/enable/divisor，此后同组合只 `bindVertexArray`。
- 容量 256 + LRU 淘汰（目标 7/8）+ **本帧使用保护**（`last_frame`）+ 草稿键不计容量 + 每布局草稿 VAO 兜底。
- `set_vertex_buffer` / `set_index_buffer` 退化为纯记录，索引绑定在组合落位时写进 VAO。

### 7. 上传与上屏

- 压缩纹理行距按块行判据 `ceil(w/block_w) * block_bytes`，逐块行喂（WebGL2 忽略 `UNPACK_ROW_LENGTH`）。
- 渲染落在离屏「屏幕纹理」，present 时以反向源矩形 blit 到默认帧缓冲（不可直画：自 blit 重叠=未定义）。

### 8. 错误与可观测

- 错误一律短码 + `log::error!`，不 `panic!`；断言仅用于「绝不该发生」的编程合同。
- 诊断探针：`frame churn configure/tex_new/tex_del`（逐帧资源重建）、`rack 重挂`（附件差分原因）、`after_flush_*` / `f_readback`（状态组别）。

## 🕳️ 经验与踩坑（症状 → 根因 → 处理）

| 症状 | 根因 | 处理 |
|---|---|---|
| 每帧刷 `INVALID_ENUM`，帧率掉到个位数 | 用 `getParameter` 查能力开关（DEPTH_TEST/CULL_FACE/SCISSOR_TEST/BLEND/STENCIL_TEST） | 改 `isEnabled` |
| `Polygon mode other than FILL` panic | `polygon_to_gl` 常量错位（Fill 被映射成 LINE） | 修正映射；WebGL 侧不下发 polygonMode |
| 清屏 `INVALID_VALUE` | `gl.scissor(0,0,i32::MAX,i32::MAX)` 在 Web 走 f32 口舍入后越界 | 删除该调用 |
| `location is not from associated program` | 程序未绑时 `uniform1i` 定位 | 采样器指位随程序切换下发 |
| `useProgram` / `bindTexture` 每秒双发 | 直发路未回写水位 → 下轮 flush 判失配 | 直发后同步水位（已补 10+ 处） |
| 桌面端绑定「永久悬空」 | 死名水位别名（GL 名被复用，水位与期望同值） | 死名一律丢期望 |
| `delete/createFramebuffer` 洪水 | release 计数归零即物理删 | Fbo 域驻留，release 不删 |
| 每 draw 重下十余条 `vertexAttribPointer` | 实例槽偏移逐批变化 | 组合 VAO 一次整备 |
| 压缩纹理上传错位 | 行距按 `width` 估算（BC 误估 4 倍） | 块行判据 + 逐行喂 |
| 画面闪、每个元素互相覆盖 | 上层每元素一 pass，且每 pass `Clear` 全附件 | 延后并按裁剪框清（诊断口） |
| 只渲染一角 | pass 继承上通道小视口 | 每 pass 复位 viewport/scissor/depthRange |
| 上层每帧重配表面、纹理逐帧重建 | `suboptimal` 恒报 true | 按真实语义报 false |

## ⚠️ 已知差异与缺口登记

| 项 | 现状 | 说明 |
|---|---|---|
| WebGL 无 `polygonMode` | Web 恒填充；非 Fill 只 warn 一次 | 语义差异登记 |
| `sample_mask` | 无驱动下发面 | 仅镜像记录 |
| 子区域清屏 `CLEAR_SCISSORED` | 默认开（暂留缺口） | 偏离 wgpu「全附件清除」，勿用于通用语义 |
| 默认帧缓冲通道 | 不复位 viewport/scissor（无尺寸来源） | 上屏走 blit，暂不暴露 |
| 管线 VAO | 仍按布局建身份位 VAO | 可退成不建 GL 对象 |
| 多 draw 合并 | 未做 `WEBGL_multi_draw` | 需 run 判定 + 扩展缺失回退 |

## ✅ 验证三轨

```bash
cargo test --lib                                        # 单元/合验
cargo check --lib --target wasm32-unknown-unknown       # wasm 编译面
powershell -ExecutionPolicy Bypass -File .\web_case\build.ps1   # 浏览器案例包重产
python -m http.server 8731 -d web_case                  # 打开 index.html 实测
```

## 🧾 提交史主题索引（路线 B 实机收敛）

| 提交 | 主题 |
|---|---|
| `591b922` | WebGL2 装配链与 u64 身份层（路线 B 起点） |
| `b5f510a` | WebGL2 浏览器案例包 |
| `2d69dc4` / `a84d8dd` | 管线混合接线 + 混合数值对账样例 |
| `f6d7bd2` | WebGL2 实机差异收口与压缩格式面 |
| `e2c6ded` | 采样器指位随程序切换、块枚举精确界定 |
| `1db82a7` | 不可回读组直发水位补齐、死名精确清退 |
| `3448097` | FBO 形状键行驻留、附件差分选架 |
| `cbe3e7b` | 顶点组合 VAO 一次整备，draw 只绑不重下 |
| `c800828` | 管线定态接线、死名丢期望、组合 VAO 容量调优 |
| `08fd7da` | 热点接口 `#[inline]`（跨 crate / 跨 CGU） |

## 🚀 快速开始

### 依赖集成
在`Cargo.toml`中添加：
```toml
[dependencies]
pi_wgpu = { version = "0.3", features = ["gl"] }  # 默认使用pi_wgpu实现
# pi_wgpu = { version = "0.3", features = ["use_wgpu"] }  # 使用原生wgpu实现
```

> 注意：内建实现启用了 `#![feature(hash_extract_if)]`，当前需 **nightly 工具链** 编译；依赖 `pi_egl`/`pi_glow` 来自私有 registry，开发仓库的 `[patch.crates-io] naga` 指向本地 `../wgpu/naga`，发布或作为外部依赖使用前需自备该 fork 或移除 patch（详见 [API参考](documents/api.md) §1）。

### 运行示例

#### Windows平台
```bash
cargo run --example triangle --features gl
```

#### Web平台
```bash
wasm-pack test --chrome --example triangle
# 访问 http://localhost:8000 查看效果
# 如果想修改访问地址， 将地址设置在`WASM_BINDGEN_TEST_ADDRESS`环境变量中， 如$env:WASM_BINDGEN_TEST_ADDRESS="0.0.0.0:8000", 即可通过ip访问
# https://rustwasm.github.io/docs/wasm-bindgen/contributing/testing.html
```

#### Android平台
```bash
cargo apk run --example triangle --lib # 打开Linux Shell,在Linux Shell中执行
adb install target/debug/apk/examples/triangle.apk # 在powershell中执行
```

## 📚 文档资源
- [功能需求](documents/requirements.md) - 库定位、功能需求与验收范围
- [模块设计](documents/module-design.md) - 源码分层、模块职责与关键机制
- [设计笔记](documents/design.md) - 架构设计论证与实现细节
 - [API参考](documents/api.md) - 完整接口文档（含与 wgpu 差异汇总、着色器与坐标约定（产品端必读）§8）
- [导出接口清单](documents/exported-api.md) - 全部 `pub` 导出符号条目（v0.3.7 自动扫描）
- [迁移指南](documents/migration.md) - 从wgpu-rs迁移指南

## 🛠️ 功能支持

### 核心模块支持
| 模块           | 支持状态 | 特性说明                              |
|----------------|----------|---------------------------------------|
| RenderPipeline | ✔️       | 完整渲染管线支持，包含混合/深度测试    |
| Texture        | ✔️       | 支持ASTC/DDS压缩纹理，自动生成Mipmap  |
| Buffer         | ✔️       | 支持动态数据上传，显存智能管理         |
| Shader         | ✔️       | GLSL 3.3标准兼容，支持UBO绑定          |

### 平台特性限制
| 平台       | 特性限制                                                                 |
|------------|--------------------------------------------------------------------------|
| Windows    | 需要OpenGL 3.3+，不支持Geometry Shader                                   |
| Android    | 要求GLES 3.0+，部分纹理格式需要扩展支持                                  |
| Web        | 依赖WebGL2规范，iOS需≥15.5版本                                           |

## 👥 贡献指南
欢迎通过Issue或PR参与贡献！请遵循：
1. 提交前运行`cargo fmt`格式化代码
2. 新增功能需包含测试用例
3. 更新相关文档

## 📜 许可证
本项目采用 [MIT License](LICENSE) - 详情请查看[许可证文件](LICENSE)。

## 💬 联系方式
- 问题追踪: [GitHub Issues](https://github.com/yourorg/pi_wgpu/issues)
- 开发者文档: [API参考手册](documents/api.md)
- 技术博客: [实现原理分析](documents/design.md)
- 社区讨论: [Discord技术频道](#)

+ `性能优化`：设置状态机，做GL的全状态比较；所以GL指令数量会比[wgpu-rs](https://github.com/gfx-rs/wgpu)少；

**注1**：为什么不直接用 [wgpu-rs](https://github.com/gfx-rs/wgpu)
；
+ `Vulkan` / `WebGPU` 因兼容性问题，近期内不能广泛使用于各个平台；

**注2**：根据微信小游戏的开发者文档，在 iPhone / iPad 上，`WebGL2` 至少要 `iOS`版本 >= 15.5 才能开启

**注3**：详细的 设计笔记，[见这里](documents/design.md)

## 2.01. 限制

+ 销毁资源：可以在录制指令时 创建 / 销毁 资源
+ 线程安全：
    - 录制指令 是 `单线程`，录制即为提交，所谓的队列提交是空实现；
    - 创建，销毁 资源，Exe / Apk `多线程`；
    - `注`：使用者自己确保，创建/释放 和 录制指令 的 线程安全性；
+ GLSL:
    - 功能: 仅支持 [gles-300 / std140-布局](https://www.khronos.org/files/webgl20-reference-guide.pdf)；
    - 语法: 仅支持 [GLSL 450 语法](https://www.khronos.org/files/webgl20-reference-guide.pdf)；

## 2.02. 销毁资源

+ 必须等该资源的物体`draw`之后，才能销毁；
+ `RenderPass` 绑定的 纹理 资源，必须等到 `RenderPass` Drop 方法调用后，才能销毁；
+ 原因：因为 不知道调用顺序，vb / ib / ubo 的调用都是先缓存，等到 draw() 内部 才 统一调用的 gl 函数；

例子：

```rust
pass.set_vertex_buffer(vb1)
// 不能 调用 vb1 的 drop
pass.set_bind_group(bg1)
// 不能 调用 bg2 的 drop
pass.set_render_pipeline(rp1)
// 不能 调用 rp1 的 drop
pass.draw()
// 这里之后，可以扔掉 vb1, bg1, rp1
pass.set_vertex_buffer(vb2)
```

## 2.03. **不** 支持

| 函数            | 支持 | 说明            |
| --------------- | ---- | --------------- |
| QuerySet        | ×    | WebGL2 本身支持 |
| hal::Fence      | ×    | WebGL2 本身支持 |
| RenderBundle    | ×    |                 |
| ComputePipeline | ×    |                 |
| ComputePass     | ×    |                 |
| hal::Barrier    | ×    |                 |

## 2.04. wgpu::util

| 函数                       | 支持 | 说明                        |
| -------------------------- | ---- | --------------------------- |
| `backend_bits_from_env`    | ✔    | 永远返回 Some(Backends::GL) |
| `create_texture_with_data` | ✔    |                             |
| `create_buffer_init`       | ✔    |                             |

## 2.05. `Instance`

| 函数               | 支持 | 说明 |
| ------------------ | ---- | ---- |
| `new`              | ✔    |      |
| `request_adapter`  | ✔    |      |
| `create_surface`   | ✔    |      |
| enumerate_adapters | ×    |      |

## 2.06. `Adapter`

| 函数                          | 支持 | 说明 |
| ----------------------------- | ---- | ---- |
| `request_device`              | ✔    |      |
| `features`                    | ✔    |      |
| `limits`                      | ✔    |      |
| `get_info`                    | ✔    |      |
| `get_downlevel_capabilities`  | ✔    |      |
| `is_surface_supported`        | ✔    |      |
| `get_texture_format_features` | ✔    |      |

## 2.07. `Surface`

| 函数                  | 支持 | 说明                                                        |
| --------------------- | ---- | ----------------------------------------------------------- |
| `configure`           | ✔    | `present_mode`: 必须是`Fifo`; `alpha_mode`: 必须是 `Opaque` |
| `get_current_texture` | ✔    |                                                             |
| get_default_config    | ×    |                                                             |
| get_capabilities      | ×    |                                                             |

## 2.08. `SurfaceTexture`

| 函数      | 支持 | 说明 |
| --------- | ---- | ---- |
| `present` | ✔    |      |

## 2.09. `Device`

**注：** 所有方法，都要等 Surface.configure(Device, &config) 调用之后，才能使用！

| 函数                           | 支持 | 说明                                                |
| ------------------------------ | ---- | --------------------------------------------------- |
| `features`                     | ✔    |                                                     |
| `limits`                       | ✔    |                                                     |
| `create_shader_module`         | ✔    |                                                     |
| `create_command_encoder`       | ✔    |                                                     |
| `create_bind_group_layout`     | ✔    |                                                     |
| `create_bind_group`            | ✔    |                                                     |
| `create_pipeline_layout`       | ✔    |                                                     |
| `create_render_pipeline`       | ✔    | 参数 layout: Option<&'a PipelineLayout> 必须 有值！ |
| `create_buffer`                | ✔    |                                                     |
| `create_texture`               | ✔    |                                                     |
| `create_sampler`               | ✔    |                                                     |
| create_shader_module_unchecked | ×    |                                                     |
| create_shader_module_spirv     | ×    |                                                     |
| create_render_bundle_encoder   | ×    |                                                     |
| create_compute_pipeline        | ×    |                                                     |
| create_texture_from_hal        | ×    |                                                     |
| create_query_set               | ×    |                                                     |
| poll                           | ×    |                                                     |
| on_uncaptured_error            | ×    |                                                     |
| push_error_scope               | ×    |                                                     |
| pop_error_scope                | ×    |                                                     |
| start_capture                  | ×    |                                                     |
| stop_capture                   | ×    |                                                     |
| as_hal                         | ×    |                                                     |

## 2.10. `Queue`

**注：** 所有方法，都要等和该Queue对应的Device 调用 Surface.configure(Device, &config) 之后，才能使用！

| 函数                           | 支持 | 说明   |
| ------------------------------ | ---- | ------ |
| `write_buffer`                 | ✔    |        |
| `write_texture`                | ✔    |        |
| `submit`                       | ✔    | 空实现 |
| on_submitted_work_done         | ×    |        |
| write_buffer_with              | ×    |        |
| copy_external_image_to_texture | ×    |        |
| get_timestamp_period           | ×    |        |

## 2.11. `CommandEncoder`

| 函数                    | 支持 | 说明               |
| ----------------------- | ---- | ------------------ |
| `finish`                | ✔    | 空实现             |
| `begin_render_pass`     | ✔    | 只支持一个渲染目标 |
| clear_texture           | ×    |                    |
| clear_buffer            | ×    |                    |
| begin_compute_pass      | ×    |                    |
| copy_buffer_to_buffer   | ×    |                    |
| copy_buffer_to_texture  | ×    |                    |
| copy_texture_to_buffer  | ×    |                    |
| copy_texture_to_texture | ×    |                    |
| insert_debug_marker     | ×    |                    |
| push_debug_group        | ×    |                    |
| pop_debug_group         | ×    |                    |
| write_timestamp         | ×    |                    |
| resolve_query_set       | ×    |                    |

## 2.12. `CommandBuffer` 空内容，空实现

## 2.13. `RenderPass`

| 函数                              | 支持 | 说明                                                   |
| --------------------------------- | ---- | ------------------------------------------------------ |
| `set_bind_group`                  | ✔    |                                                        |
| `set_pipeline`                    | ✔    |                                                        |
| `set_blend_constant`              | ✔    |                                                        |
| `set_index_buffer`                | ✔    |                                                        |
| `set_vertex_buffer`               | ✔    |                                                        |
| `set_scissor_rect`                | ✔    |                                                        |
| `set_viewport`                    | ✔    |                                                        |
| `set_stencil_reference`           | ✔    |                                                        |
| `draw`                            | ✔    | 参数 `first_instance` 必须为 0; `base_vertex` 必须为 0 |
| `draw_indexed`                    | ✔    | 参数 `first_instance` 必须为 0; `base_vertex` 必须为 0 |
| set_push_constants                | ×    |                                                        |
| insert_debug_marker               | ×    |                                                        |
| push_debug_group                  | ×    |                                                        |
| pop_debug_group                   | ×    |                                                        |
| draw_indirect                     | ×    |                                                        |
| draw_indexed_indirect             | ×    |                                                        |
| execute_bundles                   | ×    |                                                        |
| multi_draw_indirect               | ×    |                                                        |
| multi_draw_indexed_indirect       | ×    |                                                        |
| multi_draw_indirect_count         | ×    |                                                        |
| multi_draw_indexed_indirect_count | ×    |                                                        |
| write_timestamp                   | ×    |                                                        |
| begin_pipeline_statistics_query   | ×    |                                                        |
| end_pipeline_statistics_query     | ×    |                                                        |

## 2.14. `Sampler`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

## 2.15. `Texture`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

| 函数                    | 支持 | 说明 |
| ----------------------- | ---- | ---- |
| `create_view`           | ✔    |      |
| `as_image_copy`         | ✔    |      |
| `size`                  | ✔    |      |
| `width`                 | ✔    |      |
| `height`                | ✔    |      |
| `depth_or_array_layers` | ✔    |      |
| `mip_level_count`       | ✔    |      |
| `sample_count`          | ✔    |      |
| `dimension`             | ✔    |      |
| `format`                | ✔    |      |
| `usage`                 | ✔    |      |
| as_hal                  | ×    |      |
| destroy                 | ×    |      |

## 2.16. `Buffer`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

| 函数                       | 支持 | 说明 |
| -------------------------- | ---- | ---- |
| `size`                     | ✔    |      |
| `usage`                    | ✔    |      |
| `as_entire_binding`        | ✔    |      |
| `as_entire_buffer_binding` | ✔    |      |
| `slice`                    | ✔    |      |
| unmap                      | ×    |      |
| destroy                    | ×    |      |

## 2.17. `BufferSlice` 空实现

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |      |
| get_mapped_range     | ×    |      |
| get_mapped_range_mut | ×    |      |

## 2.18. `BindGroup`

`BindGroup` 会握住它使用的`Buffer`, `Texture`, `Sampler` 对象，使其不会被销毁。

如果想要对应的资源被销毁，必须同时扔掉`BindGroup`和对应的资源。

## 2.19. `ShaderModule`

+ 仅支持 Naga 编译过后，版本为 glsl 3.0 的 无 define 宏 的 glsl
+ 扩展：带上一个字段，用于说明 这个shader原始的 set-binding 和 uniform 的 名字

# 3. 附录

## 3.1. 和 `WebGL2` 相比，缺失 的 功能

+ 代码 模拟
    - `UBO`：用 uniform 模拟
    - `Sampler`：用 texture 函数 模拟
    - `VS` / `FS`: 不能指定 layout
        - **难点，需要 `可行性测试`**：要找一套 WebGL Shader 的 编译转换工具，`naga`未必支持；
            * 将 UBO 转成 Uniform；
            * 去掉 所有的 layout，同时导出成 json文件，供 rust 设置；
        - rust层，调用 WebGL 的 getXXXLocation 取 Uniform / Attribute 的 location，建立 layout 的 hash-map；
+ 看 **扩展** 有没有
    - 很多纹理格式: 比如 float 纹理，深度 纹理
    - `VAO`
    - `几何实例化`
    - 多目标渲染 / drawBuffers clearBuffer
    - 多重采样
    - FeedBack

## 3.2. [微信小游戏: `iOS` 的 `WebGL`](https://developers.weixin.qq.com/minigame/dev/guide/performance/perf-high-performance.html#%E7%AE%80%E4%BB%8B)

+ `WebGL`: 如果发现 iOS 14 的 帧率提升不上去，试试 在 canvas.getContext 接口 将 antialias: true
+ `WebGL`: iOS 版本 14.0 ~ iOS 15.3, 多个 drawcall 使用不同偏移 来 共享 `VB`/`IB`，性能非常糟糕 ！

## 3.3. 初始化流程

Instance --> Adatper --> (Device, QUeue)
         
         --(Resumed Event)--> Surface

所有的 Device / Queue 的函数，都要等：Surface.configure(Device, &config) 之后调用。

原因：因为 Web平台 没有 跨 Surface 共享的 Context 

可以将 所有的 渲染初始化，放到一个 furture，哪个furture 留到 窗口的 Resumed 事件判断并初始化。

然后，configure 还是要 等 Resized 调用（web没有Resize事件，就再 request_draw 中模拟）
