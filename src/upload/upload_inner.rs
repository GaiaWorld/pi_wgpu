// 设计：pi-wgpu-rebuild
//! upload 芯 · 即时写路径：校验（错误码记录）→ 锁附着 → 驱动子写。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：违约记 log::error!(UploadErr)；debug 档不下发、release 档留痕+下发（不谎报）。
use std::num::NonZeroU32;

use glow::HasContext;

use crate::context_inner::with_gl;
use crate::entry::{Buffer, Queue};
use crate::types::{BufferAddress, Extent3d, ImageCopyTexture, ImageDataLayout};
use crate::upload::UploadErr;

/// GL_COPY_WRITE_BUFFER（GL3.1+/ES3.0/WebGL2；非镜像/VAO 管辖的通用子写目标）。
const GL_COPY_WRITE_BUFFER: u32 = 0x8F3F;
/// GL_TEXTURE_2D。
const GL_TEXTURE_2D: u32 = 0x0DE1;
/// GL_TEXTURE0 基址。
const GL_TEXTURE0: u32 = 0x84C0;
/// GL_UNPACK_ROW_LENGTH（以像素计）。
const GL_UNPACK_ROW_LENGTH: u32 = 0x0CF2;
/// GL_UNPACK_IMAGE_HEIGHT。
const GL_UNPACK_IMAGE_HEIGHT: u32 = 0x806E;

/// 上传专用纹理单元（在镜像 0..BIND_SLOTS 之外，避免扰动绘制态绑定镜像）。
const UPLOAD_TEX_UNIT: u32 = crate::state::BIND_SLOTS as u32;

// 设计预期（伪码·示意·非唯一实现）：
// 1. 范围与对齐校验：违约=log::error!(UploadErr)；debug 档不下发，release 档留痕后仍下发（驱动兜底，不谎报）。
// 2. 取重入锁+附着（single_thread feature 下锁步退化零开销）。
// 3. 目标缓冲绑 GL_COPY_WRITE_BUFFER 后调子数据写入——即时生效，无排队、无提交点。
// 4. 数据写入不是状态：不碰状态镜像（无需 flush）。
pub(crate) fn write_buffer_impl(_q: &Queue, b: &Buffer, o: BufferAddress, d: &[u8]) {
    let Some(name) = b._h.name() else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    let mut bad = false;
    if o % 4 != 0 {
        log::error!("upload {:?} off={o}", UploadErr::Misaligned);
        bad = true;
    }
    if o + d.len() as u64 > b.size {
        log::error!(
            "upload {:?} off={o} len={} size={}",
            UploadErr::OutOfRange,
            d.len(),
            b.size
        );
        bad = true;
    }
    if bad && cfg!(debug_assertions) {
        return;
    }
    let Some(nn) = NonZeroU32::new(name.0) else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    with_gl(|gl| unsafe {
        gl.bind_buffer(GL_COPY_WRITE_BUFFER, Some(glow::NativeBuffer(nn)));
        gl.buffer_sub_data_u8_slice(GL_COPY_WRITE_BUFFER, o as i32, d);
    });
}

// 设计预期（伪码·示意·不可验证）：
// 1. convert_upload_layout 把 256 字节行距语义换算为驱动 unpack 行距/偏移参数。
// 2. 目标矩形/偏移校验，两档处理同 write_buffer（错误码 log，不谎报）。
// 3. 锁附着，上传专用单元按目标（2D/3D/2D_ARRAY/CUBE 面/CUBE_ARRAY）绑定后写到 dst 指定 mip 层与起点。
// 4. 目标带 mip 链且为全量上传 → 触发整链重生成（formats::mip_convert 的成本合同）。
/// GL 纹理目标常量（与 formats/entry 同源规范值）。
const GL_TEXTURE_3D: u32 = 0x806F;
const GL_TEXTURE_2D_ARRAY: u32 = 0x8C1A;
const GL_TEXTURE_CUBE_MAP: u32 = 0x8513;
const GL_TEXTURE_CUBE_MAP_ARRAY: u32 = 0x9009;
const GL_TEXTURE_CUBE_MAP_POSITIVE_X: u32 = 0x8515;

pub(crate) fn write_texture_impl(
    q: &Queue,
    t: &ImageCopyTexture<'_>,
    d: &[u8],
    l: &ImageDataLayout,
    e: Extent3d,
) {
    let Some(name) = t.texture._h.name() else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    let p = &t.texture.params;
    let mut bad = false;
    if l.offset % 4 != 0 {
        log::error!("upload {:?} off={}", UploadErr::Misaligned, l.offset);
        bad = true;
    }
    if let Some(bpr) = l.bytes_per_row {
        if bpr.get() % 256 != 0 {
            log::error!("upload {:?} bpr={}", UploadErr::RowPitch, bpr.get());
            bad = true;
        }
    }
    let off = l.offset as usize;
    if off > d.len() {
        log::error!(
            "upload {:?} off={off} len={}",
            UploadErr::OutOfRange,
            d.len()
        );
        bad = true;
    }
    if bad && cfg!(debug_assertions) {
        return;
    }
    let src: &[u8] = if off <= d.len() { &d[off..] } else { &[] };
    let Some(nn) = NonZeroU32::new(name.0) else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    let lay = convert_impl(l, &e);
    let bpp = p.block_bytes.max(1) as u64;
    let target = t.texture._target;
    let restore_unit = upload_restore_unit(q);
    with_gl(|gl| unsafe {
        gl.active_texture(GL_TEXTURE0 + UPLOAD_TEX_UNIT);
        gl.bind_texture(target, Some(glow::NativeTexture(nn)));
        match lay.bytes_per_row {
            Some(bpr) => gl.pixel_store_i32(GL_UNPACK_ROW_LENGTH, (bpr.get() as u64 / bpp) as i32),
            None => gl.pixel_store_i32(GL_UNPACK_ROW_LENGTH, 0),
        }
        match lay.rows_per_image {
            Some(rpi) => gl.pixel_store_i32(GL_UNPACK_IMAGE_HEIGHT, rpi.get() as i32),
            None => gl.pixel_store_i32(GL_UNPACK_IMAGE_HEIGHT, 0),
        }
        match target {
            GL_TEXTURE_3D | GL_TEXTURE_2D_ARRAY | GL_TEXTURE_CUBE_MAP_ARRAY => {
                gl.tex_sub_image_3d(
                    target,
                    t.mip_level as i32,
                    t.origin.x as i32,
                    t.origin.y as i32,
                    t.origin.z as i32,
                    e.width as i32,
                    e.height as i32,
                    e.depth_or_array_layers as i32,
                    p.format,
                    p.pixel_type,
                    glow::PixelUnpackData::Slice(src),
                );
            }
            GL_TEXTURE_CUBE_MAP => {
                let face = GL_TEXTURE_CUBE_MAP_POSITIVE_X + t.origin.z;
                gl.tex_sub_image_2d(
                    face,
                    t.mip_level as i32,
                    t.origin.x as i32,
                    t.origin.y as i32,
                    e.width as i32,
                    e.height as i32,
                    p.format,
                    p.pixel_type,
                    glow::PixelUnpackData::Slice(src),
                );
            }
            _ => {
                gl.tex_sub_image_2d(
                    target,
                    t.mip_level as i32,
                    t.origin.x as i32,
                    t.origin.y as i32,
                    e.width as i32,
                    e.height as i32,
                    p.format,
                    p.pixel_type,
                    glow::PixelUnpackData::Slice(src),
                );
            }
        }
        gl.active_texture(GL_TEXTURE0 + restore_unit);
    });
}

/// 上传专用单元的「恢复单元」=镜像记录的当前 active 单元（上传只在专用单元操作，事后恢复到镜像期望）。\n/// 恢复镜像值而非盲还原 0，以维持「驱动 active == 镜像 active」不变量（帧门跳过回读时不至于错位）。
fn upload_restore_unit(q: &Queue) -> u32 {
    match q._dev.as_ref() {
        Some(d) => d
            ._inner
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active_texture_unit
            .unwrap_or(0),
        None => 0,
    }
}

// 设计预期（伪码·示意·不可验证）：
// 1. 主线程校验：离线程调用=log::error!(OffMainThread)（锁搬运合同）。
// 2. 入参类型校验：非位图值=log::error!(NotBitmap)。
// 3. 直接把位图句柄推进上传通道——零字节路径（唯一公开分形态点，桌面/安卓不存在此步）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn write_js_impl(q: &Queue, t: &ImageCopyTexture<'_>, bitmap: wasm_bindgen::JsValue) {
    use wasm_bindgen::JsCast;
    let Some(name) = t.texture._h.name() else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    let Some(bmp) = bitmap.dyn_ref::<web_sys::ImageBitmap>() else {
        log::error!("upload {:?}", UploadErr::NotBitmap);
        return;
    };
    let p = &t.texture.params;
    let bmp = bmp.clone();
    let Some(nn) = NonZeroU32::new(name.0) else {
        log::error!("upload {:?}", UploadErr::DeadHandle);
        return;
    };
    with_gl(|gl| unsafe {
        gl.active_texture(GL_TEXTURE0 + UPLOAD_TEX_UNIT);
        gl.bind_texture(GL_TEXTURE_2D, Some(glow::NativeTexture(nn)));
        gl.tex_sub_image_2d_with_image_bitmap(
            GL_TEXTURE_2D,
            t.mip_level as i32,
            t.origin.x as i32,
            t.origin.y as i32,
            p.format,
            p.pixel_type,
            &bmp,
        );
        gl.active_texture(GL_TEXTURE0 + upload_restore_unit(q));
    });
}

// 设计预期（伪码·示意·不可验证）：
// 1. 纯参数换算：256 语义行距→驱动粒度（含缺省行的推定），只算不查——不读驱动、不校验、无状态。
pub(crate) fn convert_impl(l: &ImageDataLayout, e: &Extent3d) -> ImageDataLayout {
    ImageDataLayout {
        offset: l.offset,
        bytes_per_row: l.bytes_per_row,
        rows_per_image: l.rows_per_image.or_else(|| NonZeroU32::new(e.height)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{DirectHandle, DirectKind};
    use crate::upload::convert_upload_layout;

    /// convert：缺省层行按 extent.height 推定；bytes_per_row 缺省保持 None。
    #[test]
    fn convert_fills_rows_default_test() {
        let l = ImageDataLayout {
            offset: 8,
            bytes_per_row: None,
            rows_per_image: None,
        };
        let e = Extent3d {
            width: 4,
            height: 8,
            depth_or_array_layers: 1,
        };
        let c = convert_upload_layout(&l, &e);
        assert_eq!(c.offset, 8);
        assert_eq!(c.rows_per_image.map(|x| x.get()), Some(8));
        assert!(c.bytes_per_row.is_none());
    }

    /// 死句柄：记错误码后早退，不 panic、不触 GL。
    #[test]
    fn write_buffer_dead_handle_no_panic_test() {
        let q = Queue { _dev: None };
        let b = Buffer {
            _h: DirectHandle::placeholder(DirectKind::Buffer, 1),
            size: 16,
        };
        write_buffer_impl(&q, &b, 0, &[0u8; 4]);
    }
}
