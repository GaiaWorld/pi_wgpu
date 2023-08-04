#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/gfx-rs/wgpu/master/logo.png")]
#![warn(missing_docs, unsafe_op_in_unsafe_fn)]

mod wgpu_core;
mod wgpu_hal;

pub use wgt::*;
pub use wgpu_core::*;