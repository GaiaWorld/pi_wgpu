// 设计：pi-wgpu-rebuild
//! pi_wgpu · WebGPU 兼容子集的 GL 直替层（根面）。本文件只做模块接线与 feature 门控。
//! 每个模块 = 公开壳（类型/函数声明 + 契约注释）+ 实现芯（同名 _inner：B0–B12 已补实现）。
//! use_wgpu（B13 薄壳 facade）：整面 re-export 官方 wgpu 类型 + PiWgpuAdapter for wgpu::Adapter；
//! 与 gl 互斥——自有模块在 use_wgpu 下整体关闭（届时 crate 即 wgpu 薄壳）。
#![allow(dead_code, unused_variables, unused_imports)]

#[cfg(all(feature = "use_wgpu", feature = "gl"))]
compile_error!("use_wgpu 与 gl 互斥：请用 --no-default-features --features use_wgpu");

// —— use_wgpu 薄壳 facade ——
#[cfg(feature = "use_wgpu")]
pub use wgpu::*;
#[cfg(feature = "use_wgpu")]
mod lib_inner;

// —— 自产面（use_wgpu 下整体关闭） ——
#[cfg(not(feature = "use_wgpu"))]
pub mod types;

#[cfg(not(feature = "use_wgpu"))]
pub mod entry;

#[cfg(not(feature = "use_wgpu"))]
#[path = "entry/entry_inner.rs"]
pub(crate) mod entry_inner;

#[cfg(not(feature = "use_wgpu"))]
pub mod surface;

#[cfg(not(feature = "use_wgpu"))]
#[path = "surface/surface_inner.rs"]
pub(crate) mod surface_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod context;

#[cfg(not(feature = "use_wgpu"))]
#[path = "context/context_inner.rs"]
pub(crate) mod context_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod caps;

#[cfg(not(feature = "use_wgpu"))]
#[path = "caps/caps_inner.rs"]
pub(crate) mod caps_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod objects;

#[cfg(not(feature = "use_wgpu"))]
#[path = "objects/objects_inner.rs"]
pub(crate) mod objects_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod state;

#[cfg(not(feature = "use_wgpu"))]
#[path = "state/state_inner.rs"]
pub(crate) mod state_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod binds;

#[cfg(not(feature = "use_wgpu"))]
#[path = "binds/binds_inner.rs"]
pub(crate) mod binds_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod shader;

#[cfg(not(feature = "use_wgpu"))]
#[path = "shader/shader_inner.rs"]
pub(crate) mod shader_inner;

#[cfg(not(feature = "use_wgpu"))]
pub(crate) mod formats;

#[cfg(not(feature = "use_wgpu"))]
#[path = "formats/formats_inner.rs"]
pub(crate) mod formats_inner;

#[cfg(not(feature = "use_wgpu"))]
pub mod draw;

#[cfg(not(feature = "use_wgpu"))]
#[path = "draw/draw_inner.rs"]
pub(crate) mod draw_inner;

#[cfg(not(feature = "use_wgpu"))]
pub mod upload;

#[cfg(not(feature = "use_wgpu"))]
#[path = "upload/upload_inner.rs"]
pub(crate) mod upload_inner;

#[cfg(not(feature = "use_wgpu"))]
pub mod utils;

#[cfg(not(feature = "use_wgpu"))]
#[path = "utils/utils_inner.rs"]
pub(crate) mod utils_inner;

#[cfg(not(feature = "use_wgpu"))]
pub use draw::RenderPass;
#[cfg(not(feature = "use_wgpu"))]
pub use entry::CreateSurfaceError;
#[cfg(not(feature = "use_wgpu"))]
pub use entry::*;
#[cfg(not(feature = "use_wgpu"))]
pub use surface::{Surface, SurfaceCapabilities, SurfaceError, SurfaceTarget, SurfaceTexture};
#[cfg(not(feature = "use_wgpu"))]
pub use types::{Backends, BufferUsages, Extent3d, TextureFormat};
#[cfg(not(feature = "use_wgpu"))]
pub use utils::{
    backend_bits_from_env, parse_backends_from_comma_list, power_preference_from_env,
    BufferInitDescriptor, DeviceExt,
};

#[cfg(not(feature = "use_wgpu"))]
pub mod util {
    pub use crate::types::TextureDataOrder;
    pub use crate::utils::*;
}

use std::path::Path;

/// 伞面 trait（自产面）：把 request_device 统一成「描述符（含 trace 词位）+ trace_path 历史兼容位」双参形状。
/// 返回面 = RPITIT（不透明 future + Send 契约）：零堆分配、静态分发；含 RPITIT 方法即非 dyn-safe（本 trait 无 dyn 用法）。
#[cfg(not(feature = "use_wgpu"))]
pub trait PiWgpuAdapter {
    /// 请求设备：future 借用收体与各参（'p 显式统一），Send 合同保持（自产 futures 已证）。
    /// desc.trace=词汇位（Directory=断言 ErrTrace）；trace_path 历史兼容位=仅记录不产出（None 常态）。
    fn request_device<'p>(
        &'p self,
        desc: &'p types::DeviceDescriptor<'p>,
        trace_path: Option<&'p Path>,
    ) -> impl std::future::Future<
        Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>,
    > + Send
           + 'p;
}
#[cfg(not(feature = "use_wgpu"))]
impl PiWgpuAdapter for entry::Adapter {
    fn request_device<'p>(
        &'p self,
        desc: &'p types::DeviceDescriptor<'p>,
        trace_path: Option<&'p Path>,
    ) -> impl std::future::Future<
        Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>,
    > + Send
           + 'p {
        let r = crate::entry_inner::request_device_impl(self, desc, trace_path);
        async move { r }
    }
}

/// 同形伞词汇（use_wgpu 专轨）：与自产面 types::Trace 同形（形照官方历史 Trace 枚举、non_exhaustive、
/// Directory 恒拒止）；两面互斥故各立一名，对调用方呈现同名同形词汇。
#[cfg(feature = "use_wgpu")]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub enum Trace {
    /// 追踪关闭（唯一合法形）。
    #[default]
    Off,
    /// 追踪目录（官方词位；本面无产出，给值=违约断言）。
    Directory(std::path::PathBuf),
}

/// 伞面 trait（use_wgpu 薄壳）：官方 wgpu::Adapter 的单参 request_device 适配为双参形状（与自产面同形 RPITIT）。
#[cfg(feature = "use_wgpu")]
pub trait PiWgpuAdapter {
    /// 请求设备：适配官方单参面；trace_path 历史兼容位官方无对应物=丢弃（双轨同形合同，
    /// 自产面的追踪词位在 types::DeviceDescriptor.trace）。 @public
    fn request_device<'p>(
        &'p self,
        desc: &'p wgpu::DeviceDescriptor<'p>,
        trace_path: Option<&'p Path>,
    ) -> impl std::future::Future<
        Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>,
    > + Send
           + 'p;
}
#[cfg(feature = "use_wgpu")]
impl PiWgpuAdapter for wgpu::Adapter {
    fn request_device<'p>(
        &'p self,
        desc: &'p wgpu::DeviceDescriptor<'p>,
        trace_path: Option<&'p Path>,
    ) -> impl std::future::Future<
        Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>,
    > + Send
           + 'p {
        crate::lib_inner::uw_request_impl(self, desc, trace_path)
    }
}
