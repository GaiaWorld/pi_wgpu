# pi-wgpu-rebuild · 数据结构与不变量（L7）

不变量与所有权住壳类型注释；本册留指针+表示理由。

| 结构 | 壳 | 表示选择 / 被否方案 |
|---|---|---|
| Instance/Adapter/Device/Queue/CommandEncoder | src/entry/mod.rs | 引用链=Share：Device{_inner:Share<DeviceInner{state/cache/slots:ShareMutex}>}；CommandEncoder 持 _inner（克隆共享）；被否 Rc（线程合同破）；登记差异(B11)：DeviceInner 未存能力快照——device_features/limits 现探 caps；create_texture 创建期 tex_storage_2d 分配（绑 GL_TEXTURE_2D 未过镜像）；原「被否 Arc+refcount」条为决定⑫ 反转 |
| 设备共享内芯 DeviceInner | src/entry/mod.rs | 三字段 state/cache/slots 各 ShareMutex；锁序=state 先、with_gl 后（与 set_*/flush 同序） |
| 资源句柄对象（1:1：Buffer/Texture/Sampler/TextureView） | src/objects/mod.rs | DirectHandle=共享计数（Share<DirectHandleState>，Clone 共享驱动名、末引用 Drop 才删，决定⑫）；name()→Option<GlHandle>（死用=binds 断言 / upload 错误码）；活态 ⟺ 上下文代号=当前；Buffer 另持 size、Texture 另持 GlParams + 目标/维度/层数（_target/_dim/_layers，创建期定格；upload 与视图相容性消费） |
| 上传错误码 UploadErr | src/upload/mod.rs | DeadHandle/Misaligned/RowPitch/OutOfRange/NotBitmap/OffMainThread；公开非 Result 面=log 短码（决定⑪） |
| 复用对象（程序/VAO/FBO/RBO） | src/objects/mod.rs | 表示行合入本册「对象句柄层」行（计数真相=芯全局驻留表、行=物理名模板）；1:1 句柄已改共享计数（决定⑫），本类复用对象仍走芯全局计数表（非 Share）；VAO 生命周期=**管线级**（随 RenderPipeline 建/删，每 pass 不再建删）；通道 `RenderPass._vao` 只记名不持有 |
| BindGroup/BindGroupLayout/PipelineLayout/RenderPipeline/ShaderModule | src/entry/mod.rs | 描述符解析后持强链（三硬规则库内形态=引用图+使用方合同）；ShaderModule 另持转译产物 _compiled；RenderPipeline 另持 _vao:Reusable（管线级复用对象），Drop 释放 _program/_vao |
| 着色器产物 CompiledEs300/ShaderIr | src/shader/mod.rs | CompiledEs300{hash, modules:Vec<ShaderIr{module,info}>}；hash=源文本哈希（复用键）；GLSL 出码延到管线创建 |
| 纹理视图 TextureView | src/entry/mod.rs | {_tex:Option<Texture>, _target:u32, _dim:TextureViewDimension}；视图目标按维度覆盖；帧视图=无宿主且目标 0；经 Texture::create_view 构造（纹理切片） |
| 反射登记表 ReflectionTable | src/shader/mod.rs | entries:Vec<RefEntry{group,binding,kind:BindingKind{UniformBuffer,Texture,Sampler},name}>；与程序转译哈希同生命周期 |
| 呈现表面 Surface | src/surface/mod.rs | _cfg=RefCell<Option<SurfaceConfiguration>>（configure 取 &self）；平台表面驻 context 附着槽（Attached.surface）；帧视图=默认帧缓冲标记 TextureView{_tex:None}；suboptimal 恒真 |
| 表面操作错误码 SurfaceOpErr | src/context/context_inner.rs | NotAttached/CreateSurface/NoSurface（surface_attach/surface_swap 上递，surface 芯落 log 短码，决定⑪） |
| SurfaceTexture（帧租约） | src/surface/mod.rs | present 消自——同时至多一帧的不变量类型级成立；view()=帧视图取值（examples 片） |
| 附着体 Attachment | src/context/mod.rs | 芯内进程驻留附着槽：EGL/WGL 实例对（glow 入口由实例借出）+ 持锁线程重入门锁 + TLS 层数 + 代号原子（非 Arc 引用链）；Web 形态零 EGL、GL 入口由呈现配置链注入；代号单调增；跨线程搬运前提见壳 attach 注释条款 |
| 对象句柄层 | src/objects/mod.rs | L7 解冻重落定：GlHandle=u32 原生产物（glow native NonZeroU32 包装由芯边界转换）；DirectHandle=Share<DirectHandleState{kind, handle:Cell<Option<GlHandle>>, generation}>（决定⑫ 共享计数：Clone 共享名、末引用 Drop 三路删除分派+死名登记）；HandleCache=pi_hash XHashMap/FxHash64，行持物理名模板；计数真相与死名刷失驻留 objects/state 芯——归零即删+登记，invalidate 纯记账幂等 |
| 状态镜像 StateMirror | src/state/mod.rs | L7 解冻展齐：全驱动分组字段；值类型词汇无 PartialEq→比较器经映射原语拆量住芯；被否 dirty 位图（散落易漏）；两档重 sync=可回读档比对+不可回读组（GL 程序.current/WebGL 句柄绑定/UBO 槽/fork 掩码面）按已下发水位比对增量发（决定⑬：+water 字段=非回读组「上次实际下发」原语快照，住 state 芯 Water；水位空/死名命中=整组强发）+整面销记；决定⑭：+readback_frame 帧门（可回读档每帧一次回读，present 推进帧号）；对象死=死名登记强发 |

| 绑定槽表 SlotTable | src/binds/mod.rs | 双段定长数组 ubo[Option<u32>;16]+单元段[Option<u32>;16]（域映射 Uniform→ubo 段、Texture/Sampler→共用单元段）；索引=GL 绑定点/单元号，值=占槽 (set,binding) 打包键（set 高 16 位/binding 低 16 位）；设备级单表、建管线时按该布局重排（多管线切换正确性=已登记限制）；被否键值表（热路径）；满=断言 |
| 绑定组条目 BoundEntry | src/binds/mod.rs | 声明序条目集：binding+domain(Uniform/Texture/Sampler)+资源强引用 DirectHandle（共享计数 clone，决定⑫：组持副本不删用户资源）+offset/size/dynamic+target（纹理条目=自 TextureView 的 GL 绑定目标；非纹理=0）；动态声明数=dynamic 位计数（consume 校验源） |
| state UBO 槽值 | src/state/mod.rs | 各 UBO 绑定点=(名,偏移,尺寸) 三元——偏移/尺寸变亦必须重绑（键含二者，防换偏移漏发） |
| 渲染通道 RenderPass | src/draw/mod.rs | 借编码器（_enc，通道锁生命周期）+ 通道态（_vertex 布局拷贝/_pipe_set/_vertex_ready/_index_ready/_topology）；Drop=end 释放守卫 |
| 描述符词汇结构 | src/types/mod.rs | types 自建移植（决定③）；无运行态；合法性 entry 边界一次校验（隔离带） |
| SurfaceConfiguration 缓存 | src/surface/mod.rs | 宽/高非零断言；格式表来自 caps |
| 错误枚举 | src/entry/mod.rs | thiserror；封闭三处+其余类型上不可失败 |

持久化/迁移：无磁盘结构；跨版本词汇对表走「显式登记断裂」于迁移清单，不静默。
