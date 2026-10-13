<path>E:\Rust\PI_PI\pi_wgpu_ai\design\pi-wgpu-rebuild\01-requirements\notes.md</path>
<type>file</type>
<content>
1: # pi-wgpu-rebuild · 非功能 / 未决 / 澄清（L1）
2: 
3: ## 非功能
4: 
5: - 工具链：nightly（轮1 拍板；修正 README「1.65+」徽章与 nightly 特性用法的矛盾——documents/overview.md:138 自证）。
6: - 依赖环境：沿用私有 registry `yn` 同名 crate + naga 本地 fork（用户拍板 + L0 地形）。版本锁定与升级路径在 L5 decisions.md 定（引新工程 ⇒ L5 必做）。
7: - 性能判据：下发驱动指令数 ≤ 参考实现同样例，以抓帧比较为准（documents FR-11/NFR-03）。
8: - 体积/依赖面：核心路径不依赖官方 wgpu，仅对照开启用（documents NFR-05）。
9: 
10: ## documents 审读修正（用户授「不合理可改」）
11: 
12: 1. README「Rust-1.65+」徽章过时 → 本设计定 nightly。
13: 2. README §2.07 对 `get_capabilities` 的「不支持」记录过时（api.md §7 自证已有）→ 支持面以本设计 L7 接口册为准。
14: 3. documents 内部口径不一（module-design.md:50 说写入直接生效 vs README §2.10 沿用 wgpu「提交排队」表述）→ 统一为「即时生效」，写入需求与迁移差异清单。
15: 
16: ## 未决清单
17: 
18: | 待决项 | 归属人 | 预期解决时点 | 来源 |
19: |---|---|---|---|
20: | 无（13 问当场清零，不以合理默认代替） | — | — | — |
21: 
22: ## 待验证（承 L0，非未决：不阻门禁，写明解决时点与影响）
23: 
24: - registry `yn` 在线可解与版本兼容 → 新工程首跑 cargo check（影响 L5 依赖清单）。**✔ L7 收敛（2026-10-13）：未依赖在线 registry，path 依赖全解析+cargo check Finished。**
25: - Android 真机链路与各平台一手约束 → 用户侧提供环境时（影响 FR 呈现/特判验收级别）。
26: - documents 其余细节与真实现的漂移 → L5/L7 逐块定点反查旧仓（影响对照素材采信）。
27: 
28: ## 澄清记录（问答摘要）
29: 
30: - 轮0：①目标=本工作区重建 ②依赖=沿用同名私有 crate ③slug=pi-wgpu-rebuild。
31: - 边界：④与旧完整模式设计独立重做 ⑤documents=唯一需求素材；随后用户追加 ⑥「不依赖原仓结构」⑦「documents 非绝对正确可审改」。
32: - 轮1：⑧提交语义=沿用录制即执行 ⑨工具链=接受 nightly ⑩回读=仍不做 ⑪生命周期=沿用使用方合同。
33: - 轮2：⑫基线=wgpu-rs 25.x 兼容子集 ⑬验收=Win+浏览器先行、Android 延后 ⑭绑定上限=写死 ⑮错误面=沿用（三处结果值+断言）。
34: - 门禁：⑯L1 通过进 L5 ⑰L5+L7 都做。
35: 

(End of file - total 35 lines)
</content>
## L5 门禁校准追加记录（2026-10-13）

- L5 阶段共 9 轮澄清（后端版图→驱动形态图解→上传链→删除时机/复用范围）。
- 用户判「资产设计以 pi_render 生态（pi_render_asset）为准」：业务资产管理（键值复用/显存记账/异步加载/收缩）出本库，进 problem OUT；本库保留线程面与确定删除时机面。
- 删除时机=即删+镜像同刷失（三硬规则合同变硬）；复用范围=管线级（程序/VAO）+离屏附件（FBO/RBO），纹理/缓冲/采样器 1:1 直建。
- 依赖清单减 2：本库不再直接依赖 pi_assets / pi_time（L5 decisions 同步）。
