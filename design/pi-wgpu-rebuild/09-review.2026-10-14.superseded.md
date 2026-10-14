# L9 轻审报告（design-lite · 第三轮：B4/B5 解冻重落增量复审）

> 独立子代理判卷（无对话历史、直读分册与 src/ 壳芯）；主会话按 ❌ 全部同场修复并复验编译+测试。
> 判卷对象=五处解冻壳面+三册登记对账+词汇新词建行；实现体按授权行为未罚（口径见判卷说明）。

**结论**：☑ 放行（子代理原判=有条件放行，❌ R-1~R-9 全部同场修复，复验三组合 Finished+12 测试全绿）

## 内容核对（判卷原表）

| 检查 | 结果 |
|---|---|
| Caps 四字段注释/合同一致 | ❌→修（形态位/快照缓存副作用未登记壳注释） |
| objects DirectKind/DirectHandle | ✅ |
| HandleCache 容器落定 | ✅ |
| objects 注释密度 | ❌→修（GlHandle/ReuseKey/Reusable 逐字段） |
| objects 刷失名实 | ❌→修（登记制统一口径，六行改） |
| state 字段展开+BIND_SLOTS | ✅ |
| state 函数面契约 | ❌→修（两档口径/sample_mask 记录档/不变量自洽/队列话术） |
| state↔binds 槽同源 | ❌→修（同步点话术+互指注） |
| context attach 条款 | ✅ |
| context Attachment 字段 | ❌→修（_raw/generation 逐字段） |
| formats GlParams | ❌→修（待办话术清除） |
| changes 近段一致性 | 部分误判——:26/\:37 两行「作废旧09」=两次独立事件（首轮归档经用户确认删除；B4 解冻另生），不删；state 壳无解冻批准行→本表补账 ✔ |
| data 三行权威 | ✅*；旧制残留 ❌→修（复用对象/Arc+refcount 行改指向、duplicate StateMirror 行删除、(Handle,u32) 制式行删除） |
| call-chain B0–B14 | ✅；链尾 drop_invalidate 指针 ❌→修（→死名登记→flush 整面销记） |
| 芯 const/static/新函数注释抽查 | ✅（79 处） |
| 芯伪码时效 | ❌→修（objects「镜像直接字段作废」→整面重置口径） |
| 死名销记覆盖（真实缺口） | ❌→修：flush 整发档恒刷全期望后**整面清表**（clear_removed），根除名单单调增长；drop_invalidate 即时作废面保留注释（留给 binds/draw 点名失效，调用方待接） |
| glossary 新词建行 | ❌→修（整发档/可回读档、死名登记刷失、映射原语比较器、最近快照缓存 四行；「镜像刷失」行加机制注记） |

## ❌ 明细与修复对账

| # | 位置 | 修复落点 |
|---|---|---|
| R-1 | src/caps/mod.rs | _ext_family/probe 注释补形态位+快照驻留（壳注释解冻，签名零动） |
| R-2 | src/state/mod.rs | flush_if_changed 两档口径、不变量期望值化、sample_mask 记录档、drop_invalidate 消费方注 |
| R-3 | src/objects/mod.rs、src/context/mod.rs | GlHandle 载荷句、ReuseKey 四变体、Reusable 三字段、Attachment 两字段逐注释 |
| R-4 | src/objects/mod.rs 六行 + call-chain.md 链10 | 刷失=死名登记→flush 强发+整面销记 统一；链尾指针改 |
| R-5 | src/state/state_inner.rs flush 尾部 | clear_removed() 整面清表（原仅程序域 forget，表单调增长根除） |
| R-6 | src/formats/mod.rs | 「字段由实现批展开」待办话术清除 |
| R-7 | data.md :9/:15/:16 + state BIND_SLOTS 注 | 旧制三行收账改写；同源→同步点互指（binds 壳字面量 16 暂保留，B6 接线批统一消费） |
| R-8 | index/glossary.md | 四新词 tableAddRow；旧词注记 tableSetCell |
| R-9 | src/objects/objects_inner.rs | invalidate 注释整面重置口径 |

## 复验

- cargo check 三组合（gl/single_thread/geterror）= Finished 零新增
- cargo test --lib -- --include-ignored --test-threads=1 = 12 passed / 0 failed（含 B5 flush 幂等+死名销记断言）

## 判卷误读说明（不改账）

- changes.md 两条「作废旧 09」行非重复：第一条属 L7 返工轮（归档件后经用户确认删除收尾），第二条（:37）为 B4 解冻重落时 tableSupersede09 新生事件，两条皆事实在册。
