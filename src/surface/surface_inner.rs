// 设计：pi-wgpu-rebuild
//! surface 芯 · 附着→配置→取帧→换帧。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：configure 非 Result 面=log::error!(SurfaceErr)；get_current 走五态 Result。
use crate::entry::Device;
use crate::surface::{
    Surface, SurfaceCapabilities, SurfaceErr, SurfaceError, SurfaceTarget, SurfaceTexture,
};
use crate::types::*;

// 设计预期（伪码·示意·非唯一实现）：
// 1. 全量校验：宽高>0、(present×alpha)仅(Fifo,Opaque)——违例=log::error!(SurfaceErr)，配置不写。
// 2. 桌面/安卓：以窗口目标建平台表面并 make_current——**只在未附着时建一次**（重配置不改平台表面；
//    重 GetDC/SetPixelFormat 同一窗口 DC 是 Windows 禁止的用法）；Web：无 EGL 步（画布即表面）。
// 3. 驱动步骤全部成功后才写 _cfg=Some；中途失败=log，_cfg 保持未配置（不留半配置）。
pub(crate) fn configure_impl(s: &Surface<'_>, _d: &Device, c: &SurfaceConfiguration) {
    if c.width == 0 || c.height == 0 {
        log::error!(
            "surface {:?} w={} h={}",
            SurfaceErr::BadSize,
            c.width,
            c.height
        );
        return;
    }
    if c.usage.is_empty() {
        log::error!("surface {:?} usage empty", SurfaceErr::BadSize);
        return;
    }
    // (present × alpha) 合法面：Fifo 之外无呈现模式；alpha 收 Auto/PreMultiplied/Opaque——
    // Auto=wgpu 标准「由实现按能力选定」语义，本层解析为 PreMultiplied；Opaque=忽略 alpha（GL 天然行为）；
    // PostMultiplied/Inherit 两形态核心都无对应物=仍拒（上层 init_render/render_windows 恒传 Auto）。
    if !matches!(c.present_mode, PresentMode::Fifo)
        || !matches!(
            c.alpha_mode,
            CompositeAlphaMode::Auto
                | CompositeAlphaMode::PreMultiplied
                | CompositeAlphaMode::Opaque
        )
    {
        log::error!(
            "surface {:?} present={:?} alpha={:?}",
            SurfaceErr::BadCombo,
            c.present_mode,
            c.alpha_mode
        );
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if s._cfg.borrow().is_none() {
            let r = match &s._target {
                SurfaceTarget::Window(w) => crate::context_inner::surface_attach(&**w),
            };
            if let Err(e) = r {
                log::error!("surface {:?}", e);
                return;
            }
        }
    }
    *s._cfg.borrow_mut() = Some(c.clone());
    *s._creation.borrow_mut() += 1;
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. _cfg=None 前置：未配置=Err(Outdated)+log 码（须先 configure）。
// 2. 帧视图=默认帧缓冲标记（TextureView{_tex:None}）；suboptimal 恒 true（官方旧语义，登迁移清单）。texture 字段不提供（GL 无独立帧纹理对象，法窄，见壳注）。
// 3. Lost 代号推进由上下文侧 gen_bump 承担（本面不做失效动作，重建=上层 re-request_device）。
pub(crate) fn get_current_impl(
    s: &Surface<'_>,
) -> std::result::Result<SurfaceTexture, SurfaceError> {
    if s._cfg.borrow().is_none() {
        log::error!("surface {:?}", SurfaceErr::NotConfigured);
        return Err(SurfaceError::Outdated);
    }
    let guard = s._cfg.borrow();
    let c = match guard.as_ref() {
        Some(c) => c,
        None => {
            log::error!("surface {:?}", SurfaceErr::NotConfigured);
            return Err(SurfaceError::Outdated);
        }
    };
    let frame_desc = TextureDescriptor {
        label: None,
        size: Extent3d {
            width: c.width,
            height: c.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: c.format,
        usage: c.usage,
        view_formats: &[],
    };
    // 帧背板句柄=占位直建（名恒 0，Drop 永不触驱动）；属性面按 configure 快照真读，
    // creation 位跨重配置失效旧帧（失配消费按死名合同上递，不 panic）。
    let texture = crate::entry::Texture {
        _h: crate::objects::DirectHandle::placeholder(
            crate::objects::DirectKind::Texture,
            *s._creation.borrow() as u32,
        ),
        params: crate::formats::to_gl_params(&frame_desc, c.format),
        _target: crate::formats::target_of(&frame_desc),
        _dim: TextureDimension::D2,
        _layers: 1,
        _size: frame_desc.size,
        _mip: 1,
        _samples: 1,
        _format: c.format,
        _usage: c.usage,
    };
    Ok(SurfaceTexture {
        texture,
        suboptimal: true,
        presented: false,
        _view: crate::entry::TextureView {
            _tex: None,
            _target: 0,
            _dim: crate::types::TextureViewDimension::D2,
        },
    })
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 按附着能力位→formats 候选表×present/alpha 组合回报（纯查询，不建状态）。
pub(crate) fn capabilities_impl(_a: &crate::entry::Adapter) -> SurfaceCapabilities {
    SurfaceCapabilities {
        formats: vec![
            TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb,
        ],
        present_modes: vec![PresentMode::Fifo],
        // 回报面与 configure 合法面同源（Auto 是 wgpu 标准默认值，上游恒用之）。
        alpha_modes: vec![
            CompositeAlphaMode::Auto,
            CompositeAlphaMode::PreMultiplied,
            CompositeAlphaMode::Opaque,
        ],
    }
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 桌面/安卓：swap_buffers（换帧+同步）；Web：空操作。
// 2. 换帧报错不回传错误值（租约已消耗）：记 log::error!(码)，由下一次 get_current 以 Lost/Outdated 暴露。
pub(crate) fn present_impl(_t: SurfaceTexture) {
    if let Err(e) = crate::context_inner::surface_swap() {
        log::error!("surface {:?}", e);
    }
    // 换帧=帧边界：推进状态芯帧门（可回读档每帧只做一次驱动回读）。
    crate::state_inner::bump_frame();
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
    f.debug_struct("Surface")
        .field("target", &target)
        .field("configured", &s._cfg.borrow().is_some())
        .finish()
}
