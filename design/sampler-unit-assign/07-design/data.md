# sampler-unit-assign · 数据结构（L7）

## 新增类型

壳路径：`src/binds/mod.rs`（与「绑定→单元」语义同处一模块）。

| 字段 | 类型 | 含义 | 取值域 |
|---|---|---|---|
| `location` | `i32` | 驱动侧 uniform 位置 | `>= 0`（`-1` 表示查位失败，**不入表**） |
| `unit` | `u32` | 该采样入口要读的 GL 纹理单元 | `[0, BIND_SLOTS)`，即 `0..16` |

```text
struct SamplerUniform { location: i32, unit: u32 }
```

## 既有类型的增字段

| 宿主 | 新字段 | 类型 | 语义 |
|---|---|---|---|
| `entry::RenderPipeline` | `sampler_uniforms` | `Vec<SamplerUniform>` | 该管线创建期定格的指位表（与 `own_slots` 同生命周期） |
| `record::RetainedPipeline` | `sampler_uniforms` | `Vec<SamplerUniform>` | 录制持行体携带同一张表（延后执行下提交回放仍须知道指到哪） |
| `state::StateMirror` | `sampler_uniforms` | `Vec<SamplerUniform>` | 期望态：当前生效管线要求程序指到的位置 |

## 不变量（住类型注释，落壳后以壳为准）

1. 指位**只随 flush 的程序组下发**成对同段发生（`state_inner.rs:911-915` 分支内）。既有第二个发点 `set_program`（`state_inner.rs:514-520`，由 `exec_set_pipeline_parts` 触发）**不附指位**——其合法性来自：两发点之间不存在任何绘制命令，且每条绘制入口都先 `flush_impl` 再画；程序水位未被直发推进 → flush 必重发 `use_program` 并全量指位。故任何取样命令执行前，程序侧采样 uniform 必已就位。
   - 附带事实：程序切换会造成 `use_program` 双发（直发 + flush 重发），系**既有行为**，本设计不改、不得顺手优化掉任一侧，否则上一条覆盖论证失效。
2. 该表只由「设管线」写入镜像，绘制路径不写。
3. 表内 `(location)` 互不重复；`unit` 全部落在 `[0, BIND_SLOTS)`。
4. 空表是合法态（无采样入口的管线），不代表未初始化。

## 表示先于算法（为何存 location 而非 group,binding）

查位置必须触 GL。若存 (group,binding)，则在程序切换点（绘制热路）里查——把 GL 查询搬进每帧路径，且每次切换重复查同样的名字。存 (location,unit) 把触 GL 的部分前移到管线创建期一次做完，切换期只发整数；代价是 location 随程序对象生死（程序死则表作废，随 `RetainedPipeline` 部件同生命周期释放，天然成立）。
