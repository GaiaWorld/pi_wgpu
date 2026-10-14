# pi-wgpu-rebuild · 接口面（L7）

<!-- 双门禁通过（2026-10-13）。契约住壳：以下各行「契约」列=壳注释的原文计划（落壳批按此写入壳注释）；语义正文勿在此复述。
返工落壳补注（2026-10-13）：契约注释已全量补落 26 壳芯文件（对象表示/纯查询不失败/参数级约束/实现预期伪码），本册列=历史规划稿，对拍以壳为准；壳内 design/ 路径指针已清零。 -->

## 公开操作（entry 伞面 + object 方法族；同名对齐 wgpu 25.x = 兼容子集需求）

| 操作 | 壳 | 芯 | 契约→已落壳（下表列=规划期历史原稿；唯一居所=壳注释，对账以壳为准） | 理由/被否 |
|---|---|---|---|---|
| Instance::new | src/entry/mod.rs | src/entry/entry_inner.rs | 聚合入口取单一附着；失败后=无残留（创建期断言） | 对齐伞面 |
| Instance::create_surface | src/entry/mod.rs | src/entry/entry_inner.rs | 结果面①；错=无半附着可重试 | 结果面=轮2 拍板封闭三处 |
| Instance::request_adapter | src/entry/mod.rs | src/entry/entry_inner.rs | Future<Option<Adapter>> 同步包装；None=无可适配、无副作用 | 被否：多枚举（EGL 恒单卡） |
| Adapter::request_device | src/entry/mod.rs | src/entry/entry_inner.rs | 结果面②；错=设备未建、Adapter 仍可用；trace 参数忽略仅记录 | PiWgpuAdapter 双参统一（两 feature 同形） |
| Adapter::{features,limits,get_info,get_downlevel_capabilities,get_texture_format_features,is_surface_supported,unmake_current} | src/entry/mod.rs | src/entry/entry_inner.rs | 纯查询→不失败；unmake_current=私有差异（登迁移清单）；登记差异(B11)：is_surface_supported 按附着世代比对（非逐格式真判） | |
| Device 创建族 create_{buffer,texture,sampler,shader_module,bind_group_layout,bind_group,pipeline_layout,render_pipeline,command_encoder} | src/entry/mod.rs | src/entry/entry_inner.rs | 即时创建/命中复用；非法描述符=断言含 desc.label，无半建。登记差异(B11)：纹理存储=创建期按目标分配（2D·CUBE=tex_storage_2d / 3D·2D_ARRAY·CUBE_ARRAY=tex_storage_3d；绑目标未过镜像）；程序=新程序（refcount==1）内联编译+链接（复用命中不重链） | layout 必填 Some=参数级约束 |
| Texture::create_view | src/entry/mod.rs | src/entry/entry_inner.rs | 唯一 @public 视图入口：视图维度须与纹理相容（Cube/CubeArray 需 层数%6==0）；视图目标按维度覆盖（Cube→CUBE_MAP、CubeArray→CUBE_MAP_ARRAY）。违约=断言含 label | 纹理切片（B13 后） |
| Device::create_shader_module 产物 | src/shader/mod.rs | src/shader/shader_inner.rs | 转译产物=IR{hash,modules}；GLSL 出码延到管线创建（emit_stage 按 stage/entry_point）；解析/校验失败=错误码上递+log 诊断（错误规则）；登记(B11)：reflection_register 待接线（需 layout+可写模块，_refl 恒空，emit_stage 走 SlotTable） | wgpu 模块/入口点分离 |
| Device::{features,limits,reset_state} | src/entry/mod.rs | src/entry/entry_inner.rs | reset_state=重建镜像（外部 GL 干扰），失败后不存在；登记差异(B11)：features/limits 现探 caps（DeviceInner 未存创建时快照） | 私有差异登记 |
| Queue::submit / CommandEncoder::finish | src/entry/mod.rs | src/entry/entry_inner.rs | 空壳→无失败语义；契约：上层不得依赖提交生效点 | FR-13 |
| Queue::{write_buffer,write_texture} | src/upload/mod.rs | src/upload/upload_inner.rs | 即时下发；违约=log::error!(UploadErr)（debug 不下发/release 留痕+GL 兜底，不谎报，决定⑪） | 语义收窄进差异清单 |
| Queue::write_texture_jsbuffer [wasm] | src/upload/mod.rs | src/upload/upload_inner.rs | 私有扩展：JS 位图直传零字节拷贝；非位图/离线程=log::error!(UploadErr)（决定⑪） | 唯一公开分形态点 |
| Surface::configure | src/surface/mod.rs | src/surface/surface_inner.rs | 先全量校验（尺寸/组合）后建平台表面+make_current=不留半配置；违约=log::error!(SurfaceErr)（决定⑪）；登记差异：格式合法性未在查（Device 无 Caps） | |
| Surface::get_capabilities(&Adapter)->SurfaceCapabilities | src/surface/mod.rs | src/surface/surface_inner.rs | 纯查询→不失败；能力表来自 caps（README × 记录过时=documents 审读修正2） | 差异矩阵行 |
| Surface::get_current_texture | src/surface/mod.rs | src/surface/surface_inner.rs | 结果面③ SurfaceError{Timeout/Outdated/Lost/OutOfMemory}；未配置=Err(Outdated)+log 码；帧视图=默认帧缓冲标记；Lost=代号+1 整面失效 | |
| SurfaceTexture::present(self) | src/surface/mod.rs | src/surface/surface_inner.rs | 自消费=类型级禁重复呈现（无运行时态） | 替旧实现运行时 panic——更强且兼容 |
| CommandEncoder::begin_render_pass | src/draw/mod.rs | src/draw/draw_inner.rs | 颜色目标恰一 Some，其余断言；失败后通道未建 | |
| RenderPass::{set_pipeline,set_bind_group,set_vertex_buffer,set_index_buffer,set_viewport,set_scissor_rect,set_stencil_reference,set_blend_constant,draw,draw_indexed} | src/draw/mod.rs | src/draw/draw_inner.rs | 全经镜像差异下发；draw 参数级约束（draw 无 first_instance 形参；draw_indexed 的 base_vertex=0）=断言；Drop=通道结束（此后资源可销毁） | viewport wgpu 坐标内翻=汇合层 |
| util::DeviceExt::{create_buffer_init,create_texture_with_data} | src/utils/mod.rs | src/utils/utils_inner.rs | 组合调用（无新契约）；登记(B12)：create_texture_with_data 仅 mip0 单段直写，order 多段排布未实现 | |
| util::{backend_bits_from_env,power_preference_from_env,parse_backends_from_comma_list} | src/utils/mod.rs | src/utils/utils_inner.rs | 恒 GL 语义（登记差异：WGPU_BACKEND 不生效） | |
| trait PiWgpuAdapter | src/lib.rs | src/lib_inner.rs | 伞面统一双参 request_device（use_wgpu 下适配官方）。B13 落地=薄壳 facade：use_wgpu 下 `pub use wgpu::*` + 自有模块 cfg(not(use_wgpu)) 整体关闭 + 两条 cfg 版 trait（自有/官方各返回本侧类型）；lib_inner::uw_request_impl 承接官方单参→双参（trace 丢弃）；与 gl 互斥（compile_error） | 被否：两侧签名各异；被否 B 并存+适配层（wgpu::Device→entry::Device 无源） |

## 库内操作（@internal，芯互访面；契约同法住壳）
清单（L9 名实对账=壳实名）：objects::{ensure_reuse, release_reuse, invalidate_generations}、state::{flush_if_changed, drop_invalidate}（芯另暴露 set_*/bind_* 差异下发族：程序/VAO/目标/上传/UBO·纹理·采样器槽/开关函数/模板/视口——签名与契约见 state 芯，消费方=draw/binds）、binds::{alloc_slots, apply_layout, consume_dynamic_offsets}、shader::{compile_to_es300, reflection_register, emit_stage}、formats::{to_gl_params, mip_convert}、context::{attach, let_go, gen_bump}、context_inner::{with_gl, generation_now}（驱动调用闸/代号读，全芯共用）、context_inner::{surface_attach→Result<(),SurfaceOpErr>（建面+make_current；失败槽保持原状）、surface_swap→Result<(),SurfaceOpErr>（租约已消耗、不回传错误值、记 log）、web_bind_glow（Web GL 入口注入）}、caps::{probe, fill_features}、surface::{configure_impl, get_current_impl, present_impl, capabilities_impl, debug_impl}——方法级签名见对应壳文件。

<!-- 词汇/描述符 85 项全量对表=相-2 批；B14 落地见 07-design/types-map.md（类型粒度裁剪登记，字段级归挂接件）；相-1 已落壳仅链上 25 结构。 -->

## 落壳批注记（相-1 收口）

- 落壳清单：src/{entry,entry_inner,surface,surface_inner,context,context_inner,caps,caps_inner,objects,objects_inner,state,state_inner,binds,binds_inner,shader,shader_inner,formats,formats_inner,draw,draw_inner,upload,upload_inner,utils,utils_inner}.rs + src/types/mod.rs + src/lib.rs + src/lib_inner.rs（B13 挂接芯）+ Cargo.toml。
- use_wgpu：骨架批=lib compile_error 防呆桩（官方依赖挂接=相-2，非静默）；once_cell=1.21.3 钉死为 naga fork 传递依赖可用版（新解析缺 sync 模块）——登记依赖清单勘误。
- 壳体即空声明两处（finish/submit）=契约本身为空壳，非违规桩（决定②）。
- 词汇面=链上 25 结构落地，types 全量 85 项对表=相-2（本册顶注同源）。

- use_wgpu 侧定形【B13 已落】：`pub use wgpu::*` + impl PiWgpuAdapter for wgpu::Adapter 委托 lib_inner::uw_request_impl（官方单参→双参、trace 丢弃），官方面**无自有芯**；A/B 验收=`--features gl` / `--no-default-features --features use_wgpu` 对照编译（均已 Finished，见 call-chain B13 行）。
