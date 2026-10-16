// 设计：pi-wgpu-rebuild
//! surface · 呈现面：附着→配置→取帧→换帧四步。契约权威文本住本面注释。
use crate::types::*;
use crate::Device;

/// 官方窗口/显示句柄双 trait 的绑定别名（伞面书写用）；blanket impl 使任意实现双 trait 的类型自动满足。 @public
pub trait WindowHandle:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync
{
}
impl<T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync>
    WindowHandle for T
{
}
/// 附着目标按形态分支（桌面/安卓=窗口句柄，Web=画布或离屏画布，cfg 隔离——运行时不存在跨形态误入路径）。 @public
#[non_exhaustive]
pub enum SurfaceTarget<'target> {
    Window(Box<dyn WindowHandle + 'target>),
    #[cfg(target_arch = "wasm32")]
    Canvas(web_sys::HtmlCanvasElement),
    #[cfg(target_arch = "wasm32")]
    OffscreenCanvas(web_sys::OffscreenCanvas),
}

/// 窗口/画布句柄自动转附着目标（wgpu 式 `create_surface(window)`）。 @public
impl<'target, T: WindowHandle + 'target> From<T> for SurfaceTarget<'target> {
    fn from(t: T) -> Self {
        Self::Window(Box::new(t))
    }
}
/// 可呈现表面。不变量：configure 成功之后方可参与取帧/换帧（_cfg=None 即未就绪，get_current 返回 Err）。 @public
pub struct Surface<'surface> {
    /// 附着目标（窗口/画布；configure 时据此建平台表面）。
    pub(crate) _target: SurfaceTarget<'surface>,
    /// 当前配置（None=未配置/配置失败不写——无半配置）；configure 取 &self，故内部可变。
    pub(crate) _cfg: pi_share::ShareCell<Option<SurfaceConfiguration>>,
    /// 附着槽（代号读数；Lost 判定基准）。
    pub(crate) _attach: crate::context::AttachmentSlot,
    /// 配置代号（每次 configure +1）：跨重配置旧帧句柄的失效判据。
    pub(crate) _creation: pi_share::ShareCell<u64>,
}
impl Surface<'_> {
    /// 配置呈现。先全量校验（宽高≥1、present×alpha 仅 (Fifo, Opaque)），全部通过才建平台表面并写入配置。
    /// 违约（错误纪律，非 Result 面）：校验类=log::error!(SurfaceErr)；建面失败=log::error!(SurfaceOpErr)，_cfg 保持未写——绝不留半配置。
    /// 平台表面（窗口 DC）只附着一次，重配置只更新配置值不重建（重 GetDC/SetPixelFormat 同一 DC 是 Windows 禁止用法）（不重建）；登记差异：格式合法性未在查（Device 不持 Caps；纹理创建时已按 caps 过，surface 复用同能力面）。 @public
    pub fn configure(&self, device: &Device, config: &SurfaceConfiguration) {
        crate::surface_inner::configure_impl(self, device, config)
    }
    /// 取当前帧（封闭五态错误面；与 request_device/create_surface 同属全库三处显式 Result 面之一）。成功=帧租约 SurfaceTexture；
    /// 失败映射：Timeout=可重试、Outdated=须重新 configure、Lost=上下文代号已+1 并触发整面失效，
    /// 上层须走重建链（适配器→设备→表面重配置）、OutOfMemory=帧不可取。失败不改变既有配置。 @public
    pub fn get_current_texture(&self) -> std::result::Result<SurfaceTexture, SurfaceError> {
        crate::surface_inner::get_current_impl(self)
    }
    /// 能力查询：纯查不失败。回报面可以宽于 configure 的合法面——查到的候选不代表都配得进去。 @public
    pub fn get_capabilities(&self, adapter: &crate::entry::Adapter) -> SurfaceCapabilities {
        crate::surface_inner::capabilities_impl(adapter)
    }
}

/// 帧租约（与上游 api SurfaceTexture 同形：pub texture + pub suboptimal + presented 哨兵 +
/// present 按值自消费 + Drop 未呈现即丢弃；Clone=上游同备）。texture=帧背板句柄：
/// 默认帧缓冲无独立可挂纹理对象，该句柄为占位直建（名恒 0、永不删驱动），属性方法面
/// （width/height/format/usage 等）按 configure 快照真读；渲染附件用视图经 view() 取
/// （texture.create_view 产物指空名=本层不产出面，死名合同按错误码上递不崩）。 @public
#[derive(Debug, Clone)]
pub struct SurfaceTexture {
    /// 帧背板纹理句柄（上游同名同位；属性快照可读，见壳注）。
    pub texture: crate::entry::Texture,
    /// 帧仍可渲染但重建交换链更优（本层恒 true=旧语义迁移清单）。
    pub suboptimal: bool,
    /// 已呈现哨兵（上游同名内部位；Drop 丢弃判定用）。
    pub(crate) presented: bool,
    pub(crate) _view: crate::entry::TextureView,
}
impl SurfaceTexture {
    /// 换帧呈现（上游现形 mut self：置 presented 后消费）。
    /// 调用后本租约已消耗，再访问不可表达。 @public
    pub fn present(mut self) {
        self.presented = true;
        crate::surface_inner::present_impl(self)
    }
    /// 帧视图取值（建颜色附件用）。 @public
    pub fn view(&self) -> &crate::entry::TextureView {
        &self._view
    }
}

// 与上游同合同：未经 present 即析构=丢弃帧（GL 背板复用，无资源回收；panicking 路径静默）。
impl Drop for SurfaceTexture {
    fn drop(&mut self) {
        if !self.presented && !std::thread::panicking() {
            log::debug!("surface 帧未呈现即丢弃（背板复用，无回收动作）");
        }
    }
}

// 骨架期手写 Debug 委托（SurfaceTarget 持 dyn 对象，derive 不可达；实现批改字段级输出）。
impl core::fmt::Debug for Surface<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        crate::surface_inner::debug_impl(self, f)
    }
}

/// get_current_texture 的封闭失败态（全库三处显式错误面之一；五态与上游同集，Display=短码纪律）。 @public
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SurfaceError {
    /// 取帧超时（可重试）。
    #[error("timeout")]
    Timeout,
    /// 表面已变，须重新 configure。
    #[error("outdated")]
    Outdated,
    /// 交换链已丢，须重建链（上层 re-request_device）。
    #[error("lost")]
    Lost,
    /// 无余量分配新帧。
    #[error("out of memory")]
    OutOfMemory,
    /// 通用失败（本层暂无触发臂——词汇占位对齐上游）。
    #[error("other")]
    Other,
}
/// 表面能力快照（查询面数据，不承载 configure 的合法组合合同）。 @public
#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    pub formats: Vec<TextureFormat>,
    pub present_modes: Vec<PresentMode>,
    pub alpha_modes: Vec<CompositeAlphaMode>,
}
/// surface 错误码（configure 等非 Result 面=log 短码，错误纪律）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceErr {
    /// 未配置即取帧/换帧。
    NotConfigured,
    /// 宽或高为 0。
    BadSize,
    /// (present_mode, alpha_mode) 非法组合（仅 Fifo×Opaque 合法）。
    BadCombo,
}
