# L9 轻审报告（design-lite · L7 返工轮：B6 表示面同步）

**结论**：☑ 放行（子代理原判=有条件放行，5 ❌ 全部同场收口）

> 子代理=无对话历史、只读、按 R1–R16 逐条判；返工对象=07-design 表示层随 B6 增补（BindGroup 载体 / apply 签名 / SlotTable 值语义 / state UBO 槽值）同步。

## 内容核对（子代理原判）

| 检查 | 结果 |
|---|---|
| R1 需求够清楚 | ✅ |
| R2 问题与范围 | ✅ |
| R3 需求↔模块对得上 | ✅ |
| R4 模块切得对 | ✅ |
| R5 图与文一致 | ❌→修 |
| R6 依赖无环、无越级 | ❌→修 |
| R7 变更点三项齐 | ✅ |
| R8 名词有定义 | ❌→修 |
| R9 边界不越 | ✅ |
| R10 能照此开工 | ✅ |
| R11 决定有理由 | ✅ |
| R12 假设已验证 | ✅ |
| R13 无过度工程 | ✅ |
| R14 详细设计能落地 | ❌→修 |
| R15 详细设计不越界 | ✅ |
| R16 实现切片可照做 | ✅ |

## ❌ 明细与收口对账

| 条目 | 位置 | 问题 | 回哪份文档 | 已改 |
|---|---|---|---|---|
| R5 | 07-design/data.md SlotTable 行 | 单表单段描述，未现 ubo/单元双段与 Texture·Sampler 共用单元段；「每管线 reorder」与「设备级单表」相抵 | data.md | 改双段定义+域映射+设备级单表/重排限制 |
| R5/R6 | 05-modules/deps.md 边表+图 | 缺 binds→{state,entry,objects}、entry→state、objects→state、formats→caps、shader→entry；「环检通过」不成立 | deps.md | 补 6 条边（表+图同步）+ 改判据为「无功能环+无越级，单 crate 类型互引不计环」并列互引对 |
| R8 | index/glossary.md | 「绑定槽」定义未反映单元段共用；缺 BindDomain/BoundEntry/SlotTable 定义 | glossary.md | tableSetCell 改「绑定槽」+ tableAddRow 补 3 行 |
| R14 | 07-design/api.md draw 行 + errors.md | 写 draw「first_instance=0/base_vertex=0 两零参」，与 draw 壳「first_instance 类型面不存在」矛盾 | api.md、errors.md | api 改「draw 无 first_instance 形参；draw_indexed base_vertex=0」；errors 改「单零参」 |
| R14 | src/objects/mod.rs 壳注释 | 「见 data 册对象句柄层表示行」=指向 design/ 的悬空指针，违壳注释自足 | src/objects/mod.rs | 就地写全 Web 包装说明，删指针 |
| R14 | 07-design/api.md 内部操作清单 | state 芯 set_*/bind_* 族未登记（binds/draw 经芯互访） | api.md | 清单补「state 芯 set_*/bind_* 差异下发族」条目 |

## 复验

- cargo check（--features gl）= EXIT 0，无新增错误（objects 壳注释改动后）。

## 说明（不计 ❌）

- 实现批 B1–B8 芯已含逻辑体（非纯桩）=实现期正常演进，设计册未越界，R15 判 ✅。
