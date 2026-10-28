# pi_wgpu 设计总览（v0.4.1）

> 本文是当前分支（v4）的单篇总纲，面向「第一次接触本库」的读者。
> 权威细节：模块契约在 src/**/mod.rs 头注释；重大机制在 design/ 三册（pi-wgpu-rebuild、desc-align-wgpu25、sampler-unit-assign）；
> 具体接口与差异见 documents/api.md、documents/migration.md、documents/module-design.md、documents/design.md。

## 1. 定位

在 OpenGL 3.3 / GLES 3.0 / WebGL2 之上，实现 **wgpu 25 兼容子集** 的渲染库：

~~~text
上层引擎（wgpu 风格代码）
        │
        ├── feature = use_wgpu → 整面 re-export 官方 wgpu（A/B 对照）
        └── feature = gl（默认）→ pi_wgpu 自产实现
                                  │
                     OpenGL 3.3+ / GLES 3.0+ / WebGL2
~~~

选择理由：微信小游戏只提供 WebGL2（iOS 需 ≥ 15.5），存量 Android/Windows 设备主力仍是 GL 驱动栈；Vulkan/原生 WebGPU 短期内覆盖不了。

## 2. 当前架构（13 个功能模块 + 3 个支撑模块）

~~~text
src/
├── lib.rs / lib_inner.rs     feature 分叉、根导出、PiWgpuAdapter 双参适配
├── record.rs                 渲染命令录制与提交序回放（录制订阅）
├── types/                    公开词汇面：pub use wgpu_types::* + 引用式描述符别名
├── entry/                    公开对象面：Instance/Adapter/Device/Queue/Encoder/资源对象
├── surface/                  呈现：configure / get_current / present（离屏屏幕纹理 + 翻转 blit）
├── context/                  全库唯一上下文附着点（重入锁、跨线程搬运、代号）
├── caps/                     能力探测 + 特判表（Qualcomm / iOS18）
├── state/                    驱动状态镜像（可回读档 / 整发档 / 死名表 / 帧门 / 世代）
├── objects/                  GL 句柄层：1:1 直建三类 + 四类可复用对象
├── formats/                  格式→驱动参数表 + mipmap 链 + 纹理目标选择
├── binds/                    绑定槽表：UBO / 纹理单元 / 采样器单元三族
├── shader/                   naga 转译（WGSL/GLSL→ES300）+ 反射
├── draw/                     渲染通道：begin/set/draw/end、组合 VAO、RTT、清屏
├── upload/                   即时写入、拷贝、读回、Web 图像直传
├── utils/                    DeviceExt 便捷构造 + 环境助手
├── error/                    错误面（Error/ErrorFilter/UncapturedErrorHandler）
└── objects/gid.rs            GlHandle(u64) 与 glow 句柄值唯一闸口
~~~

每个模块 = **壳**（src/<mod>/mod.rs：类型/方法声明 + 契约注释）+ **芯**（*_inner.rs：实现）。
壳头注释必带「设计：<slug>」；改壳/设计册需走 dev-design-lite 流程。

## 3. 执行模型（v4 关键变化）

~~~text
录制期（可多编码器）
  encoder.begin_render_pass(desc)  → 校验 + 记 Begin（含附件/清除参数）
  pass.set_pipeline/set_bind_group/set_* / draw* → 只入命令表（持资源强引用）
  pass Drop                         → 记 End（不触驱动）
  encoder.finish()                  → CommandBuffer { commands }
提交期（Queue::submit）
  按 submit 参数顺序逐 CommandBuffer 回放
  通道内逐条执行：Begin → set/draw → End（此时才真正下发 GL）
~~~

- **写路径仍是即时**：Queue::write_buffer / write_texture / copy_* 在调用点生效（不等 submit）。
- **present 才换帧**：渲染落在离屏「屏幕纹理」，present 做一次 y 反向 blit 到默认帧缓冲，再推进帧号。
- 多线程并行编码因此获得「提交序」语义；通道资源在命令表存活期内被强引用，不会被提前删。

## 4. 三档状态治理

| 档 | 覆盖 | 判据 |
|---|---|---|
| 可回读档 | 活动纹理单元、各能力开关、深度函数、视口/裁剪框 | 每帧一次向驱动回读比对（帧门），差异才重发 |
| 整发档 | program / VAO / FBO / PIXEL_UNPACK / UBO 槽 / 纹理槽 / 采样器槽 / 混合 / 模板 / 图元 / 写掩码等 | 与「上次实际下发水位」water 比对，差异才发 |
| 直发路 | set_* 组 helper | 下发后**同点回写水位**，否则下一轮 flush 判失配重发 |

附加约束：

- 能力开关只能 isEnabled 回读（getParameter + cap pname = INVALID_ENUM）。
- 死名表非空 → 镜像丢掉指向死名的期望，整发档强发一轮后销记（GL 名可复用，不能拿名字当存活证据）。
- 每通道起始复位 viewport/scissor/depthRange 为全附件（有宿主尺寸时）；默认帧缓冲通道无尺寸来源，不复位（已登记）。
- 状态下发世代（epoch）变化 → 本地水位整组作废。

## 5. 对象与生命周期

| 类别 | 对象 | 生命周期 |
|---|---|---|
| 1:1 直建 | Buffer / Texture / Sampler | Share 共享计数；末引用 Drop 即删 + 登记死名 |
| 键复用 | Program / VAO / FBO / RBO | 全局计数表；键=配置哈希 |
| 驻留复用 | FBO / VAO | 计数归零**不删**，ensure 沿名复用（跨帧稳定） |
| 归零删除 | Program / RBO | 计数归零物理删 + 死名登记 |
| 纯值 | TextureView / 描述符 / BindingResource | 随宿主复制消亡 |

三条使用方硬规则（库不兜底、不校验）：资源须在该资源最后一次 draw 之后销毁；RenderPass 绑定资源须等通道结束（Drop）后销毁；被 BindGroup 持有的资源与 BindGroup 同弃。

## 6. 绑定模型

- set 域 0..3；每管线持有**自己那份** SlotTable（创建期定格），设备级槽表只作「最近一次分配」的查询面。
- 三族独立紧凑编号；槽表定长 BIND_SLOTS = 24，实际分配上限：UBO 24，纹理 16，采样器 16（GLES3 保底）：
  - ubo[i]：glBindBufferBase/Range 的绑定点号；
  - tex[i] / smp[i]：同一 GL 纹理单元空间的两个对象槽；
  - 值 = (set<<16 | binding) 打包键；索引即 GL 点/单元号。
- 组合采样器：出码期从 IR 收集「纹理↔采样器」配对，应用期把采样器对象绑到配对纹理单元（alias 表）。
- 动态偏移：偏移个数必须等于布局中 has_dynamic_offset 条目数；按声明序消耗，叠加静态偏移。
- UBO 块指位与采样器 uniform 指位是**程序对象持久态**：建管线时一次下发，之后随 useProgram 天然携带，切换零参与。

## 7. 着色器与坐标

- 输入：WGSL 单串，或 GLSL 按阶段单源（#version 由源码自承载）；SpirV/Naga 变体不移植。
- 输出：naga glsl-out → #version 300 es，binding_map 按槽表定位。
- 关键 writer flag：ADJUST_COORDINATE_SPACE（等价 wgpu GL 后端）：gl_Position.y 取反 + z 从 [0,1] 映到 [-1,1]。
- 连带：正面绕序映射按镜像反向（Ccw↔GL_CCW 的常规表反过来用）；上屏用反向源矩形 blit 翻正，copy 路径不做行翻。
- entry_point=None：该 stage 唯一入口自动选；缺失/歧义 = 出码失败（管道创建期断言）。

## 8. 纹理与上传

- 格式表：22 个显式条目（R/RG/RGBA 未压缩族 + 深度族 + BC1/2/3 + ETC2 四型）+ ASTC 参数族（14 种块尺寸 × Unorm/UnormSrgb，Hdr 表外）；能力门控经 caps 最近快照；表外格式=断言拒止，不做近似替换。
- 目标：D1/D2（层=1）→ TEXTURE_2D；D2 层>1：cube 兼容（宽=高、层%6=0、sample=1）→ CUBE_MAP / CUBE_MAP_ARRAY，否则 2D_ARRAY；D3 → TEXTURE_3D。
- 压缩纹理：块行行距判据 + 逐块行喂；Web 忽略 UNPACK_ROW_LENGTH，故按块行处理。
- mipmap：全量上传后整链重生成（无「仅刷一级」路径，成本=全链）。
- Web 专面：write_texture_jsbuffer / copy_external_image_to_texture / create_compress_texture_with_data_jsdata 零字节直传（仅主线程）。
- 读回：Buffer::get_sub_data 是上游 map_async/get_mapped_range 的同步投影替代；copy_texture_to_buffer 内部 glFinish 收敛 PBO。

## 9. 诊断与错误

- 三个显式 Result 面：request_device / create_surface / get_current_texture；其余创建族违约=断言（短码 + label），断言前不发驱动调用。
- 非 Result 写路径：log::error!(UploadErr 等短码)；debug 不下发、release 留痕后下发（不谎报）。
- 错误作用域：on_uncaptured_error / push_error_scope / pop_error_scope 已接；缺省未捕获 = log 降级（上游默认 panic，差异登记）。
- 诊断探针：frame churn configure/tex_new/tex_del、rack 重挂、组合 VAO 表满、测试计数器 probe(0/1/2)。

## 10. 验证

~~~bash
cargo test --lib
cargo check --lib --target wasm32-unknown-unknown
cargo check --all-targets
powershell -ExecutionPolicy Bypass -File .\web_case\build.ps1
python -m http.server 8731 -d web_case
~~~

样例：examples/{hello_triangle, textured_quad, clear, alpha_blend, d32_iso, rtt_probe, shadow_cube, upload_probe}；web_case 为浏览器案例包。

## 11. 当前状态与已知缺口

- 版本 0.4.1；v4 分支已完成：WebGL2 装配、u64 身份层、描述符对齐 wgpu 25、采样器/UBO 指位、提交序回放、RTT 形状键驻留、组合 VAO、每通道默认态复位、延后子区域清屏。
- 已知缺口：默认帧缓冲通道不复位 viewport/scissor；CLEAR_SCISSORED 非标准清除语义（默认开）；Web 无 polygonMode；sample_mask 仅镜像记录；多 draw 未合并；Android 真机链路待验；OffscreenCanvas 面未接；Limits 为静态保底表（非逐项真探）。
- documents/ 旧版（v0.3.7）已删除；本套文档按 v4 源码重写。
