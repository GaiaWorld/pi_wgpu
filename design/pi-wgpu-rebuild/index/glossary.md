# 登记分册：词汇表

## 词汇

| 术语 | 定义 | 出处 |
|---|---|---|
| | 无（建行由机器层 tableAddRow 追加） | — |
| 兼容子集 | 对外发布的 wgpu-rs 25.x 同名同形、语义收窄的接口面 | L1 需求 |
| 参考实现 | 编译开关整面切入的官方 wgpu-rs，用于对照排差 | L1 需求 |
| 内置实现 | 本库运行于旧图形驱动之上的渲染执行体 | L1 需求 |
| 录制即执行 | 命令录制=立即执行；提交/收尾为空壳 | L1 需求 |
| 即时生效 | 资源写入在下一次读取/绘制时已生效，与提交动作无关 | L1 需求 |
| 绑定槽 | (set,binding) 到驱动的固定映射：set 0–3；UBO 段 ≤16 + 纹理单元段 ≤16（Texture/Sampler 共用单元段）；槽值=(set,binding) 打包键（set 高 16 位/binding 低 16 位） | L1 需求 |
| 动态偏移 | 声明允许偏移的绑定条目之起始字节偏移，按声明顺序消耗 | L1 需求 |
| 延迟回收 | （作废·L1→L5 返工 2026-10-13）原指库内超时回收队列；资产出库后由「即删+镜像刷失」替代，留行防复活 | L1 需求 |
| 三硬规则 | 资源销毁三前提：draw 后/渲染通道结束后/与绑定组同弃，使用方保证 | L1 需求 |
| 取帧与呈现 | 一帧生命周期：取得可写帧纹理→渲染→呈现一次 | L1 需求 |
| 驱动形态 | 桌面 GL 3.3+ / GLES 3.0+ / WebGL2 三种内置实现形态；经 pi_glow 一入口汇合，差异收口 context/surface/caps | L5 |
| 整库切换 | 编译开关 use_wgpu 把对外整面换成官方 wgpu 参考实现的 A/B 手段 | L1 |
| 资产出库 | 业务资产键值复用/显存记账/异步加载/收缩由上游资产 crate 本库公开面之上承接，非本库职责 | L5 校准 |
| 镜像刷失 | 资源对象删除时状态镜像对该引用的作废机制——实现形态=死名登记制（objects 删除登记、flush 强制重发+整面销记，见「死名登记刷失」行） | L5 校准 |
| 上下文代号 | 附着体世代计数；lost 即 +1，整面资源/镜像/租约随代作废重登记 | L5/L7-⑥⑦⑧ |
| 交集原则 | 跨驱动形态差异仅经 caps/formats/context-surface/pi_glow 四道闸，核心模块禁形态分支（L5 判⑥） | L5-⑥ |
| 隔离带 | 合法性在 entry 边界校验一次，核心层假定干净不再校验 | L7-errors |
| 壳芯 | L7 落码形态：壳=声明+契约注释+一句委托（冻结面）；芯=实现唯一入口（失败面=错误码上递或断言；伪码注释标「示意」） | L7-落壳 |
| 相-1/相-2 | 相-1=本会话骨架集（已落已编译核验）；相-2=词汇全量对表/use_wgpu 官方挂接/wasm 面核验 | L7-落壳 |
| TODO-DECL | 芯桩统一记号（panic 消息），全文 grep 即实现批待办清单 | L7-落壳 |
| 名实对账 | 文档函数名与壳实名一致校验（L9 判出后已执行，api/units/call-chain 三册） | L9 |
| 形态子图 | 逐驱动形式的实现路径/依赖画法（deps 册节），源=差异矩阵+外部边表，仅两时序差：换帧与配置先后 | L9 补 |
| 整发档/可回读档 | 驱动状态镜像的两档重同步：可回读档=读驱动标量实值差异才发；整发档=驱动无回读口（GL 程序.current/WebGL 对象绑定/UBO 槽/fork 掩码面）恒幂等刷镜像期望 | B5 落地（实现批事实：GL_CURRENT_PROGRAM 无 glGetIntegerv 合法回读） |
| 死名登记刷失 | 对象删除时登记（域,名）而非即时改镜像；下次 flush 必覆盖重发并整面清表，镜像不假设驱动仍持已删对象 | B5/L9 增量复审定形（旧「镜像刷失」词的机制化） |
| 映射原语比较器 | types 词汇结构无 PartialEq→镜像比较全部经「枚举→GL 规范号」映射后拆量比较，比较器住芯不改壳词汇面 | B5 落地 |
| 最近快照缓存 | caps probe 末尾驻留的锁外原子读数（family/version/es），供 formats 等实现批做能力门控；未探先查=消费端断言 | B2/B3 落地 |
| BindDomain | 绑定域：Uniform=UBO 绑定点槽；Texture/Sampler=纹理单元槽（同段共用） | B6 |
| BoundEntry | 绑定组条目单位：binding+domain 资源强引用 DirectHandle+offset/size/dynamic；动态声明数=dynamic 位计数（consume 校验源） | B6 |
| 绑定槽表 SlotTable | 双段定长数组 ubo[16]+单元段[16]；设备级单表、建管线时按布局重排；被否键值表（热路径） | B6 |
| CompiledEs300 | shader 转译产物：hash（源文本哈希=复用键）+ modules:Vec<ShaderIr>；出码延到管线创建 | B7 |
| ShaderIr | 单转译单元的 naga 中间产物：module + 校验信息 info | B7 |
| ReflectionTable / RefEntry | 反射登记：entries:Vec<RefEntry{group,binding,kind:BindingKind,name}>；与程序哈希同生命周期 | B7 |
| BindingKind | 绑定类别：UniformBuffer / Texture / Sampler（决定应用走 UBO 槽还是纹理单元） | B7 |
| ShaderErr | shader 错误码枚举（ParseWgsl/ParseGlsl/Validate/Emit/NoEntry/SlotMissing/LayoutMismatch/EmptyGlsl/UnsupportedBinding） | B7 |
| 错误码上递 | 决定⑪错误处理：优先返回短错误码（Result），避免原地 panic；不便上递处 log::error!(码) 后降级；禁长文本 | B7/决定⑪ |
| UploadErr | upload 错误码枚举：DeadHandle/Misaligned/RowPitch/OutOfRange/NotBitmap/OffMainThread；公开非 Result 面=log 短码（决定⑪） | B8 |
