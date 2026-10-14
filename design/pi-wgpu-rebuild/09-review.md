# L9 轻审报告（design-lite · L7 返工轮：B7 shader 表示层同步 + 错误处理规则）

**结论**：☑ 放行（子代理原判=有条件放行，5 ❌ 全部同场收口）

> 子代理=无对话历史、只读、按 R1–R16 判。返工对象=B7 接口增量（CompiledEs300/ShaderIr/ReflectionTable/emit_stage/ShaderModule._compiled）+ 新增决定⑪错误处理规则。

## 内容核对（子代理原判）

| 检查 | 结果 |
|---|---|
| R1 需求够清楚 | ✅ |
| R2 问题与范围 | ✅ |
| R3 需求↔模块对得上 | ✅ |
| R4 模块切得对 | ✅ |
| R5 图与文一致 | ❌→修 |
| R6 依赖无环、无越级 | ✅ |
| R7 变更点三项齐 | ✅ |
| R8 名词有定义 | ❌→修 |
| R9 边界不越 | ✅ |
| R10 能照此开工 | ✅ |
| R11 决定有理由 | ✅ |
| R12 假设已验证 | ✅ |
| R13 无过度工程 | ✅ |
| R14 详细设计能落地 | ❌→修 |
| R15 详细设计不越界 | ✅ |
| R16 实现切片可照做 | ✅ |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R14 | src/entry/mod.rs + entry_inner.rs | create_shader_module 壳文/芯伪码仍「codespan→断言」，与决定⑪/api 冲突；公开非 Result 面 Err 落点未定义 | entry 壳/芯 | 改：源错=log::error!(短码)+降级无有效产物、真错延后由管线创建以码暴露；芯伪码同步 |
| R14 | call-chain.md:7,14 | 「源错=断言」未随决定⑪改；:14 节点 ensure_program 不存在于壳 | call-chain.md | :7 加 emit_stage、尾格改短码上递；:14 节点改 compile_to_es300→reflection_register→objects.ensure_reuse |
| R14 | src/shader/mod.rs | reflection_register 壳注释缺失败后状态 | shader 壳 | 补「失败后：反射表保持调用前原状（不半写）」 |
| R5 | errors.md:15,30,33 | 旧表「codespan→断言」与新增基线自相矛盾 | errors.md | :15/:30/:33 三处改短码+log；补「公开非 Result 面」边界与三 Result 面口径 |
| R8 | index/glossary.md | B7 新术语无词条；「壳芯」定义仍「panic 桩」过时 | glossary.md | tableSetCell 改「壳芯」；tableAddRow 补 6 行（CompiledEs300/ShaderIr/ReflectionTable/RefEntry/BindingKind/ShaderErr/错误码上递） |

## 附注收口

- data.md 重复的旧「反射登记 ReflectionTable」行已删（保留 B7 新行）。
- call-chain B13 前依赖「全部」→ 显式「B0–B12」（序号即实现序）。
- status.md 最后更新 → 2026-10-14。

## 复验

- cargo check（--features gl）= EXIT 0；cargo test --lib -- --include-ignored --test-threads=1 = 18 passed / 0 failed。

## 说明（不计 ❌）

- B1–B6 芯含逻辑体属实现期正常演进；错误处理按决定⑪「B7 起从严、B1–B6 待清理」登记。
