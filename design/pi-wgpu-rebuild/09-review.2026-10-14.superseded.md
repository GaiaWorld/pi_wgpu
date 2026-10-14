# L9 轻审报告（design-lite · L7 返工轮：B9 surface 表示层同步）

**结论**：☑ 放行（子代理原判=有条件放行，3 ❌ 全部同场收口）

> 子代理=无对话历史、只读、按 R1–R16 判。返工对象=B9 接口增量（Surface._cfg RefCell / context 建面换帧 / SurfaceErr·SurfaceOpErr）。

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
| R8 名词有定义 | ❌→修 |
| R9 边界不越 | ✅ |
| R10 能照此开工 | ✅ |
| R11 决定有理由 | ✅ |
| R12 假设已验证 | ✅ |
| R13 无过度工程 | ❌→修 |
| R14 详细设计能落地 | ❌→修 |
| R15 详细设计不越界 | ✅ |
| R16 实现切片可照做 | ✅ |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R8 | glossary | 新增 SurfaceOpErr 未入册 | glossary | tableAddRow 补行 |
| R13 | surface/mod.rs + context | SurfaceErr 与 SurfaceOpErr 变体重叠、SurfaceErr::CreateSurface/NotAttached 为死变体 | surface 壳 | 删两死变体；职责分工=校验类 SurfaceErr / 平台类 SurfaceOpErr |
| R14 | api/data/errors + surface 壳注释 | api 库内清单缺 context 表面函数；data/errors 未收 SurfaceOpErr；壳注释码与芯落码不一致 | api/data/errors + 壳 | api 清单补 surface_attach/surface_swap/web_bind_glow；data 增 SurfaceOpErr 行；errors configure 行两码分工；壳注释同改 |

## 附注收口

- call-chain B9 行补 context_inner::{surface_attach, surface_swap}。
- deps 顶注补「共用闸说明」（with_gl / BIND_SLOTS 不单列功能边）。
- lib.rs 顶注改「已落地批含真实实现，未落地批留桩+ TODO-DECL」。
- 00-index 指向的 09-review.md 由本轮产出。

## 复验

- cargo test --features gl --lib -- --include-ignored = 20 passed / 0 failed；三组合 Finished。

## 说明（不计 ❌）

- get_current 现仅落「未配置→Outdated」；四态映射与 Lost→gen_bump→invalidate 待续（非设计矛盾）。