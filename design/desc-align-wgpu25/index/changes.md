# 登记分册：变更日志

<!-- ddt 的 tableLog / tableAddRow 负责追行。只记「什么被改成什么、为什么」，供事后审计与返工锚定。 -->

## 变更日志

| 日期 | 阶段 | 变更 | 影响 |
|---|---|---|---|
| 2026-10-15 | L0 | 第零步骨架落册；L0 地形采集完成（上游 wgpu 25.0.2 对账基准确认） | 无 |
| 2026-10-15 | L1 | L1 四册落盘+词汇 3 行；门禁轮提问 3 次（L1 通过/L5 跳过/L7 精简+壳芯），累计本轮 6 次 | L7 按精简档+落壳芯推进；L5 不落册 |
| 2026-10-15 | L7 | 落地门禁过（死变体不抄/entry None 违约/cache 无构造占位）；S1–S4 壳芯落地；PS 编码事故 git 还原重放；check --all-targets 零错误、test --lib 19 通过；07-design/call-chain.md 落册 | examples/契约/词汇全换形；实现批承接 5 项 TODO-DECL |
| 2026-10-15 | L9 | 独立子代理轻审=有条件放行；回改 R5/R8/R12/R16+A1/A2#3/A3-2（flow 判序、词汇补行、GLSL 两测试、切片粒度声明、derive 差异登记、request_device 补对照、断言消息补参）；复验 21 passed/0 failed、all-targets 零错误；09-review.md 落册 | 结论升级为放行（运行回归交用户） |
| 2026-10-15 | 实现批 | 用户批准直做实现（不走 dev-impl）：entry_point=None 自动选（pick_entry 唯一采纳/缺失歧义 NoEntry）、defines 注入 naga 预处理、ShaderStage→naga From 收敛、emit_stage 面转 Option 透传；补 4 测试=24 passed；三例带窗冒烟过（存活4s无崩溃，clear 帧正常） | 壳注释三处同步 entry_point 语义；遗留缩至 3 项（错误码通道/Features 位表/derive 全量对账） |
| 2026-10-15 | 追加批 | BufferSize 对齐上游废包装=type alias NonZeroU64（get/构造式原样兼容，无构造点零破）；TextureFormat 全表 75 变体照抄 25.0.2 入面（名/序/Hash，无 Astc-以 checkout 为准）：形宽法窄=转换表仅收录已接 GL 映射 7 项、表外恒拒（untabled_format_rejects_test），Depth24UnormS8Uint 改名 Depth24PlusStencil8（6 处引用同步）；TODO-DECL 词汇挂账退役 | 25 passed/0 failed，all-targets 零错误；压缩族接表归后续能力批 |
