// 设计：pi-wgpu-rebuild
//! pi_wgpu · WebGPU 兼容子集的 GL 直替层（根面）。本文件只做模块接线与 feature 门控。
//! 每个模块 = 公开壳（类型/函数声明 + 契约注释，契约的权威文本就住在壳注释里）
//! + 实现芯（同名 _inner 文件：桩 + 设计预期伪码，逻辑体由后续批次补全，现以 TODO-DECL 标记）。
//! 本批验收目标 = 引用网自洽（编译通过）；不含任何运行时行为。
#![allow(dead_code, unused_variables, unused_imports)]

// use_wgpu 定形（挂接批照此落地）：整面 re-export 官方 wgpu 类型 + 为 wgpu::Adapter 补一条
// PiWgpuAdapter 实现、把官方单参 request_device 委托成双参形状（trace_path 官方侧无对应物、丢弃即可）。
// 骨架期该 feature 依赖面未连，故意用下面的 compile_error 防呆锁住，勿提前打开。
#[cfg(feature = "use_wgpu")]
compile_error!("use_wgpu 官方挂接批未落地：re-export 面与 Adapter 委托实现缺失，此防呆桩为故意设计，请勿启用该 feature");

pub mod types;

pub mod entry;
#[path = "entry/entry_inner.rs"]
pub(crate) mod entry_inner;

pub mod surface;
#[path = "surface/surface_inner.rs"]
pub(crate) mod surface_inner;

pub(crate) mod context;
#[path = "context/context_inner.rs"]
pub(crate) mod context_inner;

pub(crate) mod caps;
#[path = "caps/caps_inner.rs"]
pub(crate) mod caps_inner;

pub(crate) mod objects;
#[path = "objects/objects_inner.rs"]
pub(crate) mod objects_inner;

pub(crate) mod state;
#[path = "state/state_inner.rs"]
pub(crate) mod state_inner;

pub(crate) mod binds;
#[path = "binds/binds_inner.rs"]
pub(crate) mod binds_inner;

pub(crate) mod shader;
#[path = "shader/shader_inner.rs"]
pub(crate) mod shader_inner;

pub(crate) mod formats;
#[path = "formats/formats_inner.rs"]
pub(crate) mod formats_inner;

pub mod draw;
#[path = "draw/draw_inner.rs"]
pub(crate) mod draw_inner;

pub mod upload;
#[path = "upload/upload_inner.rs"]
pub(crate) mod upload_inner;

pub mod utils;
#[path = "utils/utils_inner.rs"]
pub(crate) mod utils_inner;

pub use entry::*;
pub use surface::{Surface, SurfaceCapabilities, SurfaceError, SurfaceTarget, SurfaceTexture};
pub use entry::CreateSurfaceError;
pub use draw::RenderPass;
pub use utils::{BufferInitDescriptor, DeviceExt, backend_bits_from_env, parse_backends_from_comma_list, power_preference_from_env};

use std::path::Path;

/// 伞面 trait：把 request_device 统一成「描述符 + trace_path」双参形状，使上层代码在
/// 自产面（本库）与官方挂接面（use_wgpu）下书写完全一致。纯类型适配、无自身行为；
/// 官方面没有对应物，由本 trait 的第二个 impl 承担委托。 @public
pub trait PiWgpuAdapter {
    fn request_device<'p>(&'p self, desc: &'p types::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>> + Send + 'p>;
}

impl PiWgpuAdapter for entry::Adapter {
    fn request_device<'p>(&'p self, desc: &'p types::DeviceDescriptor<'p>, trace_path: Option<&'p Path>) -> Box<dyn std::future::Future<Output = std::result::Result<(entry::Device, entry::Queue), entry::RequestDeviceError>> + Send + 'p> {
        Box::new(entry::Adapter::request_device(self, desc, trace_path))
    }
}

pub use types::{Backends, BufferUsages, Extent3d, TextureFormat};
