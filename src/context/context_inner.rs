// 设计：pi-wgpu-rebuild
//! context 芯：桩+设计预期编号伪码（示意·不可验证）。
use crate::context::Attachment;
// 实现桩：逻辑体由后续批次按本伪码落地。
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. 已有活附着（本线程或按锁可重入）→ 直接返回现附着，不重复建上下文。
// 2. 桌面/安卓：取显示句柄→选 config→创建/绑定上下文→取 GL 入口表（地基调用面=egl 建上下文/make_current+glow 上下文对象，参数细节由实现批按地基 API 对应）。
// 3. Web（cfg 分支）：从画布取 WebGL2 上下文；无 EGL 步，环境分支只在此一处（不在上层模块再分叉）。
// 4. 首次取锁超时=他线程未让出→断言并 trace 持锁方；无半附着残留。single_thread feature=锁步与搬运步编译期退化零开销。
pub(crate) fn attach_impl() -> Option<Attachment> { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证）：
// 1. 解绑当前线程与上下文的绑定（make_current 传空绑定）。
// 2. Web 形态=空操作（上下文隐式绑于所属画布线程）。只动线程绑定，不销毁上下文本体。
pub(crate) fn let_go_impl() { panic!("TODO-DECL") }
// 设计预期（伪码·示意·不可验证·非唯一实现）：
// 1. generation+1 写回附着体。
// 2. 仅写代号，不动对象层账目——失效全靠消费端代号失配判定；重建路径=request_device 重入取新三件套。
// 3. 返回新代号；调用方为呈现错误面（Lost 分支）。
pub(crate) fn gen_bump_impl() -> u32 { panic!("TODO-DECL") }
