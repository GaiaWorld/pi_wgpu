# L9 轻审报告（design-lite · B17 增量：state 整发档改「已下发水位」比对）

**结论**：☑ 放行（子代理原判=有条件放行，2 ❌ + ⚠️ 同场收口）

| 检查 | 结果 |
|---|---|
| R3 死名语义 | ✅（附登记） |
| R4 复位路径 | ✅ |
| R7 代码质量 | ✅ |
| R1 壳芯一致 | ❌→修 |
| R2 正确性风险 | ❌→修 |
| R6 测试充分性 | ❌→修 |
| R5 文档诚实性 | ⚠️→修 |

## ❌ 收口

| 条目 | 问题 | 收口 |
|---|---|---|
| R1 | 芯模块头仍写旧语义（按驱动未知处理=幂等整组重发 / 回读比对命中=视同差异强制重发），与水位实现矛盾 | state 芯头注改写为水位语义：不可回读组按 Water 比对、差异才发；水位空或死名表非空=整组强发一轮；表非空=强发、末尾整面销记 |
| R2 | create_texture 建后直绑当前 active 单元，不过镜像与水位；下次 flush 命中水位跳过 → 驱动残留新纹理（潜在画面错误，非仅多余调用） | 新增 state 芯 invalidate_texture_water（置全部纹理槽水位=未下发）；create_texture 建后调用，令下次 flush 重发镜像期望以复位驱动绑定 |
| R6 | 恒真断言 dbg_removed_program(...)==false（该名从未 release，恒 false，并未验证销记） | 首轮 flush 前 note_removed(0, prog_name) 造死名 → flush 强发并销记 → 断言真验销记路径 |
| R5 | changes.md「60+ 冗余调用→0」略强 | 收敛为「整发档稳态→0」（可回读档仍每 flush 约 10 次 get_parameter） |

## 附注收口（登记差异）

- draw begin/end 直 bind_framebuffer(GL_DRAW_FRAMEBUFFER, None) 旁路：set_draw_fbo 今无调用方 → 镜像 fbo_draw 恒 None，侧路写 None 无可达分歧；待离屏批接线时收口。
- 死名表为进程级 static，而 water 每 Device 一份：多设备下首个 flush 即消费强发信号（单设备无碍）。
- ubo 槽为 None 从不解绑（整发循环 if let Some(...)）：现由 bind_ubo(None) 直发解绑兜住，与 tex/sampler 的 Some(None) 解绑路径不对称。

## 复验

- cargo check --features gl --all-targets：Finished。
- cargo test --features gl --lib -- --include-ignored --test-threads=1：20 passed（含水位增量：首轮有组 / 二次零组 / 造死名→flush 销记）。
- 实测 hello_triangle 帧 2 崩率仍 6/6（崩溃触发点非「状态调用洪峰」，另行定位）。
