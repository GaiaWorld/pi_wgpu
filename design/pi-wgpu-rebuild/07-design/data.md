# pi-wgpu-rebuild · 数据结构与不变量（L7）

不变量与所有权住壳类型注释；本册留指针+表示理由。

| 结构 | 壳 | 表示选择 / 被否方案 |
|---|---|---|
| Instance/Adapter/Device/Queue | src/entry/mod.rs | Share 引用链（pi_share）；被否 Rc（线程合同破） |
| 资源句柄对象（1:1：Buffer/Texture/Sampler/TextureView） | src/objects/mod.rs | Option<Handle>（删=置 None，死用=断言）；活态 ⟺ 上下文代号=当前 |
| 复用对象（程序/VAO/FBO/RBO） | src/objects/mod.rs | Arc+refcount（HandleCache 内）；归零=物理删+镜像刷失；被否手动 Option（违约面大） |
| BindGroup/BindGroupLayout/PipelineLayout/RenderPipeline/ShaderModule | src/entry/mod.rs | 描述符解析后持强链（三硬规则库内形态=引用图+使用方合同） |
| SurfaceTexture（帧租约） | src/surface/mod.rs | present 消自——同时至多一帧的不变量类型级成立 |
| 附着体 Attachment | src/context/mod.rs | Arc{pi_egl 实例/上下文/表面 + pi_glow Context + 重入锁 + 代号}；Web 形态 egl 退化为画布上下文（cfg 表示内变体）；代号单调增 |
| 状态镜像 StateMirror | src/state/mod.rs | 全字段比较；被否 dirty 位图（散落易漏）；仅驱动成功后改，对象死=强制失效 |
| 复用缓存 HandleCache | src/objects/mod.rs | XHashMap<ReuseKey,(Handle,u32)> |
| 绑定槽表 SlotTable | src/binds/mod.rs | [16] 定长数组+每管线 reorder；被否键值表（热路径）；满=断言 |
| 反射登记 ReflectionTable | src/shader/mod.rs | (set,binding)↔location/name——naga 输出表落地 |
| 通道上下文 PassCtx | src/draw/mod.rs | 借编码器+锁守卫（生命周期=Drop 语义 Rust 强制） |
| 描述符词汇结构 | src/types/mod.rs | types 自建移植（决定③）；无运行态；合法性 entry 边界一次校验（隔离带） |
| SurfaceConfiguration 缓存 | src/surface/mod.rs | 宽/高非零断言；格式表来自 caps |
| 错误枚举 | src/entry/mod.rs | thiserror；封闭三处+其余类型上不可失败 |

持久化/迁移：无磁盘结构；跨版本词汇对表走「显式登记断裂」于迁移清单，不静默。
