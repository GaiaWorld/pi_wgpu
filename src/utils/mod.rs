// 设计：pi-wgpu-rebuild
//! utils · 便捷构造 + 环境读取助手（与官方 util 同名同形，便于上层平移）。
//! 本面全部是公开组合面，无自身新契约。
use crate::entry::{Buffer, Device, Queue, Texture};
use crate::types::{Backends, BufferUsages, Label, PowerPreference, TextureDataOrder, TextureDescriptor};

/// 缓冲一步建+写描述符：contents 在新建缓冲上单次写入。 @public
#[derive(Debug, Clone)] pub struct BufferInitDescriptor<'a> { pub label: Label<'a>, pub contents: &'a [u8], pub usage: BufferUsages }
/// 便捷组合 trait：只串联公开面（create+write），零新增契约——失败面=被组合双方的契约原样成立。 @public
pub trait DeviceExt {
    /// ＝ create_buffer + write_buffer 的单次组合；两环各自的断言/校验一处不少。委托实体见本 trait 于 Device 的 impl 头。 @public
    fn create_buffer_init(&self, desc: &BufferInitDescriptor<'_>) -> Buffer;
    /// ＝ create_texture + 按 order 排布语义的单次写入；契约同上（组合面）。委托实体见本 trait 于 Device 的 impl 头。 @public
    fn create_texture_with_data(&self, queue: &Queue, desc: &TextureDescriptor<'_>, order: TextureDataOrder, data: &[u8]) -> Texture;
}
impl DeviceExt for Device {
    fn create_buffer_init(&self, desc: &BufferInitDescriptor<'_>) -> Buffer { crate::utils_inner::buffer_init_impl(self, desc) }
    fn create_texture_with_data(&self, queue: &Queue, desc: &TextureDescriptor<'_>, order: TextureDataOrder, data: &[u8]) -> Texture { crate::utils_inner::tex_init_impl(self, queue, desc, order, data) }
}
/// 环境读面：本库恒 GL——返回值仅 GL 位或 None；官方 WGPU_BACKEND 变量在此不生效（登记为差异面）。 @public
pub fn backend_bits_from_env() -> Option<Backends> { crate::utils_inner::backend_bits_impl() }
/// 环境词→PowerPreference：只做词汇映射（取低档值），不改任何行为——单卡面无选路可改（差异登记公开面）。 @public
pub fn power_preference_from_env() -> PowerPreference { crate::utils_inner::power_env_impl() }
/// 逗号表解析：仅识别 GL 词元；其余词元静默忽略不报错（官方兼容形状）。 @public
pub fn parse_backends_from_comma_list(list: &str) -> Backends { crate::utils_inner::parse_impl(list) }
