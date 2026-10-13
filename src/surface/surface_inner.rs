// 设计：pi-wgpu-rebuild
//! surface 芯：桩+设计预期编号伪码（示意·不可验证·非唯一实现）。
use crate::entry::Device;
use crate::surface::{Surface, SurfaceCapabilities, SurfaceTexture, SurfaceError};
use crate::types::*;
fn todo() -> ! { panic!("TODO-DECL") }

// 设计预期（伪码·示意·不可验证）：
// 1. 全量校验：宽度>0、高度>0、format 在能力集内、(present_mode, alpha_mode) 仅 (Fifo, Opaque)——任一违例先断言（报违例字段），配置不写。
// 2. 桌面/安卓：在既有附着上创建/重挂窗口表面，按配置设像素格式与缓冲属性。
// 3. Web：设画布尺寸，表面即画布本身（无 EGL 步）。
// 4. 驱动步骤全部成功后才写 _cfg=Some；中途报错=断言，_cfg 回到未配置（不留半配置）。
pub(crate) fn configure_impl(_s: &Surface<'_>, _d: &Device, _c: &SurfaceConfiguration) { todo() }
// 设计预期（伪码·示意·不可验证）：
// 1. _cfg=None 前置断言（未配置不取帧，报「须先 configure」）。
// 2. 取回缓冲生成帧视图 → SurfaceTexture{ _view, _suboptimal }；suboptimal 恒 true 沿用官方旧语义（差异，登迁移清单）。
// 3. 驱动报错映射封闭四态：超时=Timeout、表面过期=Outdated、上下文丢失=Lost（context::gen_bump 推进代号——旧对象随失配自然退出；本面不做失效动作，重建由上层 re-request_device）、显存不足=OutOfMemory；错误队列取空不留残余错误。
pub(crate) fn get_current_impl(_s: &Surface<'_>) -> std::result::Result<SurfaceTexture, SurfaceError> { todo() }
// 设计预期（伪码·示意·不可验证）：
// 1. 按附着体驱动能力位→formats 候选表×present/alpha 组合回报（纯查询，不建任何状态）。
pub(crate) fn capabilities_impl(_a: &crate::entry::Adapter) -> SurfaceCapabilities { todo() }
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 桌面/安卓：交换前后缓冲+栅栏节流（节奏控制细节由实现批按平台选法）；Web：空操作（合成由浏览器做）。
// 2. 换帧报错不回传错误值（租约已消耗）：留入错误队列，由下一次 get_current 以 Lost/Outdated 面暴露。
// 3. 按值自消费签名=「一帧一呈现」类型级成立，无运行期守卫成本。
pub(crate) fn present_impl(_t: SurfaceTexture) { todo() }
// 骨架期 Debug 委托桩：只打印标签级信息；实现批改字段级格式化。
pub(crate) fn debug_impl(_s: &Surface<'_>, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { panic!("TODO-DECL") }
