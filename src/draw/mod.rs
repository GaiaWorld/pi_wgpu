// 设计：pi-wgpu-rebuild · 形制对齐改：desc-align-wgpu25
//! draw · 渲染通道录制面（begin/set/draw；Drop=结束）。
//! 隐藏决定=通道能力约束（颜色目标单实例）；录制即执行=各 set/draw 在调用点即过差异下发。
//! 契约权威文本住本面注释。
use crate::binds;
use crate::entry::{BindGroup, BufferSlice, CommandEncoder, RenderPipeline};
use crate::types::{DynamicOffset, RenderPassDescriptor};

/// 开启中的渲染通道：可变借用编码器即通道锁生命周期——通道唯一性由借用+编码器守卫双重构成。 @public
#[derive(Debug)]
pub struct RenderPass<'pass> {
    /// 编码器可变借用（通道锁生命周期）。
    pub(crate) _enc: &'pass mut CommandEncoder,
    /// 当前管线的顶点布局拷贝（set_vertex_buffer 配属性用）。
    pub(crate) _vertex: Vec<VertexLayout>,
    /// 管线是否已设（draw 前置）。
    pub(crate) _pipe_set: bool,
    /// 顶点属性是否就绪（draw 前置）。
    pub(crate) _vertex_ready: bool,
    /// 索引是否已绑（draw_indexed 前置）。
    pub(crate) _index_ready: bool,
    /// 当前图元拓扑（draw 用）。
    pub(crate) _topology: crate::types::PrimitiveTopology,
    /// 索引绑定格式（draw_indexed 选 GL 类型用）。
    pub(crate) _index_format: Option<crate::types::IndexFormat>,
    /// 索引有效字节区间 [start,end)（set_index_buffer 记入；draw_indexed 范围校验+偏移用）。
    pub(crate) _index_range: (u64, u64),
    /// 当前管线 VAO 名（管线级复用对象；由 set_pipeline 提供，通道不持有其生命周期）。
    pub(crate) _vao: Option<crate::objects::GlHandle>,
    /// 离屏附件 FBO 复用持行（begin 计数、end 交还；None=默认帧缓冲通道）。
    pub(crate) _fbo: Option<crate::objects::Reusable>,
    /// 颜色附件宿主持行：wgpu 语义=通道持有附件引用（begin 之后上游即可放手，宿主须活到通道结束）——
    /// 缺此持行则宿主被删时驱动自动从 FBO 剥离附件，绘制期报 0x8CD7。
    pub(crate) _hold_color: Option<crate::entry::Texture>,
    /// 深度附件宿主持行（同颜色持行语义）。
    pub(crate) _hold_depth: Option<crate::entry::Texture>,
    /// MSAA resolve 对（多重采样宿主, resolve 目的宿主）——end 走驱动滤波采样读+直写中转。
    pub(crate) _resolve: Option<(crate::entry::Texture, crate::entry::Texture)>,
    /// 回放通道标记：true=提交流回放的执行体（Drop 走执行芯），false=录制体（Drop 只入表）。
    pub(crate) _replay: bool,
    /// 当前管线布局的槽分配（set_pipeline 时从管线拷入）：查槽必须与所用管线同源——
    /// 设备级槽表会被后来建的任何管线整体覆盖，唯有管线自带那份与该管线出码一致。
    pub(crate) _slots: crate::binds::SlotTable,
    /// 已绑顶点槽记录（槽号, 缓冲名, 描述符基偏移）：first_instance 模拟重下 Instance 属性的基线。
    pub(crate) _vbinds: Vec<(u32, crate::objects::GlHandle, u64)>,
    /// 当前实例属性已生效的 base：与下次 draw 的 ins.start 不同才重下（含归 0）。
    pub(crate) _fi_base: u32,
}
/// 顶点缓冲布局（拥有版）：创建期从描述符拷入；draw 配属性消费。 @internal
#[derive(Debug, Clone)]
pub(crate) struct VertexLayout {
    /// 步进字节。
    pub(crate) array_stride: u64,
    /// 步进模式。
    pub(crate) step_mode: crate::types::VertexStepMode,
    /// 属性集合（拥有）。
    pub(crate) attributes: Vec<crate::types::VertexAttribute>,
}
impl<'pass> RenderPass<'pass> {
    /// 记入当前管线并通知镜像：程序切换作为差异延后到下发点执行；死句柄=断言，通道保持原管线。 @public
    pub fn set_pipeline(&mut self, p: &RenderPipeline) {
        crate::draw_inner::set_pipeline_impl(self, p)
    }
    /// 挂绑定组：set 域 0-3、动态偏移个数与声明一致——不符=断言且绑定保持原状；
    /// 查槽/取绑定/偏移消耗经 binds 面，最终绑定经状态镜像差异下发。 @public
    pub fn set_bind_group(&mut self, set: u32, g: &BindGroup, offsets: &[DynamicOffset]) {
        crate::draw_inner::set_bind_group_impl(self, set, g, offsets)
    }
    /// 记顶点槽 slot 的字节片段（形照 25.0.2：步长不再由调用方给——管线布局权威；
    /// 属性批对象按布局键走复用通道）；死句柄=断言，属性态不变。 @public
    pub fn set_vertex_buffer(&mut self, slot: u32, buffer_slice: BufferSlice<'_>) {
        crate::draw_inner::set_vertex_impl(self, slot, &buffer_slice)
    }
    /// 记索引片段绑定（形照 25.0.2：BufferSlice+格式；draw_indexed 按格式选 GL 类型、按区间定位并校验）；
    /// 违约=断言（短码）且保持原索引绑定。 @public
    pub fn set_index_buffer(
        &mut self,
        buffer_slice: BufferSlice<'_>,
        index_format: crate::types::IndexFormat,
    ) {
        crate::draw_inner::set_index_impl(self, &buffer_slice, index_format)
    }
    /// 记视口（存原始值；负尺寸=断言）。登记差异：Y 向下→Y 向上的翻换因缺渲染目标高度本批暂缓（翻换点归汇合层，待目标尺寸面）；深度区间原样透传。 @public
    pub fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, dn: f32, up: f32) {
        crate::draw_inner::set_viewport_impl(self, x, y, w, h, dn, up)
    }
    /// 记裁剪框（存原始值）。登记差异：翻换同视口，本批暂缓（待目标尺寸面）。 @public
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        crate::draw_inner::set_scissor_impl(self, x, y, w, h)
    }
    /// 记模板参考值入镜像；与驱动已知态相同则不下发（无谓状态切换零成本）。 @public
    pub fn set_stencil_reference(&mut self, v: u32) {
        crate::draw_inner::set_stencil_impl(self, v)
    }
    /// 记混合常量色入镜像、差异才下发；浮点能力闸门在管线创建时已查，此处不复检（单一设防点原则）。 @public
    pub fn set_blend_constant(&mut self, c: crate::types::Color) {
        crate::draw_inner::set_blend_impl(self, c)
    }
    /// 记推送常量（形照齐上游 25.0.2 RenderPass：字节偏移 + 字节载荷）。
    /// 本层合法域恒拒止：GL 路无 push constants，能力位恒 0/上限恒 0——一切调用=违约断言。 @public
    pub fn set_push_constants(&mut self, offset: u32, data: &[u8]) {
        crate::draw_inner::set_push_impl(self, offset, data)
    }
    /// 录制即执行的数组绘制（形照 25.0.2：顶点区间+实例区间）。前置断言=管线与属性绑定完备，
    /// 缺一=断言且零副作用；实例化经 glDrawArraysInstanced（ES3.0/WebGL2 与桌面核心共有），
    /// instance 区间起点须 0（WebGL2 无 base_instance；上游恒传 0..N）。 @public
    pub fn draw(&mut self, vertices: std::ops::Range<u32>, instances: std::ops::Range<u32>) {
        crate::draw_inner::draw_impl(self, vertices, instances)
    }
    /// 录制即执行的索引绘制（形照 25.0.2 三参）：base_vertex≠0=断言（参数级约束，报错前无副作用）；
    /// instance 区间起点须 0（WebGL2 无 base_instance）；索引未绑/区间越出绑定片段=断言；
    /// 通过后调用点即发元素绘制（实例化经 glDrawElementsInstanced）。 @public
    pub fn draw_indexed(
        &mut self,
        indices: std::ops::Range<u32>,
        base_vertex: i32,
        instances: std::ops::Range<u32>,
    ) {
        crate::draw_inner::draw_idx_impl(self, indices, base_vertex, instances)
    }
}
/// Drop=通道结束：释放编码器单通道守卫、存储动作按描述符落定；
/// 自此起本通道绑定的资源方可合法销毁（引用合同：经绑定组持有的资源受强引用保护；program/VAO 由管线存活期覆盖——管线须存活至通道结束）。 @public
impl Drop for RenderPass<'_> {
    fn drop(&mut self) {
        if self._replay {
            crate::draw_inner::exec_end_impl(self)
        } else {
            crate::draw_inner::end_impl(self)
        }
    }
}
