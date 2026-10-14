# L9 轻审报告（design-lite · L7 返工轮：B13 use_wgpu 薄壳 facade）

**结论**：☑ 放行（子代理原判=有条件放行，2 ❌ 同场收口）

> 子代理=无对话历史、只读、R1–R16 逐条判。对象=B13（src/lib.rs / src/lib_inner.rs / Cargo.toml）+ 用户拍板方案 A。

| 检查 | 结果 |
|---|---|
| R1–R5,R7–R13,R15,R16 | ✅ |
| R6 依赖无环无越级 | ❌→修 |
| R14 详细设计能落地 | ❌→修 |

## ❌ 明细与收口

| 条目 | 位置 | 问题 | 收口 |
|---|---|---|---|
| R6 | deps.md:99-110 | wgpu 外部（可选）依赖仅混在边界行，粒度与 naga/pi_share 不一致 | 补独立行 `lib(use_wgpu) → wgpu 25.0.2（path ../wgpu/wgpu，可选；与 gl 互斥）`；边界行只留否性陈述 |
| R14 | api.md:43 | 引「待落地清单 C」全册无定义=悬空（B13 已落指向失效） | 改自足表述（B13 已落 + A/B 命令 + call-chain 指向） |

## 附注收口

- deps.md:311 门控表补 use_wgpu「与 gl 互斥：需 --no-default-features」。
- glossary 补「薄壳 facade」条。
- changes.md:21「待落地清单 C」属历史日志行（不改写历史）；术语已由 api.md:43 收口。

## 复验

- cargo check --features gl = Finished；cargo check --no-default-features --features use_wgpu = Finished（wgpu 25.0.2）；cargo test = 20 passed。

## 说明（登记差异）

- B13 验收=编译级（运行/画面对拍归 examples 冒烟块）；use_wgpu 构建连带全量编译 gl 系依赖（方案 A 文字一致）。