// 设计：pi-wgpu-rebuild
//! surface · 呈现面：附着→配置→取帧→换帧四步。契约权威文本住本面注释。
use crate::Device;
use crate::types::*;

/// 官方窗口/显示句柄双 trait 的绑定别名（伞面书写用）；blanket impl 使任意实现双 trait 的类型自动满足。 @public
pub trait WindowHandle: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle {}
impl<T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle> WindowHandle for T {}
/// 附着目标按形态分支（桌面/安卓=窗口句柄，Web=画布或离屏画布，cfg 隔离——运行时不存在跨形态误入路径）。 @public
#[non_exhaustive]
pub enum SurfaceTarget<'target> {
    Window(Box<dyn WindowHandle + 'target>),
    #[cfg(target_arch = "wasm32")] Canvas(web_sys::HtmlCanvasElement),
    #[cfg(target_arch = "wasm32")] OffscreenCanvas(web_sys::OffscreenCanvas),
}

/// 可呈现表面。不变量：configure 成功之后方可参与取帧/换帧（_cfg=None 即未就绪，get_current 返回 Err）。 @public
pub struct Surface<'surface> {
    /// 附着目标（窗口/画布；configure 时据此建平台表面）。
    pub(crate) _target: SurfaceTarget<'surface>,
    /// 当前配置（None=未配置/配置失败不写——无半配置）；configure 取 &self，故内部可变。
    pub(crate) _cfg: core::cell::RefCell<Option<SurfaceConfiguration>>,
    /// 附着槽（代号读数；Lost 判定基准）。
    pub(crate) _attach: crate::context::AttachmentSlot,
}
impl Surface<'_> {
    /// 配置呈现。先全量校验（宽高≥1、present×alpha 仅 (Fifo, Opaque)），全部通过才建平台表面并写入配置。
    /// 违约（决定⑪，非 Result 面）：log::error!(SurfaceErr)，_cfg 保持未写——绝不留半配置。
    /// 登记差异：格式合法性未在查（Device 不持 Caps；纹理创建时已按 caps 过，surface 复用同能力面）。 @public
    pub fn configure(&self, device: &Device, config: &SurfaceConfiguration) { crate::surface_inner::configure_impl(self, device, config) }
    /// 取当前帧（封闭四态错误面；与 request_device/create_surface 同属全库三处显式 Result 面之一）。成功=帧租约 SurfaceTexture；
    /// 失败映射：Timeout=可重试、Outdated=须重新 configure、Lost=上下文代号已+1 并触发整面失效，
    /// 上层须走重建链（适配器→设备→表面重配置）、OutOfMemory=帧不可取。失败不改变既有配置。 @public
    pub fn get_current_texture(&self) -> std::result::Result<SurfaceTexture, SurfaceError> { crate::surface_inner::get_current_impl(self) }
    /// 能力查询：纯查不失败。回报面可以宽于 configure 的合法面——查到的候选不代表都配得进去。 @public
    pub fn get_capabilities(&self, adapter: &crate::entry::Adapter) -> SurfaceCapabilities { crate::surface_inner::capabilities_impl(adapter) }
}

/// 帧租约。present 按值自消费：同一帧对象的「呈现两次」在类型上不可表达；
/// 同时至多持有一个活跃帧=由此不变量+使用合同共同保证。 @public
pub struct SurfaceTexture { pub(crate) _view: crate::entry::TextureView, pub(crate) _suboptimal: bool }
impl SurfaceTexture {
    /// 换帧呈现。分形态：桌面/安卓=交换缓冲+节流，Web=无显式换帧调用（浏览器隐式合成）。
    /// 调用后本租约已消耗，再访问不可表达。 @public
    pub fn present(self) { crate::surface_inner::present_impl(self) }
}

// 骨架期手写 Debug 委托（SurfaceTarget 持 dyn 对象，derive 不可达；实现批改字段级输出）。
impl core::fmt::Debug for Surface<'_> { fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { crate::surface_inner::debug_impl(self, f) } }

/// get_current_texture 的封闭失败态（全库三处显式错误面之一）。 @public
#[derive(Debug, thiserror::Error)] pub enum SurfaceError { #[error("timeout")] Timeout, #[error("outdated")] Outdated, #[error("lost")] Lost, #[error("out of memory")] OutOfMemory }
/// 表面能力快照（查询面数据，不承载 configure 的合法组合合同）。 @public
#[derive(Debug, Clone)] pub struct SurfaceCapabilities { pub formats: Vec<TextureFormat>, pub present_modes: Vec<PresentMode>, pub alpha_modes: Vec<CompositeAlphaMode> }
/// surface 错误码（configure 等非 Result 面=log 短码，决定⑪）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceErr {
    /// 未配置即取帧/换帧。
    NotConfigured,
    /// 宽或高为 0。
    BadSize,
    /// (present_mode, alpha_mode) 非法组合（仅 Fifo×Opaque 合法）。
    BadCombo,
    /// 平台表面创建失败。
    CreateSurface,
    /// 附着未建立即配置。
    NotAttached,
}
