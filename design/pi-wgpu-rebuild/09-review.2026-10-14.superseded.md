# L9 轻审报告（design-lite · 图示补细节轮）

**结论**：☑ 放行（子代理原判=有条件放行，3 ❌ 全部同场收口）

> 子代理=无对话历史、只读。返工对象=call-chain 链1/链2 时序图补函数级步骤。

## 内容核对（子代理原判）

| 检查 | 结果 |
|---|---|
| R5 图与文一致 | ❌→修 |
| R14 详细设计能落地 | ✅ |
| R16 实现切片可照做 | ✅ |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R5 | call-chain 链1:46 | 图比源表多且冲突：链1 源行止于 configure，create_render_pipeline 属链3/链10 | call-chain | 链1 标题注「含链3/链10 挂点」；拆为 create_shader_module（compile_to_es300→reflection_register）+ create_render_pipeline（emit_stage→ensure_reuse） |
| R5 | 同上括注 | 漏 compile_to_es300、混入 reflection_register（属模块创建） | call-chain | 两步行分工对齐源行 |
| R5 | call-chain 链2:73 | surface_swap 画作 surface 自步，与链1（surface_attach 画作 S->>C）归属不一 | call-chain | 链2 加 participant C；改 S->>C: surface_swap |

## 附注收口

- 链2 标题「含链4/6/7 挂点」→「含链5/6/8 挂点」。
- entry/mod.rs「ensure_program」→「shader.emit_stage / objects.ensure_reuse(ReuseKey::Program)」（实名）。
- 壳注释去 design 内号：src 全量「决定⑪」→「错误纪律」（9 文件；规则正文自足，不构成契约缺口）。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed / 0 failed；三组合 Finished。

## 说明

- deps 主依赖图 42 边与表逐一相等、节点 13（子代理核）；flow.md 与 needs 无冲突；无悬空指针。