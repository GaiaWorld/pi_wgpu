// 设计：pi-wgpu-rebuild
//! caps · 能力探测：版本/扩展读数 + 已知问题设备的特判。
//! 隐藏决定=特判集合的存放位置——一切「按设备名打补丁」的判定集中本模块，禁止散写他处。
//! 契约权威文本住本面注释；本面全部 @internal。

/// 能力快照（建后即不可变）。不变量：对外可见能力=这份快照的投影；
/// 特判位集中于此——公开面只读快照，不各自查扩展字符串。 @internal
#[derive(Debug)]
pub struct Caps {
    /// GL/GLES 核心版本对（主, 次）；形态区分（ES/桌面）走 _ext_family 标记位，保持本对为纯版本读数。
    pub(crate) _gl_version: (u32, u32),
    /// 特判位=Qualcomm/Adreno 系 GPU 设备（含小米等 Adreno 机型）：按特判表命中时置位，其压缩纹理位降级消费在 fill 投影处单点执行。
    pub(crate) _qualcomm: bool,
    /// 特判位=iOS18/Safari 系设备：按特判表命中时置位，其 ASTC 位降级消费在 fill 投影处单点执行。
    pub(crate) _ios18: bool,
    /// 扩展能力位掩码：probe 按扩展表/版本判定逐项写入（芯内 CAP_* 词汇，未检出恒 0）；fill 投影为官方 Features 同名位。建后只读。
    pub(crate) _ext_family: u64,
}

/// 探测入口：附着建立后运行一次（唯一合法调用时机；早于附着=断言）。 @internal
pub(crate) fn probe() -> Caps { crate::caps_inner::probe_impl() }
/// 快照→对外 Features 位词汇表的投影（纯函数，不失败）。 @internal
pub(crate) fn fill_features(c: &Caps) -> crate::types::Features { crate::caps_inner::fill_impl(c) }
