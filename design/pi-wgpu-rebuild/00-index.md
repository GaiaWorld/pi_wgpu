# pi-wgpu-rebuild 设计索引（design-lite）

<!-- 模式一栏永远 = lite。本册是【指针册】：只列分册指针、不承载表格内容。恢复入口=读本册 → index/status.md 拿当前阶段。 -->

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
| [05-modules/modules.md](05-modules/modules.md) | 模块清单 | L5 |
| [05-modules/deps.md](05-modules/deps.md) | 依赖边 + 模块依赖图 | L5 |
| [05-modules/change-points.md](05-modules/change-points.md) | 变更点 | L5 |
| [05-modules/decisions.md](05-modules/decisions.md) | 决定与来源（理由 / 被否方案 / 可行性 / 买还是造） | L5 |
| [07-design/api.md](07-design/api.md) | 接口面 | L7 |
| [07-design/data.md](07-design/data.md) | 数据结构 | L7 |
| [07-design/units.md](07-design/units.md) | 类与子程序 | L7 |
| [07-design/call-chain.md](07-design/call-chain.md) | 预期调用链 | L7 |
| [07-design/errors.md](07-design/errors.md) | 防御与错误 | L7 |
| [09-review.md](09-review.md) | L9 轻审报告 | L9 |

<!-- L5 跳过则 05-modules/ 不存在，L7 跳过则 07-design/ 不存在；每册只承载一个内容，不把多节塞一册。
     L7 若落壳芯：文件写项目 src/（按 L0 惯例目录），壳路径在 07-design/ 各册里给指针；契约住壳注释。 -->
