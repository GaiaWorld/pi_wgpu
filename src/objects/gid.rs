// 设计：pi-wgpu-rebuild
//! gid · 对象身份层：u64 归一化句柄 ⇄ 驱动面柄值的双向转换唯一闸口。
//! 桌面=驱动 uint 名加宽（构造经 Native* 具体体——关联型投影别名不可作 tuple ctor）；
//! Web=glow slot-key `as_ffi` 全 64 位（idx+version 无损，截半即混叠，故禁用 32 位投影）。
//! 重建形（from）合法 iff 值来自同域 id_of（from_ffi 合同）——伪造数字柄入驱动的通道在本层收口，
//! 全仓唯一允许直接构造 glow 对象柄值的位点。 @internal

macro_rules! def_gid {
    ($($idname:ident, $mkname:ident, $ty:ident, $naty:ident);* $(;)?) => {
        $(
            #[doc = concat!("取 ", stringify!($ty), " 柄值的归一化身份（桌面=驱动名；Web=slot-key ffi 64 位）。")]
            pub(crate) fn $idname(v: glow::$ty) -> u64 {
                #[cfg(not(target_arch = "wasm32"))]
                let out = v.0.get() as u64;
                #[cfg(target_arch = "wasm32")]
                let out = { use slotmap::Key as _; v.data().as_ffi() };
                out
            }
            #[doc = concat!("由归一化身份重建 ", stringify!($ty), " 柄值（仅接同域 id_of 产出值）。")]
            pub(crate) fn $mkname(n: u64) -> glow::$ty {
                #[cfg(not(target_arch = "wasm32"))]
                let out = glow::$naty(std::num::NonZeroU32::new(n as u32)
                    .unwrap_or_else(|| panic!("gid 断言: 名域溢出 id={n}")));
                #[cfg(target_arch = "wasm32")]
                let out = glow::$ty::from(slotmap::KeyData::from_ffi(n));
                out
            }
        )*
    };
}

def_gid! {
    id_of_buffer, buffer_from, Buffer, NativeBuffer;
    id_of_texture, texture_from, Texture, NativeTexture;
    id_of_sampler, sampler_from, Sampler, NativeSampler;
    id_of_program, program_from, Program, NativeProgram;
    id_of_framebuffer, framebuffer_from, Framebuffer, NativeFramebuffer;
    id_of_renderbuffer, renderbuffer_from, Renderbuffer, NativeRenderbuffer;
    id_of_vao, vao_from, VertexArray, NativeVertexArray;
}
