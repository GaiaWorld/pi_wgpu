// 设计：pi-wgpu-rebuild
//! upload 芯 · 即时写路径：校验（错误码记录）→ 锁附着 → 驱动子写。契约权威文本住壳面注释。
//! 错误处理（错误纪律）：违约记 log::error!(UploadErr)；debug 档不下发、release 档留痕+下发（不谎报）。
use std::num::NonZeroU32;

use glow::HasContext;

use crate::context_inner::with_gl;
use crate::entry::{Buffer, Queue};
use crate::types::{BufferAddress, Extent3d, TexelCopyBufferLayout, TexelCopyTextureInfo};
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
pub(crate) const UPLOAD_TEX_UNIT: u32 = crate::state::BIND_SLOTS as u32;

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
    t: &TexelCopyTextureInfo<'_>,
    d: &[u8],
    l: &TexelCopyBufferLayout,
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
        if bpr % 256 != 0 {
            log::error!("upload {:?} bpr={}", UploadErr::RowPitch, bpr);
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
    crate::crash_diag::op(&format!(
        "write_texture target={target:#x} mip={} origin=({},{},{}) extent={}x{}x{} bpr={:?} rpi={:?} src_len={} fmt={:?} type={:#x} tex_size={:?} layers={}",
        t.mip_level, t.origin.x, t.origin.y, t.origin.z,
        e.width, e.height, e.depth_or_array_layers,
        lay.bytes_per_row, lay.rows_per_image, src.len(), p.format, p.pixel_type,
        t.texture._size, t.texture._layers
    ));
    let restore_unit = upload_restore_unit(q);
    with_gl(|gl| unsafe {
        gl.active_texture(GL_TEXTURE0 + UPLOAD_TEX_UNIT);
        gl.bind_texture(target, Some(glow::NativeTexture(nn)));
        match lay.bytes_per_row {
            Some(bpr) => gl.pixel_store_i32(GL_UNPACK_ROW_LENGTH, (bpr as u64 / bpp) as i32),
            None => gl.pixel_store_i32(GL_UNPACK_ROW_LENGTH, 0),
        }
        match lay.rows_per_image {
            Some(rpi) => gl.pixel_store_i32(GL_UNPACK_IMAGE_HEIGHT, rpi as i32),
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
pub(crate) fn write_js_impl(
    q: &Queue,
    t: &TexelCopyTextureInfo<'_>,
    bitmap: wasm_bindgen::JsValue,
) {
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
pub(crate) fn convert_impl(l: &TexelCopyBufferLayout, e: &Extent3d) -> TexelCopyBufferLayout {
    TexelCopyBufferLayout {
        offset: l.offset,
        bytes_per_row: l.bytes_per_row,
        rows_per_image: l.rows_per_image.or(Some(e.height)),
    }
}

// 设计预期（伪码·示意·非唯一实现）：纹理→缓冲读回=同步即时（录制即执行的读侧投影）。
// 1. 门：宿主两方活名；tex COPY_SRC、buf COPY_DST；aspect=All；深度族/数组层/越界/行对齐=拒止断言（短码）。
// 2. 通路：色宿键 FBO 重挂 target mip→逐行 readPixels（行翻=纹理顶行 0 语义对齐上游；GL 原行 0=底）
//    组装块（bpr_eff stride，行隙零填）→交还 write_buffer 子写路。无 PBO/扩枚面（ES2 可达+同步保证）。
// 3. 缓冲内容即刻可读（get_sub_data 同调用点后有效）——无 pending 态。
pub(crate) fn copy_texture_to_buffer_impl(
    _e: &mut crate::entry::CommandEncoder,
    src: TexelCopyTextureInfo<'_>,
    dst: crate::types::TexelCopyBufferInfo<'_>,
    size: Extent3d,
) {
    let tex = src.texture;
    let buf = dst.buffer;
    let tn = tex._h.name();
    assert!(
        tn.is_some(),
        "upload 断言: 拷贝源纹理死句柄（label={:?}）",
        tex._h.generation()
    );
    let bn = buf._h.name();
    assert!(bn.is_some(), "upload 断言: 拷贝目标缓冲死句柄");
    assert!(
        tex._usage.contains(crate::types::TextureUsages::COPY_SRC),
        "upload 断言 ErrCopySrc: 源缺 COPY_SRC 位（usage={:?}）",
        tex._usage
    );
    assert!(
        buf._usage.contains(crate::types::BufferUsages::COPY_DST),
        "upload 断言 ErrCopyDst: 目标缺 COPY_DST 位（usage={:?}）",
        buf._usage
    );
    assert!(
        src.aspect == crate::types::TextureAspect::All,
        "upload 断言 ErrCopyAspect: 本批仅 aspect=All 读回（aspect={:?}）",
        src.aspect
    );
    assert!(
        !crate::draw_inner::is_depth_format(tex._format),
        "upload 断言 ErrCopyDepth: 深度族 readPixels=本驱动环境拒（0x502 实测），采样路归二阶（format={:?}）",
        tex._format
    );
    assert!(
        tex._layers == 1 && src.origin.z == 0 && size.depth_or_array_layers == 1,
        "upload 断言 ErrCopyArray: 数组/深向拷贝归二阶（layers={}, z={}, d={})",
        tex._layers,
        src.origin.z,
        size.depth_or_array_layers
    );
    assert!(
        src.mip_level < tex._mip,
        "upload 断言 ErrCopyMip: mip 越出源层数（mip={} count={})）",
        src.mip_level,
        tex._mip
    );
    assert!(
        src.mip_level == 0,
        "upload 断言 ErrCopyMipLevel: 非零 mip 拷贝归二阶（mip={})）",
        src.mip_level
    );
    let w: u64 = size.width as u64;
    let h: u64 = size.height as u64;
    assert!(w >= 1 && h >= 1, "upload 断言: 拷贝范围宽高须≥1");
    let ox: u64 = src.origin.x as u64;
    let oy: u64 = src.origin.y as u64;
    assert!(
        ox + w <= tex._size.width as u64 && oy + h <= tex._size.height as u64,
        "upload 断言 ErrCopyOver: 源矩形越出纹理尺寸（x={}+{}≤{} y={}+{}≤{}）",
        ox,
        w,
        tex._size.width,
        oy,
        h,
        tex._size.height
    );
    let bpp: u32 = tex.params.block_bytes;
    let row_bytes: u64 = w * bpp as u64;
    let bpr_raw: u64 = dst.layout.bytes_per_row.unwrap_or(0) as u64;
    let align: u64 = crate::types::COPY_BYTES_PER_ROW_ALIGNMENT as u64;
    let bpr_eff: u64 = if bpr_raw == 0 {
        (row_bytes + align - 1) / align * align
    } else {
        assert!(
            bpr_raw % align == 0,
            "upload 断言 ErrCopyAlign: bytes_per_row 须 4 倍数（={}）",
            bpr_raw
        );
        bpr_raw
    };
    let total: u64 = bpr_eff * h;
    assert!(
        dst.layout.offset + total <= buf.size,
        "upload 断言 ErrCopyOut: 写入块越出缓冲（off={} + {} > size={}）",
        dst.layout.offset,
        bpr_eff * h,
        buf.size
    );
    let mut block: Vec<u8> = vec![0u8; total as usize];
    let mip: i32 = src.mip_level as i32;
    let item = {
        let mut cache = _e._inner.cache.lock().unwrap_or_else(|x| x.into_inner());
        crate::objects::ensure_reuse(
            crate::objects::ReuseKey::Fbo(crate::draw_inner::rtt_fbo_key(Some(tex), None)),
            &mut cache,
        )
    };
    let fb = item.handle.expect("upload 断言: Fbo 复用行无名");
    let tname = tex._h.name().expect("前置活名");
    let bpr_usize: usize = bpr_eff as usize;
    let row_bytes_usize: usize = row_bytes as usize;
    let layers: u64 = tex._size.height as u64;
    with_gl(|gl| unsafe {
        gl.bind_framebuffer(
            GL_FRAMEBUFFER,
            Some(glow::NativeFramebuffer(
                std::num::NonZeroU32::new(fb.0).expect("非零 fb 名"),
            )),
        );
        gl.framebuffer_texture_2d(
            GL_FRAMEBUFFER,
            GL_COLOR_ATTACHMENT0,
            tex._target,
            Some(glow::NativeTexture(
                std::num::NonZeroU32::new(tname.0).expect("非零纹理名"),
            )),
            mip,
        );
        for r in 0..(h as usize) {
            // GL 原生行 0=底；wgpu 语义行 0=顶——读取逐行翻转对齐。
            let gl_y: i32 = layers as i32 - 1 - (oy as i32 + r as i32);
            let slice: &mut [u8] = &mut block[r * bpr_usize..r * bpr_usize + row_bytes_usize];
            gl.read_pixels(
                src.origin.x as i32,
                gl_y,
                w as i32,
                1,
                tex.params.format,
                tex.params.pixel_type,
                glow::PixelPackData::Slice(slice),
            );
        }
        gl.bind_framebuffer(GL_FRAMEBUFFER, None);
    });
    crate::objects::release_reuse(&item);
    // 写回走即时子写路（COPY_WRITE 目标；范围已在门内校验）——缓冲内容调用返回后即可 get_sub_data 读。
    let q: crate::entry::Queue = crate::entry::Queue { _dev: None };
    write_buffer_impl(&q, buf, dst.layout.offset, &block);
}

/// 缓冲同步读回（get_sub_data 消费芯）：活名+界内断言；COPY_READ 目标自绑自收（不在镜像管辖），读毕即返。
pub(crate) fn get_sub_data_impl(
    b: &crate::entry::Buffer,
    offset: BufferAddress,
    size: crate::types::BufferSize,
) -> Vec<u8> {
    let n = b._h.name();
    assert!(n.is_some(), "upload 断言: 读回命中死句柄缓冲");
    let len: u64 = size.get();
    assert!(
        offset + len <= b.size,
        "upload 断言 ErrSlice: 读回越界（off={} len={} size={}）",
        offset,
        len,
        b.size
    );
    let mut v: Vec<u8> = vec![0u8; len as usize];
    with_gl(|gl| unsafe {
        let nn = std::num::NonZeroU32::new(n.expect("前置活名").0).expect("非零缓冲名");
        gl.bind_buffer(GL_COPY_READ_BUFFER, Some(glow::NativeBuffer(nn)));
        gl.get_buffer_sub_data(GL_COPY_READ_BUFFER, offset as i32, &mut v);
        gl.bind_buffer(GL_COPY_READ_BUFFER, None);
    });
    v
}

// 设计预期（伪码·示意·非唯一实现）：缓冲→纹理=上传的布局化对偶（bpr 步幅+行翻）；同步即时。
pub(crate) fn copy_buffer_to_texture_impl(
    src: crate::types::TexelCopyBufferInfo<'_>,
    dst: TexelCopyTextureInfo<'_>,
    copy_size: Extent3d,
) {
    let buf = src.buffer;
    let tex = dst.texture;
    assert!(buf._h.name().is_some(), "upload 断言: 拷贝源缓冲死句柄");
    assert!(tex._h.name().is_some(), "upload 断言: 拷贝目标纹理死句柄");
    assert!(
        buf._usage.contains(crate::types::BufferUsages::COPY_SRC),
        "upload 断言 ErrCopySrc: 源缓冲缺 COPY_SRC 位（usage={:?}）",
        buf._usage
    );
    assert!(
        tex._usage.contains(crate::types::TextureUsages::COPY_DST),
        "upload 断言 ErrCopyDst: 目标纹理缺 COPY_DST 位（usage={:?}）",
        tex._usage
    );
    assert!(
        dst.aspect == crate::types::TextureAspect::All,
        "upload 断言 ErrCopyAspect: 本层仅 aspect=All（aspect={:?}）",
        dst.aspect
    );
    assert!(
        tex._layers == 1 && dst.origin.z == 0 && copy_size.depth_or_array_layers == 1,
        "upload 断言 ErrCopyArray: 数组层拷贝归二阶（layers={}, z={}, d={}）",
        tex._layers,
        dst.origin.z,
        copy_size.depth_or_array_layers
    );
    assert!(
        dst.mip_level < tex._mip,
        "upload 断言 ErrCopyMip: 目标 mip 越界（mip={} count={}）",
        dst.mip_level,
        tex._mip
    );
    let w: u64 = copy_size.width as u64;
    let h: u64 = copy_size.height as u64;
    assert!(w >= 1 && h >= 1, "upload 断言: 拷贝范围宽高须≥1");
    let ox: u64 = dst.origin.x as u64;
    let oy: u64 = dst.origin.y as u64;
    assert!(
        ox + w <= tex._size.width as u64 && oy + h <= tex._size.height as u64,
        "upload 断言 ErrCopyOver: 目标矩形越出纹理尺寸",
    );
    let texh: u64 = tex._size.height as u64;
    assert!(texh >= h + oy || texh <= h, "upload 断言: 内部尺寸一致性",);
    let bpp: u64 = tex.params.block_bytes as u64;
    let align: u64 = crate::types::COPY_BYTES_PER_ROW_ALIGNMENT as u64;
    let bpr_raw: u64 = src.layout.bytes_per_row.unwrap_or(0) as u64;
    let bpr_eff: u64 = if bpr_raw == 0 {
        (w * bpp + align - 1) / align * align
    } else {
        assert!(
            bpr_raw % align == 0,
            "upload 断言 ErrCopyAlign: bytes_per_row 须 256 倍数（={bpr_raw}）"
        );
        bpr_raw
    };
    let need: u64 = src.layout.offset + bpr_eff * h;
    assert!(
        need <= buf.size,
        "upload 断言 ErrCopyOut: 源读取块越出缓冲（off={} + {} > size={}）",
        src.layout.offset,
        bpr_eff * h,
        buf.size
    );
    let tname = tex._h.name().expect("前置活名");
    let fname = buf._h.name().expect("前置活名");
    let level: i32 = dst.mip_level as i32;
    let row_bytes: usize = (w * bpp) as usize;
    with_gl(|gl| unsafe {
        gl.bind_buffer(
            GL_COPY_READ_BUFFER,
            Some(glow::NativeBuffer(
                std::num::NonZeroU32::new(fname.0).expect("非零"),
            )),
        );
        gl.active_texture(UPLOAD_TEX_UNIT);
        gl.bind_texture(
            tex._target,
            Some(glow::NativeTexture(
                std::num::NonZeroU32::new(tname.0).expect("非零"),
            )),
        );
        for r in 0..(h as usize) {
            // 源块行 0=纹理顶行；GL 存储行 0=底——逐行翻转直写对应 y。
            let mut row: Vec<u8> = vec![0u8; row_bytes];
            gl.get_buffer_sub_data(
                GL_COPY_READ_BUFFER,
                (src.layout.offset + (h - 1 - r as u64) * bpr_eff) as i32,
                &mut row,
            );
            gl.tex_sub_image_2d(
                tex._target,
                level,
                dst.origin.x as i32,
                r as i32,
                w as i32,
                1,
                tex.params.format,
                tex.params.pixel_type,
                glow::PixelUnpackData::Slice(&row),
            );
        }
        gl.bind_buffer(GL_COPY_READ_BUFFER, None);
    });
}

// 设计预期（伪码·示意·非唯一实现）：纹→纹=CPU 中转两程（本机 blit 需 READ/DRAW 分绑目标枚举=ES 系
// 空转面，中转零扩展依赖=最稳；性能差异登记）。门=双方活性/位/aspect/单层/同像素宽/非深度。
pub(crate) fn copy_texture_to_texture_impl(
    e: &crate::entry::CommandEncoder,
    src: TexelCopyTextureInfo<'_>,
    dst: TexelCopyTextureInfo<'_>,
    copy_size: Extent3d,
) {
    let stx = src.texture;
    let dtx = dst.texture;
    assert!(stx._h.name().is_some(), "upload 断言: 源纹理死句柄");
    assert!(dtx._h.name().is_some(), "upload 断言: 目标纹理死句柄");
    assert!(
        stx._usage.contains(crate::types::TextureUsages::COPY_SRC),
        "upload 断言 ErrCopySrc: 源缺 COPY_SRC（usage={:?}）",
        stx._usage
    );
    assert!(
        dtx._usage.contains(crate::types::TextureUsages::COPY_DST),
        "upload 断言 ErrCopyDst: 目标缺 COPY_DST（usage={:?}）",
        dtx._usage
    );
    assert!(
        src.aspect == crate::types::TextureAspect::All
            && dst.aspect == crate::types::TextureAspect::All,
        "upload 断言 ErrCopyAspect: 本层仅 aspect=All"
    );
    assert!(
        !crate::draw_inner::is_depth_format(stx._format)
            && !crate::draw_inner::is_depth_format(dtx._format),
        "upload 断言 ErrCopyDepth: 深度族纹→纹归二阶"
    );
    assert!(
        stx._layers == 1
            && dtx._layers == 1
            && src.origin.z == 0
            && dst.origin.z == 0
            && copy_size.depth_or_array_layers == 1,
        "upload 断言 ErrCopyArray: 数组层纹→纹归二阶"
    );
    assert!(
        src.mip_level < stx._mip && dst.mip_level < dtx._mip,
        "upload 断言 ErrCopyMip: mip 越出层数域"
    );
    let sbpp: u64 = stx.params.block_bytes as u64;
    let dbpp: u64 = dtx.params.block_bytes as u64;
    let w: u64 = copy_size.width as u64;
    let h: u64 = copy_size.height as u64;
    assert!(w >= 1 && h >= 1, "upload 断言: 拷贝范围宽高须≥1");
    assert!(
        sbpp == dbpp,
        "upload 断言 ErrCopyFmt: 双方像素宽度须一致（src={} ≠ dst={}）",
        sbpp,
        dbpp
    );
    assert!(
        src.origin.x as u64 + w <= stx._size.width as u64
            && src.origin.y as u64 + h <= stx._size.height as u64,
        "upload 断言 ErrCopyOver: 源矩形越出源纹理"
    );
    assert!(
        dst.origin.x as u64 + w <= dtx._size.width as u64
            && dst.origin.y as u64 + h <= dtx._size.height as u64,
        "upload 断言 ErrCopyOver: 目标矩形越出目标纹理"
    );
    let item = {
        let mut cache = e._inner.cache.lock().unwrap_or_else(|x| x.into_inner());
        crate::objects::ensure_reuse(
            crate::objects::ReuseKey::Fbo(crate::draw_inner::rtt_fbo_key(Some(stx), None)),
            &mut cache,
        )
    };
    let fb = item.handle.expect("upload 断言: Fbo 复用行无名");
    let sname = stx._h.name().expect("前置活名");
    let dname = dtx._h.name().expect("前置活名");
    let row_bytes: usize = (w * sbpp) as usize;
    let block: Vec<u8> = vec![0u8; row_bytes * h as usize];
    let sh: u64 = stx._size.height as u64;
    let slvl: i32 = src.mip_level as i32;
    let dlvl: i32 = dst.mip_level as i32;
    let (sx, sy, dx, dy, wi, dtarget) = (
        src.origin.x as i32,
        src.origin.y as i32,
        dst.origin.x as i32,
        dst.origin.y as i32,
        w as i32,
        dtx._target,
    );
    let (sfmt, styp, ddst) = (
        stx.params.format,
        stx.params.pixel_type,
        (dname, row_bytes, h),
    );
    let mut blockw: Vec<u8> = block;
    with_gl(|gl| unsafe {
        gl.bind_framebuffer(
            GL_FRAMEBUFFER,
            Some(glow::NativeFramebuffer(
                std::num::NonZeroU32::new(fb.0).expect("非零"),
            )),
        );
        gl.framebuffer_texture_2d(
            GL_FRAMEBUFFER,
            GL_COLOR_ATTACHMENT0,
            stx._target,
            Some(glow::NativeTexture(
                std::num::NonZeroU32::new(sname.0).expect("非零"),
            )),
            slvl,
        );
        for r in 0..(h as usize) {
            let gl_y: i32 = sh as i32 - 1 - (sy + r as i32);
            let slice: &mut [u8] = &mut blockw[r * row_bytes..(r + 1) * row_bytes];
            gl.read_pixels(
                sx,
                gl_y,
                wi,
                1,
                sfmt,
                styp,
                glow::PixelPackData::Slice(slice),
            );
        }
        gl.bind_framebuffer(GL_FRAMEBUFFER, None);
        gl.active_texture(UPLOAD_TEX_UNIT);
        gl.bind_texture(
            dtarget,
            Some(glow::NativeTexture(
                std::num::NonZeroU32::new(ddst.0 .0).expect("非零"),
            )),
        );
        for r2 in 0..(h as usize) {
            let srcrow: &[u8] = &blockw[r2 * row_bytes..(r2 + 1) * row_bytes];
            let grow: usize = h as usize - 1 - r2;
            let payload: Vec<u8> = srcrow.to_vec();
            gl.tex_sub_image_2d(
                dtarget,
                dlvl,
                dx,
                grow as i32,
                wi,
                1,
                sfmt,
                styp,
                glow::PixelUnpackData::Slice(&payload),
            );
        }
    });
    crate::objects::release_reuse(&item);
}
/// GL_PIXEL_PACK 通路常量组。
/// GL_FRAMEBUFFER 通用目标。
const GL_FRAMEBUFFER: u32 = 0x8D40;
/// GL_COLOR_ATTACHMENT0。
const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
const GL_COPY_READ_BUFFER: u32 = 0x8F36;
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::objects::{DirectHandle, DirectKind};
    use crate::upload::convert_upload_layout;

    /// GPU→CPU 读回全链：离屏 clear→copy_texture_to_buffer→get_sub_data 比对；
    /// 行翻语义（纹理行 0=顶）+ bpr 补齐零填 + 边界/对齐负断言。真实 GL（attach→probe 同序）。
    pub(crate) fn gl_copy_check() {
        let _caps = crate::caps_inner::probe_impl();
        let di = crate::entry::DeviceInner {
            state: std::sync::Mutex::new(crate::state::StateMirror::default()),
            cache: std::sync::Mutex::new(crate::objects::HandleCache::default()),
            slots: std::sync::Mutex::new(crate::binds::SlotTable::default()),
            error_state: std::sync::Mutex::new(crate::error_inner::ErrorState::default()),
        };
        let dev = crate::entry::Device {
            _inner: pi_share::Share::new(di),
        };
        let mut enc = crate::entry::CommandEncoder {
            _opened_pass: false,
            _inner: dev._inner.clone(),
        };
        let tex = crate::entry_inner::create_texture_impl(
            &dev,
            &crate::types::TextureDescriptor {
                label: Some("cb"),
                size: crate::types::Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: crate::types::TextureDimension::D2,
                format: crate::types::TextureFormat::Rgba8Unorm,
                usage: crate::types::TextureUsages::RENDER_ATTACHMENT
                    | crate::types::TextureUsages::COPY_SRC,
                view_formats: &[],
            },
        );
        let buf = crate::entry_inner::create_buffer_impl(
            &dev,
            &crate::types::BufferDescriptor {
                label: Some("cb-dst"),
                size: 1024,
                usage: crate::types::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let view = tex.create_view(&crate::types::TextureViewDescriptor::default());
        let ca = crate::types::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: crate::types::Operations {
                load: crate::types::LoadOp::Clear(crate::types::Color {
                    r: 0.25,
                    g: 0.5,
                    b: 0.75,
                    a: 1.0,
                }),
                store: crate::types::StoreOp::Store,
            },
        };
        let desc = crate::types::RenderPassDescriptor {
            label: Some("cb-pass"),
            color_attachments: &[Some(ca)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        drop(enc.begin_render_pass(&desc));
        let src = crate::types::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: crate::types::Origin3d { x: 0, y: 0, z: 0 },
            aspect: crate::types::TextureAspect::All,
        };
        let dst = crate::types::TexelCopyBufferInfo {
            buffer: &buf,
            layout: crate::types::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
        };
        let ext = crate::types::Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        };
        enc.copy_texture_to_buffer(src, dst, ext);
        // bpr 缺省=上游 256 对齐行距补齐（4×RGBA8=16B 有效 + 240B 零填）×4 行=1024B。
        let got = buf.get_sub_data(0, crate::types::BufferSize::new(1024).expect("1024"));
        let mut rows: usize = 0;
        for row in got.chunks(256) {
            rows = rows + 1;
            let pxs: &[u8] = &row[..16];
            for chunk in pxs.chunks(4) {
                assert!(
                    (chunk[0] as i32 - 64).abs() <= 2
                        && (chunk[1] as i32 - 128).abs() <= 2
                        && (chunk[2] as i32 - 191).abs() <= 2
                        && chunk[3] >= 253,
                    "readback px mismatch: {:?}",
                    got
                );
            }
            assert!(
                row[16..].iter().all(|b| *b == 0),
                "行距补齐区应为零: {:?}",
                row
            );
        }
        assert_eq!(rows, 4, "四行读回");
        // 负断言：越出缓冲写入块=ErrCopyOut。
        let bad = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let dstbad = crate::types::TexelCopyBufferInfo {
                buffer: &buf,
                layout: crate::types::TexelCopyBufferLayout {
                    offset: 48,
                    bytes_per_row: None,
                    rows_per_image: None,
                },
            };
            enc.copy_texture_to_buffer(src, dstbad, ext);
        }));
        assert!(bad.is_err(), "越界写入块=ErrCopyOut 断言");
    }

    /// convert：缺省层行按 extent.height 推定；bytes_per_row 缺省保持 None。
    #[test]
    fn convert_fills_rows_default_test() {
        let l = TexelCopyBufferLayout {
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
        assert_eq!(c.rows_per_image, Some(8));
        assert!(c.bytes_per_row.is_none());
    }

    /// 死句柄：记错误码后早退，不 panic、不触 GL。
    #[test]
    fn write_buffer_dead_handle_no_panic_test() {
        let q = Queue { _dev: None };
        let b = Buffer {
            _h: DirectHandle::placeholder(DirectKind::Buffer, 1),
            size: 16,
            _usage: crate::types::BufferUsages::COPY_DST,
        };
        write_buffer_impl(&q, &b, 0, &[0u8; 4]);
    }
}
