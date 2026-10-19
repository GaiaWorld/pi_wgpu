//! pi_wgpu · 错误面壳（根 enum，形= wgpu 25.0.2 `wgpu::Error`/`ErrorFilter`/`UncapturedErrorHandler`）。
//! 词面直齐上游：三变体同字段名（source / description）；通道同法（push/pop_error_scope、
//! on_uncaptured_error 挂 Device 声明面，芯在 entry_inner）。差异登记：ErrorSource 恒
//! Send+Sync（上游按 send_sync 配型，本层无 wasm 无同步路）；默认未捕获行为=log 降级非 panic
//! （AGENTS 禁 panic 上递纪律），上游=默认 panic——能力差异入册。

/// GPU API 用法错误根 enum（形= 上游 wgpu::Error：变体、字段名逐一对齐）。
/// 本层拒绝短码经 `description` 承载（短码+label 内嵌=既有断言面词形，断言迁移入批）。
#[derive(Debug)]
pub enum Error {
    /// 显存/资源耗尽（本层暂无主动判定源，通道占位=上游同形）。
    OutOfMemory {
        /// 下层错误源。
        source: ErrorSource,
    },
    /// 违约用法错误（传参/状态违反接口合同）。拒止短码目录在此登记。
    Validation {
        /// 下层错误源。
        source: ErrorSource,
        /// 违约描述（本层=短码文本，如 `ErrCache:...`）。
        description: String,
    },
    /// 内部错误（非合同预期的失败，本层=驱动/环境拒绝等）。
    Internal {
        /// 下层错误源。
        source: ErrorSource,
        /// 内部错误描述。
        description: String,
    },
}

/// 错误来源低层句柄（Send+Sync 恒定=本层桌面-only 差异登记）。
pub type ErrorSource = Box<dyn std::error::Error + Send + Sync + 'static>;

/// 错误作用域过滤器（形= 上游 wgpu::ErrorFilter，变体逐一同名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorFilter {
    /// 只捕获 OOM。
    OutOfMemory,
    /// 只捕获违约。
    Validation,
    /// 只捕获内部错误。
    Internal,
}

/// 未捕获错误回调合同（形= 上游 trait 别名面：Blantket impl 同构）。
pub trait UncapturedErrorHandler: Fn(Error) + Send + 'static {}
impl<T> UncapturedErrorHandler for T where T: Fn(Error) + Send + 'static {}

#[cfg(test)]
mod tests;
