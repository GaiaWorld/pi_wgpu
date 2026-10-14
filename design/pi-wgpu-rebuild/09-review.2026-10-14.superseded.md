# L9 轻审报告（design-lite · L7 增量：纹理切片 3D/cube）

**结论**：☑ 放行（子代理原判=有条件放行，4 ❌ + 附注全部同场收口）

> 子代理=无对话历史、只读、R1–R16 逐条判；wgpu 假设经实核（wgpu-hal get_info_from_desc:437 / is_cube_compatible:1879）。新需求=3D/cube 纹理；范围按 pi_render 用法定为全维度同族。

| 检查 | 结果 |
|---|---|
| R1–R4,R6,R7,R9–R13,R15 | ✅ |
| R5 图与文一致 | ✅（附注 deps entry→formats 可补「+目标」） |
| R8 名词有定义 | ❌→修 |
| R14 详细设计能落地 | ❌→修 |
| R16 实现切片可照做 | ❌→修 |

## ❌ 明细与收口

| 条目 | 位置 | 问题 | 收口 |
|---|---|---|---|
| R14 | api.md:15 | 存储口径错（写 CUBE=tex_storage_3d；实为 CUBE_MAP→tex_storage_2d） | 改「2D·CUBE=tex_storage_2d / 3D·2D_ARRAY·CUBE_ARRAY=tex_storage_3d」（代码镜像 wgpu-hal 正确，不改代码） |
| R14 | data.md BoundEntry 行 | 缺新增 target 字段 | 行内补 `+target` |
| R8 | glossary:38 + 缺行 | BoundEntry 缺 target；TextureView 无词汇行 | BoundEntry 补 target；新增 TextureView 行 |
| R16 | call-chain 块清单 | 止于 B14，无纹理切片块 | 新增 B15 纹理切片行（前置/验收/触及壳齐） |

## 附注收口

- create_view 维度约束收紧：Cube 限 `层数==6`（原 %6==0 会让 12 层纹理映出 CUBE_MAP 目标视图）；CubeArray 仍 %6==0。
- upload 芯注释随目标路由更新；JS 位图路径=恒 2D（不参与路由）。
- types-map §1 行区分两套维度名（TextureDimension vs TextureViewDimension）。
- errors.md 边界补 create_texture/create_view 维度校验。
- deps entry→formats 行「+目标」= 附注（非阻断，可在挂接件补）。
- R15 历史 B 号散见（state/lib/context/caps/objects 注释）非本轮引入，留待统一清理。

## 复验

- cargo check --features gl = Finished；cargo test --features gl --lib -- --include-ignored = 20 passed。

## 说明（登记差异）

- cube 判定镜像 wgpu is_cube_compatible；D2×6 与 2D-array×6 不可兼得（GL 单目标，wgpu 同）。