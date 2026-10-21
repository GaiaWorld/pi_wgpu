# sampler-unit-assign · 类与子程序（L7）

> 一致抽象：每层只做一件事，命名沿用 L0 惯例（`xxx` 壳 / `xxx_inner` 芯 / `exec_` 前缀=执行面）。

| 单元 | 壳/芯路径 | 职责（只做这一件） | 不做什么 |
|---|---|---|---|
| `sampler_targets` / `sampler_targets_impl` | `src/shader/mod.rs` / `src/shader/shader_inner.rs` | 读反射，答「哪些绑定会长成采样入口」 | 不查单元、不触 GL |
| `unit_of` | `src/binds/mod.rs` / `src/binds/binds_inner.rs` | 读槽表，答「这个绑定占哪个单元」 | 不碰 GL、不改表 |
| `collect_sampler_uniforms` | `src/entry/entry_inner.rs` | 读程序，答「这个采样入口在驱动侧叫什么位置」并合成对表 | 不做时机决策（何时指位不归它） |
| `exec_set_pipeline_parts` | `src/draw/draw_inner.rs` | 把管线的指位表交给镜像（期望态登记） | 不下发 GL |
| `flush_impl` 程序分支 | `src/state/state_inner.rs` | 在程序真换时，按期望态发指位 | 不查名字、不算单元（只消费已定格的表） |

## 无隐藏副作用声明

- `sampler_targets` / `unit_of`：纯函数。
- `collect_sampler_uniforms`：触 GL 查询（只读），不改驱动态；唯一副作用是 `log::error!` 短码。
- 设管线：只改镜像。
- flush 程序分支：只发 `glUniform1i`（写程序 uniform 态），这是其职责本身，不算隐藏副作用。
