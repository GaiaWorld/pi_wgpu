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
    /// 层/面号越界，或面族（2D/CUBE_MAP）一次写了多面。
    LayerRange,
    /// 传入值非位图（Web 专有面）。
    NotBitmap,
    /// 非主线程调用（Web 专有面）。
    OffMainThread,
    /// 压缩纹理写数组/3D 目标——WebGL2 compressedTexSubImage 系无多层形（平台限制登记）。
    CompressedArray,
    /// 外来图源上传目标非单层 2D（源形无数组/立方体路）。
    WebExternalTarget,
    /// 源区 origin/flip_y 的局部语义在 ES 源形不支持——记录后整帧下发（平台差异登记）。
    WebExternalPartial,
    /// OffscreenCanvas 在 ES texSubImage 源形无对应重载（需先转 bitmap，本层未代做）。
    WebOffscreenSource,
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
    /// 写入矩形/面号/层数按创建期快照预校验（越界=UploadErr，不下发）；矩形越界或面族多面一次写=拒。 @public
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
    /// 外部图像源→纹理指定 mip/层 拷贝（WebGL2 专面，零字节 GL 直灌）：img/canvas/bitmap/video
    /// 整帧缩放写；ImageData 走字节路（行紧排）。登记差异：源 origin/flip_y 与色彩空间/预乘注解
    /// 为记录面（非默认=log::error!(UploadErr) 仍整帧发）；非单层 2D 目标拒。主线程合同同
    /// write_texture_jsbuffer 形。 @public
    #[cfg(target_arch = "wasm32")]
    // 旧名形承 pi 系调用面（wgpu-types 25 弃用别名）——入口允许，迁册登记。
    #[allow(deprecated)]
    pub fn copy_external_image_to_texture(
        &self,
        source: &crate::types::ImageCopyExternalImage,
        destination: crate::types::CopyExternalImageDestInfo<&crate::entry::Texture>,
        copy_size: crate::types::Extent3d,
    ) {
        crate::upload_inner::copy_external_impl(self, source, &destination, &copy_size)
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
