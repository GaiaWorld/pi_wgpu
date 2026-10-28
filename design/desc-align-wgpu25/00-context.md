# 00-context · 地形（L0 事实层）

## 工作区与语言
- 项目：`E:\Rust\PI_PI\pi_wgpu_ai`（Rust 库 crate `pi_wgpu`，GL 直替层；冻结壳 `src/**/mod.rs` + 实现芯 `*_inner.rs` 惯例，头注释标 `设计：<slug>`）。
- 上游对账基准：同级 checkout `E:\Rust\PI_PI\wgpu` = wgpu 25.0.2 / wgpu-types 25.0.0 / naga 25.0.1（与 Cargo.toml `wgpu = "25.0", optional, features=["glsl"]` 同代）。

## 素材清单（三类来源）
1. 上游权威定义（可达，只读）：
   - `wgpu/src/api/render_pipeline.rs`：VertexState(:89 module/entry_point/compilation_options/buffers)、FragmentState(:121 …/targets)、RenderPipelineDescriptor(:149 label/layout:Option/primitive/depth_stencil/multisample/fragment/multiview:Option<NonZeroU32>/cache:Option<&PipelineCache>)
   - `wgpu/src/api/shader_module.rs:179`：ShaderSource = Wgsl(Cow<str>) / Glsl{shader:Cow, stage:naga::ShaderStage, defines:&[(&str,&str)]} / SpirV / Naga（后两者 feature 门控）
   - `wgpu/src/api/common_pipeline.rs:7`：PipelineCompilationOptions{constants:&[(&str,f64)], zero_initialize_workgroup_memory:bool}（Default: 空/true）；:41 PipelineCacheDescriptor
   - `wgpu/src/api/pipeline_layout.rs:25`：PipelineLayoutDescriptor{label, bind_group_layouts, push_constant_ranges:&[PushConstantRange]}（derive Default）
   - `wgpu-types/src/lib.rs:6215`：PushConstantRange{stages:ShaderStages, range:Range<u32>}
   - `wgpu/src/api/pipeline_cache.rs:71`：PipelineCache
2. 项目现状（可达）：
   - `src/types/mod.rs`：RenderPipelineDescriptor(:665 layout 必填、缺 multiview/cache)、VertexState(:464 layouts、缺 compilation_options)、ShaderSource(:709 Wgsl(Glsl 两段合一:702 带 GlslVersion:682))、PipelineLayoutDescriptor(:444 无 push 范围)、Features(:96 u64 位集无命名位)、Limits(:102 缺 max_push_constant_size，TODO-DECL 已注明对表补齐)、ShaderStages(:155 已有 VERTEX/FRAGMENT 位集)
   - `src/entry/mod.rs:258-266`：create_pipeline_layout / create_render_pipeline 契约注释含「layout 必填不提供 auto-layout」
   - `src/draw/mod.rs:43-72`：RenderPass set_* 面（无 set_push_constants）
   - examples（hello_triangle/textured_quad）：以 GlslSource 两段合一 + GlslVersion 构造，VertexState.layouts 命名字段初始化
3. 会话拍板（来源=用户，2026-10-15）：④=A 形拉齐域收紧；③强拉齐；layout=Option 形+None 违约断言；补 cache（含 PipelineCache 类型桩）；slug=desc-align-wgpu25。

## 测试与核验惯例
- 无单测框架依赖记录；以 `cargo build` + examples 端到端（hello_triangle 渲染验证，见 pi-wgpu-rebuild status 册）核验；L7 落壳后编译核验零新增错误。

## 旧设计
- `design/pi-wgpu-rebuild` 状态=已完成（2026-10-14）；本设计为独立新册，不返工旧册；旧册壳头注释 `设计：pi-wgpu-rebuild` 的文件本设计改动时按 L7 规则处理（改动处换新头注释）。
