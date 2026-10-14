# L9 轻审报告（design-lite · L7 返工轮：B8 upload 表示层同步）

**结论**：☑ 放行（子代理原判=有条件放行，4 ❌ 全部同场收口）

> 子代理=无对话历史、只读、按 R1–R16 判。返工对象=B8 接口增量（Buffer.size / Texture.params / UploadErr）。

## 内容核对（子代理原判）

| 检查 | 结果 |
|---|---|
| R1 需求够清楚 | ✅ |
| R2 问题与范围 | ✅ |
| R3 需求↔模块对得上 | ✅ |
| R4 模块切得对 | ✅ |
| R5 图与文一致 | ✅ |
| R6 依赖无环、无越级 | ✅ |
| R7 变更点三项齐 | ✅ |
| R8 名词有定义 | ✅ |
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
| R14 | errors.md:19 | 「JS 直传→断言」与决定⑪/壳 log 口径相抵（B8 漏改本行） | errors.md | 改 log::error!(UploadErr) 两档 |
| R14 | api.md:20 | write_texture_jsbuffer「非位图/越视野=断言」相抵 + 笔误 | api.md | 改「非位图/离线程=log::error!(UploadErr)」 |
| R14 | src/entry/mod.rs | Buffer.size / Texture.params 无字段级注释（违项目纪律） | entry 壳 | 两字段各补注释（含义/来源/消费点） |
| R14 | src/upload/mod.rs ↔ 芯 | 壳承诺「矩形落在 mip 层范围内」芯未实现（Texture 无尺寸） | upload 壳 | 收窄：矩形超界=驱动拒绝（release 兜底），登记差异 |

## 附注收口

- **01-requirements/notes.md 文件损坏**（内嵌 read 工具包装 + 行号前缀 + 伪尾）→ 已复写清污，保留正文与「L5 门禁校准追加记录」节；全 design 目录扫描无同类污染。
- api.md 内部操作清单补 context_inner::with_gl / generation_now。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed / 0 failed；三组合 Finished。

## 说明（不计 ❌）

- 决定⑪「B7 起从严、B1–B6 待清理」维持。