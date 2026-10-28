//! 临时诊断：访问违规（0xc0000005）时打印异常地址、模块与调用栈，便于定位原生崩溃点。
//! 仅在 Windows 生效；由环境变量 PI_WGPU_CRASH_LOG=1 开启。诊断完可移除。
#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

/// 诊断开启标志（install 时置位；未开启时 op 记录为零成本判断）。
static ENABLED: AtomicBool = AtomicBool::new(false);
thread_local! { static LAST_OP: RefCell<String> = const { RefCell::new(String::new()) }; }

/// 记录当前正在执行的 GL 操作（含关键参数），崩溃时随异常一起打印。
pub(crate) fn op(label: &str) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    LAST_OP.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.push_str(label);
    });
}

#[repr(C)]
struct ExceptionRecord {
    code: u32,
    flags: u32,
    record: *mut ExceptionRecord,
    address: *mut c_void,
    number_parameters: u32,
    _pad: u32,
    information: [usize; 15],
}

#[repr(C)]
struct ExceptionPointers {
    exception_record: *mut ExceptionRecord,
    context_record: *mut c_void,
}

#[link(name = "kernel32")]
extern "system" {
    fn AddVectoredExceptionHandler(
        first: u32,
        handler: extern "system" fn(*mut ExceptionPointers) -> i32,
    ) -> *mut c_void;
    fn GetModuleHandleExW(flags: u32, name: *const u16, out: *mut *mut c_void) -> i32;
    fn GetModuleFileNameW(module: *mut c_void, out: *mut u16, size: u32) -> u32;
}

const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x0000_0004;
const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x0000_0002;

fn module_of(addr: usize) -> String {
    unsafe {
        let mut h: *mut c_void = std::ptr::null_mut();
        let ok = GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            addr as *const u16,
            &mut h,
        );
        if ok == 0 {
            return "<unknown>".to_string();
        }
        let mut buf = [0u16; 1024];
        let n = GetModuleFileNameW(h, buf.as_mut_ptr(), buf.len() as u32);
        String::from_utf16_lossy(&buf[..n as usize])
    }
}

/// 沿 RBP 帧链走栈（x64 惯例），打印返回地址与所属模块。
unsafe fn walk_stack() -> String {
    let mut out = String::new();
    let mut rbp: usize;
    std::arch::asm!("mov {}, rbp", out(reg) rbp, options(nomem, nostack, preserves_flags));
    for i in 0..48 {
        if rbp == 0 || rbp % 8 != 0 {
            break;
        }
        let next = *(rbp as *const usize);
        let ret = *((rbp + 8) as *const usize);
        if ret == 0 {
            break;
        }
        out.push_str(&format!("\n  #{i:02} {ret:#018x} {}", module_of(ret)));
        if next <= rbp {
            break;
        }
        rbp = next;
    }
    out
}

extern "system" fn handler(info: *mut ExceptionPointers) -> i32 {
    unsafe {
        let rec = (*info).exception_record;
        if !rec.is_null() && (*rec).code == 0xC000_0005 {
            let addr = (*rec).address as usize;
            let bt = walk_stack();
            let last = LAST_OP.try_with(|c| c.borrow().clone()).unwrap_or_default();
            let msg = format!(
                "[pi_wgpu crash] AV addr={addr:#018x} module={}\nlast_op={last}\nstack:{bt}\n",
                module_of(addr)
            );
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("pi_wgpu_crash.log")
                .ok();
            if let Some(f) = f.as_mut() {
                let _ = f.write_all(msg.as_bytes());
                let _ = f.flush();
            }
            eprintln!("{msg}");
        }
    }
    0 // EXCEPTION_CONTINUE_SEARCH
}

/// 安装诊断钩子（幂等；未设环境变量则什么都不做）。
pub(crate) fn install() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("PI_WGPU_CRASH_LOG").is_none() {
            return;
        }
        ENABLED.store(true, Ordering::Relaxed);
        unsafe {
            AddVectoredExceptionHandler(1, handler);
        }
    });
}
