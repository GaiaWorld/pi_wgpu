# 登记分册：变更日志

<!-- ddt 的 tableLog / tableAddRow 负责追行。只记「什么被改成什么、为什么」，供事后审计与返工锚定。 -->

## 变更日志

| 日期 | 阶段 | 变更 | 影响 |
|---|---|---|---|
| 2026-10-15 | L0 | 第零步骨架落册；L0 地形采集完成（上游 wgpu 25.0.2 对账基准确认） | 无 |
| 2026-10-15 | L1 | L1 四册落盘+词汇 3 行；门禁轮提问 3 次（L1 通过/L5 跳过/L7 精简+壳芯），累计本轮 6 次 | L7 按精简档+落壳芯推进；L5 不落册 |
| 2026-10-15 | L7 | 落地门禁过（死变体不抄/entry None 违约/cache 无构造占位）；S1–S4 壳芯落地；PS 编码事故 git 还原重放；check --all-targets 零错误、test --lib 19 通过；07-design/call-chain.md 落册 | examples/契约/词汇全换形；实现批承接 5 项 TODO-DECL |
| 2026-10-15 | L9 | 独立子代理轻审=有条件放行；回改 R5/R8/R12/R16+A1/A2#3/A3-2（flow 判序、词汇补行、GLSL 两测试、切片粒度声明、derive 差异登记、request_device 补对照、断言消息补参）；复验 21 passed/0 failed、all-targets 零错误；09-review.md 落册 | 结论升级为放行（运行回归交用户） |
| 2026-10-15 | 实现批 | 用户批准直做实现（不走 dev-impl）：entry_point=None 自动选（pick_entry 唯一采纳/缺失歧义 NoEntry）、defines 注入 naga 预处理、ShaderStage→naga From 收敛、emit_stage 面转 Option 透传；补 4 测试=24 passed；三例带窗冒烟过（存活4s无崩溃，clear 帧正常） | 壳注释三处同步 entry_point 语义；遗留缩至 3 项（错误码通道/Features 位表/derive 全量对账） |
| 2026-10-15 | 追加批 | BufferSize 对齐上游废包装=type alias NonZeroU64（get/构造式原样兼容，无构造点零破）；TextureFormat 全表 75 变体照抄 25.0.2 入面（名/序/Hash，无 Astc-以 checkout 为准）：形宽法窄=转换表仅收录已接 GL 映射 7 项、表外恒拒（untabled_format_rejects_test），Depth24UnormS8Uint 改名 Depth24PlusStencil8（6 处引用同步）；TODO-DECL 词汇挂账退役 | 25 passed/0 failed，all-targets 零错误；压缩族接表归后续能力批 |
| 2026-10-15 | 追加批2 | TextureViewDescriptor 九字段照抄 25.0.2（format/dimension/usage Option 化、level_count→mip_level_count、NonZero→u32 域、+Default 闭合取整张便捷构造，core=None 推定维度）+ DeviceDescriptor +Default derive + Limits 36 字段全表（名/序/derive 照抄；Default=静态保底表：尺寸/绑定取 spec 保底与槽表容量 16，未实现面 0 不虚报；自定义旧字段 max_uniform_buffer_bindings 退役对位 per_stage=16）；request_device 对照扩 over_limits 全 36 项；样例 view 换 Default 式 | 公开形变（view 构造式换代）—examples 已适配；25 tests green、all-targets 零错误；map_label 泛型投影登记不抄 |
| 2026-10-15 | trace批 | trace 面按用户拍板采官方历史 Trace 枚举形（Off/Directory(PathBuf)，non_exhaustive+自有所有权注记，cfg(trace) 门不移植=恒拒止目录项）：request_device 参数 Option<&Path>→Trace；公开壳同步改 RPITIT（-> impl Future，WasmNotSend 登记不抄）+ desc 生命周期解耦 25.0.2；拒止用 if/panic（assert!+matches! 消息求值拷贝 PathBuf 触发 catch_unwind×全局分配器死锁，事故在案注记芯注释）；双轨 facade 同形改：not 侧走 types::Trace、uw 侧 lib.rs 立同形 pub Trace（互斥轨各一名）+ lib_inner 拒止同判；顺带发现 TextureDescriptor.dim→dimension 名实已换（上游正名）、formats_inner/样例两残留点续改（含 write_texture 布局按值形——上游 25 签名按值） | 公开面 request_device 双参形变（样例 None→Trace::Off）；25 tests green×2 轮（1 轮 flake=inject_latest 全局快照互污既存债，新测扩注面、serial 化挂账实现批）；双 cargo check 轨全绿 |
| 2026-10-15 | 绘制批 | BufferSlice/绘制面/通道附件面强拉齐 25.0.2：BufferSlice{buffer,offset,size:Option<BufferSize>}+Buffer::slice&二次 slice（RangeBounds+slice_bounds_impl 界校验 ErrSlice；map 族登记不抄）；set_vertex_buffer(slot, BufferSlice)（步长由管线布局权威、原比对随形退役）set_index_buffer(BufferSlice, format)（区间记 _index_range，范围校验移 draw 期）；draw(vertices:Range, instances:Range) draw_indexed(indices:Range, base_vertex, instances:Range)（instances 恒 0..1=ErrInstances 断言，base≠0 照旧）；Operations<T>/LoadOp<T> 泛型化（Color<f32><u32> 三参实例）；RenderPassDescriptor +Default +timestamp_writes(ErrQuery 拒)+occlusion_query_set(拒)+QuerySet 无构造占位（PipelineCache 同式）+RenderPassTimestampWrites 词位；Buffer 手落 PartialEq（句柄名+长度，上游投影）；样例三处换形制（vbuf.slice(..)/0..n/降双 None）；context 自动通道补 None | 公开绘制面破坏形（调用方需 Range+slice）——examples/tests 全适配；25 lib tests、双 cargo 轨、三例冒烟稳；可见性目检留用户 |
| 2026-10-15 | trace修正批 | trace 词位改居 DeviceDescriptor.trace（Trace 词汇在场、Directory=断言 ErrTrace 拒止，借用模式修复）；request_device 接口参数恢复历史双参 trace_path: Option<&std::path::Path>（Some=debug 记录后忽略、永不失败；公开面+双 facad 轨+lib_inner 同复）；uw 轨 lib-local Trace 枚举保留为公开词位；三例 desc 补 trace: Trace::Off + 双参 None | 调用形回摆=历史兼容面对表恢复；25 tests、双 cargo 轨、三例 ALIVE 冒烟全绿 |
