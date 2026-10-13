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

/// 可呈现表面。不变量：configure 成功之后方可参与取帧/换帧（_cfg=None 即未就绪，get_current 先断言）。 @public
pub struct Surface<'surface> { pub(crate) _target: SurfaceTarget<'surface>, pub(crate) _cfg: Option<SurfaceConfiguration>, pub(crate) _attach: crate::context::AttachmentSlot }
impl Surface<'_> {
    /// 配置呈现。先全量校验（宽高≥1、格式在能力集内、present×alpha 仅 (Fifo, Opaque) 合法），
    /// 全部通过才挂接驱动并写入配置——违约=断言（报违例字段），_cfg 保持未写，绝不留半配置。 @public
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
