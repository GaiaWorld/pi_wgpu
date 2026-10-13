# pi-wgpu-rebuild · 变更点（L5 · 资产出库校准版）

| 变更点 | 变更方向 | 隔离位置（模块） | 影响面（受影响的调用方） |
|---|---|---|---|
| 上游 wgpu 出新版本 | 词汇对表/子集裁剪 | types | 全模块词汇集合、entry 签名对表、样例编译 |
| 错误面扩大 | 更多路径改结果值 | entry | 样例/上层代码、L7 接口册、迁移清单 |
| 呈现约束放开（如新合法组合） | 合法组合增 | surface | entry（configure 委托方） |
| 新增问题驱动特判 | 登记项+1 | caps | surface/draw 消费点（经能力位） |
| 镜像字段扩展 | 纳入更多状态比较 | state | draw/upload（绑定应用路径） |
| 复用键冲突或粒度调整 | 缓存策略（与删除时机同属释放规则） | objects | 调用方=entry（create 族）、state（刷失）、upload（写入定位） |
| [同上行合并注] 删除时机即删改回延迟=释放规则的另一面 | 语义变化（键+删除两面同向改） | objects（归零即删）→state（刷失） | 调用方=entry/utils 资源面+上层文档 |
| 绑定槽上限放宽 | 绑定模型 | binds | draw 的 set_bind_group、entry 布局创建、shader 反射登记 |
| 转译后端换 naga 版本/形态 | 着器桥 | shader | objects（program 缓存键哈希源） |
| 通道扩多目标 | 能力扩展 | draw | formats（附件组合）、样例 |
| 写入时机改提交栅栏（翻轮1决定） | 语义回调 | upload | 调用方=entry（Queue 面）、utils（create_texture_with_data 组合）；文档三处口径、验收用例 |
| 驱动形态约束变化（浏览器新增限制等） | 平台适配 | caps+context+surface | 调用方=entry（伞面）、upload（探针）；镜像/缓存/绑定不动（交集原则） |
| 写链扩展面变化（JS 直传面收缩/扩大） | API 边界 | upload | 迁移差异清单、上层资产 crate |
| 线程模型调整（全主线程部署/新增搬运） | 合同收紧/放宽 | context | 下涉各模块调用点（锁面） |
| 便捷面扩项 | API 增补 | utils | 样例、上层工具链 |
| 安卓 swappy 帧调度开启（vsync 链改） | 呈现实现细节 | surface+context 芯 | 调用方=entry（present 委托链）；与判⑩ fps 不开同源决策 |
| 业务资产复用面回渗库内（上游要求键能力） | OUT 边界回调 | 无模块可隔离→回 L1（免责注记：此变更点故意不落模块=OUT 属性） | 判⑦作废、objects 职责重写 |

<!-- 原「回收超时参数/出队点」「收缩面转发」两行作废：队列与收缩均出本库（资产校准 2026-10-13）。 -->
