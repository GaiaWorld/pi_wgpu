// 设计：pi-wgpu-rebuild
//! upload 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::entry::{Buffer, Queue};
use crate::types::{BufferAddress, Extent3d, ImageCopyTexture, ImageDataLayout};
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 范围与对齐校验：debug=断言（报 offset/长度/边界）；release=trace 留痕后继续发驱动调用（由 GL 拒绝，错误入队列——不谎报）。
// 2. 取重入锁+附着（single_thread feature 下锁步退化零开销）。
// 3. 选定目标缓冲即调子数据写入——即时生效，无排队、无提交点。
// 4. 数据写入不是状态：不碰状态镜像（无需 flush）。
pub(crate) fn write_buffer_impl(_q: &Queue, _b: &Buffer, _o: BufferAddress, _d: &[u8]) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. convert_upload_layout 把 256 字节行距语义换算为 unpack 行距/偏移参数。
// 2. 目标矩形/偏移校验，两档处理同 write_buffer（debug 断言 / release 留痕，不谎报）。
// 3. 锁附着，写到 dst 指定 mip 层与起点。
// 4. 目标带 mip 链且为全量上传 → 触发整链重生成（ formats::mip_convert 的成本合同）。
pub(crate) fn write_texture_impl(_q: &Queue, _t: &ImageCopyTexture<'_>, _d: &[u8], _l: &ImageDataLayout, _e: Extent3d) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 主线程校验：离线程调用=断言（锁搬运合同）。
// 2. 入参类型校验：非位图值=断言（报类型）。
// 3. 直接把位图句柄推进上传通道——零字节路径，不在 JS 侧展开像素缓冲（唯一公开分形态点，桌面/安卓不存在此步）。
#[cfg(target_arch = "wasm32")] pub(crate) fn write_js_impl(_q: &Queue, _t: &ImageCopyTexture<'_>, _b: wasm_bindgen::JsValue) { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 纯参数换算：256 语义行距→驱动粒度（含缺省行的推定），只算不查——不读驱动、不校验、无状态。
pub(crate) fn convert_impl(_l: &ImageDataLayout, _e: &Extent3d) -> ImageDataLayout { panic!("TODO-DECL") }
