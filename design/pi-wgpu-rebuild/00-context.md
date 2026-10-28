# pi-wgpu-rebuild · context（L0）

<!-- 事实层：不随 L1/L5 返工作废。 -->

## 地形（客观事实，原仓结构仅作背景、**不采用**）

- 本工作区 `E:\\Rust\\PI_PI\\pi_wgpu_ai`：仅 documents/（6 篇）+ README.md，**无 src / 无 Cargo.toml** —— 重建落点为空。
- 语言：Rust，edition 2021、参考基线 v0.3.7（pi_wgpu/Cargo.toml:3-4）；参考实现 `#![feature(hash_extract_if)]` → nightly。此为旧实现事实，**非本次约束**（选型在 L1/L5 重定）。
- 私有依赖本机可得：`E:\\Rust\\PI_PI\\pi_egl / pi_glow / pi_share / pi_assets / pi_hash / pi_time / pi_naga` 与 `wgpu`（含 `../wgpu/naga` fork）；registry `yn` 已在 `~/.cargo/config.toml` 配置。
- 旧仓 v0.3.7 分层 wgc/wgt/hal/util、其完整模式重建的壳+芯布局（pi_wgpu/src）**记录在此仅为地形事实，用户判「不采用其结构」**——新模块切分见 L5、接口见 L7，从零定。
- 测试位置参考：旧实现无独立 tests/，验收=examples 冒烟（`cargo run --example triangle`、`wasm-pack test --chrome`、`cargo apk run`）+ `[[example]] test = true`。**此为惯例参照，测试策略在 L1/L5 重定**。
- 构建命令（L7 壳芯核验用）：新工程落定后用 `cargo check --features <选定>`；**当前基线＝空工作区未建工程，编译核验暂不可用**（L7 落壳芯时照摆此代价）。

## 素材

- `documents/`（6 篇）+ README：**输入素材**——用户判「不合理处可改、不作绝对正确」（见来源·对话⑥）。已通读全文，其自述「真相源」的权威被下调：L1 起逐条审读其结论，冲突/存疑/落后处就地提出并修正。
- 旧仓 `pi_wgpu/{src,design}`：结构**不采用**；「v0.3.7 实际长什么样」可作背景反查，非判据。未逐文件细读——影响：L1 审读 documents 结论若需与真实现核对，再定点查阅。

## 来源（三类）

- 对话（**最高权威**）：用户拍板 —— ①本工作区重建 pi_wgpu ②沿用同名私有 crate ③slug=pi-wgpu-rebuild ④与旧完整模式设计**独立重做** ⑤**不依赖原仓结构** ⑥**documents/ 非绝对正确，不合理处可改**。
- 文件（documents/ + README）：需求/设计的**素材**，须经 L1–L7 审读，允许推翻修订，不作绝对源。
- 组织既有方案：旧仓 pi_wgpu（源码+完整模式设计）——结构不采用、结论不默认继承，仅背景参考。目标平台侧（微信小游戏/iOS/存量设备）以 documents 内引用为线索，**无一手渠道 → 相关平台约束标待验证**（见下表），不得拿通用惯例顶替。

## 待验证

| 待验证项 | 归属 | 影响结论 | 何时解决 |
|---|---|---|---|
| registry `yn` 在线可解 + 各私有 crate 版本兼容（pi_egl 0.2 / pi_glow 0.13 / pi_assets 0.15 / naga 25.* …） | 依赖环境 | L5 依赖清单与构建可行性 | 新工程首跑 `cargo check` |
| 工具链：nightly 是否硬约束、`hash_extract_if` 有无 stable 替代 | 构建 | NFR 工具链条目 | L1 澄清 |
| 目标平台一手约束（微信小游戏 WebGL2 上限、iOS≥15.5、iPhone VB/IB 共享偏移性能） | 平台文档 | L1 非功能/兼容需求、L5 特判 | L1 向用户核 |
| documents 结论与真实现是否漂移（如 get_capabilities 文档标 × 实有、write_* 即时生效 vs 文档「下次 submit」） | 素材可靠性 | L1 审读结论 | L1–L5 逐条核 |
