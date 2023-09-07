use super::super::wgt::{self, Backends, PowerPreference};

/// Always returns WEBGPU on wasm over webgpu.
#[cfg(all(target_arch = "wasm32", not(feature = "wgc")))]
pub fn parse_backends_from_comma_list(_string: &str) -> Backends {
    Backends::GL
}

/// Get a set of backend bits from the environment variable WGPU_BACKEND.
pub fn backend_bits_from_env() -> Option<Backends> {
    Some(Backends::GL)
}

/// Get a power preference from the environment variable WGPU_POWER_PREF
pub fn power_preference_from_env() -> Option<PowerPreference> {
    Some(
        match std::env::var("WGPU_POWER_PREF")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("low") => PowerPreference::LowPower,
            Ok("high") => PowerPreference::HighPerformance,
            _ => return None,
        },
    )
}

/// Choose which DX12 shader compiler to use from the environment variable `WGPU_DX12_COMPILER`.
///
/// Possible values are `dxc` and `fxc`. Case insensitive.
pub fn dx12_shader_compiler_from_env() -> Option<wgt::Dx12Compiler> {
    Some(
        match std::env::var("WGPU_DX12_COMPILER")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("dxc") => wgt::Dx12Compiler::Dxc {
                dxil_path: None,
                dxc_path: None,
            },
            Ok("fxc") => wgt::Dx12Compiler::Fxc,
            _ => return None,
        },
    )
}
