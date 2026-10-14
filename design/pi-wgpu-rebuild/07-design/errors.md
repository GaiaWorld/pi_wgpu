# pi-wgpu-rebuild · 防御与错误（L7）

## 边界与防御（13 点）

| 边界点 | 校验 | 失败后恢复状态 | 归属 |
|---|---|---|---|
| create_surface 目标 | 句柄/画布类型 | 结果值可重试 | entry |
| request_device | 超能力请求 | 结果值；设备未建 | entry |
| configure 组合+尺寸/建面 | 校验类=SurfaceErr；平台类=SurfaceOpErr | log::error!(码)；未配置可再调（不留半配置） | surface |
| begin_render_pass 附件 | 颜色恰一 | 断言；通道未建 | draw |
| set 范围/槽满 | 0–3·16 | 断言；绑定未变 | binds |
| 动态偏移计数 | 与 layout 对表 | 断言；未消耗 | binds |
| draw 参数约束 | 单零参（draw_indexed base_vertex=0） | 断言；无副作用 | draw |
| write_* 偏移/行步/界 | 错误码(UploadErr) | debug 档不下发 / release 档留痕+下发（不谎报） | upload |
| Shader 源 | naga 解析 | 短错误码上递+log 诊断（决定⑪） | shader |
| get_current_texture | 底层态 | 封闭四态 | surface |
| present 重复 | 类型级消灭 | 不可能态 | surface |
| 锁超时（try_lock_for） | 等待上限 | 断言+trace | context |
| JS 直传（非位图/离线程） | Web 边界 | log::error!(UploadErr)（debug 不下发/release 留痕+降级，决定⑪） | upload |

内部已保证处=断言不重复校验；隔离带=entry 校验一次成型。

## 错误策略（≤2/模块）

| 模块 | 策略集 | 吞错核查 |
|---|---|---|
| entry | 结果值三处+断言 | 断言含 label；无吞 |
| surface | 四态+断言 | 无吞 |
| upload | debug 断言+release 留痕记录 | 日志必行（FR-15 trace 伪代码） |
| shader | 结果值(ShaderErr)+log | 不吞诊断（原文走 log） |
| 其余 | 单一断言 | 无 |

geterror/trace=诊断机制，非错误策略。链失败/降级收口：仅链8 有降级（lost 重建）；余=上递错误码，仅编程合同违反才断言。描述符违约=编程合同违反（entry 创建族，含 create_texture/create_view 的维度/目标校验），故断言。

## 错误处理基线（2026-10-14 新增·决定⑪）

- **优先上递错误**（`Result<_, ErrCode>`），**禁止原地 `panic!` 让运行崩溃**。
- 不便上递的逻辑节点：`log::error!(错误码)` 记录后降级/继续（不静默吞）。
- 错误信息=**短错误码**（枚举），避免长文本；诊断原文（编译器 codespan 等）走 `log::error!`，不进 panic 消息。
- `assert/panic` 仅保留「绝不该发生」的编程合同违反，且消息用短码。
- 应用：B7 起从严；B1–B6 既有断言面登记为「待清理」清单（后续批按需收敛）。
- **边界（公开非 Result 面）**：输入类失败（如着色器源语法错）=log::error!(短码)+降级（不留半建），真错延后上递；仅「绝不该发生」的编程合同违反（如零句柄名、计数成对破）才断言。三条显式 Result 面（request_device/create_surface/get_current_texture）走结果值。**非 Result 面的延后真错无处上递**：如 create_render_pipeline（返回 RenderPipeline 无错误通道）遇空产物 emit_stage 失败=编程合同违反，短码断言（模块创建期已 log 降级）。
