// 设计：pi-wgpu-rebuild
//! upload · 即时写路径：写入在调用内生效（录制即执行的直接推论——不存在提交批的等待点）。
//! 隐藏决定=写入生效时机（即时，无延迟队列）。契约权威文本住本面注释。
use crate::entry::{Buffer, Queue};
use crate::types::{BufferAddress, Extent3d, ImageCopyTexture, ImageDataLayout};

impl Queue {
    /// 写缓冲：offset+data 长度须落在缓冲范围内、offset 为 4 字节倍数。
    /// 违约两档：debug 构建=断言（报 offset/长度/边界三值）；release=记录留痕后仍下发，
    /// 由驱动自行拒绝并在错误队列留证——绝不谎报写入成功。违约条目整体不生效，缓冲保持原状。 @public
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) { crate::upload_inner::write_buffer_impl(self, buffer, offset, data) }
    /// 写纹理：偏移须 4 字节倍数、bytes_per_row 给定时须 256 倍数、写入矩形须落在目标 mip 层范围内。
    /// 违约处理两档同 write_buffer（debug 断言 / release 留痕不谎报）。 @public
    pub fn write_texture(&self, dst: &ImageCopyTexture<'_>, data: &[u8], layout: &ImageDataLayout, extent: Extent3d) { crate::upload_inner::write_texture_impl(self, dst, data, layout, extent) }
    /// 私有扩展（全库唯一公开分形态点，登迁移清单）：直接把浏览器位图句柄提交给上传通道（零字节路径，
    /// 不经 JS 侧解码缓冲）。合同：仅主线程（离线程=断言，与锁搬运同合同）；
    /// 传入非位图值/矩形越界=断言（报字段）。 @public
    #[cfg(target_arch = "wasm32")] pub fn write_texture_jsbuffer(&self, dst: &ImageCopyTexture<'_>, bitmap: wasm_bindgen::JsValue) { crate::upload_inner::write_js_impl(self, dst, bitmap) }
}
/// 256 字节行距语义→驱动 unpack 行距/偏移参数换算。单职责：纯换算、零校验零状态——
/// 合法性检查是写入口的责任，此处不重复设防。 @internal
pub(crate) fn convert_upload_layout(layout: &ImageDataLayout, extent: &Extent3d) -> ImageDataLayout { crate::upload_inner::convert_impl(layout, extent) }
