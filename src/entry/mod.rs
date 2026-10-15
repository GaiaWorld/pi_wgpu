// 设计：pi-wgpu-rebuild · 形制对齐改：desc-align-wgpu25
//! entry · 公开对象面：上层拿到的全部对象与方法。方法体都是一句委托（实现芯或对应模块芯）；
//! 契约的权威文本住本面注释。失败封闭原则：除三个显式 Result 面（request_device / create_surface /
//! get_current_texture）外，一切操作要么不失败、要么以断言失败——描述符违约在创建边界一次校验。
use crate::draw::RenderPass;
use pi_share::{Share, ShareMutex};

/// request_device 的失败信号（封闭类型）：请求的能力/上限超出适配器快照。
/// 失败后设备未建，同一 Adapter 不受影响、可反复重试。 @public
#[derive(Debug)]
pub struct RequestDeviceError;
/// create_surface 的失败信号（封闭类型）：附着目标类型不符或当前无附着。
/// 失败后不留半附着体，可修正输入后重试。 @public
#[derive(Debug)]
pub struct CreateSurfaceError;

// TODO-DECL：根错误聚合枚举为伞面兼容保留——当前零签名引用、不导出；挂接批按官方错误面对表决定收编范围。

/// 聚合入口。表示=唯一附着槽：GL 上下文的生死随它一体。创建期全部校验/探测完成，
/// 任一违约=断言且不留残留体。 @public
#[derive(Debug)]
pub struct Instance {
    pub(crate) _attach: crate::context::AttachmentSlot,
}
/// 适配器=探测时刻的能力不可变快照。此后所有查询面读的都是这份快照；
/// 可对同一 Adapter 反复 request_device，失败不产生任何副作用。 @public
#[derive(Debug)]
pub struct Adapter {
    pub(crate) _caps: crate::caps::Caps,
}
/// 设备共享内芯：录制/校验面经共享柄触达镜像/缓存/槽表（引用链=Share；三件套各由 Mutex 护）。
/// 锁序约定：先取 state 锁、后进 with_gl（与 set_*/flush 同序，无反向）。 @internal
#[derive(Debug)]
pub(crate) struct DeviceInner {
    /// 驱动状态镜像（差异下发）。
    pub(crate) state: ShareMutex<crate::state::StateMirror>,
    /// 可复用对象缓存。
    pub(crate) cache: ShareMutex<crate::objects::HandleCache>,
    /// 绑定槽表。
    pub(crate) slots: ShareMutex<crate::binds::SlotTable>,
}
/// 设备句柄：持共享内芯（镜像/缓存/槽表），可被命令编码器克隆共享。
/// 引用链为 Share（被否方案：Rc——违反单线程合同）。 @public
#[derive(Debug)]
pub struct Device {
    pub(crate) _inner: Share<DeviceInner>,
}
/// 队列=随设备创建的伴生面。提交点语义为空壳：本库「录制即执行」，
/// submit 不做任何事，上层不得依赖提交批生效时机（保留官方伞面形状的兼容面）。 @public
#[derive(Debug)]
pub struct Queue {
    pub(crate) _dev: Option<Device>,
}
/// 命令编码器=通道开关守卫。不变量：任意时刻至多一个开启中的 RenderPass；
/// 嵌套 begin_render_pass=断言，且不会产生第二个通道。 @public
#[derive(Debug)]
pub struct CommandEncoder {
    pub(crate) _opened_pass: bool,
    pub(crate) _inner: Share<DeviceInner>,
}
/// 空类型：录制即执行之下没有可记录的内容，此类型仅维系官方伞面形状。契约=无内容、无资源。 @public
#[derive(Debug)]
pub struct CommandBuffer;
/// 查询集句柄：RenderPassDescriptor.timestamp_writes / occlusion_query_set 词位的照齐占位类型。
/// 本层 GL 路无对应实现、创建建面不提供（create_query_set 不立）——&QuerySet 按构造不可达，
/// Some 值仅作防御拒止词汇位（短码 ErrQuery，begin 通道芯断言）。形宽法窄，与 PipelineCache 同式。 @public
#[derive(Debug)]
pub struct QuerySet {
    /// 占位：不存在任何可达构造路径。
    pub(crate) _never: core::convert::Infallible,
}
/// 缓冲字节片段（形照 25.0.2 公开面：句柄+偏移+可选长度、Copy/PartialEq）。
/// 上游 map_async/get_mapped_range 族不移植——本层无映射面（录制即执行，写入走 Queue），登记有意差异。 @public
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BufferSlice<'a> {
    pub(crate) buffer: &'a Buffer,
    pub(crate) offset: crate::types::BufferAddress,
    pub(crate) size: Option<crate::types::BufferSize>,
}
impl<'a> BufferSlice<'a> {
    /// 源缓冲引用。 @public
    pub fn get_buffer(&self) -> &'a Buffer {
        self.buffer
    }
    /// 片段起始字节。 @public
    pub const fn get_offset(&self) -> crate::types::BufferAddress {
        self.offset
    }
    /// 片段长度（None=到缓冲末尾）。 @public
    pub fn get_size(&self) -> Option<crate::types::BufferSize> {
        self.size
    }
    /// 片段的绝对字节范围 [start,end)。 @public
    pub fn get_range(&self) -> std::ops::Range<crate::types::BufferAddress> {
        self.offset..self.offset + self.size.map_or(self.buffer.size, |s| s.get())
    }
    /// 本片段之内的再切片（相对界；越界或空段=断言短码，同 Buffer::slice 合同）。 @public
    pub fn slice<S: std::ops::RangeBounds<crate::types::BufferAddress>>(
        &self,
        bounds: S,
    ) -> BufferSlice<'a> {
        let rng = self.get_range();
        let total = rng.end - rng.start;
        let (off, size) = crate::entry_inner::slice_bounds_impl(total, bounds);
        BufferSlice {
            buffer: self.buffer,
            offset: self.offset + off,
            size,
        }
    }
}
impl Buffer {
    /// 取字节片段（形照上游：任意半界/全界 RangeBounds；越界或空段=断言短码，无半建片段）。 @public
    pub fn slice<S: std::ops::RangeBounds<crate::types::BufferAddress>>(
        &self,
        bounds: S,
    ) -> BufferSlice<'_> {
        let (off, size) = crate::entry_inner::slice_bounds_impl(self.size, bounds);
        BufferSlice {
            buffer: self,
            offset: off,
            size,
        }
    }
}
/// 一对一直建的 GL 缓冲。句柄契约：活态⟺与当前附着代号一致；使用死句柄=错误码；
/// 末引用 Drop 才调驱动删除（共享计数：绑定组与用户双持时互不删）；死句柄使用=错误码。 @public
#[derive(Debug)]
pub struct Buffer {
    /// 直建句柄（活态判定 + 驱动名）。
    pub(crate) _h: crate::objects::DirectHandle,
    /// 缓冲字节长度（创建期定格、4 字节倍数）；upload 范围校验消费。
    pub(crate) size: crate::types::BufferAddress,
}
/// 缓冲同一性 = 直建句柄名与定格长度全同（上游 wgpu 对 Dispatch 代理的 eq 之本层投影；
/// 两名皆失活（None）判不为等——死句柄不参与相等推理）。
impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        match (self._h.name(), other._h.name()) {
            (Some(a), Some(b)) => a == b && self.size == other.size,
            _ => false,
        }
    }
}
/// 一对一直建的 GL 纹理。句柄契约同 Buffer：末引用 Drop 才删驱动资源、死句柄使用=错误码。
/// 目标/维度/层数创建期定格（决定 upload 写路径与视图相容性）。 @public
#[derive(Debug, Clone)]
pub struct Texture {
    /// 直建句柄（活态判定 + 驱动名；Clone=共享计数）。
    pub(crate) _h: crate::objects::DirectHandle,
    /// 驱动参数组（创建期由 formats::to_gl_params 定格）；upload 写路径消费 format/pixel_type/bpp。
    pub(crate) params: crate::formats::GlParams,
    /// GL 目标（创建期由 formats::target_of 定格：2D/3D/2D_ARRAY/CUBE_MAP/CUBE_MAP_ARRAY）。
    pub(crate) _target: u32,
    /// 纹理维度（视图相容性判定用）。
    pub(crate) _dim: crate::types::TextureDimension,
    /// 数组层/深度（cube 面数=6）。
    pub(crate) _layers: u32,
}
impl Texture {
    /// 整张缺省拷贝目标（上游同式：mip 0/原点/All）——write_texture/copy_texture 消费。 @public
    pub fn as_image_copy(&self) -> crate::types::TexelCopyTextureInfo<'_> {
        crate::types::TexelCopyTextureInfo {
            texture: self,
            mip_level: 0,
            origin: crate::types::Origin3d::ZERO,
            aspect: crate::types::TextureAspect::All,
        }
    }
    /// 建纹理视图（唯一 @public 视图入口，形制=25.0.2 九字段）：dimension/format/usage/count
    /// None=按宿主纹理取默认（维度依纹理维/层数推定）；维度相容（Cube/CubeArray 需层数%6==0）；
    /// 视图目标按维度覆盖（Cube→CUBE_MAP、CubeArray→CUBE_MAP_ARRAY，余同纹理目标）。违约=断言含 desc.label。
    /// 本层「取整张」：base/count 切片暂不生效（越界接表归实现批）。 @public
    pub fn create_view(&self, desc: &TextureViewDescriptor<'_>) -> TextureView {
        crate::entry_inner::create_view_impl(self, desc)
    }
}
/// 纯值视图：随宿主 Texture 复制与消亡。视图本身无驱动对应物，不独立触发任何删除。
/// 目标/维度由视图维度定格（cube/cube-array 覆盖纹理目标；帧视图=无宿主且目标 0）。 @public
#[derive(Debug, Clone)]
pub struct TextureView {
    pub(crate) _tex: Option<Texture>,
    pub(crate) _target: u32,
    pub(crate) _dim: crate::types::TextureViewDimension,
}
/// 一对一直建的 GL 采样器。句柄契约同 Buffer。 @public
#[derive(Debug)]
pub struct Sampler {
    pub(crate) _h: crate::objects::DirectHandle,
}
/// 绑定组布局：持有解析后的条目强链（合法性已在创建边界全部检查，此后不再复查）。
/// 生命周期合同：布局必须先于使用它的绑定组/管线布局销毁。 @public
#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) _entries: Vec<BindGroupLayoutEntry>,
}
/// 绑定组：解析后的条目集（声明序）——每条=资源强引用句柄+应用元数据（偏移/尺寸/动态位）。
/// 生命周期合同：被绑资源先于绑定组销毁=断言；动态偏移条目的声明数在此登记，set_bind_group 时按序消耗对表。 @public
#[derive(Debug)]
pub struct BindGroup {
    pub(crate) _bound: Vec<crate::binds::BoundEntry>,
}
/// 管线布局：持成员绑定组布局强链。set 归属域 0-3 且成员不重复，创建时校验。 @public
#[derive(Debug)]
pub struct PipelineLayout {
    pub(crate) _bgls: Vec<BindGroupLayout>,
}
/// 渲染管线：持转译产物的可复用程序对象（复用键=源码转译哈希）、管线级 VAO（复用键=布局哈希）与顶点属性布局。
/// 复用对象引用归零时物理删除并刷失镜像，由对象层保证。
/// 生命周期合同：管线须存活至引用它的 RenderPass 结束（通道只记名、不持复用对象强引用；管线 Drop 释放 _program/_vao）。 @public
#[derive(Debug)]
pub struct RenderPipeline {
    /// 转译哈希键复用的程序对象（管线级）。
    pub(crate) _program: crate::objects::Reusable,
    /// 布局哈希键复用的 VAO（管线级；随管线建/删）。
    pub(crate) _vao: crate::objects::Reusable,
    /// 顶点属性布局（拥有版；供通道配属性）。
    pub(crate) _vertex: Vec<crate::draw::VertexLayout>,
    /// 图元拓扑（供 draw 选 GL 图元）。
    pub(crate) _topology: crate::types::PrimitiveTopology,
}
/// 着色器模块：持反射登记表（(set,binding)↔类别/名映射）与转译产物（IR，供管线创建按入口出码）。 @public
#[derive(Debug)]
pub struct ShaderModule {
    pub(crate) _refl: crate::shader::ReflectionTable,
    pub(crate) _compiled: crate::shader::CompiledEs300,
}
/// 管线缓存句柄：照齐上游 RenderPipelineDescriptor.cache 字段形制的占位类型。
/// 本库 GL 路无对应物、创建建面不提供——&PipelineCache 按构造不可达，
/// cache=Some 仅为防御断言词汇位（短码 ErrCache，芯 TODO-DECL）。 @public
#[derive(Debug)]
pub struct PipelineCache {
    /// 占位：不存在任何可达构造路径。
    pub(crate) _never: core::convert::Infallible,
}

/// Instance 的创建与顶层查询入口。 @public
impl Instance {
    /// 建实例：描述符校验→取唯一上下文附着→能力探测→装配。违约=断言；失败后无任何残留体。 @public
    pub fn new(descriptor: &InstanceDescriptor) -> Self {
        crate::entry_inner::new_impl(descriptor)
    }
    /// 附着呈现目标。失败（Err=目标类型不符/无附着）后不留半附着、可直接重试——用户输入面走错误值不走断言。 @public
    pub fn create_surface<'s>(
        &'s self,
        target: impl Into<crate::surface::SurfaceTarget<'s>>,
    ) -> Result<crate::surface::Surface<'s>, CreateSurfaceError> {
        crate::entry_inner::create_surface_impl(target.into())
    }
    /// 适配器选择：官方 Future 形状的同步包装，恒等返回即时有值。
    /// None=当前无可用附着/探测未过，实例不受影响可再试。单卡面（EGL 恒一块：被否多卡枚举）；
    /// options 的选择字段记录不路由。Lost 后的重建回路以此为重入点：代号推进完成后对同一 Instance 再次调用（内部自动重建附着并重探测），无需重建 Instance。 @public
    pub async fn request_adapter<'s>(
        &self,
        options: &RequestAdapterOptions<'s>,
    ) -> Option<Adapter> {
        crate::entry_inner::request_adapter_impl(options)
    }
}

/// Adapter 的查询面与设备创建面。 @public
impl Adapter {
    /// 按描述符建设备（-> impl Future 形照 25.0.2；WasmNotSend 不适用 GL 面、登记不抄）。
    /// 校验=对照本适配器能力快照全量 36 字段逐字段检查；任一超出=Err，
    /// 设备未建且 Adapter 仍可用。失败面仅此一种；成功后三件套完整装配。
    /// trace=追踪词位现居 DeviceDescriptor.trace（Directory=违约断言 ErrTrace）；
    /// trace_path 维持官方历史双参兼容形：仅记录不产出（Some=入 debug 日志后忽略，永不失败）。 @public
    pub fn request_device(
        &self,
        desc: &DeviceDescriptor<'_>,
        trace_path: Option<&std::path::Path>,
    ) -> impl std::future::Future<Output = std::result::Result<(Device, Queue), RequestDeviceError>>
    {
        let r = crate::entry_inner::request_device_impl(self, desc, trace_path);
        async move { r }
    }
    /// 能力查询：纯读快照，永不失败。 @public
    pub fn features(&self) -> Features {
        crate::entry_inner::adapter_features(self)
    }
    /// 上限查询：纯读快照（兼容子集各值按能力下限拍板），永不失败。 @public
    pub fn limits(&self) -> Limits {
        crate::entry_inner::adapter_limits(self)
    }
    /// 适配器信息：读 EGL/GL 查询串聚合拼装；单卡面不枚举多设备，永不失败。 @public
    pub fn get_info(&self) -> AdapterInfo {
        crate::entry_inner::adapter_info(self)
    }
    /// 降级能力：GL 路=汇总静态表（供下兼容描述，非逐项真探），永不失败。 @public
    pub fn get_downlevel_capabilities(&self) -> DownlevelCapabilities {
        crate::entry_inner::adapter_downlevel(self)
    }
    /// 单格式能力=能力位×格式转换表合成（在库内组合，扩展不存在时不虚报），永不失败。 @public
    pub fn get_texture_format_features(&self, format: TextureFormat) -> TextureFormatFeatures {
        crate::entry_inner::adapter_format_features(self, format)
    }
    /// 表面相容性：判表面附着与本次 probe 的上下文是否同一，永不失败。 @public
    pub fn is_surface_supported(&self, surface: &crate::surface::Surface<'_>) -> bool {
        crate::entry_inner::adapter_surface_supported(self, surface)
    }
    /// 私有差异（登迁移清单）：让出当前线程的 GL 上下文绑定，供跨线程搬运。 @public
    pub fn unmake_current(&self) {
        crate::entry_inner::adapter_let_go(self)
    }
}

/// Device 的资源创建面与查询面。创建族共同合同：即时创建、描述符在边界一次全量校验，
/// 违约=断言（报 desc.label 与违约字段）且断言前不发任何驱动调用、无半建对象。 @public
impl Drop for RenderPipeline {
    /// 管线销毁=释放其持有的程序与管线级 VAO（引用归零即物理删）。
    /// 限界：invalidate_generations 清计数后仍存活时，此释放命中零计数=断言（当前仅测试面调用/未接线 Lost 路径）。 @internal
    fn drop(&mut self) {
        crate::objects::release_reuse(&self._program);
        crate::objects::release_reuse(&self._vao);
    }
}

impl Device {
    /// 建缓冲。检查项：usage 非空且在支持位内；size>0 且为 4 字节倍数。一对一创建，无对象级复用。 @public
    pub fn create_buffer(&self, desc: &BufferDescriptor<'_>) -> Buffer {
        crate::entry_inner::create_buffer_impl(self, desc)
    }
    /// 建纹理。检查项：宽高≥1；mip_level_count≥1；sample_count∈{1,4}；
    /// 格式在能力支持集内；view_formats 与主格式同族。违约后果同创建族共同合同。 @public
    pub fn create_texture(&self, desc: &TextureDescriptor<'_>) -> Texture {
        crate::entry_inner::create_texture_impl(self, desc)
    }
    /// 建采样器（形制=25.0.2 十二字段）。检查项：anisotropy_clamp ≥1、非 1 时三过滤全 Linear
    /// （上游合同；lod 夹值/GL 各向异性下发归实现批）。（compare 与绑定类型相容性属绑定组对表，不在本步。）
    /// 违约后果同创建族共同合同。 @public
    pub fn create_sampler(&self, desc: &SamplerDescriptor<'_>) -> Sampler {
        crate::entry_inner::create_sampler_impl(self, desc)
    }
    /// 建着色器模块。WGSL/GLSL 经 naga 转译为 GLSL ES 3.0（出码延到管线创建）。
    /// 失败面（错误纪律，非 Result 面）：源语法/转译错=log::error!(短码)+降级（无有效产物、不留半建），
    /// 真错（空产物）延后到管线创建——create_render_pipeline 非 Result 面，出码失败=编程合同违反（短码断言）；诊断原文走 log。 @public
    pub fn create_shader_module(&self, desc: ShaderModuleDescriptor<'_>) -> ShaderModule {
        crate::entry_inner::create_shader_impl(self, desc)
    }
    /// 建绑定组布局。检查项：条目 binding 域合法、visibility 非空、数组长度>0、类型在兼容子集内。
    /// 违约后果同创建族共同合同。 @public
    pub fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDescriptor<'_>,
    ) -> BindGroupLayout {
        crate::entry_inner::create_bgl_impl(self, desc)
    }
    /// 建绑定组。检查项：条目与布局逐一对表（缺失/多余/类型不符=断言）、资源句柄全部活态、
    /// 缓冲偏移满足对齐与边界、动态偏移条目数在此登记。违约=未建组、无残留。 @public
    pub fn create_bind_group(&self, desc: &BindGroupDescriptor<'_>) -> BindGroup {
        crate::entry_inner::create_bg_impl(self, desc)
    }
    /// 建管线布局。检查项：成员 set 归属 0-3、不重复、组数不超上限；
    /// push_constant_ranges 非空=违约断言短码（GL 路恒拒止，能力位恒 0/上限恒 0，芯 TODO-DECL）。持成员强链。 @public
    pub fn create_pipeline_layout(&self, desc: &PipelineLayoutDescriptor<'_>) -> PipelineLayout {
        crate::entry_inner::create_pl_impl(self, desc)
    }
    /// 建渲染管线。形全量照齐 wgpu 25.0.2；合法域收窄（形宽法窄）：layout=None、multiview=Some、
    /// cache=Some、非默认编译选项=违约断言（短码含 label，无半建管线）；
    /// entry_point=None=该 stage 唯一入口自动选（缺失/歧义=出码失败断言）。
    /// 程序对象按转译哈希、顶点布局按布局哈希走复用通道。违约后果同创建族共同合同。 @public
    pub fn create_render_pipeline(&self, desc: &RenderPipelineDescriptor<'_>) -> RenderPipeline {
        crate::entry_inner::create_rp_impl(self, desc)
    }
    /// 建命令编码器：仅置通道守卫初值，零驱动调用；除 label 外没有可失败项（纯结构不可能非 label 违约）。 @public
    pub fn create_command_encoder(&self, desc: &CommandEncoderDescriptor<'_>) -> CommandEncoder {
        crate::entry_inner::create_encoder_impl(self, desc)
    }
    /// 能力查询：读创建时能力快照，纯查不失败。 @public
    pub fn features(&self) -> Features {
        crate::entry_inner::device_features(self)
    }
    /// 上限查询：读创建时能力快照，纯查不失败。 @public
    pub fn limits(&self) -> Limits {
        crate::entry_inner::device_limits(self)
    }
    /// 私有差异（登迁移清单）：外部 GL 干扰后的恢复——丢弃状态镜像并按驱动真态逐项重同步。
    /// 幂等面，无失败态。 @public
    pub fn reset_state(&self) {
        crate::entry_inner::device_reset_state(self)
    }
}

/// Queue：提交面空壳——上层不得依赖提交生效点（录制即执行的公开合同）。写路径见 upload 模块。 @public
impl Queue {
    pub fn submit(&self, _buffer: impl Into<Option<CommandBuffer>>) {} // 真·空壳：不委托芯——「无提交点语义」本身就是契约。
}

/// CommandEncoder 的通道开关面。 @public
impl CommandEncoder {
    /// 收束编码器：产出空 CommandBuffer，无失败语义（内容已在执行点消化完毕）。 @public
    pub fn finish(self) -> CommandBuffer {
        crate::entry_inner::encoder_finish(self)
    }
    /// 开渲染通道。检查项（一次全量）：color_attachments 中 Some 恰好 1 个。违约=断言（短码），通道未开、编码器守卫不变。
    /// 登记差异：深度/模板附件与 store 语义本批暂缓（通道态暂无描述符载体）；离屏 FBO 路径本批暂停（走默认帧缓冲）。 @public
    pub fn begin_render_pass<'pass>(
        &'pass mut self,
        desc: &RenderPassDescriptor<'pass>,
    ) -> RenderPass<'pass> {
        crate::draw_inner::begin_impl(self, desc)
    }
}

// 词汇面整面再导出（伞面白名单对表属挂接批；骨架批全量导出）。
pub use crate::types::*;
