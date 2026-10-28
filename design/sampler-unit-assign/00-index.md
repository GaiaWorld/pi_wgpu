# sampler-unit-assign 设计索引（design-lite）

**恢复入口**：`index/status.md` 的 `当前阶段` 格。

## 登记分册（index/）
| 分册 | 内容 |
|---|---|
| [index/status.md](index/status.md) | 状态表（模式/当前阶段/最后更新） |
| [index/glossary.md](index/glossary.md) | 词汇表（非 ID 表：术语/定义/出处） |
| [index/changes.md](index/changes.md) | 变更日志 |

## 内容分册（design/ 根）
| 分册 | 内容 | 阶段 |
|---|---|---|
| [00-context.md](00-context.md) | context 节（地形/素材/来源） | L0 |
| [01-requirements/problem.md](01-requirements/problem.md) | 问题与范围 | L1 |
| [01-requirements/needs.md](01-requirements/needs.md) | 需求条目 | L1 |
| [01-requirements/flow.md](01-requirements/flow.md) | 业务流程图 | L1 |
| [01-requirements/notes.md](01-requirements/notes.md) | 非功能 / 未决 / 澄清 | L1 |
| [07-design/api.md](07-design/api.md) | 接口面 | L7 |
| [07-design/data.md](07-design/data.md) | 数据结构 | L7 |
| [07-design/units.md](07-design/units.md) | 类与子程序 | L7 |
| [07-design/call-chain.md](07-design/call-chain.md) | 预期调用链 + 实现切片 | L7 |
| [07-design/errors.md](07-design/errors.md) | 防御与错误 | L7 |
| [09-review.md](09-review.md) | L9 轻审报告 | L9 |

> 本次跳过 L5（不引新依赖/新语言、不新增模块）；L7 走全册档（用户拍板）。
