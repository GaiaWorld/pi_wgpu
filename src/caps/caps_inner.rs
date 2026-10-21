// 设计：pi-wgpu-rebuild
//! caps 芯 · 能力探测（probe）与官方 Features 投影（fill）。契约权威文本见 caps 壳注释。
//! 特判表=全库唯一「按设备名打补丁」容器：新增问题驱动特判只在 SPECIAL_ROWS 加一行——
//! probe 据此写快照标志位、fill 据此算降级清除位，两处消费同源于表，禁止调用点散写。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use glow::HasContext;

use crate::caps::Caps;
use crate::context_inner::with_gl;

/// GL_VENDOR 查询参数号（0x1F00，GL 规范值；pi_glow 根面未导出 GL_* 枚举行名，按规范内联）。
const GL_VENDOR_PARAM: u32 = 0x1F00;
/// GL_RENDERER 查询参数号（0x1F01，GL 规范值；同上内联原因）。
const GL_RENDERER_PARAM: u32 = 0x1F01;

/// —— 扩展能力位词汇（芯内索引；probe 写、fill 投影、B3 起 formats 直载校验消费；未检出恒 0=不谎报）——
/// S3TC/DXT 解码能力（DDS 主力族；桌面永非核心，仅扩展证据）。
pub(crate) const CAP_S3TC: u64 = 1 << 0;
/// BPTC（BC6H/BC7）解码能力（桌面 GL>=4.2 核心或扩展）。
pub(crate) const CAP_BPTC: u64 = 1 << 1;
/// RGTC（BC4/BC5）解码能力（桌面 GL>=4.2 核心或扩展）。
pub(crate) const CAP_RGTC: u64 = 1 << 2;
/// ASTC LDR 解码能力（仅扩展证据）。
pub(crate) const CAP_ASTC_LDR: u64 = 1 << 3;
/// ASTC HDR 解码能力（本投影域 u64 无官方同名位，fill 不消费；去向登记在后续对表批）。
pub(crate) const CAP_ASTC_HDR: u64 = 1 << 4;
/// ETC2/EAC 解码能力（ES>=3.0 或桌面>=4.3 核心或兼容扩展，probe 处按形态确证）。
pub(crate) const CAP_ETC2: u64 = 1 << 5;
/// FLOAT32 纹理线性过滤能力。
pub(crate) const CAP_FLOAT_LINEAR: u64 = 1 << 6;
/// 浮点颜色附件可渲染能力（供 B3 formats 直载校验；Features 投影不占位=保守不谎报）。
pub(crate) const CAP_FLOAT_RENDER: u64 = 1 << 7;
/// 形态标记位：GLES/WebGL2=true——版本对保持纯读数，形态规则差异只在本芯判定、不泄入投影。
pub(crate) const CAP_EMBEDDED: u64 = 1 << 8;
/// 半浮点（16F）颜色附件可渲染能力：ES3 需 EXT_color_buffer_half_float（或更强的浮点扩展）；
/// 桌面 GL3.0 起 RGBA16F 核心可渲染=不置位也不影响（formats 门控只对 ES 形态生效）。
pub(crate) const CAP_RENDER_HALF: u64 = 1 << 9;

/// —— 官方 Features 位值（对齐本地 wgpu fork 的 webgpu 组低 u64 域=本壳 Features 唯一投影域；全表对账后续批次）——
/// DEPTH32FLOAT_STENCIL8：官方位 1<<1。
const FEAT_DEPTH32FLOAT_STENCIL8: u64 = 1 << 1;
/// TEXTURE_COMPRESSION_BC：官方位 1<<2（本库口径：S3TC∨BPTC 任一确证即置位）。
const FEAT_TEXTURE_COMPRESSION_BC: u64 = 1 << 2;
/// TEXTURE_COMPRESSION_ETC2：官方位 1<<4。
const FEAT_TEXTURE_COMPRESSION_ETC2: u64 = 1 << 4;
/// TEXTURE_COMPRESSION_ASTC：官方位 1<<5。
const FEAT_TEXTURE_COMPRESSION_ASTC: u64 = 1 << 5;
/// FLOAT32_FILTERABLE：官方位 1<<11。
const FEAT_FLOAT32_FILTERABLE: u64 = 1 << 11;

/// —— 最近一次探测快照缓存（芯内驻留；probe 写·formats 等实现批门控读）——
/// 能力位 family 读数（CAP_* 全位域，含形态标记位）。写序：family→meta→READY(release)。
static CAPS_FAMILY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// 元读数打包：主版低 8 | 次版 8..16 | embedded 1<<16 | qualcomm 1<<17 | ios18 1<<18。
static CAPS_META: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// 快照有效标志：false=尚未 attach 后即探（消费端=断言，不静默放行）。
static CAPS_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// 探测快照的锁外轻量读数（门控消费形制；权威仍是 Caps 快照本体——此处为同值副本）。
pub(crate) struct LatestCaps {
    /// 能力位 family（含形态标记位）。
    pub(crate) family: u64,
    /// 核心版本对（主, 次）。
    pub(crate) version: (u32, u32),
    /// 形态标记：GLES/WebGL2。
    pub(crate) es: bool,
}

/// probe 末尾写入缓存（release 序：读者见 READY=true 时 family/meta 必然同代）。
fn store_latest(c: &Caps) {
    CAPS_FAMILY.store(c._ext_family, Ordering::Release);
    let meta: u64 = (c._gl_version.0 as u64 & 0xFF)
        | ((c._gl_version.1 as u64 & 0xFF) << 8)
        | (if (c._ext_family & CAP_EMBEDDED) != 0 {
            1 << 16
        } else {
            0
        })
        | (if c._qualcomm { 1 << 17 } else { 0 })
        | (if c._ios18 { 1 << 18 } else { 0 });
    CAPS_META.store(meta, Ordering::Release);
    CAPS_READY.store(true, Ordering::Release);
}

/// 锁外读最近快照；未探=None（消费端断言话术自带上下文）。
pub(crate) fn latest_caps() -> Option<LatestCaps> {
    if !CAPS_READY.load(Ordering::Acquire) {
        return None;
    }
    let family = CAPS_FAMILY.load(Ordering::Acquire);
    let meta = CAPS_META.load(Ordering::Acquire);
    Some(LatestCaps {
        family,
        version: ((meta as u32) & 0xFF, ((meta >> 8) as u32) & 0xFF),
        es: (meta & (1 << 16)) != 0,
    })
}

/// 测试注入口：无 GL 环境验证门控表路径（仅测试构型可见，产品路径由 probe 写）。
#[cfg(test)]
pub(crate) fn inject_latest(family: u64, version: (u32, u32), es: bool) {
    let mut f = family;
    if es {
        f |= CAP_EMBEDDED;
    }
    let mut c = Caps {
        _gl_version: version,
        _qualcomm: false,
        _ios18: false,
        _ext_family: f,
    };
    if es {
        c._ext_family |= CAP_EMBEDDED;
    } else {
        c._ext_family &= !CAP_EMBEDDED;
    }
    store_latest(&c);
}

/// 特判标志路由：命中特判行=置 Caps 快照的哪个布尔位（与壳快照字段一一对应）。
enum JudgmentFlag {
    /// Qualcomm/Adreno 系（含小米等 Adreno 机型）。
    Qualcomm,
    /// iOS18/Safari 系。
    Ios18,
}

/// 特判表行：设备串关键字命中 → 置标志位；投影时该标志驱动清除声明的能力位集合（保守降级，不谎报驱动虚报）。
struct SpecialRow {
    /// 审计名（日志追溯与变更点登记用）。
    name: &'static str,
    /// 归一化关键字（小写）：对 vendor∨renderer∨形态附加源合并串做子串匹配，任一命中即算。
    keywords: &'static [&'static str],
    /// 命中后投影中清除的 CAP_* 位集合（表行声明，probe/fill 同源消费的唯一来源）。
    clears: u64,
    /// 写入的快照标志位。
    flag: JudgmentFlag,
}

/// 全库唯一特判表——新增问题驱动特判只往这里加一条。
const SPECIAL_ROWS: &[SpecialRow] = &[
    SpecialRow {
        name: "qualcomm-adreno",
        keywords: &["qualcomm", "adreno"],
        clears: CAP_S3TC | CAP_BPTC | CAP_RGTC | CAP_ASTC_LDR | CAP_ASTC_HDR,
        flag: JudgmentFlag::Qualcomm,
    },
    SpecialRow {
        name: "ios18-safari-astc",
        keywords: &["os 18_", "cpu os 18_"],
        clears: CAP_ASTC_LDR | CAP_ASTC_HDR,
        flag: JudgmentFlag::Ios18,
    },
];

/// 扩展名归一：去 GL_ 前缀（native 形态带、web 形态不带）+ 小写（匹配表全小写字面）。
fn norm_ext(raw: &str) -> String {
    raw.strip_prefix("GL_").unwrap_or(raw).to_ascii_lowercase()
}

/// 特判分类：全部特判行关键字对合并设备串逐一子串匹配，输出两标志位（清除集投影时按表重derive，快照只存真相）。
fn classify(haystack: &str) -> (bool, bool) {
    let mut qualcomm = false;
    let mut ios18 = false;
    for row in SPECIAL_ROWS {
        if row.keywords.iter().any(|k: &&str| haystack.contains(k)) {
            match row.flag {
                JudgmentFlag::Qualcomm => qualcomm = true,
                JudgmentFlag::Ios18 => ios18 = true,
            }
        }
    }
    (qualcomm, ios18)
}

/// 快照标志位对应的投影清除集（表驱动唯一派生处）。
fn clears_of(qualcomm: bool, ios18: bool) -> u64 {
    let mut c: u64 = 0;
    for row in SPECIAL_ROWS {
        let on = match row.flag {
            JudgmentFlag::Qualcomm => qualcomm,
            JudgmentFlag::Ios18 => ios18,
        };
        if on {
            c |= row.clears;
        }
    }
    c
}

/// 形态附加匹配源：Web=UA 串（GL 设备串不含系统版本，iOS18 判定只能借 UA）；其余形态=空串（UA 关键字恒不命中）。
#[cfg(target_arch = "wasm32")]
fn extra_haystack() -> String {
    web_sys::window()
        .map(|w| w.navigator())
        .and_then(|n| n.user_agent())
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// 非 Web 形态无附加源。
#[cfg(not(target_arch = "wasm32"))]
fn extra_haystack() -> String {
    String::new()
}

// 1. 读版本对、扩展集、设备名（全部驱动查询统一经 with_gl 重入锁内执行）。
// 2. 压缩族/浮点族逐项判定：核心版本路径 ∨ 扩展证据路径；未检出恒 0。
// 3. 特判表只写标志位——快照=版本对+形态位+能力位+特判标志，投影与降级全部可从中纯函数还原。
// 4. 读数无效（版本 0.0）=trace 后断言；附着不可用归创建期错误面处理，探测处不吞错。
pub(crate) fn probe_impl() -> Caps {
    with_gl(|gl| {
        let ver = gl.version();
        if ver.major == 0 && ver.minor == 0 {
            log::error!("caps probe: GL 版本读数 0.0（附着无效或环境异常）");
            panic!("caps probe 断言: 版本读数无效，须附着建立后探测");
        }
        let mut exts: Vec<String> = gl
            .supported_extensions()
            .iter()
            .map(|s| norm_ext(s))
            .collect();
        exts.sort();
        exts.dedup();
        let has = |names: &[&str]| -> bool {
            names
                .iter()
                .any(|n: &&str| exts.iter().any(|e: &String| e.as_str() == *n))
        };
        let es = ver.is_embedded;
        let mut family: u64 = 0;
        if es {
            family |= CAP_EMBEDDED;
        }
        if has(&["texture_compression_s3tc", "texture_compression_dxt1"]) {
            family |= CAP_S3TC;
        }
        if has(&["texture_compression_bptc"]) || (!es && (ver.major, ver.minor) >= (4, 2)) {
            family |= CAP_BPTC;
        }
        if has(&["texture_compression_rgtc"]) || (!es && (ver.major, ver.minor) >= (4, 2)) {
            family |= CAP_RGTC;
        }
        if has(&[
            "khr_texture_compression_astc_ldr",
            "ext_texture_compression_astc_decoder",
            "webgl_compressed_texture_astc",
        ]) {
            family |= CAP_ASTC_LDR;
        }
        if has(&[
            "khr_texture_compression_astc_hdr",
            "webgl_compressed_texture_astc_hdr",
        ]) {
            family |= CAP_ASTC_HDR;
        }
        if (es && (ver.major, ver.minor) >= (3, 0))
            || (!es && (ver.major, ver.minor) >= (4, 3))
            || has(&["arb_es3_compatibility", "ext_texture_compression_eac"])
        {
            family |= CAP_ETC2;
        }
        if (!es && (ver.major, ver.minor) >= (3, 0)) || has(&["oes_texture_float_linear"]) {
            family |= CAP_FLOAT_LINEAR;
        }
        if has(&[
            "arb_color_buffer_float",
            "ext_color_buffer_float",
            "webgl_color_buffer_float",
        ]) {
            family |= CAP_FLOAT_RENDER;
            // 浮点附件扩展同时覆盖 16F（WebGL2 的 EXT_color_buffer_float 即含半浮点）。
            family |= CAP_RENDER_HALF;
        } else if has(&["ext_color_buffer_half_float"]) {
            family |= CAP_RENDER_HALF;
        }
        let vendor = unsafe { gl.get_parameter_string(GL_VENDOR_PARAM) }.to_ascii_lowercase();
        let renderer = unsafe { gl.get_parameter_string(GL_RENDERER_PARAM) }.to_ascii_lowercase();
        let haystack = format!("{vendor} {renderer} {}", extra_haystack());
        let (qualcomm, ios18) = classify(&haystack);
        log::debug!(
            "caps probe: v{}.{} exts={} qualcomm={qualcomm} ios18={ios18}",
            ver.major,
            ver.minor,
            exts.len()
        );
        let _ = vendor;
        let caps = Caps {
            _gl_version: (ver.major, ver.minor),
            _qualcomm: qualcomm,
            _ios18: ios18,
            _ext_family: family,
        };
        store_latest(&caps);
        caps
    })
}

// 1. 快照→官方 Features 同名位逐项映射；本库无对应物/未确证的能力保持 0。
// 2. 特判降级按标志位经表派生清除集后过滤——纯函数，快照即全部真相。
pub(crate) fn fill_impl(c: &Caps) -> crate::types::Features {
    let family = c._ext_family & !clears_of(c._qualcomm, c._ios18);
    let mut flags = crate::types::Features::empty();
    if c._gl_version >= (3, 0) {
        // D32F_S8：桌面 GL3.0 与 ES3.0 均已入核心，版本确证即可报。
        flags |= crate::types::Features::DEPTH32FLOAT_STENCIL8;
    }
    if (family & (CAP_S3TC | CAP_BPTC)) != 0 {
        flags |= crate::types::Features::TEXTURE_COMPRESSION_BC;
    }
    if (family & CAP_ETC2) != 0 {
        flags |= crate::types::Features::TEXTURE_COMPRESSION_ETC2;
    }
    if (family & CAP_ASTC_LDR) != 0 {
        flags |= crate::types::Features::TEXTURE_COMPRESSION_ASTC;
    }
    if (family & CAP_FLOAT_LINEAR) != 0 {
        flags |= crate::types::Features::FLOAT32_FILTERABLE;
    }
    // 形态事实：本层纹理格式面=静态白名单（adapter_format_features 恒等返回，不依赖驱动逐格式查询），
    // 故「按适配器逐格式能力」这一位恒成立——GL 面全部格式已实现，该位按 wgpu 语义本就多余
    // （见 wgpu-types features.rs 注释）。不报它=render_core 在 Functionality 优先级下强加的请求被拒。
    if c._gl_version >= (3, 0) {
        flags |= crate::types::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 纯投影：能力位→官方位映射全路径校验。
    #[test]
    fn fill_projection_test() {
        let c = Caps {
            _gl_version: (3, 3),
            _qualcomm: false,
            _ios18: false,
            _ext_family: CAP_S3TC | CAP_FLOAT_LINEAR,
        };
        let f = fill_impl(&c);
        use crate::types::Features;
        assert!(f.contains(Features::DEPTH32FLOAT_STENCIL8));
        assert!(f.contains(Features::TEXTURE_COMPRESSION_BC));
        assert!(f.contains(Features::FLOAT32_FILTERABLE));
        assert!(!f.contains(Features::TEXTURE_COMPRESSION_ASTC));
        assert!(!f.contains(Features::TEXTURE_COMPRESSION_ETC2));
    }

    /// 特判降级：Qualcomm 清 BC 族+ASTC、iOS18 只清 ASTC；保留能力位不受他标志误伤。
    #[test]
    fn fill_special_case_downgrade_test() {
        let base = CAP_S3TC | CAP_ASTC_LDR | CAP_FLOAT_LINEAR | CAP_ETC2;
        let q = Caps {
            _gl_version: (3, 2),
            _qualcomm: true,
            _ios18: false,
            _ext_family: base,
        };
        let fq = fill_impl(&q);
        use crate::types::Features;
        assert_eq!(
            fq,
            Features::DEPTH32FLOAT_STENCIL8
                | Features::TEXTURE_COMPRESSION_ETC2
                | Features::FLOAT32_FILTERABLE
                | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        );
        let i = Caps {
            _gl_version: (3, 2),
            _qualcomm: false,
            _ios18: true,
            _ext_family: base,
        };
        let fi = fill_impl(&i);
        assert_eq!(
            fi,
            Features::DEPTH32FLOAT_STENCIL8
                | Features::TEXTURE_COMPRESSION_BC
                | Features::TEXTURE_COMPRESSION_ETC2
                | Features::FLOAT32_FILTERABLE
                | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        );
    }

    /// 特判表分类匹配：子串命中与全不命中路径。
    #[test]
    fn classify_test() {
        assert_eq!(classify("ati radeon x"), (false, false));
        assert_eq!(classify("qualcomm inc adreno (tm) 640"), (true, false));
        assert_eq!(
            classify("apple a17 gpu mozilla/5.0 cpu os 18_2 like"),
            (false, true)
        );
    }

    /// 扩展名归一化：GL_ 前缀去除+小写；无前缀输入原样小写。
    #[test]
    fn norm_ext_test() {
        assert_eq!(
            norm_ext("GL_EXT_texture_compression_astc_decoder"),
            "ext_texture_compression_astc_decoder"
        );
        assert_eq!(
            norm_ext("WEBGL_compressed_texture_astc"),
            "webgl_compressed_texture_astc"
        );
    }

    // 实机探测冒烟由 B1+B2 合并用例承载（context_inner::gl_state_machine_test，单线程合验）。
}
