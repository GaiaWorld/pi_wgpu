# pi_wgpu 模块设计（v0.4.1）

> 描述「当前实现是什么」。为什么这样做见 design.md；接口面见 api.md；与 wgpu 的差异见 migration.md。
> 每个模块 = 公开壳（mod.rs，类型/方法声明 + 契约注释）+ 实现芯（*_inner.rs）。壳变更走 dev-design-lite。

## 1. 模块清单

| 模块 | 职责 | 隐藏的易变决定 | 壳 / 芯 |
|---|---|---|---|
| types | 公开词汇面：pub use wgpu_types::* + 引用式描述符别名（BufferDescriptor<'a> 等） | 跟随上游版本对表 | types/mod.rs |
| entry | 公开对象面：Instance/Adapter/Device/Queue/Encoder/Buffer/Texture/Sampler/BindGroup(Layout)/Pipeline(Layout)/ShaderModule | 对象面失败与持有合同 | entry/mod.rs → entry_inner.rs |
| surface | 附着→configure→取帧→present；离屏屏幕纹理 + 翻转 blit | 呈现约束与取帧错误态 | surface/mod.rs → surface_inner.rs |
| context | 全库唯一 GL 附着点：重入锁、跨线程搬运、附着代号、state epoch | 线程合同 | context/mod.rs → context_inner.rs |
| caps | 版本/扩展探测；Features 投影；设备特判表 | 特判集合（新增只加 SPECIAL_ROWS 一行） | caps/mod.rs → caps_inner.rs |
| state | 驱动状态镜像与差异下发；死名表；帧门；水位 | 镜像字段集与比较粒度 | state/mod.rs → state_inner.rs |
| objects | GL 句柄层：1:1 直建三件套 + Program/VAO/FBO/RBO 键复用 | 复用范围与归零删除规则 | objects/mod.rs → objects_inner.rs |
| formats | 格式→GL 参数表、纹理目标选择、mipmap 链、能力门控 | 格式支持集 | formats/mod.rs → formats_inner.rs |
| binds | (set,binding)→三族槽位；reorder/alias；动态偏移 | 槽位分配规则 | binds/mod.rs → binds_inner.rs |
| shader | naga 转译（WGSL/GLSL→ES300）、反射登记、按入口出码 | 转译输出形态与反射契约 | shader/mod.rs → shader_inner.rs |
| draw | 通道录制/执行：begin/set/draw/end、组合 VAO、RTT、清屏守卫 | 通道能力约束与状态下发点 | draw/mod.rs → draw_inner.rs |
| upload | 即时写入、拷贝、读回、Web 图像直传 | 写入生效时机（即时） | upload/mod.rs → upload_inner.rs |
| utils | DeviceExt 便捷构造 + 环境助手 | 附加便利面 | utils/mod.rs → utils_inner.rs |
| record | 渲染命令表与提交序回放 | 命令粒度（通道整体 + 状态/绘制逐条） | record.rs |
| error | Error/ErrorFilter/UncapturedErrorHandler + 作用域栈 | 错误面形状 | error/mod.rs → error_inner.rs |
| gid | GlHandle(u64) ⇄ glow 句柄值唯一闸口（Web 保留 slot-key 全 64 位） | 身份表示 | objects/gid.rs |

## 2. 依赖边

~~~mermaid
flowchart TD
  types["types · 词汇"]
  entry["entry · 对象面"]
  surface["surface · 呈现"]
  context["context · 附着"]
  caps["caps · 能力"]
  state["state · 镜像"]
  objects["objects · 句柄"]
  formats["formats · 格式"]
  binds["binds · 槽"]
  shader["shader · 转译"]
  draw["draw · 通道"]
  upload["upload · 写"]
  utils["utils · 便捷"]
  record["record · 回放"]
  error["error · 错误面"]

  entry --> types
  entry --> surface
  entry --> context
  entry --> caps
  entry --> state
  entry --> formats
  entry --> objects
  entry --> binds
  entry --> shader
  entry --> draw
  entry --> upload
  entry --> error
  utils --> types
  utils --> entry
  surface --> types
  surface --> context
  surface --> caps
  surface --> entry
  context --> types
  caps --> types
  state --> types
  state --> context
  state --> objects
  objects --> types
  objects --> context
  objects --> formats
  objects --> state
  formats --> types
  formats --> caps
  formats --> context
  binds --> types
  binds --> state
  binds --> entry
  binds --> objects
  shader --> types
  shader --> binds
  shader --> entry
  draw --> types
  draw --> state
  draw --> binds
  draw --> context
  draw --> entry
  draw --> objects
  draw --> record
  upload --> types
  upload --> objects
  upload --> context
  upload --> formats
  upload --> entry
  record --> entry
  record --> draw
  record --> upload
  record --> objects
  record --> binds
  record --> types
~~~

说明：单 crate 内存在双向类型引用（entry↔binds/state/shader、objects↔state、surface↔entry），按「无功能调用环」判定不计环。所有芯经 context_inner::with_gl 取驱动入口，经 state::BIND_SLOTS 共享槽常量。

## 3. 公开对象与内部数据结构

| 结构 | 位置 | 关键字段 / 不变量 |
|---|---|---|
| Instance | entry/mod.rs | _attach: AttachmentSlot（唯一附着槽）；创建期完成附着与探测。 |
| Adapter | entry/mod.rs | _caps: Caps（不可变能力快照）；可反复 request_device，失败无副作用。 |
| DeviceInner | entry/mod.rs | state / cache / slots / error_state 四把 std Mutex；锁序：state 先、with_gl 后。 |
| Device | entry/mod.rs | Share<DeviceInner>；编码器可克隆共享。 |
| Queue | entry/mod.rs | _dev: Option<Device>；submit 回放命令，write_* 即时。 |
| CommandEncoder | entry/mod.rs | _opened_pass 守卫 + _commands: Vec<Command>；至多一个开启通道。 |
| CommandBuffer | entry/mod.rs | 命令表；submit 按参数顺序回放。 |
| Buffer | entry/mod.rs | _h: DirectHandle + size + _usage；PartialEq 按活名+size。 |
| Texture | entry/mod.rs | _h + GlParams + _target/_dim/_layers + _size/_mip/_samples/_format/_usage + _surface 标记。 |
| TextureView | entry/mod.rs | {_tex: Option<Texture>, _target, _dim}；帧视图=无宿主且 target=0。 |
| Sampler | entry/mod.rs | _h: DirectHandle。 |
| BindGroupLayout | entry/mod.rs | _entries: Vec<BindGroupLayoutEntry>（创建期已校验）。 |
| BindGroup | entry/mod.rs | _bound: Vec<BoundEntry>（声明序，持资源强引用）。 |
| PipelineLayout | entry/mod.rs | _bgls: Vec<BindGroupLayout>（成员强链）。 |
| RenderPipeline | entry/mod.rs | _program/_vao: Reusable + _vertex/_topology/_depth_stencil/_blend/_primitive/_color_writes/_alpha_to_coverage/_slots。 |
| ShaderModule | entry/mod.rs | _refl: ReflectionTable + _compiled: CompiledEs300。 |
| RenderPass<'pass> | draw/mod.rs | _enc: &mut CommandEncoder + 通道态（_vertex/_pipe_set/_vao/_fbo/_hold_color/_hold_depth/_resolve/_clear/_slots/_vbinds/_index_*）。 |
| DirectHandle | objects/mod.rs | Share<DirectHandleState{kind, handle: ShareU64, generation}>；Clone 共享名，末引用删。 |
| Reusable | objects/mod.rs | key: ReuseKey + handle: Option<GlHandle> + refcount 快照。 |
| HandleCache | objects/mod.rs | XHashMap<ReuseKey, Vec<Reusable>>（pi_hash）。 |
| StateMirror | state/mod.rs | 全字段期望态 + water + generation + epoch_seen + readback_frame。 |
| Water | state_inner.rs | 非回读组「上次实际下发」原语快照（u64/元组/数组）。 |
| SlotTable | binds/mod.rs | ubo/tex/smp: [Option<u32>; 24] + alias；分配上限：UBO 24、纹理 16、采样器 16。 |
| BoundEntry | binds/mod.rs | binding/domain/handle/offset/size/dynamic/target。 |
| CompiledEs300 | shader/mod.rs | hash + Vec<ShaderIr{module, info}>；出码延到管线创建。 |
| ReflectionTable | shader/mod.rs | Vec<RefEntry{group,binding,kind,name}>。 |
| Surface | surface/mod.rs | _target + _cfg + _attach + _creation + _stage。 |
| SurfaceTexture | surface/mod.rs | pub texture + pub suboptimal + presented 哨兵 + _view；present 按值自消费。 |
| Attachment | context/mod.rs | 代号读数载体；真实附着驻留 context 芯全局槽。 |
| Caps | caps/mod.rs | 版本对 + qualcomm/ios18 标志 + 扩展位掩码（含形态位）。 |
| Command | record.rs | Begin/End/SetPipeline/SetBindGroup/SetVertex/SetIndex/SetViewport/SetScissor/SetStencilRef/SetBlendConstant/Draw/DrawIndexed/CopyT2B/CopyB2T/CopyT2T。 |

## 4. 生命周期与引用链

~~~text
BindGroup ──强──▶ Buffer / Texture(─▶TextureView) / Sampler（DirectHandle 共享计数）
RenderPipeline ──强──▶ Program(Reusable) + VAO(Reusable)
CommandBuffer ──强──▶ RetainedPipeline(program/vao 计数+1) / RetainedBindGroup(句柄 clone) / 附件宿主 Texture
RenderPass ──借用──▶ CommandEncoder + 设备 state/cache 锁
Queue ──▶ DeviceInner（Share）
Surface ──▶ AttachmentSlot + ShareCell 配置/屏幕纹理
~~~

销毁规则：

1. Buffer/Texture/Sampler：末引用 Drop 即调驱动删除 + note_removed 登记死名。
2. Program/RBO：全局计数归零即删 + 死名登记。
3. FBO/VAO：归零**不删**（驻留），ensure 命中同名行直接复用；VAO 另有组合缓存 256 上限 + LRU（本帧使用保护）。
4. 纹理删除同步摘 ROW_ATT 签名（GL 会从 FBO 自动剥离附件）。
5. RenderPipeline Drop 释放 program/vao；命令表的持行体在命令销毁时成对 release。

## 5. 线程与锁

~~~text
录制（编码器）：不触 GL；只写命令表（持资源强引用）
提交（Queue::submit）：回放线程逐条触 GL
创建/销毁/写：可跨线程，经 context_inner::with_gl
with_gl = claim_and_attach() → f(glow) → release_one_layer()
  同线程重入=计层快路；异线程等待上限 5s（超时 trace + 断言）
  single_thread feature：锁步与 make/unmake current 整体退化
~~~

- 包裹锁用 std::sync::Mutex（poison 用 unwrap_or_else(into_inner)）。
- 锁序：state → cache → with_gl；binds 应用期持 state 后再 bind_*。
- 状态 epoch：任一线程完成一次下发即 +1；其他编码器的水位跨世代作废，整组强发。

## 6. 错误归属

| 模块 | 策略 |
|---|---|
| entry | 三个 Result 面（request_device/create_surface/get_current_texture）+ 创建族断言（短码 + label） |
| surface | configure=log 短码（SurfaceErr/SurfaceOpErr），失败不写 _cfg；取帧=五态 Result |
| draw | 合同违反断言（嵌套通道/死句柄/参数级），断言前无副作用 |
| binds | 计数不符/槽满/死句柄=断言，且在任何 bind 之前预检 |
| upload | log::error!(UploadErr)；debug 不下发，release 留痕后下发 |
| shader | Result<_, ShaderErr> + log 诊断原文 |
| caps/formats | 探测无效/格式无表/能力未确证=断言（短码） |
| context | 锁超时=断言 + trace；附着缺失=断言 |
| error | 作用域捕获/未捕获回调/log 降级 |

## 7. 文件结构

~~~text
src/
  lib.rs lib_inner.rs record.rs crash_diag.rs
  types/mod.rs
  entry/{mod.rs, entry_inner.rs}
  surface/{mod.rs, surface_inner.rs}
  context/{mod.rs, context_inner.rs}
  caps/{mod.rs, caps_inner.rs}
  state/{mod.rs, state_inner.rs}
  objects/{mod.rs, objects_inner.rs, gid.rs}
  formats/{mod.rs, formats_inner.rs}
  binds/{mod.rs, binds_inner.rs}
  shader/{mod.rs, shader_inner.rs}
  draw/{mod.rs, draw_inner.rs}
  upload/{mod.rs, upload_inner.rs}
  utils/{mod.rs, utils_inner.rs}
  error/{mod.rs, error_inner.rs, tests.rs}
design/               设计册（pi-wgpu-rebuild / desc-align-wgpu25 / sampler-unit-assign）
documents/            本套文档
examples/             hello_triangle, textured_quad, clear, alpha_blend, d32_iso, rtt_probe, shadow_cube, upload_probe
web_case/             浏览器案例包（build.ps1 + src + index.html）
~~~
