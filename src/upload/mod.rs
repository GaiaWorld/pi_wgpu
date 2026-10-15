// 设计：pi-wgpu-rebuild
//! upload · 即时写路径：写入在调用内生效（录制即执行的直接推论——不存在提交批的等待点）。
//! 隐藏决定=写入生效时机（即时，无延迟队列）。契约权威文本住本面注释。
//! 错误处理（错误纪律）：公开非 Result 面 → 违约=log::error!(短码)，debug 档不下发、release 档留痕+下发（不谎报）。
use crate::entry::{Buffer, Queue};
use crate::types::{BufferAddress, Extent3d, TexelCopyBufferLayout, TexelCopyTextureInfo};

/// 上传错误码（上递/记录；诊断原文走 log::error!，短码，错误纪律）。 @internal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UploadErr {
    /// 目标资源句柄已死（None）。
    DeadHandle,
    /// 偏移未按 4 字节对齐。
    Misaligned,
    /// 行距未按 256 字节对齐。
    RowPitch,
    /// 写入范围越出资源/数据边界。
    OutOfRange,
    /// 传入值非位图（Web 专有面）。
    NotBitmap,
    /// 非主线程调用（Web 专有面）。
    OffMainThread,
}

impl Queue {
    /// 写缓冲：offset+data 长度须落在缓冲范围内、offset 为 4 字节倍数。
    /// 违约（错误纪律）：log::error!(UploadErr)；debug 档不下发，release 档留痕后仍下发——
    /// 由驱动自行拒绝并在错误队列留证，绝不谎报写入成功。 @public
    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        crate::upload_inner::write_buffer_impl(self, buffer, offset, data)
    }
    /// 写纹理（形照 25.0.2：目标按值 TexelCopyTextureInfo、布局按值 TexelCopyBufferLayout）：
    /// 偏移须 4 字节倍数、bytes_per_row 给定时须 256 倍数；Some(0)/缺省=行宽自动。
    /// 违约处理两档同 write_buffer（log::error!(UploadErr)；debug 不下发/release 留痕+下发）。
    /// 写入矩形超界=由驱动拒绝（release 档兜底；Texture 不持尺寸、本库不预校验——登记差异）。 @public
    pub fn write_texture(
        &self,
        texture: TexelCopyTextureInfo<'_>,
        data: &[u8],
        data_layout: TexelCopyBufferLayout,
        size: Extent3d,
    ) {
        crate::upload_inner::write_texture_impl(self, &texture, data, &data_layout, size)
    }
    /// 私有扩展（全库唯一公开分形态点，登迁移清单）：直接把浏览器位图句柄提交给上传通道（零字节路径，
    /// 不经 JS 侧解码缓冲）。合同：仅主线程（离线程=log::error!(OffMainThread)）；
    /// 传入非位图值=log::error!(NotBitmap)。 @public
    #[cfg(target_arch = "wasm32")]
    pub fn write_texture_jsbuffer(
        &self,
        texture: TexelCopyTextureInfo<'_>,
        bitmap: wasm_bindgen::JsValue,
    ) {
        crate::upload_inner::write_js_impl(self, &texture, bitmap)
    }
}
/// 256 字节行距语义→驱动 unpack 行距/偏移参数换算。单职责：纯换算、零校验零状态——
/// 合法性检查是写入口的责任，此处不重复设防。 @internal
pub(crate) fn convert_upload_layout(
    layout: &TexelCopyBufferLayout,
    extent: &Extent3d,
) -> TexelCopyBufferLayout {
    crate::upload_inner::convert_impl(layout, extent)
}
