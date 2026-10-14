# pi-wgpu-rebuild · 数据结构与不变量（L7）

不变量与所有权住壳类型注释；本册留指针+表示理由。

| 结构 | 壳 | 表示选择 / 被否方案 |
|---|---|---|
| Instance/Adapter/Device/Queue | src/entry/mod.rs | Share 引用链（pi_share）；被否 Rc（线程合同破） |
| 资源句柄对象（1:1：Buffer/Texture/Sampler/TextureView） | src/objects/mod.rs | Option<Handle>（删=置 None，死用=断言）；活态 ⟺ 上下文代号=当前 |
| 复用对象（程序/VAO/FBO/RBO） | src/objects/mod.rs | 表示行合入本册「对象句柄层」行（计数真相=芯全局驻留表、行=物理名模板）；被否 Arc+refcount 与手动 Option 口径不变 |
| BindGroup/BindGroupLayout/PipelineLayout/RenderPipeline/ShaderModule | src/entry/mod.rs | 描述符解析后持强链（三硬规则库内形态=引用图+使用方合同）；ShaderModule 另持转译产物 _compiled |
| 着色器产物 CompiledEs300/ShaderIr | src/shader/mod.rs | CompiledEs300{hash, modules:Vec<ShaderIr{module,info}>}；hash=源文本哈希（复用键）；GLSL 出码延到管线创建 |
| 反射登记表 ReflectionTable | src/shader/mod.rs | entries:Vec<RefEntry{group,binding,kind:BindingKind{UniformBuffer,Texture,Sampler},name}>；与程序转译哈希同生命周期 |
| SurfaceTexture（帧租约） | src/surface/mod.rs | present 消自——同时至多一帧的不变量类型级成立 |
| 附着体 Attachment | src/context/mod.rs | 芯内进程驻留附着槽：EGL/WGL 实例对（glow 入口由实例借出）+ 持锁线程重入门锁 + TLS 层数 + 代号原子（非 Arc 引用链）；Web 形态零 EGL、GL 入口由呈现配置链注入；代号单调增；跨线程搬运前提见壳 attach 注释条款 |
| 对象句柄层 | src/objects/mod.rs | L7 解冻重落定：GlHandle=u32 原生产物（glow native NonZeroU32 包装由芯边界转换）；DirectHandle 带 DirectKind 域（三路删除分派）；HandleCache=pi_hash XHashMap/FxHash64，行持物理名模板；计数真相与死名刷失驻留 objects/state 芯——归零即删+登记，invalidate 纯记账幂等 |
| 状态镜像 StateMirror | src/state/mod.rs | L7 解冻展齐：全驱动分组字段；值类型词汇无 PartialEq→比较器经映射原语拆量住芯；被否 dirty 位图（散落易漏）；两档重 sync=可回读档比对+不可回读组（GL 程序.current/WebGL 句柄绑定/UBO 槽/fork 掩码面）幂等整发+整面销记；对象死=死名登记强发 |

| 绑定槽表 SlotTable | src/binds/mod.rs | 双段定长数组 ubo[Option<u32>;16]+单元段[Option<u32>;16]（域映射 Uniform→ubo 段、Texture/Sampler→共用单元段）；索引=GL 绑定点/单元号，值=占槽 (set,binding) 打包键（set 高 16 位/binding 低 16 位）；设备级单表、建管线时按该布局重排（多管线切换正确性=已登记限制）；被否键值表（热路径）；满=断言 |
| 绑定组条目 BoundEntry | src/binds/mod.rs | 声明序条目集：binding+domain(Uniform/Texture/Sampler)+资源强引用 DirectHandle+offset/size/dynamic；动态声明数=dynamic 位计数（consume 校验源） |
| state UBO 槽值 | src/state/mod.rs | 各 UBO 绑定点=(名,偏移,尺寸) 三元——偏移/尺寸变亦必须重绑（键含二者，防换偏移漏发） |
| 通道上下文 PassCtx | src/draw/mod.rs | 借编码器+锁守卫（生命周期=Drop 语义 Rust 强制） |
| 描述符词汇结构 | src/types/mod.rs | types 自建移植（决定③）；无运行态；合法性 entry 边界一次校验（隔离带） |
| SurfaceConfiguration 缓存 | src/surface/mod.rs | 宽/高非零断言；格式表来自 caps |
| 错误枚举 | src/entry/mod.rs | thiserror；封闭三处+其余类型上不可失败 |

持久化/迁移：无磁盘结构；跨版本词汇对表走「显式登记断裂」于迁移清单，不静默。
