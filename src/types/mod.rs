// 设计：pi-wgpu-rebuild · 形制对齐改：desc-align-wgpu25
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

use std::num::NonZeroU16;
use std::num::NonZeroU32;

/// 缓冲字节大小的正值（形制对齐 wgpu 25.0.2：官方已废包装结构，直接别名 NonZeroU64；
/// `BufferSize::new(v)` 即 `NonZeroU64::new` 返 Option，官方调用式原样可编译）。
pub type BufferSize = core::num::NonZeroU64;
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

/// 后端选位集合（位标记）。本库 GL 单面：GL=1 是唯一有效位，EMPTY=0 表示不选后端；其余位恒不可用。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Backends(pub u32);
impl Backends {
    pub const GL: Self = Self(1);
    pub const EMPTY: Self = Self(0);
}
flag_bitor!(Backends);

/// Instance 创建输入。默认 backends=GL；填其他位只会在创建校验处失败，不会多出后端。
#[derive(Debug, Clone, Copy)]
pub struct InstanceDescriptor {
    pub backends: Backends,
}
impl Default for InstanceDescriptor {
    fn default() -> Self {
        Self {
            backends: Backends::GL,
        }
    }
}

/// 适配器类型标签：仅随 AdapterInfo 上报展示；单卡面，不参与任何选择路由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Cpu,
    IntegratedGpu,
    DiscreteGpu,
    VirtualGpu,
    Other,
}
/// 功耗偏好：兼容收录的提示词；EGL 单卡面记录但不改变选路（无多卡可选）。
#[derive(Debug, Clone, Copy, Default)]
pub enum PowerPreference {
    #[default]
    Default,
    LowPower,
    HighPerformance,
}
/// 内存倾向提示；收录词汇，无驱动侧映射行为。
#[derive(Debug, Clone, Copy)]
pub enum MemoryHints {
    Performance,
    Memory,
    Auto,
}
impl Default for MemoryHints {
    fn default() -> Self {
        Self::Auto
    }
}

/// 能力位集合（u64 位标记）。有效位由 caps 探测填充；探测不到的能力恒 0，不虚报。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Features(pub u64);
flag_bitor!(Features);

/// 资源上限全表（字段名与序照抄 wgpu 25.0.2，36 项）。request_device 拿它与能力快照逐字段对照，
/// 任一超出即失败。Default=静态下限拍板（宁低报不虚高；0=本层无该面，真机能力以 caps 快照为准）：
/// 尺寸/绑定类取 ES/WebGL2 spec 保底与槽表实际容量（ubo/sampler 各 16 共享记 16/16）；
/// 自定义旧字段 max_uniform_buffer_bindings 退役，语义对位 max_uniform_buffers_per_shader_stage=16。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Limits {
    pub max_texture_dimension_1d: u32,
    pub max_texture_dimension_2d: u32,
    pub max_texture_dimension_3d: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
    pub max_bindings_per_bind_group: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    pub max_sampled_textures_per_shader_stage: u32,
    pub max_samplers_per_shader_stage: u32,
    pub max_storage_buffers_per_shader_stage: u32,
    pub max_storage_textures_per_shader_stage: u32,
    pub max_uniform_buffers_per_shader_stage: u32,
    pub max_binding_array_elements_per_shader_stage: u32,
    pub max_binding_array_sampler_elements_per_shader_stage: u32,
    pub max_uniform_buffer_binding_size: u32,
    pub max_storage_buffer_binding_size: u32,
    pub max_vertex_buffers: u32,
    pub max_buffer_size: BufferAddress,
    pub max_vertex_attributes: u32,
    pub max_vertex_buffer_array_stride: u32,
    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub max_inter_stage_shader_components: u32,
    pub max_color_attachments: u32,
    pub max_color_attachment_bytes_per_sample: u32,
    pub max_compute_workgroup_storage_size: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub max_compute_workgroups_per_dimension: u32,
    pub min_subgroup_size: u32,
    pub max_subgroup_size: u32,
    pub max_push_constant_size: u32,
    pub max_non_sampler_bindings: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_texture_dimension_1d: 2048,
            max_texture_dimension_2d: 4096,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 256,
            max_bind_groups: 4,
            max_bindings_per_bind_group: 32,
            max_dynamic_uniform_buffers_per_pipeline_layout: 12,
            max_dynamic_storage_buffers_per_pipeline_layout: 0,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 0,
            max_storage_textures_per_shader_stage: 0,
            max_uniform_buffers_per_shader_stage: 16,
            max_binding_array_elements_per_shader_stage: 0,
            max_binding_array_sampler_elements_per_shader_stage: 0,
            max_uniform_buffer_binding_size: 16384,
            max_storage_buffer_binding_size: 0,
            max_vertex_buffers: 16,
            max_buffer_size: 1 << 28,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_inter_stage_shader_components: 60,
            max_color_attachments: 4,
            max_color_attachment_bytes_per_sample: 128,
            max_compute_workgroup_storage_size: 0,
            max_compute_invocations_per_workgroup: 0,
            max_compute_workgroup_size_x: 0,
            max_compute_workgroup_size_y: 0,
            max_compute_workgroup_size_z: 0,
            max_compute_workgroups_per_dimension: 0,
            min_subgroup_size: 0,
            max_subgroup_size: 0,
            max_push_constant_size: 0,
            max_non_sampler_bindings: 32,
        }
    }
}

/// 纹理格式词汇全表（变体名与序照抄 wgpu 25.0.2，75 项；该版本面无 Astc 族，对账 checkout 为准）。
/// 形宽法窄：词汇全表在场，实际支持=转换表条目 ∩ 逐格式能力门控——表无条目（压缩族与未接非色族）
/// 在创建校验处断言拒止（报格式名+label），特性查询面返回不支持。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,
    R16Uint,
    R16Sint,
    R16Unorm,
    R16Snorm,
    R16Float,
    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,
    R32Uint,
    R32Sint,
    R32Float,
    Rg16Uint,
    Rg16Sint,
    Rg16Unorm,
    Rg16Snorm,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgb9e5Ufloat,
    Rgb10a2Uint,
    Rgb10a2Unorm,
    Rg11b10Ufloat,
    R64Uint,
    Rg32Uint,
    Rg32Sint,
    Rg32Float,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Float,
    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,
    Stencil8,
    Depth16Unorm,
    Depth24Plus,
    Depth24PlusStencil8,
    Depth32Float,
    Depth32FloatStencil8,
    NV12,
    Bc1RgbaUnorm,
    Bc1RgbaUnormSrgb,
    Bc2RgbaUnorm,
    Bc2RgbaUnormSrgb,
    Bc3RgbaUnorm,
    Bc3RgbaUnormSrgb,
    Bc4RUnorm,
    Bc4RSnorm,
    Bc5RgUnorm,
    Bc5RgSnorm,
    Bc6hRgbUfloat,
    Bc6hRgbFloat,
    Bc7RgbaUnorm,
    Bc7RgbaUnormSrgb,
    Etc2Rgb8Unorm,
    Etc2Rgb8UnormSrgb,
    Etc2Rgb8A1Unorm,
    Etc2Rgb8A1UnormSrgb,
    Etc2Rgba8Unorm,
    Etc2Rgba8UnormSrgb,
    EacR11Unorm,
    EacR11Snorm,
    EacRg11Unorm,
    EacRg11Snorm,
}
impl TextureFormat {
    /// 压缩块宽（上游同式）。 @public
    #[must_use]
    pub fn block_width(&self) -> u32 {
        self.block_dimensions().0
    }
    /// 压缩块高（上游同式）。 @public
    #[must_use]
    pub fn block_height(&self) -> u32 {
        self.block_dimensions().1
    }
    /// 压缩块粒度（上游同表；本层无 Astc 分支——25.0.2 词汇面无该族，压缩族接表归能力批）。 @public
    #[must_use]
    pub fn block_dimensions(&self) -> (u32, u32) {
        match *self {
            Self::R8Unorm
            | Self::R8Snorm
            | Self::R8Uint
            | Self::R8Sint
            | Self::R16Uint
            | Self::R16Sint
            | Self::R16Unorm
            | Self::R16Snorm
            | Self::R16Float
            | Self::Rg8Unorm
            | Self::Rg8Snorm
            | Self::Rg8Uint
            | Self::Rg8Sint
            | Self::R32Uint
            | Self::R32Sint
            | Self::R32Float
            | Self::Rg16Uint
            | Self::Rg16Sint
            | Self::Rg16Unorm
            | Self::Rg16Snorm
            | Self::Rg16Float
            | Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Rgba8Snorm
            | Self::Rgba8Uint
            | Self::Rgba8Sint
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::Rgb9e5Ufloat
            | Self::Rgb10a2Uint
            | Self::Rgb10a2Unorm
            | Self::Rg11b10Ufloat
            | Self::R64Uint
            | Self::Rg32Uint
            | Self::Rg32Sint
            | Self::Rg32Float
            | Self::Rgba16Uint
            | Self::Rgba16Sint
            | Self::Rgba16Unorm
            | Self::Rgba16Snorm
            | Self::Rgba16Float
            | Self::Rgba32Uint
            | Self::Rgba32Sint
            | Self::Rgba32Float
            | Self::Stencil8
            | Self::Depth16Unorm
            | Self::Depth24Plus
            | Self::Depth24PlusStencil8
            | Self::Depth32Float
            | Self::Depth32FloatStencil8
            | Self::NV12 => (1, 1),
            Self::Bc1RgbaUnorm
            | Self::Bc1RgbaUnormSrgb
            | Self::Bc2RgbaUnorm
            | Self::Bc2RgbaUnormSrgb
            | Self::Bc3RgbaUnorm
            | Self::Bc3RgbaUnormSrgb
            | Self::Bc4RUnorm
            | Self::Bc4RSnorm
            | Self::Bc5RgUnorm
            | Self::Bc5RgSnorm
            | Self::Bc6hRgbUfloat
            | Self::Bc6hRgbFloat
            | Self::Bc7RgbaUnorm
            | Self::Bc7RgbaUnormSrgb => (4, 4),
            Self::Etc2Rgb8Unorm
            | Self::Etc2Rgb8UnormSrgb
            | Self::Etc2Rgb8A1Unorm
            | Self::Etc2Rgb8A1UnormSrgb
            | Self::Etc2Rgba8Unorm
            | Self::Etc2Rgba8UnormSrgb
            | Self::EacR11Unorm
            | Self::EacR11Snorm
            | Self::EacRg11Unorm
            | Self::EacRg11Snorm => (4, 4),
        }
    }
}
/// 纹理用途位集合（组合）；支持位之外在 create_texture 校验处断言。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureUsages(pub u32);
impl TextureUsages {
    pub const COPY_DST: Self = Self(1);
    pub const RENDER_ATTACHMENT: Self = Self(2);
    pub const TEXTURE_BINDING: Self = Self(4);
}
flag_bitor!(TextureUsages);
/// 缓冲用途位集合（组合）；空组合与非支持位在 create_buffer 校验处断言。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferUsages(pub u32);
impl BufferUsages {
    pub const VERTEX: Self = Self(1);
    pub const INDEX: Self = Self(2);
    pub const UNIFORM: Self = Self(4);
    pub const COPY_DST: Self = Self(8);
}
flag_bitor!(BufferUsages);
/// 绑定可见阶段位集合；空集在 create_bind_group_layout 校验处断言。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ShaderStages(pub u32);
impl ShaderStages {
    pub const VERTEX: Self = Self(1);
    pub const FRAGMENT: Self = Self(2);
}
flag_bitor!(ShaderStages);

/// 尺寸三元组，单位纹素/元素；用它的场合要求宽高≥1（0 值在创建校验处断言）。
#[derive(Debug, Clone, Copy, Default)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}
impl Extent3d {
    /// 格式的内存物理尺寸（块粒度向上取整；照抄上游——压缩纹理上传/mip 换算用）。
    #[must_use]
    pub fn physical_size(&self, format: TextureFormat) -> Self {
        let (block_width, block_height) = format.block_dimensions();
        let width = self.width.div_ceil(block_width) * block_width;
        let height = self.height.div_ceil(block_height) * block_height;
        Self {
            width,
            height,
            depth_or_array_layers: self.depth_or_array_layers,
        }
    }
    /// 最大 mip 级数（按维度视深度；上游同式）。
    #[must_use]
    pub fn max_mips(&self, dim: TextureDimension) -> u32 {
        match dim {
            TextureDimension::D1 => 1,
            TextureDimension::D2 => 32 - self.width.max(self.height).leading_zeros(),
            TextureDimension::D3 => {
                32 - self
                    .width
                    .max(self.height.max(self.depth_or_array_layers))
                    .leading_zeros()
            }
        }
    }
    /// 指定 mip 级逻辑尺寸（不含内存对齐补块；上游同式）。
    #[must_use]
    pub fn mip_level_size(&self, level: u32, dim: TextureDimension) -> Self {
        Self {
            width: u32::max(1, self.width >> level),
            height: match dim {
                TextureDimension::D1 => 1,
                _ => u32::max(1, self.height >> level),
            },
            depth_or_array_layers: match dim {
                TextureDimension::D1 => 1,
                TextureDimension::D2 => self.depth_or_array_layers,
                TextureDimension::D3 => u32::max(1, self.depth_or_array_layers >> level),
            },
        }
    }
}
/// 拷贝起点，单位纹素坐标。
#[derive(Debug, Clone, Copy, Default)]
pub struct Origin3d {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl Origin3d {
    /// 原点常量（上游同形；as_image_copy 等缺省构造消费）。 @public
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };
}
/// 纹理维度；本面收录词汇，实际放行按格式能力表在创建时校验。
#[derive(Debug, Clone, Copy, Default)]
pub enum TextureDimension {
    #[default]
    D2,
    D1,
    D3,
}
/// 纹理视图维度。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextureViewDimension {
    #[default]
    D2,
    D2Array,
    Cube,
    CubeArray,
    D1,
    D3,
}
/// 视图取用的宽高面（深度/模板分离用）。
#[derive(Debug, Clone, Copy, Default)]
pub enum TextureAspect {
    #[default]
    All,
    Stencil,
    DepthOnly,
}
/// 索引格式；绘制时索引缓冲值宽必须与之一致，违约=断言。
#[derive(Debug, Clone, Copy)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}
/// 顶点属性格式子集；与 offset 组合须满足 4 字节对齐约束。
#[derive(Debug, Clone, Copy)]
pub enum VertexFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Uint8x4,
    Sint16x2,
}
/// 顶点步进模式：逐顶点或逐实例。
#[derive(Debug, Clone, Copy, Default)]
pub enum VertexStepMode {
    #[default]
    Vertex,
    Instance,
}

/// 缓冲绑定类型：仅 Uniform——WebGL2 无 storage 缓冲，兼容子集按能力交集建模。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferBindingType {
    Uniform,
}
/// 采样器绑定类型；Comparison 要求采样器带 compare 配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerBindingType {
    Filtering,
    NonFiltering,
    Comparison,
}
/// 纹理取样数值类型；filterable=false 时绑定它的采样器必须是 Nearest 系。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureSampleType {
    Float { filterable: bool },
    Sint,
    Uint,
}
/// 绑定条目类型三形。has_dynamic_offset：与官方同名同形同义（动态偏移经 set_bind_group 传入，无差异）；
/// min_binding_size=None 表示整段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingType {
    Buffer {
        ty: BufferBindingType,
        has_dynamic_offset: bool,
        min_binding_size: Option<BufferSize>,
    },
    Sampler(SamplerBindingType),
    Texture {
        sample_type: TextureSampleType,
        view_dimension: TextureViewDimension,
        multisampled: bool,
    },
}
/// 绑定组布局条目：槽号 + 可见阶段 + 数组长度（None=单个）+ 类型。
#[derive(Debug, Clone)]
pub struct BindGroupLayoutEntry {
    pub binding: u32,
    pub visibility: ShaderStages,
    pub count: Option<NonZeroU32>,
    pub ty: BindingType,
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
/// 缓冲创建输入。size 须>0 且 4 字节对齐；usage 非空且在支持集内；
/// mapped_at_creation 仅作兼容收录——本库无持久映射语义，写入一律走 Queue::write_buffer 即时生效面。
#[derive(Debug, Clone)]
pub struct BufferDescriptor<'a> {
    pub label: Label<'a>,
    pub usage: BufferUsages,
    pub size: BufferAddress,
    pub mapped_at_creation: bool,
    pub memory_hints: MemoryHints,
}
/// 纹理创建输入。约束：宽高≥1；mip_level_count≥1；sample_count 仅 1 或 4；
/// view_formats 须为主格式同族；违约=断言含 label，无半建纹理。
#[derive(Debug, Clone)]
pub struct TextureDescriptor<'a> {
    pub label: Label<'a>,
    pub usage: TextureUsages,
    pub dimension: TextureDimension,
    pub size: Extent3d,
    pub format: TextureFormat,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub view_formats: &'a [TextureFormat],
}
/// 纹理视图描述（形制照抄 wgpu 25.0.2 九字段）。None=按宿主纹理取默认（上游语义）：
/// format=宿主格式、dimension=按宿主维度/层数推定、usage=宿主全量、count=None=余量全取。
/// 本层「取整张」：base/count 切片暂不生效（视图目标覆盖逻辑不变），越界切片接表归实现批。
/// 全 Default 即「取整张」——TODO-DECL 便捷构造由 Default derive 闭合。
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureViewDescriptor<'a> {
    pub label: Label<'a>,
    pub format: Option<TextureFormat>,
    pub dimension: Option<TextureViewDimension>,
    pub usage: Option<crate::types::TextureUsages>,
    pub aspect: TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

/// 过滤模式（Default=Nearest，上游同形）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FilterMode {
    #[default]
    Nearest,
    Linear,
}
/// 寻址模式（u/v/w 三轴独立配置；Default=ClampToEdge，上游同形）。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AddressMode {
    #[default]
    ClampToEdge,
    Repeat,
    MirroredRepeat,
}
/// 边界颜色候选（25.0.2 全四变体照抄；旧代 Float 色形已退役不加）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerBorderColor {
    TransparentBlack,
    OpaqueBlack,
    OpaqueWhite,
    /// 无 alpha 纹理按 OpaqueBlack（Metal 侧差异）；需 ADDRESS_MODE_CLAMP_TO_ZERO 能力，本层无位恒拒。
    Zero,
}
/// 深度/模板比较函数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}
/// 采样器创建输入（形制照抄 25.0.2 十二字段序）：anisotropy_clamp 须 ≥1，非 1 时三过滤全 Linear
/// （上游合同；本层旧「仅 2 的幂集」校验随形退役，GL 驱动自钳制）；compare=Some 即 comparison 采样器；
/// lod 夹值 [lod,32] 语义收录、GL 消费接表归实现批。越界=断言含 label，无半建。
#[derive(Debug, Clone, PartialEq)]
pub struct SamplerDescriptor<'a> {
    pub label: Label<'a>,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: FilterMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
    pub border_color: Option<SamplerBorderColor>,
}
impl Default for SamplerDescriptor<'_> {
    fn default() -> Self {
        Self {
            label: Default::default(),
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: Default::default(),
            min_filter: Default::default(),
            mipmap_filter: Default::default(),
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        }
    }
}

/// RGBA 颜色，各通道 f32；清屏/混合语义按驱动解释（0.0-1.0）。
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
/// 渲染通道开始的加载动作：沿用既有内容，或先用给定值清除（清除值随附件通道类型参数化，形照上游）。
#[derive(Debug, Clone, Copy)]
pub enum LoadOp<T> {
    Clear(T),
    Load,
}
/// 渲染通道结束的存储动作。
#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Store,
    Discard,
}
/// 单附件的 load+store 组合（形照上游通用参：颜色 Operations<Color>、深度 Operations<f32>、模板 Operations<u32>）。
#[derive(Debug, Clone, Copy)]
pub struct Operations<T> {
    pub load: LoadOp<T>,
    pub store: StoreOp,
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
/// 命令编码器创建输入：仅标签（创建本身不触驱动）。
#[derive(Debug, Clone, Copy)]
pub struct CommandEncoderDescriptor<'a> {
    pub label: Label<'a>,
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
/// 推送常量范围（上游词汇形状占位）：每阶段至多一个范围，一个范围可服务多阶段。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantRange {
    pub stages: ShaderStages,
    pub range: std::ops::Range<u32>,
}
/// 单顶点属性：格式 + 字节偏移（须 4 对齐）+ 着色器 location。
#[derive(Debug, Clone, Copy)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: BufferAddress,
    pub shader_location: u32,
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
/// 混合等式的一个分量（因子对 + 运算）。
#[derive(Debug, Clone, Copy)]
pub struct BlendComponent {
    pub operation: BlendOperation,
    pub color_factor: BlendFactor,
    pub alpha_factor: BlendFactor,
}
/// 混合运算。
#[derive(Debug, Clone, Copy)]
pub enum BlendOperation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}
/// 混合因子子集。
#[derive(Debug, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}
/// 颜色/透明双通道混合状态。
#[derive(Debug, Clone, Copy)]
pub struct BlendState {
    pub color: BlendComponent,
    pub alpha: BlendComponent,
}
/// 通道级颜色写掩码（位集合）。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ColorWrites(pub u8);
impl ColorWrites {
    pub const R: Self = Self(1);
    pub const G: Self = Self(2);
    pub const B: Self = Self(4);
    pub const A: Self = Self(8);
    pub const ALL: Self = Self(15);
}
flag_bitor!(ColorWrites);
/// 输出目标：格式 + 可选混合 + 写掩码；格式须与实际渲染附件一致。
#[derive(Debug, Clone, Copy)]
pub struct ColorTargetState {
    pub format: TextureFormat,
    pub blend: Option<BlendState>,
    pub write_mask: ColorWrites,
}
impl From<TextureFormat> for ColorTargetState {
    fn from(format: TextureFormat) -> Self {
        Self {
            format,
            blend: None,
            write_mask: ColorWrites::default(),
        }
    }
}
/// 片元着色器段；fragment=None 表示无颜色写回（形制对齐上游，违约口径同 VertexState）。
#[derive(Debug, Clone)]
pub struct FragmentState<'a> {
    pub module: &'a crate::entry::ShaderModule,
    pub entry_point: Option<&'a str>,
    pub compilation_options: PipelineCompilationOptions<'a>,
    pub targets: &'a [Option<ColorTargetState>],
}
/// 朝向哨兵；公开侧经 CullMode 表达剔除与否。
#[derive(Debug, Clone, Copy)]
pub enum Face {
    Undef,
    Front,
    Back,
}
/// 剔除模式：Some=剔除该朝向面，None=不剔除。
pub type CullMode = Option<Face>;
/// 正面绕序，默认 Ccw（与 wgpu 默认一致）。
#[derive(Debug, Clone, Copy, Default)]
pub enum FrontFace {
    #[default]
    Ccw,
    Cw,
}
/// 面绘制模式；Fill 恒支持，Line/Point 按能力探测。
#[derive(Debug, Clone, Copy, Default)]
pub enum PolygonMode {
    #[default]
    Fill,
    Line,
    Point,
}
/// 图元拓扑；TriangleList 恒支持，其余按能力探测。
#[derive(Debug, Clone, Copy, Default)]
pub enum PrimitiveTopology {
    #[default]
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
    PointList,
}
/// 深度偏移参数；默认全 0。
#[derive(Debug, Clone, Copy)]
pub struct DepthBiasState {
    pub constant: f32,
    pub slope: f32,
    pub clamp: f32,
}
impl Default for DepthBiasState {
    fn default() -> Self {
        Self {
            constant: 0.0,
            slope: 0.0,
            clamp: 0.0,
        }
    }
}
/// 模板单朝向状态；默认 Always 比较 + 全 Keep。
#[derive(Debug, Clone, Copy)]
pub struct StencilFaceState {
    pub compare: CompareFunction,
    pub fail_op: StencilOperation,
    pub depth_fail_op: StencilOperation,
    pub pass_op: StencilOperation,
}
/// 模板操作。
#[derive(Debug, Clone, Copy)]
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}
impl Default for StencilFaceState {
    fn default() -> Self {
        Self {
            compare: CompareFunction::Always,
            fail_op: StencilOperation::Keep,
            depth_fail_op: StencilOperation::Keep,
            pass_op: StencilOperation::Keep,
        }
    }
}
/// 深度/模板固定功能状态；format 须与附件实际格式一致。
#[derive(Debug, Clone, Copy)]
pub struct DepthStencilState {
    pub format: TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
    pub stencil: StencilState,
    pub bias: DepthBiasState,
}
/// 前后双面模板状态。
#[derive(Debug, Clone, Copy, Default)]
pub struct StencilState {
    pub front: StencilFaceState,
    pub back: StencilFaceState,
}
/// 图元装配状态；默认：TriangleList 拓扑、CCW 正面、剔除背面、填充模式。
#[derive(Debug, Clone, Copy)]
pub struct PrimitiveState {
    pub topology: PrimitiveTopology,
    pub strip_index_format: Option<IndexFormat>,
    pub front_face: FrontFace,
    pub cull_mode: CullMode,
    pub polygon_mode: PolygonMode,
    pub unclipped_depth: bool,
    pub conservative: bool,
}
impl Default for PrimitiveState {
    fn default() -> Self {
        Self {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        }
    }
}
/// 多重采样状态：count 与纹理 sample_count 同域（1/4）；mask 默认全 1。
#[derive(Debug, Clone, Copy)]
pub struct MultisampleState {
    pub count: u32,
    pub mask: u64,
    pub alpha_to_coverage_enabled: bool,
}
impl Default for MultisampleState {
    fn default() -> Self {
        Self {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }
}
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

/// 追踪开关（形照官方历史 Trace 枚举：non_exhaustive 护未来开关项、Directory 持 PathBuf 所有权
/// 免无 feature 期脱逸生命周期——两处原文事由本层一并保留）。本层无 trace 产出面：
/// Off=合法无操作；Directory=词位在场但恒拒止（请求侧违约断言 ErrTrace，形宽法窄）。
/// 官方 cfg(feature="trace") 门控不移植：本层不开产出 build option，条目恒可见恒拒。
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub enum Trace {
    /// 追踪关闭（本层唯一合法形）。
    #[default]
    Off,
    /// 追踪目录（官方词位；本层无产出面，给值=违约断言）。
    Directory(std::path::PathBuf),
}
/// 设备创建输入。required_features/required_limits 与 Adapter 能力快照全量逐字段对照：
/// 任一超出=Err(RequestDeviceError)——设备未建，Adapter 仍可反复重试。
/// trace=追踪词位（形宽法窄）：Off=唯一合法形，Directory=违约断言 ErrTrace（本层无产出面）。
#[derive(Debug, Clone, Default)]
pub struct DeviceDescriptor<'a> {
    pub label: Label<'a>,
    pub required_features: Features,
    pub required_limits: Limits,
    pub memory_hints: MemoryHints,
    pub trace: Trace,
}
/// 呈现模式；configure 合法组合面仅 Fifo（Mailbox/Immediate 为词汇保留，能否用由能力查询面回报）。
#[derive(Debug, Clone, Copy)]
pub enum PresentMode {
    Fifo,
    Mailbox,
    Immediate,
}
impl Default for PresentMode {
    fn default() -> Self {
        Self::Fifo
    }
}
/// 合成透明模式；configure 合法组合面仅 Opaque。
#[derive(Debug, Clone, Copy)]
pub enum CompositeAlphaMode {
    Opaque,
    PreMultiplied,
}
impl Default for CompositeAlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}
/// 表面配置：宽高≥1；present_mode×alpha_mode 仅 (Fifo, Opaque) 合法——查询面可宽、合法面窄，
/// 违约=断言，不留半配置。
#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub present_mode: PresentMode,
    pub alpha_mode: CompositeAlphaMode,
}

// —— 创建路径辅词汇（被描述符与查询面引用） ——
/// 动态绑定偏移，单位字节。
pub type DynamicOffset = u32;
/// 后端标签（本库仅 GL）。
#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Gl,
}
/// 适配器信息快照：字符串来自 EGL/GL 查询串聚合拼装；单卡面不枚举多设备。
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub backend: Backend,
    pub device_type: DeviceType,
    pub vendor: String,
    pub name: String,
    pub driver_info: String,
}
/// 降级能力位集合（GL 路=静态汇总表输出，非逐项真探）。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DownlevelFlags(pub u64);
/// 降级能力快照：标志位 + 对应生效上限。
#[derive(Debug, Clone)]
pub struct DownlevelCapabilities {
    pub flags: DownlevelFlags,
    pub limits: Limits,
}
/// 单格式特性位集合。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TextureFormatFeatureFlags(pub u64);
/// 单格式能力快照：允许用途交集 + 特性位。
#[derive(Debug, Clone)]
pub struct TextureFormatFeatures {
    pub allowed_usages: TextureUsages,
    pub flags: TextureFormatFeatureFlags,
}
/// 纹素拷贝目标信息（正名照抄 wgpu 25.0.2 规范名 TexelCopyTextureInfo）。
/// 上游为泛型 <T> 投影（core/wasm 分面共用）；本层单轨直接持纹理引用（登记泛型投影差异）。
#[derive(Debug, Clone, Copy)]
pub struct TexelCopyTextureInfo<'a> {
    pub texture: &'a crate::entry::Texture,
    pub mip_level: u32,
    pub origin: Origin3d,
    pub aspect: TextureAspect,
}
/// 兼容别名：本层历史名（同 ImageDataLayout 别名口径）。 @public
pub type ImageCopyTexture<'a> = TexelCopyTextureInfo<'a>;
/// Texel 拷贝缓冲布局（正名照抄 wgpu 25.0.2 规范名 TexelCopyBufferLayout）：
/// offset 须 4 字节对齐；bytes_per_row 给定时须 256 倍数；Some(0)/缺省=行宽自动
/// （v25 起值域回归裸 u32，NonZero 旧代形制退役）。违约后果见 Queue::write_texture 契约
/// （debug=断言 / release=记录留痕、由驱动兜底，不谎报）。
#[derive(Debug, Clone, Copy, Default)]
pub struct TexelCopyBufferLayout {
    pub offset: BufferAddress,
    pub bytes_per_row: Option<u32>,
    pub rows_per_image: Option<u32>,
}
/// 兼容别名：本层历史名（非上游在场符号，登记为兼容延长，后续批可退役）。 @public
pub type ImageDataLayout = TexelCopyBufferLayout;
/// 多维纹理数据在内存中的排布顺序。
#[derive(Debug, Clone, Copy)]
pub enum TextureDataOrder {
    LayerMajor,
    MipMajor,
}
/// 适配器选择输入：compatible_surface 仅作相容性校验；power_preference/force_fallback 收录但无多卡路由语义。
#[derive(Debug, Clone, Copy)]
pub struct RequestAdapterOptions<'a> {
    pub compatible_surface: Option<&'a crate::surface::Surface<'a>>,
    pub force_fallback_adapter: bool,
    pub power_preference: PowerPreference,
}
