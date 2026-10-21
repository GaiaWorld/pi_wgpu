# 登记分册：变更日志

## 变更日志

| 日期 | 阶段 | 变更 | 影响 |
|---|---|---|---|
| 2026-10-21 | L0+L1 | 落 00-context 与 01-requirements 四册（problem/needs/flow/notes），骨架三册（00-index、index/status、index/glossary、index/changes）。本轮提问 3 次（共用程序是否必须支持布局不同各自正确 / 产品出画是否硬验收 / 走哪些阶段），答分别为「必须，程序切换时按当前管线布局重指」「是，必须出画」「L7 全册档（跳过 L5）」。另记一条已排除项：录制序与提交序之颠倒已由 desc-align-wgpu25 的命令缓冲摆正，非本设计成因 | L1 需求五条带可测验收；L5 按用户决定跳过；下游 L7 全册待落 |
| 2026-10-21 | L7 落地 | 落壳芯四处（binds/mod.rs 的 SamplerUniform 与 unit_of 壳、binds_inner.rs 桩、shader/mod.rs 与 shader_inner.rs 的 sampler_targets 壳桩），编译核验零新增错误、warning 与基线对拍 1440→1440 零新增，31 lib 通过。落地时发现已批准文本一处不成立并**经批准回改**：api.md 的 unit_of 理由与 call-chain.md 的伪码原写「复用 lookup_slot」，但 lookup_slot 未命中即 panic（binds_inner.rs:33-41 合同=布局与组不对表即断言），与 unit_of 须返回 None 的合同冲突（指位期「着色器声明纹理绑定而本管线布局未给」是预期分支）；改为「扫描 domain_array(Texture) + bind_base」，单元算式仍与 apply_impl 同源。本轮提问 2 次（修正批准 / 落地门禁），答为「批准按 diff 回改」「通过，进 L9 轻审」 | 五册原文与壳注释一致；下游 L9 判的即此版本 |
| 2026-10-21 | L9 | 派独立子代理轻审（只读、无对话历史、直读分册与 src 壳芯）。结论=有条件放行：R12 一条、R14 两条、另有一条关键坑。主上下文逐条回源复核全部为真：state_inner.rs:514-520 确有第二个 apply_program 直发点且不推水位；naga mod.rs:1454/1479 资源全局名第三段为 entry_point.stage.to_str()，故同 (g,b) 在 VS/FS 是两条 uniform 两个 location。经批准按四条回改：00-context 待验证表改三列并记静态核证证据（utype 项作废）、notes 未决清单同步、data.md 不变量1 改述两发点覆盖论证并注明 use_program 双发系既有行为、call-chain 伪码补注与切片3加双 stage 用例、errors.md 删下轮重试承诺改为声明指位无失败分支、api.md collect 行加按名字入表禁 (g,b) 去重。处置后结论=放行。本轮提问 2 次（L9 结论确认 / 是否交接实现），答为批准四条按表回改后收尾、只交付设计与壳芯实现另开 | 五册 + context + notes 与审查结论一致；设计基线冻结，实现从冻芯写起 |
