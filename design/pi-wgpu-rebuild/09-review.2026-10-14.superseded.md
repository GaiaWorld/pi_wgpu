# L9 轻审报告（design-lite · L7 返工轮：B10 draw 芯 + 设备接线）

**结论**：☑ 放行（子代理原判=有条件放行，9 ❌ 全部同场收口）

> 子代理=无对话历史、只读、R1–R16 逐条判。返工对象=B10（Device=Share<DeviceInner>、CommandEncoder 持 _inner、RenderPipeline/RenderPass 字段、apply_layout 去 pipe、draw_inner）。

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
| R15 详细设计不越界 | ❌→修 |
| R16 实现切片可照做 | ✅ |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R14 | draw 面 | 索引格式/偏移无载体、硬编码类型、偏移被丢 | draw 壳+芯 | RenderPass +_index_format/+_index_offset/+_vao；draw_idx 按格式选类型、按 o+start×width 定位 |
| R14 | draw 面 | begin 校验深度附件/end 落定 store 契约无载体 | draw 壳 | 契约收窄（深度/store 本批暂缓·登记差异） |
| R14 | objects_inner/formats_inner | 芯注释引册内代号（链2/链10） | src | 改语义自含 |
| R15 | draw_inner | 恒真断言（视图占位） | src | 删 |
| R15 | draw_inner | 哑消费静默丢弃索引入参 | src | 由索引字段修复承接（消费 o/f/c） |
| R15 | draw_inner | VAO 每次新建不删、不走复用/镜像 | draw 芯 | objects.ensure_reuse(ReuseKey::Vao 布局哈希) + set_vao + 通道内持有/Drop 释放 |
| R15 | draw 壳 | set_viewport 壳称唯一翻换点 vs 芯不翻换 | draw 壳 | 改「存原始值；翻换本批暂缓·登记差异」 |
| R5 | deps.md | 注记与表/图/壳不符；缺 formats→context、draw→entry | deps.md | 表+图补 3 边 |
| R5 | 00-index | 指向 09-review.md（本轮产出） | — | 本轮产出即闭 |
| R8 | glossary | SurfaceErr 含死变体 | glossary | 改定义（三变体；平台归 SurfaceOpErr） |

## 附注收口

- errors.md 链号更正（降级=链8）＋「描述符违约=编程合同违反故断言」。
- DeviceInner 单测用 std::sync::Mutex = ShareMutex 别名（非 rc 默认），一致。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed / 0 failed；三组合 Finished。

## 说明（登记差异，未做）

- 深度/模板附件与 store 语义、离屏 FBO 路径、viewport/scissor 的 Y 内翻（缺目标尺寸）。