// 设计：pi-wgpu-rebuild
//! lib_inner · use_wgpu 薄壳挂接芯：官方单参 request_device → 本库双参伞面形状
//! （trace 官方无对应物：Off 丢弃放行、Directory=与自产面同判断言拒止，双轨同形合同）。

pub(crate) fn uw_request_impl<'p>(
    adapter: &'p wgpu::Adapter,
    desc: &'p wgpu::DeviceDescriptor<'p>,
    _trace_path: Option<&'p std::path::Path>,
) -> impl std::future::Future<
    Output = std::result::Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>,
> + Send
       + 'p {
    // 历史兼容位仅记录不产出；官方描述符面无追踪字段（自产面对位 types::DeviceDescriptor.trace）。
    adapter.request_device(desc)
}
