// 设计：pi-wgpu-rebuild
//! surface 芯 · 附着→配置→取帧→换帧。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：configure 非 Result 面=log::error!(SurfaceErr)；get_current 走四态 Result。
use crate::entry::Device;
use crate::surface::{Surface, SurfaceCapabilities, SurfaceErr, SurfaceError, SurfaceTarget, SurfaceTexture};
use crate::types::*;

// 设计预期（伪码·示意·非唯一实现）：
// 1. 全量校验：宽高>0、(present×alpha)仅(Fifo,Opaque)——违例=log::error!(SurfaceErr)，配置不写。
// 2. 桌面/安卓：以窗口目标建平台表面并 make_current；Web：无 EGL 步（画布即表面）。
// 3. 驱动步骤全部成功后才写 _cfg=Some；中途失败=log，_cfg 保持未配置（不留半配置）。
pub(crate) fn configure_impl(s: &Surface<'_>, _d: &Device, c: &SurfaceConfiguration) {
    if c.width == 0 || c.height == 0 {
        log::error!("surface {:?} w={} h={}", SurfaceErr::BadSize, c.width, c.height);
        return;
    }
    if !matches!((c.present_mode, c.alpha_mode), (PresentMode::Fifo, CompositeAlphaMode::Opaque)) {
        log::error!("surface {:?} present={:?} alpha={:?}", SurfaceErr::BadCombo, c.present_mode, c.alpha_mode);
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let r = match &s._target {
            SurfaceTarget::Window(w) => crate::context_inner::surface_attach(&**w),
        };
        if let Err(e) = r {
            log::error!("surface {:?}", e);
            return;
        }
    }
    *s._cfg.borrow_mut() = Some(c.clone());
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. _cfg=None 前置：未配置=Err(Outdated)+log 码（须先 configure）。
// 2. 帧视图=默认帧缓冲标记（TextureView{_tex:None}）；suboptimal 恒 true（官方旧语义，登迁移清单）。
// 3. Lost 代号推进由上下文侧 gen_bump 承担（本面不做失效动作，重建=上层 re-request_device）。
pub(crate) fn get_current_impl(s: &Surface<'_>) -> std::result::Result<SurfaceTexture, SurfaceError> {
    if s._cfg.borrow().is_none() {
        log::error!("surface {:?}", SurfaceErr::NotConfigured);
        return Err(SurfaceError::Outdated);
    }
        Ok(SurfaceTexture { _view: crate::entry::TextureView { _tex: None, _target: 0, _dim: crate::types::TextureViewDimension::D2 }, _suboptimal: true })
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 按附着能力位→formats 候选表×present/alpha 组合回报（纯查询，不建状态）。
pub(crate) fn capabilities_impl(_a: &crate::entry::Adapter) -> SurfaceCapabilities {
    SurfaceCapabilities {
        formats: vec![TextureFormat::Bgra8Unorm, TextureFormat::Bgra8UnormSrgb, TextureFormat::Rgba8Unorm, TextureFormat::Rgba8UnormSrgb],
        present_modes: vec![PresentMode::Fifo],
        alpha_modes: vec![CompositeAlphaMode::Opaque],
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 桌面/安卓：swap_buffers（换帧+同步）；Web：空操作。
// 2. 换帧报错不回传错误值（租约已消耗）：记 log::error!(码)，由下一次 get_current 以 Lost/Outdated 暴露。
pub(crate) fn present_impl(_t: SurfaceTexture) {
    if let Err(e) = crate::context_inner::surface_swap() {
        log::error!("surface {:?}", e);
    }
}

/// Debug 委托：字段级输出（目标形态 + 是否已配置）。
pub(crate) fn debug_impl(s: &Surface<'_>, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let target = match &s._target {
        SurfaceTarget::Window(_) => "Window",
        #[cfg(target_arch = "wasm32")]
        SurfaceTarget::Canvas(_) => "Canvas",
        #[cfg(target_arch = "wasm32")]
        SurfaceTarget::OffscreenCanvas(_) => "OffscreenCanvas",
    };
    f.debug_struct("Surface").field("target", &target).field("configured", &s._cfg.borrow().is_some()).finish()
}
