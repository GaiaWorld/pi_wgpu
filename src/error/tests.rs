//! pi_wgpu · 错误面形状/通道单测（无 GL 依赖）。

use crate::error::{Error, ErrorFilter};
use crate::error_inner::{internal, validation};
use std::future::Future as _;

#[test]
fn error_variants_shape_and_source_chain() {
    let v: Error = validation("ErrCache: 占位类型无构造面");
    assert!(matches!(v, Error::Validation { .. }));
    let s: &dyn std::error::Error = &v;
    assert_eq!(
        s.to_string(),
        "Validation error: ErrCache: 占位类型无构造面"
    );
    // String 定形：source 字段即短码文本直读；std source() 链恒 None（无下层装载对象=诚实形）。
    let Error::Validation { source, .. } = &v else { unreachable!() };
    assert_eq!(source, "ErrCache: 占位类型无构造面");
    assert!(std::error::Error::source(&v).is_none(), "链面=None 定形");
    let i: Error = internal("ErrGlEnv: 多采样读拒绝");
    assert!(matches!(i, Error::Internal { .. }));
    assert!(std::error::Error::source(&i).is_none(), "Internal 链面=None");
}

#[test]
fn filter_routing_and_captures() {
    assert!(crate::error_inner::filter_matches(
        ErrorFilter::Validation,
        &validation("x")
    ));
    assert!(!crate::error_inner::filter_matches(
        ErrorFilter::Validation,
        &internal("x")
    ));
    assert!(crate::error_inner::filter_matches(
        ErrorFilter::Internal,
        &internal("x")
    ));
    assert!(!crate::error_inner::filter_matches(
        ErrorFilter::OutOfMemory,
        &validation("x")
    ));
}

struct NoopWake;
impl std::task::Wake for NoopWake {
    fn wake(self: std::sync::Arc<Self>) {}
}

fn bare_device() -> crate::entry::Device {
    crate::entry::Device {
        _inner: pi_share::Share::new(crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        }),
    }
}

#[test]
fn scope_capture_nesting_and_filter_routing() {
    let dev = bare_device();
    let waker = std::task::Waker::from(std::sync::Arc::new(NoopWake));
    let mut cx = std::task::Context::from_waker(&waker);
    let mut f0 = std::pin::pin!(dev.pop_error_scope());
    match f0.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(x) => assert!(x.is_none(), "空栈 pop=None"),
        std::task::Poll::Pending => panic!("ready future 不应 Pending"),
    }
    dev.push_error_scope(ErrorFilter::Internal);
    dev.push_error_scope(ErrorFilter::Validation);
    let ok = crate::entry_inner::report_error(&dev._inner, internal("ErrGlEnv: x"));
    assert!(ok, "外层 Internal 应接住内层不匹配错误");
    let ok2 = crate::entry_inner::report_error(&dev._inner, validation("ErrCache: y"));
    assert!(ok2);
    let popped = dev._inner.with_error(|st| st.pop());
    match popped {
        Some(Error::Validation { description, .. }) => {
            assert_eq!(description, "ErrCache: y");
        }
        other => panic!("内层首条应为 Validation，实={other:?}"),
    }
    let popped2 = dev._inner.with_error(|st| st.pop());
    assert!(
        matches!(popped2, Some(Error::Internal { .. })),
        "外层收 Internal"
    );
    assert!(dev._inner.with_error(|st| st.pop()).is_none(), "栈空");
}

#[test]
fn uncaptured_then_log_degrade() {
    let dev = bare_device();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let seen2 = seen.clone();
    dev.on_uncaptured_error(Box::new(move |e: Error| {
        *seen2.lock().unwrap_or_else(|x| x.into_inner()) = e.to_string();
    }));
    let ok = crate::entry_inner::report_error(&dev._inner, validation("ErrNoLayout: z"));
    assert!(ok);
    assert_eq!(
        *seen.lock().unwrap_or_else(|x| x.into_inner()),
        "Validation error: ErrNoLayout: z"
    );
    dev._inner.with_error(|st| st.uncaptured = None);
    let ok2 = crate::entry_inner::report_error(&dev._inner, validation("again"));
    assert!(!ok2, "无通道无回调=false 降级留痕不 panic");
}
