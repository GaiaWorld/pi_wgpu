// 设计：pi-wgpu-rebuild
//! caps · 能力探测：版本/扩展读数 + 已知问题设备的特判。
//! 隐藏决定=特判集合的存放位置——一切「按设备名打补丁」的判定集中本模块，禁止散写他处。
//! 契约权威文本住本面注释；本面全部 @internal。

/// 能力快照（建后即不可变）。不变量：对外可见能力=这份快照的投影；
/// 特判位集中于此——公开面只读快照，不各自查扩展字符串。 @internal
#[derive(Debug)] pub struct Caps { pub(crate) _gl_version: (u32, u32), pub(crate) _qualcomm: bool, pub(crate) _ios18: bool }

/// 探测入口：附着建立后运行一次（唯一合法调用时机；早于附着=断言）。 @internal
pub(crate) fn probe() -> Caps { crate::caps_inner::probe_impl() }
/// 快照→对外 Features 位词汇表的投影（纯函数，不失败）。 @internal
pub(crate) fn fill_features(c: &Caps) -> crate::types::Features { crate::caps_inner::fill_impl(c) }
