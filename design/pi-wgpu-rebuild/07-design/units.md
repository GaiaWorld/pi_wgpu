# pi-wgpu-rebuild · 类与子程序（L7）

| 类 | 壳 | 隐藏复杂度（存在理由） | 公开操作同抽象层自查 |
|---|---|---|---|
| HandleCache | src/objects/mod.rs | 键计算·refcount·代号失效 | 是（纯句柄面，无 EGL/锁混入） |
| StateMirror | src/state/mod.rs | 差异比较与刷失 | 是（纯状态，不碰资源创建） |
| LayoutApplier | src/binds/mod.rs | reorder·动态偏移消耗·16 槽约束 | 是（纯绑定，不碰 draw 语义） |
| ShaderBridge | src/shader/mod.rs | naga 转译+反射登记 | 是 |
| ContextAttachment | src/context/mod.rs | EGL+锁+搬运+代号 | 是（不含渲染语义） |
| Presenter | src/surface/mod.rs | 换帧分形态收口（configure 建平台表面；present=swap/Web 空操作） | 是 |
| PassRecorder | src/draw/mod.rs | 单目标 FBO（帧=0；离屏=登记差异）·clear/Store；属性批配属性；Drop=end | 是 |

| 子程序 | 壳 | 职责/副作用核查 |
|---|---|---|
| ensure_reuse | src/objects/mod.rs | 查缓存→缺则建→登记；断言无半项（名实对账后=壳实名） |
| set_vertex_buffer | src/draw/mod.rs | 属性批绑定（VAO 批经镜像差异） |
| flush_if_changed | src/state/mod.rs | 比较→下发 |
| get_current_texture | src/surface/mod.rs→surface_inner::get_current_impl | 底层错→封闭四态映射 |
| release_reuse | src/objects/mod.rs | 归零物理删+刷失（单向不回读） |
| invalidate_generations | src/objects/mod.rs | lost 整面失效（幂等） |
| convert_upload_layout | src/upload/mod.rs | 256B→驱动参数换算 |
| consume_dynamic_offsets | src/binds/mod.rs | 计数校验+顺序消耗（顺序性在名） |
| compile_to_es300 | src/shader/mod.rs | naga 调用（WGSL/GLSL 前端+校验）；错误=短码上递+log 诊断 |
| emit_stage | src/shader/mod.rs | 按 (stage,entry_point) 出 GLSL ES 3.0（binding_map 按槽表定位） |
| probe | src/caps/mod.rs | 探测→结构 |

命名无 And、无空动词；副作用均可从名/注释看判。
