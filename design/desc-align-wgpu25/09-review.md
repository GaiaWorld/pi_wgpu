# L9 轻审报告（design-lite）

> 审校=无对话历史子代理（只读，按内容清单判定）；审后主上下文完成回改并复验。

**结论**：放行（原「有条件放行」的全部条件已处置复验，运行回归一项除外→交用户）

## 内容核对（子代理原判 + 处置态）

| 检查 | 结果 |
|---|---|
| R1 ✅ | 验收均可写成测试/实现、无空话 |
| R2 ✅ | 问题册无方案名词，「不做」清单在场 |
| R3 不适用 | 未启用 L5 |
| R4 不适用 | 未启用 L5 |
| R5 ✅（回改后） | 原 ❌：flow 判序与芯断言序不符——已按芯序（layout→entry_point→编译选项→多视角→缓存）重排图与源表 |
| R6 不适用 | 未启用 L5 |
| R7 不适用 | 未启用 L5；call-chain 触及文件清单够审计粒度（子代理认可） |
| R8 ✅（回改后） | 原 ❌：Err* 短码族未入词汇表——已追行 |
| R9 ✅ | 只触达描述符域，全景对账挂后续批 |
| R10 ✅ | 壳注释+notes+遗留清单可开工；坑位已被遗留清单显式覆盖 |
| R11 ✅ | 拍板四条均带来源/被否方向 |
| R12 ✅（回改后） | 原 ❌：#version 头自承载无测试证据——补 GLSL 单源 compile 过+语法错 Err 两用例实证（缺头=naga 默认回退，已注记） |
| R13 ✅ | 新公开符号均指得出消费者或对账理由 |
| R14 ✅ | 契约要素齐住壳注释、桩不吞错不调未实现物、指针单向 |
| R15 ✅ | 无越界逻辑体（复用键合成=注册在案缺陷修复）；naga 不泄公开面；可见性同式 |
| R16 ✅（回改后） | 原 ❌：块级独立验收不成立——已声明验收粒度=批尾+每块判据注记 |

## ❌ 明细（审后处置）

| 条目 | 问题 | 回落点 | 处置 |
|---|---|---|---|
| R5 | flow 判序与芯断言序不符、缺 ErrEntryPoint 节点 | 01-requirements/flow.md | ✅ 图+源表按芯序重排 |
| R8 | Err* 短码族散在壳/芯无词汇条目 | index/glossary.md（ddt） | ✅ 已补行 |
| R12 | 版本头自承载无测试证据 | src/shader/shader_inner.rs tests | ✅ 补 2 用例，21 passed |
| R16 | 切片块级无独立验收 | 07-design/call-chain.md | ✅ 粒度声明+块判据注记 |
| A1 | VertexBufferLayout derive 差 | 07-design/call-chain.md 有意差异节 | ✅ 登记（传染链归后续批） |
| A2#3/A3-1 | request_device 漏 max_push_constant_size 对照 | src/entry/entry_inner.rs | ✅ 已补入比对链，重测全绿 |
| A3-2 | ErrPush 断言无定位参数 | src/draw/draw_inner.rs | ✅ 消息带 offset/len；无标签源登记有意差异 |
| A2#6 | 带窗运行回归无记录 | 结论确认门禁 | ⏸ 交用户执行 |

## 复验记录

- 回改后 cargo test --lib = 21 passed / 0 failed / 1 ignored；cargo check --all-targets = Finished 零错误；
  两例既存 drop-ref warning 位于未触达行、非本设计新增；带窗运行回归（hello_triangle 可见+稳定 / textured_quad 渐变方块）待用户执行。
- 审中事故与修复：PS5.1 Get/Set-Content 写坏两例编码 → git checkout 还原 + edit 重放 S4；编码由 cargo UTF-8 校验背书。