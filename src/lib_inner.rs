// 设计：pi-wgpu-rebuild
//! lib_inner · use_wgpu 薄壳挂接芯：官方单参 request_device → 本库双参伞面形状（trace_path 官方无对应物，丢弃）。
use std::path::Path;

pub(crate) fn uw_request_impl<'p>(
    adapter: &'p wgpu::Adapter,
    desc: &'p wgpu::DeviceDescriptor<'p>,
    _trace_path: Option<&'p Path>,
) -> impl std::future::Future<Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> + Send + 'p {
    adapter.request_device(desc)
}
