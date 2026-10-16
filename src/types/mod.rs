// 设计：pi-wgpu-rebuild · 形制对齐改：desc-align-wgpu25
//! types · 公开 API 词汇面：全为纯值/纯数据，无运行态、不触驱动。
//! 不变量：合法性不在本面判定——一律由 entry 创建边界一次性校验（违约=断言，报字段与 desc.label）。
//! 所有权：复制语义，随宿主对象存续。本面全部 @public。

/// 对象可选标签；仅用于断言与 trace 报错时定位来源对象。
pub type Label<'a> = Option<&'a str>;

use std::num::NonZeroU16;
use std::num::NonZeroU32;

// 词汇面终态映射（对齐批）：上游 wgpu-types 25.x 为唯一真相源，整体 star re-use——
// 等价官方 wgpu api 层做法；pi 特有面仅保留引用式 descriptor/别名类型。
pub use wgpu_types::*;

// —— 官方 api 层同款别名收口（wgt 泛位 → 'a 单参面，上游 wgpu::api 同法） ——
pub type BufferDescriptor<'a> = wgpu_types::BufferDescriptor<Label<'a>>;
pub type TextureDescriptor<'a> = wgpu_types::TextureDescriptor<Label<'a>, &'a [TextureFormat]>;
pub type SamplerDescriptor<'a> = wgpu_types::SamplerDescriptor<Label<'a>>;
pub type DeviceDescriptor<'a> = wgpu_types::DeviceDescriptor<Label<'a>>;
pub type CommandEncoderDescriptor<'a> = wgpu_types::CommandEncoderDescriptor<Label<'a>>;
pub type SurfaceConfiguration = wgpu_types::SurfaceConfiguration<Vec<TextureFormat>>;
pub type TextureViewDescriptor<'a> = wgpu_types::TextureViewDescriptor<Label<'a>>;
pub type TexelCopyTextureInfo<'a> = wgpu_types::TexelCopyTextureInfo<&'a crate::entry::Texture>;
pub type ImageCopyTexture<'a> = wgpu_types::TexelCopyTextureInfo<&'a crate::entry::Texture>;

/// 位集合 `|` 算子生成宏：位并集（组合两个 flags 的位）——所有位集合壳共用；不校验（超集位由消费方校验断言）。`&`/contains 等其余算子后续批按需补。
macro_rules! flag_bitor {
    ($t:ty) => {
        impl core::ops::BitOr for $t {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }
    };
}

/// Instance 创建输入。默认 backends=GL；填其他位只会在创建校验处失败，不会多出后端。
#[derive(Debug, Clone, Copy)]
pub struct InstanceDescriptor<'a> {
    pub name: &'a str,
    pub flags: InstanceFlags,
    pub backends: Backends,
}
impl<'a> Default for InstanceDescriptor<'a> {
    fn default() -> Self {
        Self {
            name: "",
            flags: InstanceFlags::default(),
            backends: Backends::GL,
        }
    }
}

/// 绑定组布局创建输入；槽位归组在布局面声明（set 域 0-3），条目违约=断言含 label。
#[derive(Debug, Clone)]
pub struct BindGroupLayoutDescriptor<'a> {
    pub label: Label<'a>,
    pub entries: &'a [BindGroupLayoutEntry],
}
/// 一次绑定的缓冲具例：offset 须 4 字节对齐；size=None 取到缓冲末尾；
/// offset+size 不得超过缓冲实际大小；引用死句柄=断言。
#[derive(Debug, Clone, Copy)]
pub struct BufferBinding<'a> {
    pub buffer: &'a crate::entry::Buffer,
    pub offset: BufferAddress,
    pub size: Option<BufferSize>,
}
/// 绑定资源三选一（缓冲/采样器/纹理视图），必须是活引用。
#[derive(Debug, Clone, Copy)]
pub enum BindingResource<'a> {
    Buffer(BufferBinding<'a>),
    Sampler(&'a crate::entry::Sampler),
    TextureView(&'a crate::entry::TextureView),
}
/// 绑定组单条具例：binding 号须与布局条目一致。
#[derive(Debug, Clone)]
pub struct BindGroupEntry<'a> {
    pub binding: u32,
    pub resource: BindingResource<'a>,
}
/// 绑定组创建输入：对给定布局逐一供给资源；条目缺失/多余/类型对不上=断言含 label，未建组。
#[derive(Debug, Clone)]
pub struct BindGroupDescriptor<'a> {
    pub label: Label<'a>,
    pub layout: &'a crate::entry::BindGroupLayout,
    pub entries: &'a [BindGroupEntry<'a>],
}

/// 颜色附件（参数名照抄上游 'tex：视图借用维度独立于描述符收集切片）。
/// view 必填；resolve_target 仅多重采样解析目标用。
#[derive(Debug, Clone, Copy)]
pub struct RenderPassColorAttachment<'tex> {
    pub view: &'tex crate::entry::TextureView,
    pub resolve_target: Option<&'tex crate::entry::TextureView>,
    pub ops: Operations<Color>,
}
/// 深度/模板附件（'tex 同颜色附件口径）：depth_ops/stencil_ops 按附件实际所含宽高面给出。
#[derive(Debug, Clone, Copy)]
pub struct RenderPassDepthStencilAttachment<'tex> {
    pub view: &'tex crate::entry::TextureView,
    pub depth_ops: Option<Operations<f32>>,
    pub stencil_ops: Option<Operations<u32>>,
}
/// 通道时间戳写入安排（形制对齐上游词位）：本层 GL 无对应实现且查询集无构造面——
/// timestamp_writes=Some 恒违约断言（短码含 label，形宽法窄）。
#[derive(Debug, Clone)]
pub struct RenderPassTimestampWrites<'a> {
    pub query_set: &'a crate::entry::QuerySet,
    pub beginning_of_pass_write_index: Option<u32>,
    pub end_of_pass_write_index: Option<u32>,
}
/// 渲染通道描述（形照 25.0.2 五字段）：单目标约束——color_attachments 里 Some 必须恰好 1 个，
/// 违约=断言含 label、通道未建；timestamp_writes/occlusion_query_set=Some 恒拒止（GL 路无对应物）。
#[derive(Debug, Clone, Default)]
pub struct RenderPassDescriptor<'a> {
    pub label: Label<'a>,
    pub color_attachments: &'a [Option<RenderPassColorAttachment<'a>>],
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a>>,
    pub timestamp_writes: Option<RenderPassTimestampWrites<'a>>,
    pub occlusion_query_set: Option<&'a crate::entry::QuerySet>,
}

/// 管线布局创建输入：成员绑定组布局按 set 序给出；set 域 0-3、不得重复，建时校验。
/// 形制对齐 wgpu 25.0.2：推送范围字段在场；本层 GL 路无对应物=恒拒止——
/// 非空为违约（断言短码含 label；拒止分支芯 TODO-DECL 后续批）。
#[derive(Debug, Clone, Default)]
pub struct PipelineLayoutDescriptor<'a> {
    pub label: Label<'a>,
    pub bind_group_layouts: &'a [&'a crate::entry::BindGroupLayout],
    pub push_constant_ranges: &'a [PushConstantRange],
}
/// 顶点缓冲布局：array_stride 须 4 字节对齐且不小于末属性末端。
#[derive(Debug, Clone)]
pub struct VertexBufferLayout<'a> {
    pub array_stride: BufferAddress,
    pub step_mode: VertexStepMode,
    pub attributes: &'a [VertexAttribute],
}
/// 每阶段管线编译选项（形状照抄上游 25.0.2）：本层仅接受默认值——
/// naga 转译管道不消费覆盖常量，非默认=违约（短码断言，芯 TODO-DECL 后续批）。
#[derive(Debug, Clone)]
pub struct PipelineCompilationOptions<'a> {
    pub constants: &'a [(&'a str, f64)],
    pub zero_initialize_workgroup_memory: bool,
}
impl Default for PipelineCompilationOptions<'_> {
    fn default() -> Self {
        Self {
            constants: &[],
            zero_initialize_workgroup_memory: true,
        }
    }
}
/// 顶点着色器段：模块 + 入口名 + 编译选项 + 属性布局表（形制对齐上游）。
/// entry_point=None=该 stage 唯一入口自动选（实现批已落地；缺失/歧义=出码失败断言）；
/// 非默认编译选项=违约断言（短码含 label）。
#[derive(Debug, Clone)]
pub struct VertexState<'a> {
    pub module: &'a crate::entry::ShaderModule,
    pub entry_point: Option<&'a str>,
    pub compilation_options: PipelineCompilationOptions<'a>,
    pub buffers: &'a [VertexBufferLayout<'a>],
}
/// 片元着色器段；fragment=None 表示无颜色写回（形制对齐上游，违约口径同 VertexState）。
#[derive(Debug, Clone)]
pub struct FragmentState<'a> {
    pub module: &'a crate::entry::ShaderModule,
    pub entry_point: Option<&'a str>,
    pub compilation_options: PipelineCompilationOptions<'a>,
    pub targets: &'a [Option<ColorTargetState>],
}
/// 剔除模式：Some=剔除该朝向面，None=不剔除。
pub type CullMode = Option<Face>;
// 浮点按位比较（照抄上游 manual 三件套impl，字段正名 slope_scale 同上游）

/// 渲染管线创建输入。形全量照抄 wgpu 25.0.2；合法域收窄（形宽法窄）——
/// layout=None（auto-layout 不提供）、multiview=Some、cache=Some、非默认编译选项均为违约：
/// 断言短码含 label，无半建管线；entry_point=None=唯一入口自动选（实现批落地，缺失/歧义=出码失败断言）。
#[derive(Debug, Clone)]
pub struct RenderPipelineDescriptor<'a> {
    pub label: Label<'a>,
    pub layout: Option<&'a crate::entry::PipelineLayout>,
    pub vertex: VertexState<'a>,
    pub primitive: PrimitiveState,
    pub depth_stencil: Option<DepthStencilState>,
    pub multisample: MultisampleState,
    pub fragment: Option<FragmentState<'a>>,
    pub multiview: Option<NonZeroU32>,
    pub cache: Option<&'a crate::entry::PipelineCache>,
}

/// GLSL 段的归属阶段（对位上游 naga::ShaderStage）：本层不泄 naga，公开侧用本地词汇；
/// 与 naga 侧的等价映射已由转译芯 From 收敛实现（desc-align-wgpu25 实现批）。
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ShaderStage {
//     Vertex,
//     Fragment,
// }
pub use naga::ShaderStage;

/// 着色器源（形制对齐 wgpu 25.0.2 的 Wgsl/Glsl 在场变体；SpirV/Naga 属 feature 死面不入，
/// 变体形制对齐 + 本枚举 #[non_exhaustive] 保留后续扩充余地）。WGSL 走 naga 转译为 GLSL ES 3.0；
/// GLSL 按阶段单源，目标版本由源码 #version 头自承载（校验透传）。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShaderSource<'a> {
    Wgsl(std::borrow::Cow<'a, str>),
    Glsl {
        shader: std::borrow::Cow<'a, str>,
        stage: ShaderStage,
        defines: &'a [(&'a str, &'a str)],
    },
}
/// 着色器模块创建输入；源语法错误=转译期 codespan 报定位后断言，无半建模块。
#[derive(Debug, Clone)]
pub struct ShaderModuleDescriptor<'a> {
    pub label: Label<'a>,
    pub source: ShaderSource<'a>,
}
