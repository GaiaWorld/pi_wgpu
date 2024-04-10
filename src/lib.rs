// #![feature(hash_drain_filter)]
#![feature(hash_extract_if)]

#[cfg(feature = "use_wgpu")]
pub use wgpu::*;

#[cfg(not(feature = "use_wgpu"))]
mod pi_wgpu;
#[cfg(not(feature = "use_wgpu"))]
pub use pi_wgpu::*;

#[macro_use]
extern crate lazy_static;