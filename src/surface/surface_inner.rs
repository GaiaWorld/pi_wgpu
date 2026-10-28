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
    crate::state_inner::churn_add(0);
    *s._cfg.borrow_mut() = Some(c.clone());
    *s._creation.borrow_mut() += 1;
    // 离屏「屏幕」纹理：尺寸/格式随配置定格（旧的一路 Clone 引用计数收尾，跨帧租约仍安全）。
    let stage_desc = TextureDescriptor {
        label: Some("surface-stage"),
        size: Extent3d {
            width: c.width,
            height: c.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: c.format,
        usage: c.usage | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let stage = crate::entry_inner::create_texture_impl(_d, &stage_desc);
    *s._stage.borrow_mut() = Some(stage);
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. _cfg=None 前置：未配置=Err(Outdated)+log 码（须先 configure）。
// 2. 帧=离屏「屏幕」纹理的真实克隆（_surface 仅作标记；视图走 create_view 实路）；suboptimal 恒 true（官方旧语义，登迁移清单）。
// 3. Lost 代号推进由上下文侧 gen_bump 承担（本面不做失效动作，重建=上层 re-request_device）。
pub(crate) fn get_current_impl(
    s: &Surface<'_>,
) -> std::result::Result<SurfaceTexture, SurfaceError> {
    if s._cfg.borrow().is_none() {
        log::error!("surface {:?}", SurfaceErr::NotConfigured);
        return Err(SurfaceError::Outdated);
    }
    if s._cfg.borrow().is_none() {
        log::error!("surface {:?}", SurfaceErr::NotConfigured);
        return Err(SurfaceError::Outdated);
    }
    let stage = s._stage.borrow().as_ref().cloned();
    let Some(stage) = stage else {
        log::error!("surface {:?}", SurfaceErr::NotConfigured);
        return Err(SurfaceError::Outdated);
    };
    // 帧=屏幕纹理的克隆（同一驱动对象；_surface 仅作标记），视图走真实纹理视图。
    let mut texture = stage;
    texture._surface = true;
    let view = crate::entry_inner::create_view_impl(&texture, &TextureViewDescriptor::default());
    // suboptimal=「本帧面相对配置已劣化，须重配」。本层离屏屏幕纹理即配置产物、尺寸格式恒贴合，
    // 恒报 false；旧形恒 true 会让上层每帧重新 configure（屏幕纹理逐帧重建，产品实测 RTT 跨帧变化/闪黑）。
    Ok(SurfaceTexture {
        texture,
        suboptimal: false,
        presented: false,
        _view: view,
    })
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 按附着能力位→formats 候选表×present/alpha 组合回报（纯查询，不建状态）。
pub(crate) fn capabilities_impl(_a: &crate::entry::Adapter) -> SurfaceCapabilities {
    SurfaceCapabilities {
        // Web=ES 域：sRGB×BGRA 无核心组合（formats 门拒路），候选只报 RGBA 族（登记差异）；
        // 桌面/GL 域四候选全报（swapchain 习惯形保留）。
        formats: if cfg!(target_arch = "wasm32") {
            vec![TextureFormat::Rgba8Unorm, TextureFormat::Rgba8UnormSrgb]
        } else {
            vec![
                TextureFormat::Bgra8Unorm,
                TextureFormat::Bgra8UnormSrgb,
                TextureFormat::Rgba8Unorm,
                TextureFormat::Rgba8UnormSrgb,
            ]
        },
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
// 渲染全部落在离屏「屏幕」纹理（configure 期建，帧视图=它的真实视图），与 wgpu 的
// surface texture 同语义；出码侧 ADJUST_COORDINATE_SPACE 让内容按 wgpu「行 0=顶」方向存，
// 默认帧缓冲仍是 GL「行 0=底」——present 用一次反向源矩形 blit 翻正（不可直画：自 blit 重叠=未定义，
// glViewport 负高在 ES 非法）。
const GL_FRAMEBUFFER: u32 = 0x8D40;
const GL_READ_FRAMEBUFFER: u32 = 0x8CA8;
const GL_DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
const GL_NEAREST: u32 = 0x2600;

const GL_SCISSOR_TEST: u32 = 0x0C11;
/// 帧面色空间转换开关——仅桌面 core 可发：WebGL2 不暴露该 cap 枚举（enable/is_enabled 均
/// INVALID_ENUM 污染错误队列），其 sRGB 读写转换由格式语义自动承担，无开关位可对齐亦无需对齐。
#[cfg(not(target_arch = "wasm32"))]
const GL_FRAMEBUFFER_SRGB: u32 = 0x8DB9;

/// 上屏 blit 的 FBO 缓存行（键=屏幕纹理名 + 上下文代号；纹理随 configure 重建即换键）。
struct PresentFbo {
    /// FBO 驱动名。
    fbo: u64,
    /// 挂在该 FBO 上的屏幕纹理名（gid 归一化）。
    tex: u64,
    /// 上下文代号（重附着后旧名失效）。
    gen: u32,
}
/// 进程级唯一上屏 FBO（单附着语义；屏幕纹理换名即重建）。
static PRESENT_FBO: std::sync::Mutex<Option<PresentFbo>> = std::sync::Mutex::new(None);

/// 取/建上屏 FBO（把屏幕纹理挂 COLOR0）：键不匹配即删旧建新；跨代号只丢记录不删。
fn ensure_present_fbo(tex: u64, target: u32) -> u64 {
    let gen = crate::context_inner::generation_now();
    let mut guard = PRESENT_FBO.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = guard.as_ref() {
        if s.tex == tex && s.gen == gen {
            return s.fbo;
        }
        if s.gen == gen {
            let fbo = s.fbo;
            crate::context_inner::with_gl(|gl| unsafe {
                gl.delete_framebuffer(crate::gid::framebuffer_from(fbo));
            });
        }
    }
    let fbo = crate::context_inner::with_gl(|gl| unsafe {
        let fbo = gl.create_framebuffer().expect("surface 断言: 上屏 FBO 建失败");
        gl.bind_framebuffer(GL_FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(
            GL_FRAMEBUFFER,
            GL_COLOR_ATTACHMENT0,
            target,
            Some(crate::gid::texture_from(tex)),
            0,
        );
        let st = gl.check_framebuffer_status(GL_FRAMEBUFFER);
        if st != 0x8CD5 {
            log::error!("surface: 上屏 FBO 不完整 status={st:#x}（present 将不可见）");
        }
        gl.bind_framebuffer(GL_FRAMEBUFFER, None);
        crate::gid::id_of_framebuffer(fbo)
    });
    *guard = Some(PresentFbo { fbo, tex, gen });
    fbo
}

/// 屏幕纹理上屏：一次翻转 blit（READ=屏幕 FBO，DRAW=默认帧缓冲，源矩形 y 反向）。
/// SCISSOR/sRGB 临时关闭后按原值恢复（blit 受裁剪测试影响；sRGB 段仅桌面——WebGL2 无该开关，
/// 帧面色空间由 canvas framebufferColorSpace 决定，本层不触碰，登记差异）。
fn blit_present(t: &crate::entry::Texture) {
    let Some(name) = t._h.name() else {
        log::error!("surface: 上屏纹理无驱动名（present 跳过）");
        return;
    };
    let w = t._size.width as i32;
    let h = t._size.height as i32;
    if w <= 0 || h <= 0 {
        return;
    }
    let fbo = ensure_present_fbo(name.0, t._target);
    let fb = crate::gid::framebuffer_from(fbo);
    crate::context_inner::with_gl(|gl| unsafe {
        let scissor = gl.is_enabled(GL_SCISSOR_TEST);
        if scissor {
            gl.disable(GL_SCISSOR_TEST);
        }
        // sRGB 帧面：blit 期间的转换行为各驱动不一，统一关掉再恢复（wgpu-hal 同法）。
        #[cfg(not(target_arch = "wasm32"))]
        let srgb = gl.is_enabled(GL_FRAMEBUFFER_SRGB);
        #[cfg(not(target_arch = "wasm32"))]
        if srgb {
            gl.disable(GL_FRAMEBUFFER_SRGB);
        }
        gl.bind_framebuffer(GL_READ_FRAMEBUFFER, Some(fb));
        gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
        gl.blit_framebuffer(0, h, w, 0, 0, 0, w, h, GL_COLOR_BUFFER_BIT, GL_NEAREST);
        gl.bind_framebuffer(GL_READ_FRAMEBUFFER, None);
        gl.bind_framebuffer(GL_DRAW_FRAMEBUFFER, None);
        #[cfg(not(target_arch = "wasm32"))]
        if srgb {
            gl.enable(GL_FRAMEBUFFER_SRGB);
        }
        if scissor {
            gl.enable(GL_SCISSOR_TEST);
        }
    });
    crate::state_inner::check_err("present_blit");
}

// 设计预期（伪码·示意·非唯一实现）：
// 1. 桌面/安卓：一次翻转 blit 上屏，再 swap_buffers（换帧+同步）；Web：同 blit 上默认帧缓冲后
//    空 swap（隐式合成，浏览器出帧）。
// 2. 换帧报错不回传错误值（租约已消耗）：记 log::error!(码)，由下一次 get_current 以 Lost/Outdated 暴露。
pub(crate) fn present_impl(t: SurfaceTexture) {
    crate::state_inner::check_err("present");
    blit_present(&t.texture);
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

/// Web 形态：自画布取 WebGL2 上下文并包 glow 入口，**首个呈现面创建时**注入（幂等：已有入口跳过；
/// OffscreenCanvas 面未接=Err 拒面，登记差异）。注：WebGL2 版串须可被 glow Version 解析（实装自察）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn web_inject_from_canvas(t: &crate::surface::SurfaceTarget<'_>) -> bool {
    if crate::context_inner::web_gl_present() {
        return true;
    }
    let canvas: web_sys::HtmlCanvasElement = match t {
        crate::surface::SurfaceTarget::Canvas(c) => c.clone(),
        crate::surface::SurfaceTarget::OffscreenCanvas(_) => {
            log::error!("surface WebOffscreenCanvasUnsupported");
            return false;
        }
        _ => {
            log::error!("surface WebWindowTargetInvalid");
            return false;
        }
    };
    use wasm_bindgen::JsCast as _;
    // 上下文属性属库级差异收口（旧仓先例：present/blit 差异不入用例）：默认 antialias:true
    // 使画布默认 fb 采样=4，本层 1 采样 present blit 0→4 恒拒；alpha=false=WebGPU 不透明面对齐。
    let attrs: web_sys::WebGlContextAttributes = web_sys::WebGlContextAttributes::new();
    attrs.set_antialias(false);
    attrs.set_alpha(false);
    let opts: wasm_bindgen::JsValue = attrs.into();
    let js: wasm_bindgen::JsValue = match canvas.get_context_with_context_options("webgl2", &opts) {
        Ok(Some(g)) => g.into(),
        _ => {
            log::error!("surface WebContextMissing");
            return false;
        }
    };
    let gl2: web_sys::WebGl2RenderingContext = match js.dyn_into() {
        Ok(g) => g,
        Err(_) => {
            log::error!("surface WebContextNotWebGl2");
            return false;
        }
    };
    let gl = glow::Context::from_webgl2_context(gl2);
    crate::context_inner::web_bind_glow(gl);
    log::info!("surface: Web GL 入口注入完成");
    true
}

