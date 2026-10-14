# L9 轻审报告（design-lite · L7 返工轮：B11 entry 编排 + 决策⑫）

**结论**：☑ 放行（子代理原判=有条件放行，9 ❌ 全部同场收口）

> 子代理=无对话历史、只读、R1–R16 逐条判。返工对象=B11 entry 全编排（src/entry/entry_inner.rs）+ 决策⑫（DirectHandle 共享计数）。

## 内容核对（子代理原判）

| 检查 | 结果 |
|---|---|
| R1 需求够清楚 | ✅ |
| R2 问题与范围 | ✅ |
| R3 需求↔模块 | ✅ |
| R4 模块切得对 | ✅ |
| R5 图与文一致 | ❌→修 |
| R6 依赖无环无越级 | ✅ |
| R7 变更点三项齐 | ✅ |
| R8 名词有定义 | ❌→修 |
| R9 边界不越 | ✅ |
| R10 能照此开工 | ❌→修 |
| R11 决定有理由 | ✅ |
| R12 假设已验证 | ✅ |
| R13 无过度工程 | ✅ |
| R14 详细设计能落地 | ❌→修 |
| R15 详细设计不越界 | ❌→修 |
| R16 实现切片可照做 | ❌→修 |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R5/R14/R16 | call-chain:14/46 + api:16 | 链图把 reflection_register 放 create_shader_module，但签名需 &PipelineLayout；实现从未调用、_refl 恒空 | call-chain/api | 链行+时序改为「模块创建 compile_to_es300 → 管线期 emit_stage」，reflection_register 标「待接线」（需 layout+可写模块；_refl 零消费者，emit_stage 走 SlotTable） |
| R14 | entry_inner:117 | 实现选择④ adapter_surface_supported 世代比对未落册 | api | Adapter 查询族行补「登记差异(B11)」 |
| R14/R16 | entry_inner:183/245 + errors:42 + 壳注释 | create_rp 对空产物 emit_stage 失败即断言，与「错误码暴露」口径相悖 | errors + entry 壳 | 统一为「create_render_pipeline 非 Result 面无错误通道 → 空产物=编程合同违反（短码断言）」 |
| R15 | entry_inner:200-224 vs 壳:133 | create_bind_group 壳称「资源活态/类型对表」，实现缺 | entry_inner | 补资源类型×布局对表 + 句柄活态断言 |
| R15 | entry_inner:192-198 vs 壳:130 | create_bgl 壳称「binding 域合法」，实现缺 | entry_inner | 补 binding<65536 域断言 |
| R15 | entry_inner:226-229 vs 壳:136 | create_pl 壳称「成员不重复」，实现缺 | entry_inner | 补指针去重断言 |
| R15 | entry_inner:156-175 vs 壳:123 | create_sampler 壳称「compare 与绑定类型相容」，实现缺（且采样器不持绑定类型） | entry 壳 | 壳注释收窄：相容性属绑定组对表，不在本步 |
| R15 | entry_inner:2 | 芯注释残留 design 内号「决定⑪」 | src | 去内号，留「错误纪律」自含 |
| R8 | glossary | 无「共享计数/DirectHandle」条目 | glossary | 补 2 行（决定⑫） |

## 附注收口

- objects::mark_dead 共享计数下「连坐删除」——补适用前提（单持有/整面失效；常规死亡走末引用 Drop）。
- data.md:9 死用口径改写「binds 断言 / upload 错误码」；entry 行补 create_texture 未过镜像差异 + ⑫ 反转注。
- 册内 B 号/链号历史遗留散见（objects:57、state:63、context 测试段、caps_inner 等）——非本轮引入，留 L9 历史债。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed / 0 failed；三组合 Finished。

## 说明（登记差异，未做）

- reflection_register 接线（签名需 layout+可写模块，待接口面决策）；纹理创建绑 GL_TEXTURE_2D 未过镜像；device_features/limits 现探。