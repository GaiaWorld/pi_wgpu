# L9 轻审报告（design-lite · L7 返工轮：B12 utils 芯）

**结论**：☑ 放行（子代理原判=有条件放行，3 ❌ 全部同场收口）

> 子代理=无对话历史、只读、R1–R16 逐条判。对象=B12 utils（src/utils/utils_inner.rs）。

| 检查 | 结果 |
|---|---|
| R1–R4,R6,R7,R9–R13 | ✅ |
| R3 需求↔模块落点 | ❌→修 |
| R5 图与文一致 | ❌→修 |
| R8 名词有定义 | ✅ |
| R14 详细设计能落地 | ❌→修 |
| R15 详细设计不越界 | ✅ |
| R16 实现切片可照做 | ❌→修 |

## ❌ 明细与收口

| 条目 | 位置 | 问题 | 收口 |
|---|---|---|---|
| R14/R5 | utils 壳:13 + api:27 | 壳承诺「按 order 排布语义」，芯丢弃 order 只写 mip0；差异未随壳/api 登记 | 壳注释+api 行补「仅 mip0 单段直写，order 多段排布未实现」；`let _ = order;`→形参 `_order` |
| R3/R5/R16 | call-chain:98 + modules:19 | B12 目标含「锁包装」，utils 职责含「锁包装+调用 trace 单点」，实现皆无、changes 未登记 | modules/call-chain 标「待落（登记差异，不静默删）」 |
| R5 | call-chain:15/17 | 便捷面行只列 create_buffer_init；create_texture_with_data + 3 env 助手无链引用，闭包注自否 | 便捷面行补 create_texture_with_data→entry.create_texture+upload；闭包注例外加 3 env 助手 |

## 附注收口

- backend_bits 壳措辞收窄「恒返回 Some(GL)」（原「GL 位或 None」None 分支不存在）。
- 芯内 Queue 由共享内芯直接构造（dev._inner.clone()）——芯互访，无公开契约外泄。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed；三组合 Finished。

## 说明（登记差异）

- order 多段排布未实现（仅 mip0 单段直写）；锁包装 + 公开入口 trace 单点待落；env 恒 GL。