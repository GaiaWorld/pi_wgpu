//! pi_wgpu · 错误面芯：Display/std::error::Error 出码、filter 匹配、短码源与拒止登记。

use crate::error::{Error, ErrorFilter, ErrorSource};

/// 由短码文本构造 Validation 错误（source=短码本体＝字段直读；std source() 链=None 定形）。
pub(crate) fn validation(code: &str) -> Error {
    Error::Validation {
        source: code.to_string(),
        description: code.to_string(),
    }
}

/// 由短码文本构造 Internal 错误。
pub(crate) fn internal(code: &str) -> Error {
    Error::Internal {
        source: code.to_string(),
        description: code.to_string(),
    }
}

/// filter→变体归属判定（形= 上游 scope 捕获语义：过滤器只收同名变体类）。
pub(crate) fn filter_matches(filter: ErrorFilter, err: &Error) -> bool {
    matches!(
        (filter, err),
        (ErrorFilter::OutOfMemory, Error::OutOfMemory { .. })
            | (ErrorFilter::Validation, Error::Validation { .. })
            | (ErrorFilter::Internal, Error::Internal { .. })
    )
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OutOfMemory { .. } => write!(f, "Out of memory"),
            Error::Validation { description, .. } => write!(f, "Validation error: {description}"),
            Error::Internal { description, .. } => write!(f, "Internal error: {description}"),
        }
    }
}

impl std::error::Error for Error {}

/// Device 错误通道态（被 entry_inner 的 DeviceInner 持有）：
/// scopes=push/pop 栈（内层优先捕获），uncaptured=用户回调槽。
#[derive(Default)]
pub(crate) struct ErrorState {
    /// 作用域栈：每层 =（过滤器，捕获队列按发生序）。
    pub(crate) scopes: Vec<(ErrorFilter, Vec<Error>)>,
    /// 未捕获错误回调（on_uncaptured_error 落位；None=缺省 log 降级）。
    pub(crate) uncaptured: Option<Box<dyn crate::error::UncapturedErrorHandler>>,
}

impl ErrorState {
    /// push 开层（上游同法）。
    pub(crate) fn push(&mut self, filter: ErrorFilter) {
        self.scopes.push((filter, Vec::new()));
    }

    /// pop 收层：取最内层队列首条（无=None），层整层出栈（上游 pop 终态）。
    pub(crate) fn pop(&mut self) -> Option<Error> {
        let (_filter, q) = self.scopes.pop()?;
        q.into_iter().next()
    }

    /// report 捕获路：最内匹配层收入；不匹配错误逐层外找（首个匹配层），全空=回调/log。
    /// 返 true=已被通道接管。
    pub(crate) fn capture(&mut self, err: &Error) -> bool {
        for (filter, q) in self.scopes.iter_mut().rev() {
            if filter_matches(*filter, err) {
                q.push(match err {
                    Error::Validation { description, .. } => validation(description),
                    Error::Internal { description, .. } => internal(description),
                    Error::OutOfMemory { .. } => internal("OutOfMemory"),
                });
                return true;
            }
        }
        false
    }
}

impl std::fmt::Debug for ErrorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorState")
            .field("scopes", &self.scopes.len())
            .field("uncaptured", &self.uncaptured.is_some())
            .finish()
    }
}
