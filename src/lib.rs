#![feature(hash_drain_filter)]

#[cfg(feature = "wgpu")]
pub use wgpu::*;

#[cfg(not(feature = "wgpu"))]
mod pi_wgpu;
#[cfg(not(feature = "wgpu"))]
pub use pi_wgpu::*;