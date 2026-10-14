# 登记分册：变更日志

## 变更日志

| 日期 | 阶段 | 变更 | 影响 |
|---|---|---|---|
| | | 无（尚无回溯与提问记录） | 无 |
| 2026-10-12 | L0 | 第零步落骨架（slug=pi-wgpu-rebuild，重名检测：本工作区无冲突；兄弟仓旧完整设计同名仅参考） | 无 |
| 2026-10-12 | L0 | 本轮提问 5 次（第零步 3：目标/依赖/slug；来源边界 2：设计关系/真相源） | 无 |
| 2026-10-12 | L0 | 回溯修订：原「documents/=唯一真相源、结构照旧仓」下调。用户加判 ——「不依赖原仓结构」+「documents 非绝对正确、不合理可改」。context 来源/素材/待验证三节改写。 | L1 起需求须审读、允许推翻 documents 结论；L5 模块从零定不继承旧分层；L0 事实层保留（不随返工作废） |
| 2026-10-12 | L1 | L1 四册落盘（问题/需求 16 条/流程图/注记）+ 词汇表建行 10；documents 审读修正 3 处 | 下游 L5 引用兼容子集/录制即执行/绑定槽等词汇口径 |
| 2026-10-12 | L1 | 门禁轮提问 2 次（L1 通过 / L7 取舍）；本轮累计提问 8+2 | 无 |
| 2026-10-13 | L1→L5 | L1 行级返工（资产出库判：needs ⑨拆两行/⑩措辞/⑯硬度 + problem OUT 增行）——推翻判=「GPU 资源统一记账+延迟回收在库内」（原按 documents FR-10 立）；改为即删+刷失、复用仅管线级与离屏附件。作废面=needs 相关行+L5 提案重出（未落盘即全废） | L5 修正版落盘；依赖清单减 pi_assets/pi_time；三硬规则合同变硬 |
| 2026-10-13 | L5 | L5 阶段提问 9 轮共 16 次（含门禁与反质询）；最终资产出库版通过 | 无 |
| 2026-10-13 | L5 | L5 四册落定（modules 12+账本/deps 边表+四图+差异矩阵+上传链/change-points 16 行/decisions ①-⑨+验证+来源）。最终门禁=资产出库版通过 | L7 输入：12 模块名、边表、决定⑥⑦⑧⑨ |
| 2026-10-13 | L7 | 相-1 壳芯落 src/（24 壳芯对+types/lib/Cargo）；cargo check --features gl=Finished 零新增（空基线）：引用网自洽实证；L0 待验证结转（registry 在线/依赖可解/nightly 链）当场收敛 ✔。once_cell=1.21.3 钉版；use_wgpu=compile_error 防呆桩相-2 | 壳=冻结面：改签名须回序解冻重落；实现权在芯桩（下游批次） |
| 2026-10-13 | L7 | 落地轮提问 0 次（门禁已批范围执行） | 无 |
| 2026-10-13 | L9 | L9 审卷❌全部就地修复：flow 节点作废/entry+objects 双决定拆分/变更点影响面 6 行改实名调用方/汇合图补 share 4 边+formats 节点/units·call-chain·api@29 名实对账/registry 实证收账/增判⑩ feature 痕迹/壳侧 utils·surface·types·Cargo·entry 违例清除/词汇表作废 1+补 7 行。cargo check 复跑 Finished | 壳体形式已改（签名未动）：utils 委托/Debug 委托/Error 注释化=委托合规非解冻 |
| 2026-10-13 | L9 | 独立子代理轻审：结论=有条件放行（15 判全跑），❌条目全部同场修复+cargo check 复验 Finished；09-review.md 落盘 | 待用户结论确认→tableDone+提交块渲染 |
| 2026-10-13 | L9 | 用户索图→deps 新增「形态子图」三画法（桌面/安卓/Android+Web 路径+逐形态依赖）+call-chain 分叉注+词汇 1；L9 再问结论 | 无 |
| 2026-10-13 | L9 | 用户追问 use_wgpu 侧 PiWgpuAdapter——定形补录⑥批注（整面 re-export+第二 impl 委托芯，官方单参双参差=trait 存在理由），当前锁 compile_error=设计内 | 待落地清单 C 项口径明确 |
| 2026-10-13 | L9 | 目录重组（用户拍板）：src/ 仅留 lib.rs+types/，12 模块收进各自目录（壳=mod.rs、芯=inner.rs，#[path] 声明），编译复验 Finished；07-design 五册壳芯路径全量同步 | 壳芯命名机制不变（xx 壳/xx_inner 芯的 lite 约定，仅文件布局变体） |
| 2026-10-13 | L9 | 芯名遵 lite 约定字面：入收目录内改名为 xx_inner.rs（用户拍板，壳=目录 mod.rs）；lib #[path] 与 5 册路径同步；编译复跑 | 无（声明层更名） |
| 2026-10-13 | L9定稿后补丁 | 用户追问 pi_egl/pi_glow 引用→deps 新增《地基接法注》：glow=单 Context·HasContext 全函数面 cfg 选型+句柄别名清单；egl=create_context/make_current/swap_buffers+platform 目录+gl 桥；模块→API 对位表（attach/let_go/present/get_current/各驱动芯）；context/surface 芯伪码点名真实 API；data Attachment 表示精确化 | 实现批按对位表动手不猜 API；壳签名未动 |
| 2026-10-13 | 定稿后补丁 | 地基门控实核：pi_glow 无 features（native/web=纯 target cfg 同签名）；pi_egl features=swappy/shared-stdcxx/fps 全默认 off+platform{windows,android,web}目录（web 路径判不接入）。deps 增《门控对位表》+变更点 swappy 行；被否：库内对地基加 feature 包装层=第二门控散落 | 相-2 实现批据此接线 cfg，环境分支无第二套 |
| 2026-10-13 | L7 | 作废旧 09 报告 | 09-review.md |
| 2026-10-13 | L7 | 返工(L7 起点·用户确认)：调用链册缺《实现切片·块清单》；壳芯契约注释不足→契约迁入壳注释、md 换指针、芯桩补实现预期伪码；动 26 壳芯文件，编译基线对照 | 作废 09 + 回退阶段；07-design 四册契约格转指针 |
| 2026-10-13 | L7 | 返工落地完成：call-chain 册补《实现切片·块清单》B0–B14；64 处注释补落 15 个壳芯文件（对象表示/查询不失败/参数断言/芯伪码）；壳内 design/ 路径指针清零（lib/types 两处改语义自带）；cargo check 默认+single_thread+geterror 三组合 Finished 零新增 | api 册顶注登记对拍以壳为准；待 L9 复审 |
| 2026-10-13 | L7 | 芯壳文案全量重写（用户判=注释/伪码质量差；门禁确认三口径）：去册内代号（链N/决定⑧/账本⑦/FR-13/结果面①/相-2 等）与指向 design/ 的悬空指针；同句样板拆为逐方法具体不变量+失败后状态；芯伪码一行箭头串升级为编号步块（校验→驱动→镜像改时→失败残留+形态分支）；types 85 项逐条用途/字段域/默认语义；@public 重复清零；伞面/防呆锁注释自含化；落批修复接线（entry/surface import、context 芯 gen_bump 返回补 u32=连带），公开签名未触（仅 context 三个 @internal 函数随注释改名） | 07-design 四册契约格已转指针，壳面即真相源；待 L9 复审 |
| 2026-10-13 | L9 | 独立子代理二复审=有条件放行；8❌+附注全部同场修复（trace 落点/entry 恰一/链号口径/④记号/lost 账目=invalidate_generations 加 cache 参改显式预清+request_adapter 公开再入合同/binds TODO/utils 委托实体移 impl DeviceExt for Device（trait 签名不变）/types has_dynamic_offset 矛盾/Cargo 代号话术）；cargo 三组合零新增复验；09-review.md 落盘 | 待用户结论确认→tableDone+提交块渲染 |
| 2026-10-13 | 收尾 | 经用户确认删除归档件 09-review.2026-10-13.superseded.md（旧轮审卷审计痕迹，用户判无追溯价值；设计已定稿完成） | design/pi-wgpu-rebuild/ 仅存活文件：09-review.md 为定稿版，目录零归档残留 |
| 2026-10-13 | L7 | 壳解冻重落（用户口头批准 A 项，本条后补登记）：Caps 快照增能力位载体字段 _ext_family: u64，struct 四字段全量补注释；attach/probe/fill 签名与全部不变量未动 | 解锁 B2 能力位判定与 B3 formats 直载查位；L9 判卷面含此行 |
| 2026-10-13 | L7 | 落壳口径补充（用户拍板）：每个类型/字段/常量/静态量/接口必须带注释，壳芯同规；已落面回补=context 芯注释全量补齐+Caps struct 逐字段注释 | B3 起各块壳芯注释密度按此判 |
| 2026-10-13 | B1 | 实现批事实修正：WGL 隐窗 DC 属创建线程资源，该线程终止后其他线程再 attach 会被 pi_egl make_current 断言拒（诊断坐实=同线程/存活线程跨绑均通过，唯创建线程死亡后跨绑失败）；跨线程搬运合同的未成文前提=附着创建线程存活 | 测试收拢为单线程 GL 状态机用例解锁 B2；attach 壳注释增补条款提案待门禁拍板 |
| 2026-10-13 | 实现批 | 门禁批准解冻重落：context 壳 attach 注释增补「Windows 搬运前提=创建线程存活」条款（纯注释，签名/委托未动）；data 册 Attachment 表示行同步为已落形制（驻留槽非 Arc 链） | 壳注释即合同权威文本更新；后续块跨线程搬运测试按此前提设计 |
| 2026-10-13 | 实现批 | B3 formats 芯落地（门禁批准 GlParams 五字段展开逐字执行）：7 词汇格式对 GL4.6 规范值转换表内联（glow 根面无 GL_* 枚举导出，沿用参数内联先例）；能力门控经 caps 最近快照缓存（probe 末驻留写·锁外读·未探=断言）；GLES 拒 D24S8 与 sRGB×BGRA 组合（断言报格式名+label）；mip_convert=levels>1 在附着持有期内整链 generate，按层数选 2D/2DArray 目标，零尺寸=断言 | B4 objects 可消费 GlParams 直建；词汇子集外压缩直载=表无条目路径已留（变更点：表加块行） |
| 2026-10-13 | L7 | 作废旧 09 报告 | 09-review.md |
| 2026-10-13 | L7 | 返工（落壳芯·最小解冻，用户门禁=批准解冻免 L9 复审）：objects 壳两行解冻重落（DirectHandle 增 kind 域字段+DirectKind 枚举=mark_dead 分派合同所需；HandleCache 容器按 TODO-DECL 落定 pi_hash::XHashMap/FxHash64）；函数签名/委托/不变量零变动；state 壳未触。芯侧新增=create_direct（entry 链消费）+ 死名刷失驻留面（state 芯 note/is/forget/clear，B5 flush 消费）；glow native 句柄=NonZeroU32 包装，边界转换收芯（壳 GlHandle 保持 u32 原生产物） | B4 验收达成：复用成对合同/归零即删/行覆写/幂等清场实机合验；09 已作废旧报告；免复审直推已完成（用户门禁选择） |
| 2026-10-13 | L7 | B5 state 芯达成（三组合 Finished+12 测试含实机合验）：StateMirror 全组展开+映射原语比较器（types 无 PartialEq=词汇面事实，比较器住芯）；set_* 同值零下发程序组探针实证；flush 两档落定——重要事实修正：GL_CURRENT_PROGRAM 无 glGetIntegerv 合法回读（驱动恒读 0），程序组归入整发档（恒幂等刷期望+死名销记）；采样掩码组=fork glow 无下发面=镜像记录档（B14 差异登记）；clamp 词汇位同理不落地 | B6/B8/B10 消费 set_* 全族就绪；「可回读档/整发档」分档口径成文（回读不可得的组=幂等整发，契约仍成立） |
| 2026-10-13 | L9 | 增量复审补账：独立子代理三审（判 B4/B5 解冻壳面+登记对账）=有条件放行，❌R-1 至 R-9 全部同场修复（壳注释解冻×8 处/芯 flush 整面销记修死名表单调增长/call-chain 链尾指针/data 三旧行收账/词汇 4 建行+旧词注记）；changes 两条作废行判为误读未删（两次独立事件在册）；复验三组合 Finished+12 测试绿。09-review.md 落盘=放行结论 | B4/B5 解冻面判卷闭环；B6 消费 set_* 全族与死名表语义已收账可直接开工 |
| 2026-10-14 | 实现批 | B6 binds 芯落地（门禁批准 A/B/C 增量+D 边界）：alloc=布局条目分域紧凑编号（槽键=pack(set,binding)；满16=断言且不半写）；apply=预检（句柄活态/槽齐备/动态计数相符，动作前）+逐条经状态镜像差异下发（动态偏移按声明序叠加静态偏移）；consume=计数校验+游标前移（不符不消耗任何份额）。接口增量（提案→批准→ddt 簿记）：binds 壳 SlotTable 值语义定案+新增 BindDomain/BoundEntry+apply_layout 加 set/table/state 借用；entry BindGroup._held→_bound（条目载偏移/尺寸/动态位+资源强引用）；state ubo 槽值→(名,偏移,尺寸)三元（偏移/尺寸变亦重绑，防漏发）。验收=芯内 alloc/consume 用例3枚+统一 GL 测试 binds 段（apply→镜像落槽+死句柄动作前断言）；15 测试全绿+三组合 Finished。设备接线（RenderPass/编码器→Device、Device 内部可重入）登记为 B10/B11 欠账。 | B10 draw 消费 apply_layout（需补设备侧借用传递链）；B7 反射批收口 (set,binding)→block/单元的程序侧绑定与纹理×采样器配对；槽表暂设备级单表（多管线切换正确性=已登记限制）。 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 返工（实现驱动）：B6 增补改了表示面（BindGroup 载体/apply_layout 签名/SlotTable 值语义/state UBO 槽值类型），07-design 表示层随动——data.md 改 SlotTable 行值语义+增 BoundEntry/state UBO 槽值 两行；call-chain.md B6 行改预检口径并把「反射表对接」更正为留 B7（pipe 仅保留位）。返工起点=L7（07-design 面）。 | 07-design 表示层与 B6 壳芯一致；旧 09 作废待重跑 |
| 2026-10-14 | L9 | L7 返工轮 L9 复审（子代理独立判卷）：有条件放行→5 ❌ 同场收口——data SlotTable 改双段定义、deps 补 6 边并改环检判据、glossary 改绑定槽+补 3 行、api/errors 修 draw first_instance 矛盾、objects 壳去 design 悬空指针、api 内部清单补 state set_*/bind_* 族；09-review.md 落盘=放行；cargo check 复验绿。 | 07-design/05-modules/index 与 B6 壳芯全一致，设计基线可提交 |
| 2026-10-14 | 实现批 | B7 shader 芯落地（门禁批准 A 据 pi_render 线索）：compile_impl=WGSL 前端解析（错误 emit_to_string 断言不吞）/GLSL 逐段前端解析 → 校验取 ModuleInfo → 产物持 IR（复用键=源文本哈希）；register_impl=遍历 global_variables 收 (set,binding)↔BindingKind/名（去重）→ 与布局逐条对表（源缺/多绑定=断言）→ 落 ReflectionTable；emit_stage_impl=按 (stage,entry_point) 选模块 → BindingMap 按布局槽表定位 → glsl-out 出 GLSL ES 3.0。接口增量（批准）：CompiledEs300{hash,modules:Vec<ShaderIr>}、ShaderIr{module,info}、BindingKind/RefEntry、ReflectionTable{entries}、新芯 emit_stage、ShaderModule 加 _compiled；Cargo naga 加 wgsl-in feature。验收=芯内 3 用例（转译+出码+语法错断言+反射对表）；17 测试全绿+三组合 Finished。 | B11 管线创建消费 emit_stage（按 stage/entry_point 出码）+ ShaderModule._refl 反射；B6 pipe 程序侧 block→slot 绑定可由反射条目收口。 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 返工（B7 表示层同步 + 新增错误处理规则）：①07-design 随 B7 接口增量同步——api/data/units/call-chain 补 emit_stage 与产物形态（CompiledEs300/ShaderIr/ReflectionTable/RefEntry/BindingKind，ShaderModule 持 _compiled），deps 外部表 naga 补 wgsl-in；②新增决定⑪「错误处理」记入 decisions+errors+项目 AGENTS.md：优先上递错误码、避免原地 panic、node log::error!、错误信息=短码禁长文本、断言仅留编程合同违反；③B7 芯按其改造：shader 失败=短错误码 ShaderErr 上递（ParseWgsl/ParseGlsl/Validate/Emit/NoEntry/SlotMissing/LayoutMismatch/EmptyGlsl/UnsupportedBinding），诊断原文走 log::error!；壳签名 → Result。复验三组合 Finished+17 测试绿（含 GL）。 | B8 起从严错误处理；B1–B6 断言面登记待清理；B11 create_shader_module 消费 Result。 |
| 2026-10-14 | L9 | B7 返工轮 L9 复审（子代理）：有条件放行→5 ❌ 同场收口——entry 壳/芯口径改错误码降级、call-chain 两行+节点、shader 壳失败后状态、errors 三处+边界、glossary 6 行+壳芯定义；附注 data 去重/B13 前依赖/status。09-review.md 落盘=放行；复验绿。 | 设计与 B7 壳芯一致；错误规则落定（决定⑪） |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 返工（B8 表示层同步）：07-design 随 B8 接口增量同步——data 资源句柄行补 Buffer.size/Texture.params + 增 UploadErr 行；api Queue::write_buffer/write_texture 契约改错误码；errors upload 行改两档错误码；call-chain 写链改错误码；deps 补 entry→formats 边（Texture 持 GlParams）+ 图同步；glossary 补 UploadErr。 | 设计表示层与 B8 壳芯一致；B9 surface 可开工。 |
| 2026-10-14 | L9 | B8 返工轮 L9 复审（子代理）：有条件放行→4 ❌ 同场收口——errors/api 的 JS 直传口径改错误码、entry 两字段补注释、upload 矩形校验收窄登记；附注 notes.md 损坏已复写（全目录扫无同类污染）、api 清单补 with_gl/generation_now。09-review.md=放行；20 测试绿。 | B8 表示层与壳芯一致 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 返工（B9 表示层同步）：07-design 随 B9 接口增量同步——data 增 Surface 行（_cfg RefCell/平台表面驻附着槽/帧视图标记）；api configure/get_current 契约改错误码+未配置口径；errors configure 行改错误码；units Presenter 补建面/换帧；glossary 补 SurfaceErr。 | 设计表示层与 B9 壳芯一致；B10 draw 可开工。 |
| 2026-10-14 | L9 | B9 返工轮 L9 复审（子代理）：有条件放行→3 ❌ 同场收口——glossary 补 SurfaceOpErr、SurfaceErr 去死变体并分工 SurfaceErr/SurfaceOpErr、api/data/errors 补登并改壳注释码一致；附注 call-chain B9 行/deps 共用闸注/lib.rs 话术。09-review.md=放行；20 测试绿。 | B9 表示层与壳芯一致 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 小返工（图示补细节）：call-chain 链1 加 surface_attach（建平台表面+make_current）与 create_render_pipeline（emit_stage→reflection_register→ensure_reuse）；链2 加 surface_swap（换帧/Web 空操作）。图随表（链节点结构未变，仅补函数级步骤）。 | 图示与 B7/B9 实现对齐；图已原文贴入会话。 |
| 2026-10-14 | L9 | 图示补细节轮 L9 复审（子代理）：有条件放行→3 ❌ 同场收口——链1 拆 shader/管线两步并对齐源行、链2 surface_swap 归属 context（加 participant C）；附注链2 挂点号/ensure_program 实名/壳注释去 design 内号（决定⑪→错误纪律，9 文件）。09-review.md=放行；20 测试绿。 | 图示与源表/B7·B9 实现一致 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | 返工（B10 表示层同步）：data 更新 Instance/Adapter/Device/Queue/CommandEncoder 行（Device=Share<DeviceInner{三字段 ShareMutex}>）+ 新增 DeviceInner 行 + PassCtx→RenderPass（通道态字段）；units PassRecorder 补属性/离屏差异；call-chain 绑定应用行标 apply_layout 无 pipe。设备接线=路线 A（据 pi_render/wgpu: 编码器持共享设备柄）。 | 设计表示层与 B10 壳芯一致；B11 entry 编排可开工。 |
| 2026-10-14 | L9 | B10 返工轮 L9 复审（子代理）：有条件放行→9 ❌ 同场收口——索引格式/偏移载体、begin/end 契约收窄、芯册号自含、恒真断言删、哑消费修、VAO 走复用+镜像、视口双口径统一、deps 补 3 边、glossary SurfaceErr 死变体清；附注 errors 链号+描述符断言口径。09-review.md=放行；20 测试绿。 | B10 表示层与壳芯一致 |
| 2026-10-14 | L7 | 决策⑫：DirectHandle 改共享计数（Share）；B11 整合暴露持有型句柄与 create_bind_group(&Buffer) 冲突。据 pi_render 强引用线索（Share<wgpu::BindGroup>/Arc<RWBufferRange>）落共享化；改 objects 壳+芯、draw/binds/upload 取用点、测试占位构造 placeholder；data 册 4 行同步（资源句柄/复用对象/对象句柄层/BoundEntry）。 | 对象句柄层所有权语义 + 绑定组资源存续 |
| 2026-10-14 | L7 | B11 entry 芯落地：全编排（new/create_surface/request_adapter/request_device + Adapter/Device 查询族 + create_* 全族 + reset_state/finish/encoder）。实现选择（待 L9 复核/登记）：①程序编译+链接（create_shader/create_shader→shader_source→compile_shader→attach_shader→link_program）内联 entry，新程序（refcount==1）才链；②create_texture 内联 tex_storage_2d 整链分配（绑 GL_TEXTURE_2D 落当前 active 单元，未过镜像——待复核）；③device_features/limits 现探 caps（DeviceInner 未存创建快照——登记差异）；④adapter_surface_supported 按附着代号比对。测试 20 绿。 | entry 编排面全量接线；三条显式 Result 面语义成立 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L9 | B11 返工轮 L9 复审（子代理）：有条件放行→9 ❌ 同场收口——反射步归属改链图/时序（compile_to_es300 于模块创建、emit_stage 于管线创建，reflection_register 标待接线）、create_bg/bgl/pl 补壳声明校验（资源类型×布局对表+活态+binding 域+成员去重）、create_sampler 壳注释收窄、create_rp 空产物口径统一（错误纪律边界补 errors）、去 entry 芯内号、glossary 补共享计数/DirectHandle、data 死用口径+差异注、mark_dead 共享前提。09-review.md=放行；20 测试绿。 | B11 表示层与壳芯一致；决策⑫入库 |
| 2026-10-14 | 返工 | 作废旧 09 报告 | 09-review.md |
| 2026-10-14 | L7 | B12 utils 芯落地：create_buffer_init（create_buffer+write_buffer，Queue 由共享内芯构造）/ create_texture_with_data（create_texture+write_texture，mip0 直写）/ env 助手（backend_bits 恒 Some(GL)、power_preference_from_env 映射 WGPU_POWER_PREF、parse_backends 逗号表仅 GL）。纯组合面零新契约；order 多段排布差异登记（当前单段直写）。api 行 27 DeviceExt 已覆盖。测试 20 绿。 | 便捷面与直调等价；env 恒 GL 语义（WGPU_BACKEND 不生效） |
| 2026-10-14 | L9 | B12 返工轮 L9 复审（子代理）：有条件放行→3 ❌ 同场收口——壳/api 登记 order 未实现+`_order`、modules/call-chain 标「锁包装+trace 单点=待落（登记差异）」、call-chain 便捷面行补 create_texture_with_data + 闭包注加 3 env 助手、backend_bits 壳措辞收窄恒 Some(GL)。09-review.md=放行；20 测试绿。 | B12 表示层与壳芯一致；组合面零新契约 |
