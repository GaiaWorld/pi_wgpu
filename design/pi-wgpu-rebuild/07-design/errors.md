# pi-wgpu-rebuild · 防御与错误（L7）

## 边界与防御（13 点）

| 边界点 | 校验 | 失败后恢复状态 | 归属 |
|---|---|---|---|
| create_surface 目标 | 句柄/画布类型 | 结果值可重试 | entry |
| request_device | 超能力请求 | 结果值；设备未建 | entry |
| configure 组合+尺寸 | 唯一合法组合、非零 | 断言；未配置可再调 | surface |
| begin_render_pass 附件 | 颜色恰一 | 断言；通道未建 | draw |
| set 范围/槽满 | 0–3·16 | 断言；绑定未变 | binds |
| 动态偏移计数 | 与 layout 对表 | 断言；未消耗 | binds |
| draw 参数约束 | 单零参（draw_indexed base_vertex=0） | 断言；无副作用 | draw |
| write_* 偏移/行步/界 | debug 必查 | release：记录+GL 兜底（不谎报） | upload |
| Shader 源 | naga 解析 | codespan→断言 | shader |
| get_current_texture | 底层态 | 封闭四态 | surface |
| present 重复 | 类型级消灭 | 不可能态 | surface |
| 锁超时（try_lock_for） | 等待上限 | 断言+trace | context |
| JS 直传（非位图/越界） | Web 边界 | 断言 | upload |

内部已保证处=断言不重复校验；隔离带=entry 校验一次成型。

## 错误策略（≤2/模块）

| 模块 | 策略集 | 吞错核查 |
|---|---|---|
| entry | 结果值三处+断言 | 断言含 label；无吞 |
| surface | 四态+断言 | 无吞 |
| upload | debug 断言+release 留痕记录 | 日志必行（FR-15 trace 伪代码） |
| shader | codespan+断言 | 不吞诊断 |
| 其余 | 单一断言 | 无 |

geterror/trace=诊断机制，非错误策略。链失败/降级收口：仅链7 有降级（重建）；余=终止断言。
