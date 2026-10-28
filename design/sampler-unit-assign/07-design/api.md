# sampler-unit-assign · 接口面（L7）

> 全部为 `pub(crate)` 内部面；**本次不动任何 `mod.rs` 的公开壳签名**（新增的三个内部操作按项目惯例经所属模块壳注释给契约，见各壳路径）。
> 可见性档：crate 内（替身库无对外新增 API）。

| 操作 | 壳路径 | 一句语义 | 失败后状态 | 参数域与单位 | 各分支返回 | 不变量 / 副作用 / 幂等 | 理由 |
|---|---|---|---|---|---|---|---|
| `shader::sampler_targets` | `src/shader/mod.rs` → `src/shader/shader_inner.rs::sampler_targets_impl` | 从反射表挑出会作为 GLSL 采样入口存在的 (group,binding) | 纯函数，不失败；空表=本程序无采样入口 | 入=管线创建期已登记的 `ReflectionTable`；出=(group,binding) 二元组 | 恒返回表 | 无副作用；幂等；输出按 (group,binding) 升序去重 | naga 对 ES300 跳过 Sampler 全局（`naga/src/back/glsl/mod.rs:964`），合并后的 `sampler2D` 即纹理全局，故候选集=反射里 `kind==Texture` 全体 |
| `binds::unit_of` | `src/binds/mod.rs` → `src/binds/binds_inner.rs::unit_of` | 查某纹理域绑定在本槽表里占用的 GL 纹理单元号 | `None`=该 (set,binding) 不在本表；**不改任何状态** | 入=槽表 + set/binding（声明号，非单元号） | 命中=`Some(unit)`；未命中=`None` | 纯查询；幂等；unit ∈ `[0, BIND_SLOTS)` | 复用 `domain_array(BindDomain::Texture)` 扫描，返回的下标**即**单元号（紧凑编号、无 set 基址），与 `apply_impl` 实绑单元天然同源、杜绝两处算式漂移；但**不得调用 `lookup_slot`**——其未命中即 panic（`binds_inner.rs:33-41` 合同=布局与组不对表即断言），与本操作可返回 `None` 的合同冲突（指位期「着色器声明了纹理绑定但本管线布局未给」是预期分支） |
| `entry::collect_sampler_uniforms` | `src/entry/entry_inner.rs`（内部，不出壳） | 把「采样入口 → 单元」翻译成驱动侧 (uniform location, unit) 对表 | 单条解析不出前缀=静默跳过；解析出但 `unit_of=None`=`log::error!` 短码后跳过该条；**整体不失败**，返回已收集部分 | 入=活程序句柄 + 本管线槽表 + targets 表；须处在 GL 段内 | 恒返回表（可为空） | 触 GL（枚举与查位）；幂等（重复调用得同表）；不改变驱动绑定态。**按名字入表，禁止按 (group,binding) 去重**——naga 名带 stage 后缀（`_group_G_binding_B_vertex` / `_fragment`），同一绑定被 VS 与 FS 各采一次时是**两条 uniform、两个 location、同一 unit**，去重即丢半 | 位置查询要触 GL，放管线创建期一次做完；切换期只发整数，成本可忽略。判据不用 `utype` 枚举集（19 个 GL_SAMPLER_* 不引入维护面），改用「名字前缀解析 + targets 白名单」 |
| `exec_set_pipeline_parts`（改） | `src/draw/draw_inner.rs` | 设管线时把该管线携带的指位表写入镜像期望态 | 与现状一致：整表覆盖，无半写 | 入=通道 + 管线部件 | 无返回 | 只改镜像，不触 GL；幂等 | 期望态住镜像=本库既有架构（镜像+水位差异下发） |
| `flush_impl` 程序分支（改） | `src/state/state_inner.rs` | 下发 `use_program` 之后，紧跟逐条 `uniform_1_i32(location, unit)` | 与现状一致；单条指位失败只记短码，不影响后续绘制 | 无新参数（读镜像） | 无返回 | 只在程序水位差异触发时执行；与 `use_program` 同 GL 段、同原子面 | 采样器 uniform 是**程序态**：程序没换就不必重指 |

## 不做什么

- 不改命令缓冲/提交序语义（`desc-align-wgpu25` 已定基线）。
- 不动 P2 的 `SlotTable.alias` 与采样器对象下发路径。
- 不引入分离采样器语法（目标 GLSL ES 3.00 无此写法）。
- 不改任何 `mod.rs` 公开壳签名，故对外 API 面无变化。
