# sampler-unit-assign · 防御与错误（L7）

## 边界设防

| 边界 | 设防 | 落点 |
|---|---|---|
| 名字不是 `_group_G_binding_B_` 形态 | 静默跳过（绝大多数 uniform 属此类，不是错） | `collect_sampler_uniforms` |
| 解析出 (G,B) 但槽表里没有 | `log::error!` 短码 `SamplerUnitMissing` + 跳过该条，其余继续 | `collect_sampler_uniforms` |
| `get_uniform_location` 返回 -1（被驱动优化掉） | 跳过，不算错 | `collect_sampler_uniforms` |
| `get_active_uniforms` 返回 0 | 空表，正常态 | `collect_sampler_uniforms` |
| 程序句柄已死 | 走既有 `DeadHandle` 短码面，不新增错误变体 | `entry` 既有面 |
| targets 为空（无采样入口） | 整条链短路，不查 GL | `collect_sampler_uniforms` |
| 单元越界（`unit >= BIND_SLOTS`） | 绝不该发生=保留断言（编程合同违反），消息用短码 | `binds` |

## 隔离带

- 新逻辑全部住 `_inner` 与既有分支内；不改 `mod.rs` 公开壳签名，对外 API 面无变化。
- 指位只在程序水位差异触发；不新增绘制期 GL 调用种类（`glUniform1i` 是本库首次使用，但属程序态写入，不影响绑定/帧生命周期）。
- 不改状态镜像既有字段语义；新增字段独立成组，缺省空表=行为与改动前一致（对不采样管线零影响）。

## 错误策略（≤2，不吞错）

1. **跳过并记短码**：单条指位不可得 → `log::error!` 后继续，绝不 panic、绝不静默。
2. **不新增错误变体**：对外仍以既有 `ShaderErr` / `DeadHandle` 面表达；本设计不引入新错误类型。

## 失败后全局状态

- 指位动作本身**无失败分支**：入表前提已是「活程序句柄 + `location >= 0`」，`glUniform1i` 在此前提下不产生可检错误；故水位照常推进，**不设重试语义**（原稿「下轮程序切换会重试」承诺已删——它与伪码无条件推 `water.program` 相互矛盾）。
- 若驱动仍报错，由既有 `check_err` 面（`PI_GL_CHECK` 门控）观察，不新增分支、不新增错误变体。
- 唯一 panic 面是单元越界断言（编程合同违反，按 `AGENTS.md` 允许，消息用短码）。
