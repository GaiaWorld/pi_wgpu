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
pub use entry::*;
#[cfg(not(feature = "use_wgpu"))]
pub use surface::{Surface, SurfaceCapabilities, SurfaceError, SurfaceTarget, SurfaceTexture};
#[cfg(not(feature = "use_wgpu"))]
pub use entry::CreateSurfaceError;
#[cfg(not(feature = "use_wgpu"))]
pub use draw::RenderPass;
#[cfg(not(feature = "use_wgpu"))]
pub use utils::{BufferInitDescriptor, DeviceExt, backend_bits_from_env, parse_backends_from_comma_list, power_preference_from_env};
#[cfg(not(feature = "use_wgpu"))]
pub use types::{Backends, BufferUsages, Extent3d, TextureFormat};

use std::path::Path;

/// 伞面 trait（自产面）：把 request_device 统一成「描述符 + trace_path」双参形状。 @public
#[cfg(not(feature = "use_wgpu"))]
pub trait PiWgpuAdapter {
    fn request_device<'p>(&'p self, desc: &'p types::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>> + Send + 'p>;
}
#[cfg(not(feature = "use_wgpu"))]
impl PiWgpuAdapter for entry::Adapter {
    fn request_device<'p>(&'p self, desc: &'p types::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>> + Send + 'p> {
        Box::new(entry::Adapter::request_device(self, desc, trace_path))
    }
}

/// 伞面 trait（use_wgpu 薄壳）：官方 wgpu::Adapter 的单参 request_device 适配为双参形状。 @public
#[cfg(feature = "use_wgpu")]
pub trait PiWgpuAdapter {
    fn request_device<'p>(&'p self, desc: &'p wgpu::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> + Send + 'p>;
}
#[cfg(feature = "use_wgpu")]
impl PiWgpuAdapter for wgpu::Adapter {
    fn request_device<'p>(&'p self, desc: &'p wgpu::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> + Send + 'p> {
        Box::new(crate::lib_inner::uw_request_impl(self, desc, trace_path))
    }
}
