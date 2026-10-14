// 设计：pi-wgpu-rebuild
//! state · 驱动状态镜像：只在「与驱动已知态不同」时才下发调用（避免重复 GL 调用的开销面）。
//! 隐藏决定=镜像字段集与比较粒度。契约权威文本住本面注释；本面全部 @internal。
use crate::objects::GlHandle;
/// 镜像分组数=绑定槽上限（同步点：binds 壳的槽宽合同放宽时**须同改本常量与 binds 壳**——两处字面量各有注释互指）。 @internal
pub(crate) const BIND_SLOTS: usize = 16;
/// 镜像快照：驱动侧可缓存副作用的字段逐组记录（字段集=L7 解冻展齐：draw set_* 全族+binds 槽表+upload 组）。
/// 不变量：字段值=**镜像期望态**；下发后驱动按 GL 命令队列最终执行（同步错误探测面不存在，成功以驱动队列为准——
/// 错误可见性归 upload 探针与 lost 面，本模块不自证成功、不谎报回读）。比较策略=全字段逐组比对
/// （被否方案：脏位图——置位散落在各写入点，易漏置位导致镜像虚新）。非回读组比较基准=上次下发水位（water 字段）。 @internal
#[derive(Debug, Default)]
pub(crate) struct StateMirror {
    /// 程序组：当前 useProgram 名（None=从未设）。
    pub(crate) program: Option<GlHandle>,
    /// 属性组：当前 VAO 绑定名（顶点属性与 ELEMENT_ARRAY 均被 VAO 吸收，不再单记）。
    pub(crate) vao: Option<GlHandle>,
    /// 目标组：当前 DRAW framebuffer 绑定名（pass begin/end 经本组）。
    pub(crate) fbo_draw: Option<GlHandle>,
    /// 上传组：PIXEL_UNPACK_BUFFER 绑定名（upload 芯经本组）。
    pub(crate) pixel_unpack_buffer: Option<GlHandle>,
    /// uniform 缓冲槽表：各绑定点基名+偏移+尺寸三元（偏移/尺寸变亦必须重绑，否则换偏移不重发=漏发）。
    pub(crate) ubo_bindings: [Option<(GlHandle, u32, u32)>; BIND_SLOTS],
    /// 纹理组：最近 ACTIVE_TEXTURE 单元号。
    pub(crate) active_texture_unit: Option<u32>,
    /// 纹理槽表：各单元绑定 (对象名, GL 目标) 二元（active 单元切换后逐名+目标比对下发；目标随视图维度）。
    pub(crate) texture_bindings: [Option<(GlHandle, u32)>; BIND_SLOTS],
    /// 采样器槽表：各单元绑定采样器名（WebGL2 兼容语义，binds 消费）。
    pub(crate) sampler_bindings: [Option<GlHandle>; BIND_SLOTS],
    /// 视口组：最终下发六元组（x,y,w,h,min_depth,max_depth；内翻换算在 draw 汇合层完成后入镜像）。
    pub(crate) viewport: Option<[f32; 6]>,
    /// 裁剪组：scissor 框四元组（x,y,w,h）。
    pub(crate) scissor: Option<[i32; 4]>,
    /// 裁剪开关：SCISSOR_TEST 使能。
    pub(crate) scissor_test: Option<bool>,
    /// 深度组：深度测试使能。
    pub(crate) depth_test_enabled: Option<bool>,
    /// 深度组：深度写掩码。
    pub(crate) depth_write_enabled: Option<bool>,
    /// 深度组：深度比较函数。
    pub(crate) depth_compare: Option<crate::types::CompareFunction>,
    /// 混合组：整体混合态（Some(None)=关闭混合哨兵，与 None=未设区分）。
    pub(crate) blend: Option<Option<crate::types::BlendState>>,
    /// 混合常量色（set_blend_constant 消费）。
    pub(crate) blend_constant: Option<crate::types::Color>,
    /// 写掩码组：颜色通道写位集合。
    pub(crate) color_writes: Option<crate::types::ColorWrites>,
    /// 图元组：剔除模式（嵌套三态：未设/不剔除/剔除某向——CullMode 词=Option<Face>）。
    pub(crate) cull_mode: Option<crate::types::CullMode>,
    /// 图元组：正面绕序。
    pub(crate) front_face: Option<crate::types::FrontFace>,
    /// 图元组：填充模式。
    pub(crate) polygon_mode: Option<crate::types::PolygonMode>,
    /// 深度偏置组：常数/斜率/钳制三元。
    pub(crate) depth_bias: Option<crate::types::DepthBiasState>,
    /// 模板组：正面状态。
    pub(crate) stencil_front: Option<crate::types::StencilFaceState>,
    /// 模板组：背面状态。
    pub(crate) stencil_back: Option<crate::types::StencilFaceState>,
    /// 模板参考值（set_stencil_reference）。
    pub(crate) stencil_reference: Option<u32>,
    /// 多重采样组：覆盖采样使能。
    pub(crate) sample_alpha_to_coverage: Option<bool>,
    /// 多重采样组：采样掩码值（u64 词汇面；本 fork glow 无掩码下发接口=镜像仅记录期望、无驱动效果，差异登记 B14）。
    pub(crate) sample_mask: Option<u64>,
    /// 代号快照：建/重探镜像时的附着代号；失配=整面作废重探测。
    pub(crate) generation: u32,
    /// 整发档已下发水位（优化件：非回读组「上次实际下发」的原语快照，差异才发；缺省=从未发=首轮全发）。
    /// 死名表非空或整面重置（reset_all）时归零=下轮强发一轮。值域与比较语义见 state 芯 Water 注释。 @internal
    pub(crate) water: crate::state_inner::Water,
    /// 可回读档帧门：上次执行驱动回读比对的帧号（None=从未）；与当前帧号不等时才重做回读。
    pub(crate) readback_frame: Option<u32>,
}
impl StateMirror {
    /// 两档重同步：可回读档（标量参数：开关/函数/单元/视口框）读驱动实值比对、差异才下发；
    /// 不可回读档（GL 无 current-program 回读、WebGL 对象绑定不可查、UBO 绑定点无标量口、fork 掩码无下发面）
    /// 按上次下发水位（water）比对、差异才发（水位空或死名登记命中=整组强发一次）。死名命中下发后销记。
    /// 可回读档另受帧门约束：每帧（surface.present 推进帧号）只做一次驱动回读比对，帧内复用镜像态。失败面：无同步错误通道（见不变量），
    /// 下发即入驱动队列，驱动侧恢复语义由消费面（探针/lost 重建）闭合。 @internal
    pub(crate) fn flush_if_changed(&mut self) { crate::state_inner::flush_impl(self) }
    /// 对象死亡即时通知（逐名）：置空引用该句柄名的镜像字段——下次下发按「镜像未知」强制重设，
    /// 不假设驱动仍持有已删对象。常规死亡路径走芯死名登记（objects 删除时）+flush 强发；
    /// 本即时作废面留给 binds/draw 接线批的点名失效场景（在册：当前调用方待实现批接入）。 @internal
    pub(crate) fn drop_invalidate(&mut self, dead: GlHandle) { crate::state_inner::invalidate_impl(self, dead) }
}
