// 设计：pi-wgpu-rebuild
//! types · 公开 API 词汇面：全为纯值/纯数据，无运行态、不触驱动。
//! 不变量：合法性不在本面判定——一律由 entry 创建边界一次性校验（违约=断言，报字段与 desc.label）。
//! 所有权：复制语义，随宿主对象存续。本面全部 @public。

/// 对象可选标签；仅用于断言与 trace 报错时定位来源对象。
pub type Label<'a> = Option<&'a str>;
/// GPU 内存地址/尺寸，单位字节。
pub type BufferAddress = u64;
/// buffer 与 texture 互拷时 bytes_per_row 必须为此值的整数倍（WebGL2 行距对齐约束）。
pub const COPY_BYTES_PER_ROW_ALIGNMENT: u32 = 256;
/// 缓冲尺寸与写入偏移必须 4 字节对齐。
pub const COPY_BUFFER_ALIGNMENT: u64 = 4;
/// 顶点步长（array_stride）与属性偏移必须 4 字节对齐。
pub const VERTEX_STRIDE_ALIGNMENT: BufferAddress = 4;

use std::num::NonZeroU32;

/// 缓冲字节大小的正值包装：严格大于 0，上限 2^32-1（兼容子集差异：官方为 u64 域）。
#[repr(transparent)] #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct BufferSize(pub NonZeroU32);
impl BufferSize { pub const fn get(self) -> u64 { self.0.get() as u64 } }

/// 后端选位集合（位标记）。本库 GL 单面：GL=1 是唯一有效位，EMPTY=0 表示不选后端；其余位恒不可用。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] pub struct Backends(pub u32);
impl Backends { pub const GL: Self = Self(1); pub const EMPTY: Self = Self(0); }

/// Instance 创建输入。默认 backends=GL；填其他位只会在创建校验处失败，不会多出后端。
#[derive(Debug, Clone, Copy)] pub struct InstanceDescriptor { pub backends: Backends }
impl Default for InstanceDescriptor { fn default() -> Self { Self { backends: Backends::GL } } }

/// 适配器类型标签：仅随 AdapterInfo 上报展示；单卡面，不参与任何选择路由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum DeviceType { Cpu, IntegratedGpu, DiscreteGpu, VirtualGpu, Other }
/// 功耗偏好：兼容收录的提示词；EGL 单卡面记录但不改变选路（无多卡可选）。
#[derive(Debug, Clone, Copy, Default)] pub enum PowerPreference { #[default] Default, LowPower, HighPerformance }
/// 内存倾向提示；收录词汇，无驱动侧映射行为。
#[derive(Debug, Clone, Copy)] pub enum MemoryHints { Performance, Memory, Auto }
impl Default for MemoryHints { fn default() -> Self { Self::Auto } }

/// 能力位集合（u64 位标记）。有效位由 caps 探测填充；探测不到的能力恒 0，不虚报。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct Features(pub u64);

/// 资源上限子集。request_device 拿它与能力快照对照，任一超出即失败。
/// TODO-DECL：字段集为兼容子集，挂接批与官方全量 Limits 字段对表补齐。
#[derive(Debug, Clone, Copy)] pub struct Limits { pub max_texture_dimension_2d: u32, pub max_buffer_size: BufferAddress, pub max_bind_groups: u32, pub max_uniform_buffer_bindings: u32 }
impl Default for Limits { fn default() -> Self { Self { max_texture_dimension_2d: 4096, max_buffer_size: 1 << 28, max_bind_groups: 4, max_uniform_buffer_bindings: 16 } } }

/// 纹理格式词汇子集；实际支持集由逐格式能力表决定，创建/用作目标时校验。
/// TODO-DECL：Astc/Bc 压缩族与格式能力表扩充由后续批次落地。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum TextureFormat { Rgba8Unorm, Rgba8UnormSrgb, Bgra8Unorm, Bgra8UnormSrgb, Rgba16Float, Depth24UnormS8Uint, Depth32Float }
/// 纹理用途位集合（组合）；支持位之外在 create_texture 校验处断言。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct TextureUsages(pub u32);
impl TextureUsages { pub const COPY_DST: Self = Self(1); pub const RENDER_ATTACHMENT: Self = Self(2); pub const TEXTURE_BINDING: Self = Self(4); }
/// 缓冲用途位集合（组合）；空组合与非支持位在 create_buffer 校验处断言。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct BufferUsages(pub u32);
impl BufferUsages { pub const VERTEX: Self = Self(1); pub const INDEX: Self = Self(2); pub const UNIFORM: Self = Self(4); pub const COPY_DST: Self = Self(8); }
/// 绑定可见阶段位集合；空集在 create_bind_group_layout 校验处断言。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct ShaderStages(pub u32);
impl ShaderStages { pub const VERTEX: Self = Self(1); pub const FRAGMENT: Self = Self(2); }

/// 尺寸三元组，单位纹素/元素；用它的场合要求宽高≥1（0 值在创建校验处断言）。
#[derive(Debug, Clone, Copy, Default)] pub struct Extent3d { pub width: u32, pub height: u32, pub depth_or_array_layers: u32 }
/// 拷贝起点，单位纹素坐标。
#[derive(Debug, Clone, Copy, Default)] pub struct Origin3d { pub x: u32, pub y: u32, pub z: u32 }
/// 纹理维度；本面收录词汇，实际放行按格式能力表在创建时校验。
#[derive(Debug, Clone, Copy, Default)] pub enum TextureDimension { #[default] D2, D1, D3 }
/// 纹理视图维度。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] pub enum TextureViewDimension { #[default] D2, D2Array, Cube, CubeArray, D1, D3 }
/// 视图取用的宽高面（深度/模板分离用）。
#[derive(Debug, Clone, Copy, Default)] pub enum TextureAspect { #[default] All, Stencil, DepthOnly }
/// 索引格式；绘制时索引缓冲值宽必须与之一致，违约=断言。
#[derive(Debug, Clone, Copy)] pub enum IndexFormat { Uint16, Uint32 }
/// 顶点属性格式子集；与 offset 组合须满足 4 字节对齐约束。
#[derive(Debug, Clone, Copy)] pub enum VertexFormat { Float32, Float32x2, Float32x3, Float32x4, Uint8x4, Sint16x2 }
/// 顶点步进模式：逐顶点或逐实例。
#[derive(Debug, Clone, Copy, Default)] pub enum VertexStepMode { #[default] Vertex, Instance }

/// 缓冲绑定类型：仅 Uniform——WebGL2 无 storage 缓冲，兼容子集按能力交集建模。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum BufferBindingType { Uniform }
/// 采样器绑定类型；Comparison 要求采样器带 compare 配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum SamplerBindingType { Filtering, NonFiltering, Comparison }
/// 纹理取样数值类型；filterable=false 时绑定它的采样器必须是 Nearest 系。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum TextureSampleType { Float { filterable: bool }, Sint, Uint }
/// 绑定条目类型三形。has_dynamic_offset：与官方同名同形同义（动态偏移经 set_bind_group 传入，无差异）；
/// min_binding_size=None 表示整段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum BindingType {
    Buffer { ty: BufferBindingType, has_dynamic_offset: bool, min_binding_size: Option<BufferSize> },
    Sampler(SamplerBindingType),
    Texture { sample_type: TextureSampleType, view_dimension: TextureViewDimension, multisampled: bool },
}
/// 绑定组布局条目：槽号 + 可见阶段 + 数组长度（None=单个）+ 类型。
#[derive(Debug, Clone)] pub struct BindGroupLayoutEntry { pub binding: u32, pub visibility: ShaderStages, pub count: Option<NonZeroU32>, pub ty: BindingType }
/// 绑定组布局创建输入；槽位归组在布局面声明（set 域 0-3），条目违约=断言含 label。
#[derive(Debug, Clone)] pub struct BindGroupLayoutDescriptor<'a> { pub label: Label<'a>, pub entries: &'a [BindGroupLayoutEntry] }
/// 一次绑定的缓冲具例：offset 须 4 字节对齐；size=None 取到缓冲末尾；
/// offset+size 不得超过缓冲实际大小；引用死句柄=断言。
#[derive(Debug, Clone, Copy)] pub struct BufferBinding<'a> { pub buffer: &'a crate::entry::Buffer, pub offset: BufferAddress, pub size: Option<BufferSize> }
/// 绑定资源三选一（缓冲/采样器/纹理视图），必须是活引用。
#[derive(Debug, Clone, Copy)] pub enum BindingResource<'a> { Buffer(BufferBinding<'a>), Sampler(&'a crate::entry::Sampler), TextureView(&'a crate::entry::TextureView) }
/// 绑定组单条具例：binding 号须与布局条目一致。
#[derive(Debug, Clone)] pub struct BindGroupEntry<'a> { pub binding: u32, pub resource: BindingResource<'a> }
/// 绑定组创建输入：对给定布局逐一供给资源；条目缺失/多余/类型对不上=断言含 label，未建组。
#[derive(Debug, Clone)] pub struct BindGroupDescriptor<'a> { pub label: Label<'a>, pub layout: &'a crate::entry::BindGroupLayout, pub entries: &'a [BindGroupEntry<'a>] }
/// 缓冲创建输入。size 须>0 且 4 字节对齐；usage 非空且在支持集内；
/// mapped_at_creation 仅作兼容收录——本库无持久映射语义，写入一律走 Queue::write_buffer 即时生效面。
#[derive(Debug, Clone)] pub struct BufferDescriptor<'a> { pub label: Label<'a>, pub usage: BufferUsages, pub size: BufferAddress, pub mapped_at_creation: bool, pub memory_hints: MemoryHints }
/// 纹理创建输入。约束：宽高≥1；mip_level_count≥1；sample_count 仅 1 或 4；
/// view_formats 须为主格式同族；违约=断言含 label，无半建纹理。
#[derive(Debug, Clone)] pub struct TextureDescriptor<'a> { pub label: Label<'a>, pub usage: TextureUsages, pub dim: TextureDimension, pub size: Extent3d, pub format: TextureFormat, pub mip_level_count: u32, pub sample_count: u32, pub view_formats: &'a [TextureFormat] }
/// 纹理视图描述：全默认即「取整张」；维度/宽高面须与宿主纹理兼容。
#[derive(Debug, Clone, Copy)] pub struct TextureViewDescriptor<'a> { pub label: Label<'a>, pub format: TextureFormat, pub dimension: TextureViewDimension, pub aspect: TextureAspect, pub base_mip_level: u32, pub level_count: Option<NonZeroU32>, pub base_array_layer: u32, pub array_layer_count: Option<NonZeroU32> }
// TODO-DECL：TextureViewDescriptor 的「按宿主纹理取默认」便捷构造，挂接批定形。

/// 过滤模式。
#[derive(Debug, Clone, Copy)] pub enum FilterMode { Nearest, Linear }
/// 寻址模式（u/v/w 三轴独立配置）。
#[derive(Debug, Clone, Copy)] pub enum AddressMode { ClampToEdge, Repeat, MirroredRepeat }
/// 边界颜色候选。
#[derive(Debug, Clone, Copy)] pub enum SamplerBorderColor { TransparentBlack, OpaqueBlack, OpaqueWhite }
/// 深度/模板比较函数。
#[derive(Debug, Clone, Copy)] pub enum CompareFunction { Never, Less, Equal, LessEqual, Greater, NotEqual, GreaterEqual, Always }
/// 采样器创建输入。anisotropy_clamp 取 None 或 1/2/4/8/16；compare=Some 即 comparison 采样器；
/// 越界=断言含 label，无半建。
#[derive(Debug, Clone)] pub struct SamplerDescriptor<'a> { pub label: Label<'a>, pub address_mode_u: AddressMode, pub address_mode_v: AddressMode, pub address_mode_w: AddressMode, pub mag_filter: FilterMode, pub min_filter: FilterMode, pub anisotropy_clamp: Option<NonZeroU16>, pub border_color: Option<SamplerBorderColor>, pub compare: Option<CompareFunction> }
use std::num::NonZeroU16;

/// RGBA 颜色，各通道 f32；清屏/混合语义按驱动解释（0.0-1.0）。
#[derive(Debug, Clone, Copy)] pub struct Color { pub r: f32, pub g: f32, pub b: f32, pub a: f32 }
/// 渲染通道开始的加载动作：沿用既有内容，或先用给定颜色清除。
#[derive(Debug, Clone, Copy)] pub enum LoadOp { Clear(Color), Load }
/// 渲染通道结束的存储动作。
#[derive(Debug, Clone, Copy)] pub enum StoreOp { Store, Discard }
/// 单附件的 load+store 组合。
#[derive(Debug, Clone, Copy)] pub struct Operations { pub load: LoadOp, pub store: StoreOp }
/// 颜色附件：view 必填；resolve_target 仅多重采样解析目标用。
#[derive(Debug, Clone, Copy)] pub struct RenderPassColorAttachment<'a> { pub view: &'a crate::entry::TextureView, pub resolve_target: Option<&'a crate::entry::TextureView>, pub ops: Operations }
/// 深度/模板附件：depth_ops/stencil_ops 按附件实际所含宽高面给出。
#[derive(Debug, Clone, Copy)] pub struct RenderPassDepthStencilAttachment<'a> { pub view: &'a crate::entry::TextureView, pub depth_ops: Option<Operations>, pub stencil_ops: Option<Operations> }
/// 渲染通道描述：单目标约束——color_attachments 里 Some 必须恰好 1 个，违约=断言含 label、通道未建。
#[derive(Debug, Clone)] pub struct RenderPassDescriptor<'a> { pub label: Label<'a>, pub color_attachments: &'a [Option<RenderPassColorAttachment<'a>>], pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a>> }
/// 命令编码器创建输入：仅标签（创建本身不触驱动）。
#[derive(Debug, Clone, Copy)] pub struct CommandEncoderDescriptor<'a> { pub label: Label<'a> }

/// 管线布局创建输入：成员绑定组布局按 set 序给出；set 域 0-3、不得重复，建时校验。
#[derive(Debug, Clone)] pub struct PipelineLayoutDescriptor<'a> { pub label: Label<'a>, pub bind_group_layouts: &'a [&'a crate::entry::BindGroupLayout] }
/// 单顶点属性：格式 + 字节偏移（须 4 对齐）+ 着色器 location。
#[derive(Debug, Clone, Copy)] pub struct VertexAttribute { pub format: VertexFormat, pub offset: BufferAddress, pub shader_location: u32 }
/// 顶点缓冲布局：array_stride 须 4 字节对齐且不小于末属性末端。
#[derive(Debug, Clone)] pub struct VertexBufferLayout<'a> { pub array_stride: BufferAddress, pub step_mode: VertexStepMode, pub attributes: &'a [VertexAttribute] }
/// 顶点着色器段：模块 + 入口名 + 属性布局表。
#[derive(Debug, Clone)] pub struct VertexState<'a> { pub module: &'a crate::entry::ShaderModule, pub entry_point: Label<'a>, pub layouts: &'a [VertexBufferLayout<'a>] }
/// 混合等式的一个分量（因子对 + 运算）。
#[derive(Debug, Clone, Copy)] pub struct BlendComponent { pub operation: BlendOperation, pub color_factor: BlendFactor, pub alpha_factor: BlendFactor }
/// 混合运算。
#[derive(Debug, Clone, Copy)] pub enum BlendOperation { Add, Subtract, ReverseSubtract, Min, Max }
/// 混合因子子集。
#[derive(Debug, Clone, Copy)] pub enum BlendFactor { Zero, One, SrcAlpha, OneMinusSrcAlpha, DstAlpha, OneMinusDstAlpha }
/// 颜色/透明双通道混合状态。
#[derive(Debug, Clone, Copy)] pub struct BlendState { pub color: BlendComponent, pub alpha: BlendComponent }
/// 通道级颜色写掩码（位集合）。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct ColorWrites(pub u8);
/// 输出目标：格式 + 可选混合 + 写掩码；格式须与实际渲染附件一致。
#[derive(Debug, Clone, Copy)] pub struct ColorTargetState { pub format: TextureFormat, pub blend: Option<BlendState>, pub write_mask: ColorWrites }
/// 片元着色器段；fragment=None 表示无颜色写回。
#[derive(Debug, Clone)] pub struct FragmentState<'a> { pub module: &'a crate::entry::ShaderModule, pub entry_point: Label<'a>, pub targets: &'a [Option<ColorTargetState>] }
/// 朝向哨兵；公开侧经 CullMode 表达剔除与否。
#[derive(Debug, Clone, Copy)] pub enum Face { Undef, Front, Back }
/// 剔除模式：Some=剔除该朝向面，None=不剔除。
pub type CullMode = Option<Face>;
/// 正面绕序，默认 Ccw（与 wgpu 默认一致）。
#[derive(Debug, Clone, Copy, Default)] pub enum FrontFace { #[default] Ccw, Cw }
/// 面绘制模式；Fill 恒支持，Line/Point 按能力探测。
#[derive(Debug, Clone, Copy, Default)] pub enum PolygonMode { #[default] Fill, Line, Point }
/// 图元拓扑；TriangleList 恒支持，其余按能力探测。
#[derive(Debug, Clone, Copy, Default)] pub enum PrimitiveTopology { #[default] TriangleList, TriangleStrip, LineList, LineStrip, PointList }
/// 深度偏移参数；默认全 0。
#[derive(Debug, Clone, Copy)] pub struct DepthBiasState { pub constant: f32, pub slope: f32, pub clamp: f32 }
impl Default for DepthBiasState { fn default() -> Self { Self { constant: 0.0, slope: 0.0, clamp: 0.0 } } }
/// 模板单朝向状态；默认 Always 比较 + 全 Keep。
#[derive(Debug, Clone, Copy)] pub struct StencilFaceState { pub compare: CompareFunction, pub fail_op: StencilOperation, pub depth_fail_op: StencilOperation, pub pass_op: StencilOperation }
/// 模板操作。
#[derive(Debug, Clone, Copy)] pub enum StencilOperation { Keep, Zero, Replace, IncrementClamp, DecrementClamp, Invert, IncrementWrap, DecrementWrap }
impl Default for StencilFaceState { fn default() -> Self { Self { compare: CompareFunction::Always, fail_op: StencilOperation::Keep, depth_fail_op: StencilOperation::Keep, pass_op: StencilOperation::Keep } } }
/// 深度/模板固定功能状态；format 须与附件实际格式一致。
#[derive(Debug, Clone, Copy)] pub struct DepthStencilState { pub format: TextureFormat, pub depth_write_enabled: bool, pub depth_compare: CompareFunction, pub stencil: StencilState, pub bias: DepthBiasState }
/// 前后双面模板状态。
#[derive(Debug, Clone, Copy, Default)] pub struct StencilState { pub front: StencilFaceState, pub back: StencilFaceState }
/// 图元装配状态；默认：TriangleList 拓扑、CCW 正面、剔除背面、填充模式。
#[derive(Debug, Clone, Copy)] pub struct PrimitiveState { pub topology: PrimitiveTopology, pub strip_index_format: Option<IndexFormat>, pub front_face: FrontFace, pub cull_mode: CullMode, pub polygon_mode: PolygonMode, pub unclipped_depth: bool, pub conservative: bool }
impl Default for PrimitiveState { fn default() -> Self { Self { topology: PrimitiveTopology::TriangleList, strip_index_format: None, front_face: FrontFace::Ccw, cull_mode: Some(Face::Back), polygon_mode: PolygonMode::Fill, unclipped_depth: false, conservative: false } } }
/// 多重采样状态：count 与纹理 sample_count 同域（1/4）；mask 默认全 1。
#[derive(Debug, Clone, Copy)] pub struct MultisampleState { pub count: u32, pub mask: u64, pub alpha_to_coverage_enabled: bool }
impl Default for MultisampleState { fn default() -> Self { Self { count: 1, mask: !0, alpha_to_coverage_enabled: false } } }
/// 渲染管线创建输入。兼容子集硬约束：layout 必填（官方 auto-layout 不提供）；
/// 违约=断言含 label，无半建管线。
#[derive(Debug, Clone)] pub struct RenderPipelineDescriptor<'a> { pub label: Label<'a>, pub layout: &'a crate::entry::PipelineLayout, pub vertex: VertexState<'a>, pub fragment: Option<FragmentState<'a>>, pub depth_stencil: Option<DepthStencilState>, pub multisample: MultisampleState, pub primitive: PrimitiveState }

/// WGSL 源码包装（经 naga 转译为 GLSL ES 3.0 后使用）。
#[derive(Debug, Clone)] pub struct WgslSource<'a> { pub code: &'a str }
/// GLSL 目标版本词汇。
#[derive(Debug, Clone, Copy)] pub enum GlslVersion { Glsl460, Glsl450, Esgl310, Esgl300 }
impl Default for GlslVersion { fn default() -> Self { Self::Glsl450 } }
/// GLSL 单段源码。
#[derive(Debug, Clone)] pub struct GlslFragment<'a>  { pub code: &'a str }
/// GLSL 顶点段（与片元段同形）。
pub type GlslVertex<'a> = GlslFragment<'a>;
/// GLSL 源码：版本 + 顶点/片元段（至少其一）。
#[derive(Debug, Clone)] pub struct GlslSource<'a> { pub version: GlslVersion, pub fragment: Option<GlslFragment<'a>>, pub vertex: Option<GlslVertex<'a>> }
/// 着色器源双形态：WGSL 走转译，GLSL 走校验透传。
#[derive(Debug, Clone)] pub enum ShaderSource<'a> { Wgsl(WgslSource<'a>), Glsl(GlslSource<'a>) }
/// 着色器模块创建输入；源语法错误=转译期 codespan 报定位后断言，无半建模块。
#[derive(Debug, Clone)] pub struct ShaderModuleDescriptor<'a> { pub label: Label<'a>, pub source: ShaderSource<'a> }

/// 设备创建输入。required_features/required_limits 与 Adapter 能力快照全量对照：
/// 任一超出=Err(RequestDeviceError)——设备未建，Adapter 仍可反复重试。
#[derive(Debug, Clone)] pub struct DeviceDescriptor<'a> { pub label: Label<'a>, pub required_features: Features, pub required_limits: Limits, pub memory_hints: MemoryHints }
/// 呈现模式；configure 合法组合面仅 Fifo（Mailbox/Immediate 为词汇保留，能否用由能力查询面回报）。
#[derive(Debug, Clone, Copy)] pub enum PresentMode { Fifo, Mailbox, Immediate }
impl Default for PresentMode { fn default() -> Self { Self::Fifo } }
/// 合成透明模式；configure 合法组合面仅 Opaque。
#[derive(Debug, Clone, Copy)] pub enum CompositeAlphaMode { Opaque, PreMultiplied }
impl Default for CompositeAlphaMode { fn default() -> Self { Self::Opaque } }
/// 表面配置：宽高≥1；present_mode×alpha_mode 仅 (Fifo, Opaque) 合法——查询面可宽、合法面窄，
/// 违约=断言，不留半配置。
#[derive(Debug, Clone)] pub struct SurfaceConfiguration { pub width: u32, pub height: u32, pub format: TextureFormat, pub present_mode: PresentMode, pub alpha_mode: CompositeAlphaMode }



// —— 创建路径辅词汇（被描述符与查询面引用） ——
/// 动态绑定偏移，单位字节。
pub type DynamicOffset = u32;
/// 后端标签（本库仅 GL）。
#[derive(Debug, Clone, Copy)] pub enum Backend { Gl }
/// 适配器信息快照：字符串来自 EGL/GL 查询串聚合拼装；单卡面不枚举多设备。
#[derive(Debug, Clone)] pub struct AdapterInfo { pub backend: Backend, pub device_type: DeviceType, pub vendor: String, pub name: String, pub driver_info: String }
/// 降级能力位集合（GL 路=静态汇总表输出，非逐项真探）。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct DownlevelFlags(pub u64);
/// 降级能力快照：标志位 + 对应生效上限。
#[derive(Debug, Clone)] pub struct DownlevelCapabilities { pub flags: DownlevelFlags, pub limits: Limits }
/// 单格式特性位集合。
#[repr(transparent)] #[derive(Debug, Clone, Copy, Default)] pub struct TextureFormatFeatureFlags(pub u64);
/// 单格式能力快照：允许用途交集 + 特性位。
#[derive(Debug, Clone)] pub struct TextureFormatFeatures { pub allowed_usages: TextureUsages, pub flags: TextureFormatFeatureFlags }
/// 纹素拷贝目标具例：纹理 + mip 层 + 起点 + 宽高面。
#[derive(Debug, Clone, Copy)] pub struct ImageCopyTexture<'a> { pub texture: &'a crate::entry::Texture, pub mip_level: u32, pub origin: Origin3d, pub aspect: TextureAspect }
/// 拷贝源内存布局：offset 须 4 字节对齐；bytes_per_row 给定时须 256 倍数；行/层数缺省按 extent 推定。
/// 违约后果见 Queue::write_texture 契约（debug=断言 / release=记录留痕、由驱动兜底，不谎报）。
#[derive(Debug, Clone, Copy)] pub struct ImageDataLayout { pub offset: BufferAddress, pub bytes_per_row: Option<NonZeroU32>, pub rows_per_image: Option<NonZeroU32> }
/// 多维纹理数据在内存中的排布顺序。
#[derive(Debug, Clone, Copy)] pub enum TextureDataOrder { LayerMajor, MipMajor }
/// 适配器选择输入：compatible_surface 仅作相容性校验；power_preference/force_fallback 收录但无多卡路由语义。
#[derive(Debug, Clone, Copy)] pub struct RequestAdapterOptions<'a> { pub compatible_surface: Option<&'a crate::surface::Surface<'a>>, pub force_fallback_adapter: bool, pub power_preference: PowerPreference }
