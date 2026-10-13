# L9 轻审报告（design-lite · 第二轮·返工复审）

> 独立子代理判卷（无对话历史、直读分册与 src/ 壳芯）；主会话按 ❌ 全部就地修复并复验编译。

**结论**：☐ 放行 ｜ ☑ 有条件放行（❌ 全部同场修复）｜ ☐ 不放行

## 内容核对

| 检查 | 结果 | 一句依据（修复前判语） |
|---|---|---|
| R1 需求照着能写实现/测试 | ✅ | 17 条带可执行验收 |
| R2 问题册无技术名词、范围内外 | ✅ | OUT 五行划定 |
| R3 需求↔模块落点 | ❌→修 | trace 日志无落点→补进 utils 职责+B12 验收 |
| R4 每模块易变决定恰一 | ❌→修 | entry 行两决定→收敛为「公开对象面合同形态」 |
| R5 图与文一致 | ❌→修 | 12/13 计数与链号双轨→口径注+改两处引用 |
| R6 依赖无环无越级 | ✅ | 13 边对合 |
| R7 变更点三要素 | ✅ | 17 行实名 |
| R8 新名词有定义 | ✅ | 词汇表在册 |
| R9 范围不越·选型留痕 | ✅ | 出库同步落 Cargo |
| R10 能照此开工 | ✅(修后) | 坑=Lost 重建回路→已在公开面补再入契约（见修复⑤） |
| R11 决定有理由 | ✅ | ①–⑩带被否 |
| R12 假设已验证 | ✅ | registry 等留痕 |
| R13 无空预留 | ✅ | deferred 全带 TODO-DECL+消费者 |
| R14 详细设计能落地 | ❌→修 | 代号残留④+契约-签名断裂⑤⑥⑦+矛盾⑧→全修，残留扫描=src 干净 |
| R15 不越界·壳合规 | ✅ | submit/finish=册记空壳契约例外；档标一致 |
| R16 实现切片可照做 | ✅ | B0–B14 序全 |

## ❌ 明细与修复对账

| # | 位置 | 问题 | 修复落点 |
|---|---|---|---|
| 1 | needs↔modules↔call-chain | 全量调用 trace 无模块/块落点 | modules utils 职责补 trace 单点；call-chain B12 验收列随批注 |
| 2 | modules entry 行 | 易变决定两项违恰一 | 收敛「公开对象面合同形态（错误面+引用链=同体两面）」 |
| 3 | modules/deps 计数、call-chain 链号 0/1 基混用 | 图节点口径+引用错行 | deps 顶注口径注（12+types=13）；链号=首表行号声明；表第2行与闭包段两处改 1 基 |
| 4 | src/surface/mod.rs「封闭错误面③」 | 册编号记号入壳 | 改写自含三处显式 Result 面全名 |
| 5 | context.gen_bump / objects.invalidate_generations | 整面失效账签名走不通+公开面无 Lost 再入点 | 账目改「代号失配自然退场+重走 request_device=公开重建」；invalidate_generations 增 cache 参改为显式预清工具（常规路径不依赖）；entry request_adapter 壳+芯补再入合同 |
| 6 | binds alloc/apply 与契约不符 | 「记入管线」「经槽映射查槽」无载体 | 契约改「写入设备级槽表」；apply 芯形参欠缺标 TODO-DECL |
| 7 | utils create_*_init 无 device | 芯伪码 step1 悬空 | 芯形参加 device、委托实体移入 impl DeviceExt for Device（trait 签名不变） |
| 8 | types「has_dynamic_offset=pi 扩展位」 | 与官方同名自相矛盾 | 改「同名同形同义」 |
| 附 | Cargo.toml 三处代号 | 相-1/decisions④/topin | 改自含话术 |

## 复验

- src 全文件残留代号正则扫描：0 命中
- cargo check：默认（gl）/single_thread/geterror 三组合 = **零错**（仅兄弟仓既有 warning，与本设计无关）

## R10 回答（子代理原文）

能开工——每壳自带职责/参数域/失败后状态/不变量、芯伪码可照做；最可能踩的坑=Lost 后的重建回路——已在本轮修复（公开面补「同 Instance/Adapter 重走 attach+request_device，无需重建 Instance」的再入合同），坑填平。