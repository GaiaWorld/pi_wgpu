# L9 轻审报告（design-lite · L7 增量：examples 冒烟骨架 B16）

**结论**：☑ 放行（子代理原判=有条件放行，4 ❌ 全部同场收口）

> 子代理=无对话历史、只读、R1–R16。方案 B（用户拍板可见窗口）；winit 0.30.13（rwh_06）。

| 检查 | 结果 |
|---|---|
| R1–R4,R6–R8,R10–R15 | ✅ |
| R5 册↔src/Cargo/examples | ❌→修 |
| R9 边界/新依赖留痕 | ❌→修 |
| R16 实现切片可照做 | ❌→修 |

## ❌ 明细与收口

| 条目 | 位置 | 问题 | 收口 |
|---|---|---|---|
| R5 | api.md:24 | view 芯列指 surface_inner（实为壳内联）；契约格串入 present 半句 | 芯列改「—（壳内联）」；删串位半句 |
| R9 | deps.md | dev-dep winit/pollster 未入 L5 册 | 外部边表补「examples(dev) → winit 0.30 / pollster 0.3」 |
| R16 | call-chain B16 | 对接链含链4/链5（例子未触达：无绑定组、顶点走便捷面） | 改「链1/链2/链3（+链11 便捷面）」 |

## 附注收口

- examples 改用 `create_surface(window)`（走新 From 实现），使 @public From 有真实消费者（免 R13 空预留）。
- examples 注释去 design 内号（链号→文字）。
- status.md 当前阶段被 tableStage 清空——tableDone 后恢复填回。
- glossary 未补 view/From/WindowHandle/SurfaceTarget 行（可选，与 B15 补 TextureView 同理）；From 无第二消费者时保持。

## 复验

- cargo check --features gl --examples = Finished；clear 运行 12s 稳定无崩。

## 说明

- 例子为可运行参照；`'static` 泄漏（instance/window）为样例简化（规避 Instance↔Surface 自引用），非库面。