// 设计：pi-wgpu-rebuild
//! surface 芯 · 附着→配置→取帧→换帧。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：configure 非 Result 面=log::error!(SurfaceErr)；get_current 走五态 Result。
use crate::entry::Device;
use glow::HasContext;
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
    crate::state_inner::check_err("configure");
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
        _surface: true,
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

// —— 上屏翻转（坐标系对齐 wgpu 的最后一环）——
// 出码侧 ADJUST_COORDINATE_SPACE 让离屏内容按 wgpu「行 0=顶」方向存；默认帧缓冲仍是
// GL「行 0=底」（且直画不可翻，glViewport 负高在 ES 非法）。做法同 wgpu-hal 的 GL 后端：
// 先把默认帧缓冲整幅落到中转纹理，再以反向源矩形 blit 回去；中转按 尺寸/内部格式/目标/
// 上下文代号 复用（重附着后旧名失效=按代号重建）。
#[cfg(not(target_arch = "wasm32"))]
const GL_FRAMEBUFFER: u32 = 0x8D40;
#[cfg(not(target_arch = "wasm32"))]
const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
#[cfg(not(target_arch = "wasm32"))]
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
#[cfg(not(target_arch = "wasm32"))]
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
#[cfg(not(target_arch = "wasm32"))]
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
#[cfg(not(target_arch = "wasm32"))]
const GL_NEAREST: u32 = 0x2600;
#[cfg(not(target_arch = "wasm32"))]
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
#[cfg(not(target_arch = "wasm32"))]
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
#[cfg(not(target_arch = "wasm32"))]
const GL_SCISSOR_TEST: u32 = 0x0C11;
#[cfg(not(target_arch = "wasm32"))]
const GL_FRAMEBUFFER_SRGB: u32 = 0x8DB9;

/// 上屏翻转中转行（显存常驻；尺寸/格式/上下文任一变更即重建）。
#[cfg(not(target_arch = "wasm32"))]
struct Stage {
    /// 中转纹理驱动名。
    tex: u32,
    /// 中转 FBO 驱动名。
    fbo: u32,
    /// 宽（像素）。
    w: u32,
    /// 高（像素）。
    h: u32,
    /// GL 内部格式（换格式=重建）。
    ifmt: u32,
    /// GL 目标（帧面通常 GL_TEXTURE_2D）。
    target: u32,
    /// 上下文代号（重附着后旧名失效）。
    gen: u32,
}
/// 进程级唯一中转（单附着/单窗口语义；多窗口同尺寸共享亦安全——只在换帧期占用）。
#[cfg(not(target_arch = "wasm32"))]
static STAGE: std::sync::Mutex<Option<Stage>> = std::sync::Mutex::new(None);

/// 取/建中转 FBO（返回驱动名）：键不匹配即删旧建新；跨代号只丢记录不删（旧名属旧上下文）。
#[cfg(not(target_arch = "wasm32"))]
fn ensure_stage(t: &crate::entry::Texture) -> u32 {
    let w = t._size.width;
    let h = t._size.height;
    let ifmt = t.params.internal_format;
    let target = t._target;
    let gen = crate::context_inner::generation_now();
    let mut guard = STAGE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = guard.as_ref() {
        if s.w == w && s.h == h && s.ifmt == ifmt && s.target == target && s.gen == gen {
            return s.fbo;
        }
        if s.gen == gen {
            let (tex, fbo) = (s.tex, s.fbo);
            crate::context_inner::with_gl(|gl| unsafe {
                gl.delete_framebuffer(glow::NativeFramebuffer(nz(fbo)));
                gl.delete_texture(glow::NativeTexture(nz(tex)));
            });
        }
    }
    let (tex, fbo) = crate::context_inner::with_gl(|gl| unsafe {
        let tex = gl.create_texture().expect("surface 断言: 中转纹理建失败");
        gl.bind_texture(target, Some(tex));
        gl.tex_storage_2d(target, 1, ifmt, w as i32, h as i32);
        gl.tex_parameter_i32(target, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
        gl.tex_parameter_i32(target, GL_TEXTURE_MAG_FILTER, GL_NEAREST as i32);
        let fbo = gl.create_framebuffer().expect("surface 断言: 中转 FBO 建失败");
        gl.bind_framebuffer(GL_FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, target, Some(tex), 0);
        gl.bind_framebuffer(GL_FRAMEBUFFER, None);
        (tex.0.get(), fbo.0.get())
    });
    *guard = Some(Stage {
        tex,
        fbo,
        w,
        h,
        ifmt,
        target,
        gen,
    });
    fbo
}

/// 非零 GL 名包装（0 非法=断言）。
#[cfg(not(target_arch = "wasm32"))]
fn nz(name: u32) -> std::num::NonZeroU32 {
    std::num::NonZeroU32::new(name).expect("surface 断言: 驱动名为 0")
}

/// 把默认帧缓冲内容翻正：默认→中转（原序），中转→默认（源矩形 y 反向）。
/// SCISSOR 临时关闭后按原值恢复（blit 受裁剪测试影响；SCISSOR_BOX 不动，镜像不失效）。
#[cfg(not(target_arch = "wasm32"))]
fn flip_back_buffer(t: &crate::entry::Texture) {
    let w = t._size.width as i32;
    let h = t._size.height as i32;
    if w <= 0 || h <= 0 {
        return;
    }
    let fbo = ensure_stage(t);
    let fb = glow::NativeFramebuffer(nz(fbo));
    crate::context_inner::with_gl(|gl| unsafe {
        let scissor = gl.is_enabled(GL_SCISSOR_TEST);
        if scissor {
            gl.disable(GL_SCISSOR_TEST);
        }
        // sRGB 帧面：blit 期间的转换行为各驱动不一，统一关掉再恢复（wgpu-hal 同法）。
        let srgb = gl.is_enabled(GL_FRAMEBUFFER_SRGB);
        if srgb {
            gl.disable(GL_FRAMEBUFFER_SRGB);
        }
        gl.bind_framebuffer(GL_READ_FRAMEBUFFER, None);
        gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, Some(fb));
        gl.blit_framebuffer(0, 0, w, h, 0, 0, w, h, GL_COLOR_BUFFER_BIT, GL_NEAREST);
        gl.bind_framebuffer(GL_READ_FRAMEBUFFER, Some(fb));
        gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
        gl.blit_framebuffer(0, 0, w, h, 0, h, w, 0, GL_COLOR_BUFFER_BIT, GL_NEAREST);
        gl.bind_framebuffer(GL_READ_FRAMEBUFFER, None);
        gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
        if srgb {
            gl.enable(GL_FRAMEBUFFER_SRGB);
        }
        if scissor {
            gl.enable(GL_SCISSOR_TEST);
        }
    });
    crate::state_inner::check_err("present_flip");
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 桌面/安卓：先翻正默认帧缓冲，再 swap_buffers（换帧+同步）；Web：空操作。
// 2. 换帧报错不回传错误值（租约已消耗）：记 log::error!(码)，由下一次 get_current 以 Lost/Outdated 暴露。
pub(crate) fn present_impl(t: SurfaceTexture) {
    crate::state_inner::check_err("present");
    #[cfg(not(target_arch = "wasm32"))]
    flip_back_buffer(&t.texture);
    #[cfg(target_arch = "wasm32")]
    let _ = &t;
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
