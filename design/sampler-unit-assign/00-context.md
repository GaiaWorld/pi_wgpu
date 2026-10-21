# sampler-unit-assign · context（L0）

> 事实层：不随 L1/L5 返工作废——L1 写需求、L7 落详细设计都引用本册。

## 地形

- 语言/工程：Rust，crate `pi_wgpu`（`E:\Rust\PI_PI\pi_wgpu_ai`），wgpu 25.0.2 语义对齐中的 GL 后端替身。
- 命名惯例：`src/<模块>/mod.rs` = 冻壳（契约注释住壳）；`src/<模块>/<模块>_inner.rs` = 芯（出处 `src/shader/mod.rs:79-95`、`src/entry/entry_inner.rs:1099`）。
- 错误纪律（出处 `AGENTS.md`）：优先上递短码错误；不便上递处 `log::error!(短码)` 后降级继续，禁原地 panic；`assert/panic` 只留给绝不该发生的合同违反。
- 每类型/字段/常量/静态量/接口必须有注释（出处 `AGENTS.md`）。

## 被触达模块的现有形状

| 位置 | 现有职责 |
|---|---|
| `src/shader/shader_inner.rs` | 转译出 ES300 GLSL（`compile_impl`）、反射登记（`reflection_register`）、从 IR 取组合采样配对（`sampler_pairs_impl:267`） |
| `src/shader/mod.rs:20-31` | 反射条目 `(group,binding,kind,name)`，kind 含 Buffer/Texture/Sampler |
| `src/entry/entry_inner.rs:1058-1063` | 程序复用键 = 顶点+片元转译哈希合成；仅 `refcount==1` 时 `link_program` 一次 |
| `src/entry/entry_inner.rs:1099` | `link_program(name, vs, fs)`：attach + link |
| `src/binds/mod.rs` / `binds_inner.rs` | `SlotTable`（含 P2 的 `alias` 配对表）、`bind_base(set,slot)` 算 GL 单元、`apply_impl` 下发纹理/采样器 |
| `src/state/mod.rs` / `state_inner.rs` | `StateMirror`（`program` 组 + `water` 水位）；`flush_impl` 在差异时下发程序与各槽绑定 |
| `src/record.rs` / `src/draw/draw_inner.rs` | `RetainedPipeline` 持 `program/vao/slots` 部件；`exec_set_pipeline_parts` 设管线 |

## 测试位置与框架

- 库内单测：`cargo test --lib`（当前 31 passed / 1 ignored）。
- 真 GL 全链：`#[ignore]` 测试，`cargo test --lib -- --ignored --test-threads=1`。
- 示例：`examples/`（`shadow_cube` 等，`cargo build --examples`）。
- 产品验收体：`E:\Rust\PI_PI\pi_3d_n\examples\instance\instance_color_tilloff.rs`。

## 素材清单（全部已读）

- `E:\Rust\PI_PI\wgpu\naga\src\back\glsl\mod.rs:206` —— `supports_explicit_locations() = >= Desktop(420) || >= gles(310)`；`:917` 据此决定是否写 `binding=`；`:1444/1454/1476` 资源全局命名 `_group_X_binding_Y_<名>`。
- `E:\Rust\PI_PI\pi_glow\src\lib.rs:174/176/538/727` —— `get_active_uniforms` / `get_active_uniform` / `get_uniform_location` / `uniform_1_i32`。
- `E:\Rust\PI_PI\pi_glow\src\gl46.rs` —— GL 枚举权威表（`GL_TEXTURE_MIN_FILTER=0x2801`、`GL_TEXTURE_MAG_FILTER=0x2800`）。
- 实测取证（本轮排查，产品带 `PI_GL_CHECK`）：合成通道镜像 `tex=[6@13] smp=[2@13]`，下发 `issue tex u=13 t=6` 与 `issue smp u=13 s=2` 均无 GL 错误；该绘制后回读全 0，而场景通道绘制后回读有内容（局部 25/38/44/30）。全库 `uniform_1_i32` 调用点为 0。

## 来源

- 对话 = 用户 NVIDIA 观察「实例内容渲染发生在 Copy 到交换 FBO 之后」；用户指定查 `ScreenWithPostprocess` 与 `RendererRenderTarget::FinalRender`（`E:\Rust\PI_PI\pi_3d_n\crates\pi_scene_context\src\renderers\graphic\graphic.rs:148,310`）。
- 文件 = 上列 path:行。
- 组织既有方案 = `design/desc-align-wgpu25/`（已完成；其 `07-design/record-buffer.md` 是命令缓冲基线，本设计不改其语义）。

## 待验证

| 来源 | 结论 | 状态与证据 | 何时关闭 |
|---|---|---|---|
| naga GLSL 后端 | 合并后的采样入口全局名取**纹理**侧 (group,binding) | **静态核证成立**：`back/glsl/mod.rs:206` 对 gles(300)=false → `:917` 不写 `binding=`；`:964` Sampler 全局 `continue` 不出码；`:1454/1479` 资源全局名 = `_group_{g}_binding_{b}_{stage}` | 动态实测随切片 3 真 GL 测试关闭 |
| 同上（stage 后缀） | 同 (g,b) 在 VS/FS 各出一条 uniform、两个 location | **静态核证成立**：名第三段为 `entry_point.stage.to_str()` | 已入 api/call-chain 设计（按名字入表，禁按 (g,b) 去重） |
| pi_glow | `get_active_uniform` 的 `utype` 枚举集 | **作废**——collect 不消费 `utype`，改按「名字前缀解析 + targets 白名单」过滤，枚举集不再是判据 | 已关闭（见 changes.md 回改行） |
