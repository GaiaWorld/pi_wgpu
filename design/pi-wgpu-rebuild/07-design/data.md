# pi-wgpu-rebuild · 数据结构与不变量（L7）

不变量与所有权住壳类型注释；本册留指针+表示理由。

| 结构 | 壳 | 表示选择 / 被否方案 |
|---|---|---|
| Instance/Adapter/Device/Queue | src/entry/mod.rs | Share 引用链（pi_share）；被否 Rc（线程合同破） |
| 资源句柄对象（1:1：Buffer/Texture/Sampler/TextureView） | src/objects/mod.rs | Option<Handle>（删=置 None，死用=断言）；活态 ⟺ 上下文代号=当前 |
| 复用对象（程序/VAO/FBO/RBO） | src/objects/mod.rs | Arc+refcount（HandleCache 内）；归零=物理删+镜像刷失；被否手动 Option（违约面大） |
| BindGroup/BindGroupLayout/PipelineLayout/RenderPipeline/ShaderModule | src/entry/mod.rs | 描述符解析后持强链（三硬规则库内形态=引用图+使用方合同） |
| SurfaceTexture（帧租约） | src/surface/mod.rs | present 消自——同时至多一帧的不变量类型级成立 |
| 附着体 Attachment | src/context/mod.rs | 芯内进程驻留附着槽：EGL/WGL 实例对（glow 入口由实例借出）+ 持锁线程重入门锁 + TLS 层数 + 代号原子（非 Arc 引用链）；Web 形态零 EGL、GL 入口由呈现配置链注入；代号单调增；跨线程搬运前提见壳 attach 注释条款 |
| 对象句柄层 | src/objects/mod.rs | L7 解冻重落定：GlHandle=u32 原生产物（glow native NonZeroU32 包装由芯边界转换）；DirectHandle 带 DirectKind 域（三路删除分派）；HandleCache=pi_hash XHashMap/FxHash64，行持物理名模板；计数真相与死名刷失驻留 objects/state 芯——归零即删+登记，invalidate 纯记账幂等 |
| 状态镜像 StateMirror | src/state/mod.rs | L7 解冻展齐：全驱动分组字段（程序/属性/目标/上传/UBO·纹理·采样器×16 槽/视口·裁剪/开关函数族/模板两面/偏置/掩码记录/代号）；值类型词汇无 PartialEq→比较器经映射原语拆量住芯；flush=可回读档比对+不可回读组（含 GL 程序.current、WebGL 句柄绑定、UBO 槽、fork 掩码面）幂等整发；set_* helper 差异才下发 |
| 状态镜像 StateMirror | src/state/mod.rs | 全字段比较；被否 dirty 位图（散落易漏）；仅驱动成功后改，对象死=强制失效 |
| 复用缓存 HandleCache | src/objects/mod.rs | XHashMap<ReuseKey,(Handle,u32)> |
| 绑定槽表 SlotTable | src/binds/mod.rs | [16] 定长数组+每管线 reorder；被否键值表（热路径）；满=断言 |
| 反射登记 ReflectionTable | src/shader/mod.rs | (set,binding)↔location/name——naga 输出表落地 |
| 通道上下文 PassCtx | src/draw/mod.rs | 借编码器+锁守卫（生命周期=Drop 语义 Rust 强制） |
| 描述符词汇结构 | src/types/mod.rs | types 自建移植（决定③）；无运行态；合法性 entry 边界一次校验（隔离带） |
| SurfaceConfiguration 缓存 | src/surface/mod.rs | 宽/高非零断言；格式表来自 caps |
| 错误枚举 | src/entry/mod.rs | thiserror；封闭三处+其余类型上不可失败 |

持久化/迁移：无磁盘结构；跨版本词汇对表走「显式登记断裂」于迁移清单，不静默。
