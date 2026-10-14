# L9 轻审报告（design-lite · B18 增量：可回读档加帧门）

**结论**：☑ 放行（子代理原判=有条件放行，1 ❌ + ⚠️ 同场收口）

| 检查 | 结果 |
|---|---|
| R1 壳芯一致 | ⚠️→修 |
| R3 帧边界语义 | ✅（附登记） |
| R4 复位路径 | ✅ |
| R5 并发/原子性 | ⚠️→修 |
| R7 文档诚实性 | ⚠️→标注 |
| R8 其它 | ⚠️→登记 |
| R2 正确性 | ❌→修 |
| R6 测试充分性 | ⚠️→修 |

## ❌ 收口

| 条目 | 问题 | 收口 |
|---|---|---|
| R2 | upload 在专用单元上传后盲还 glActiveTexture(GL_TEXTURE0) 且不改镜像 active_texture_unit → 帧门跳过回读后，「驱动 active ≠ 镜像 active」的自愈延到下一帧，bind_texture 守卫（按镜像判已激活）可能跳过切换、在错误单元绑定纹理 | 新增 upload 芯 upload_restore_unit(q)（读镜像 active_texture_unit）；上传后恢复到该值（native write_texture_impl + wasm write_js_impl 两路径），恢复「驱动 active == 镜像 active」不变量 |

## ⚠️ 收口

| 条目 | 问题 | 收口 |
|---|---|---|
| R5 | 门内两次 frame_now() 非原子对：他线程 present 于两行之间推进会把 readback_frame 盖成 N+1，令下一帧回读被误跳过 | 单次取值 let rb_frame = frame_now();，比较与落值同源 |
| R6 | 回读段无 probe，帧门跳过/重做不可观测；无 bump 用例 | 新增 probe(2)=回读段执行次数（TEST_PROBE 扩到 3 槽）；gl_state_machine_test 补断言：首帧回读执行 / 同帧二次 flush 跳过 / bump_frame 后重做 |
| R1 | 壳 flush_if_changed 方法级注释未提帧门 | state/mod.rs flush_if_changed 注释补「可回读档另受帧门约束：每帧（present 推进帧号）一次回读比对，帧内复用镜像态」 |

## 附注收口（登记差异）

- FRAME 为进程级 static：多 Device 共享 → 他设备 present 令本设备多回读（仅多余，不错误）；无复位点；u32 回绕（约 2^32 帧）理论可等值跳一帧。
- 一帧内多次 present（多 SurfaceTexture 租约）未受强约束：仅多回读，无错误；登记「多租约=违约用法」。
- get_current 不推进帧号（present 已在帧尾推进）——首帧靠 readback_frame=None≠Some(0) 兜底，正确。
- 崩率「帧 2 仍 6/6」为环境实测（需 NVIDIA + 窗口），仓内无脚本佐证、不可离线复核；已在 changes 行标注。

## 复验

- cargo check --features gl --all-targets：Finished。
- cargo test --features gl --lib -- --include-ignored --test-threads=1：20 passed（回读帧门：首帧执行 / 同帧跳过 / bump 后重做）。
- 实测 hello_triangle 帧 2 崩率仍 6/6（与状态调用量无关，另行定位）。
