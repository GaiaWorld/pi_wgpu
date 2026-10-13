// 设计：pi-wgpu-rebuild
//! state · 驱动状态镜像：只在「与驱动已知态不同」时才下发调用（避免重复 GL 调用的开销面）。
//! 隐藏决定=镜像字段集与比较粒度。契约权威文本住本面注释；本面全部 @internal。
use crate::objects::GlHandle;
/// 镜像分组数=绑定槽上限（与 binds 写死 16 槽合同同源：UBO/纹理/采样器三张槽表）。 @internal
pub(crate) const BIND_SLOTS: usize = 16;
/// 镜像快照：驱动侧可缓存副作用的字段逐组记录（字段集=L7 解冻展齐：draw set_* 全族+binds 槽表+upload 组）。
/// 不变量：字段值=驱动已成功生效的态，绝不在调用发出时先改。比较策略=全字段逐组比对
/// （被否方案：脏位图——置位散落在各写入点，易漏置位导致镜像虚新）。 @internal
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
    /// uniform 缓冲槽表：UNIFORM_BUFFER 各绑定点基名（binds 应用经本表差异位下发）。
    pub(crate) ubo_bindings: [Option<GlHandle>; BIND_SLOTS],
    /// 纹理组：最近 ACTIVE_TEXTURE 单元号。
    pub(crate) active_texture_unit: Option<u32>,
    /// 纹理槽表：各单元 TEXTURE_2D 绑定对象名（active 单元切换后逐名比对下发）。
    pub(crate) texture_bindings: [Option<GlHandle>; BIND_SLOTS],
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
    /// 多重采样组：采样掩码值（u64 词汇面；下发按驱动 int 宽度截位）。
    pub(crate) sample_mask: Option<u64>,
    /// 代号快照：建/重探镜像时的附着代号；失配=整面作废重探测。
    pub(crate) generation: u32,
}
impl StateMirror {
    /// 逐组比较→差异下发→成功后同步镜像。失败面：驱动报错的字段不改镜像、错误入队列，
    /// 下次比对自然重下发（自洽恢复，无谎报态）。 @internal
    pub(crate) fn flush_if_changed(&mut self) { crate::state_inner::flush_impl(self) }
    /// 对象死亡通知：置空引用该句柄名的镜像字段——下次下发按「镜像未知」强制重设，
    /// 不假设驱动仍持有已删对象。 @internal
    pub(crate) fn drop_invalidate(&mut self, dead: GlHandle) { crate::state_inner::invalidate_impl(self, dead) }
}
