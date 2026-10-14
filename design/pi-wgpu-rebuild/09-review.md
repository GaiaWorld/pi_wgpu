# L9 轻审报告（design-lite · L7 增量：VAO 生命周期改管线级复用）

**结论**：☑ 放行（子代理原判=有条件放行，2 ❌ + 附注同场收口）

| 检查 | 结果 |
|---|---|
| R1–R13,R16 | ✅ |
| R14 详细设计能落地 | ❌→修 |
| R15 不越界·壳合规 | ❌→修 |

## ❌ 收口

| 条目 | 问题 | 收口 |
|---|---|---|
| R14 | 通道只记名不持强引用，管线 Drop 真删 program/VAO → 存续合同缺口 | RenderPipeline 补生命周期合同（管线须存活至引用它的 RenderPass 结束）；draw Drop 注释限定「强引用=经绑定组持有的资源，program/VAO 由管线存活期覆盖」 |
| R15 | ①Drop 注释「会话级」≠「管线级」；②`_vao` 缺字段注释 | 改「管线级」；RenderPipeline 改多行 + 逐字段注释 |

## 附注收口

- data.md RenderPass/复用对象行补「通道 `_vao` 只记名不持有」。
- 附注5（重要）：`invalidate_generations` 清计数后仍存活的管线 Drop 会命中零计数断言——已登记为限界（Drop 注释内注明；当前仅测试面/未接线 Lost 路径）。
- 附注3/4（decisions 补 VAO 生命周期行、call-chain B10/B16 补 Drop 验收）留作后续登记（非阻断）。

## 复核（R13 Drop 配对）

- 子代理确认：每次 ensure_reuse 各 +1，Drop 对 _program/_vao 各 −1；Reusable 非 Clone/Copy 且无 Drop → 无双释放；同程序多管线复用计数正确聚合。

## 复验

- 20 测试绿；examples 编译绿；崩率仍 6/6（NVIDIA 驱动侧，非本库可修；本轮达成「每帧不再建删 VAO」降载目标）。