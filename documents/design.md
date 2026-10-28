# pi_wgpu 关键机制与论证（v0.4.1）

> 本文记录「为什么这样实现」以及实机踩坑。接口见 api.md，模块职责见 module-design.md。
> 结论均来自 v4 分支源码与 design/ 三册；旧 v0.3.7 文档已删除。

## 1. 身份层：GlHandle(u64) 唯一闸口

- 对外/镜像/缓存一律用 GlHandle(u64)；进出 glow 必须过 objects/gid.rs。
- 桌面：驱动对象名是 u32，加宽零损。
- Web：glow 句柄是 slot-key（idx+version），必须取 ffi 全 64 位——截 32 位会把「删除后同 idx 新对象」混叠成别名。
- **名字不是存活证据**：GL 名可被驱动回收复用。删除路径把 (域, 名) 写进 state 死名表；flush 遇死名**丢弃期望**（置 None），而不是把水位置成死名——否则同名新对象会因水位相等永久悬空。

## 2. 三档状态镜像

| 档 | 组 | 机制 |
|---|---|---|
| 可回读 | ACTIVE_TEXTURE、DEPTH_TEST、DEPTH_FUNC、DEPTH_WRITEMASK、CULL_FACE(_MODE)、FRONT_FACE、SCISSOR_TEST/BOX、BLEND、STENCIL_TEST、VIEWPORT/DEPTH_RANGE | 每帧一次 glGet 比对，差异才重发 |
| 整发 | program、VAO、DRAW_FBO、PIXEL_UNPACK_BUFFER、UBO 槽、texture 槽、sampler 槽、blend、blend_constant、color_writes、polygon_mode、depth_bias、stencil 双面、alpha_to_coverage | 与 Water（上次实际下发）比对，差异才发 |
| 直发 | set_* helper | 下发后同点回写 water |

三条纪律：

1. 能力开关只能用 isEnabled 回读；getParameter + cap pname 在 WebGL 是 INVALID_ENUM。
2. 名称类只能比水位，不能查驱动（WebGL 对象绑定查询返回 JS 对象；UBO 绑定点无标量回读）。
3. 死名表非空 → 整发档强发一轮，成功后整面销记。

补充：

- **帧门**：可回读档每帧只回读一次（present 推进 FRAME）；帧内后续 draw 复用镜像态。理由：每 draw 7~11 次同步 glGet 是固定开销，而帧内状态只能由本库改（单上下文所有者）。
- **epoch**：任一线程/编码器完成一次下发即 STATE_EPOCH+1；本地 epoch_seen 不符 → water 整组作废。否则甲编码器的水位不能证明乙没改过驱动。
- **死名即删配套**：删对象时镜像不立即改写，而是下次 flush 依据死名表丢期望并强发；避免「删后立刻重建同名对象」与旧水位相等。

## 3. 录制 ≠ 执行：提交序回放（v4 核心变化）

早期实现是「录制即执行、submit 空壳」。v4 改为：

~~~text
begin_render_pass → 校验 + 记 Begin（附件/清除参数）
set_* / draw*     → 只入命令表（RetainedPipeline / RetainedBindGroup / Buffer clone）
Drop              → 记 End
finish            → CommandBuffer
Queue::submit     → 按提交顺序逐缓冲、逐通道 replay（此时才触 GL）
~~~

理由：

- 多线程各自编码后，提交顺序必须由 submit 列表决定（对齐上游语义），立即执行做不到。
- 命令表持资源强引用，允许上层在录制后立刻放手资源（上游合同）。
- 通道内仍按录制序执行；状态差异下发与水位逻辑不变（回放时走同一 exec_* 芯）。

边界：

- Queue::write_buffer / write_texture / copy_* **仍是即时**（调用点生效，不等 submit）。
- 跨通道的拷贝命令不能在通道内出现（record.rs 断言）。
- 测试钩子 flush_encoder 立即回放某编码器（不改变提交序合同）。

## 4. 每通道默认态

WebGPU 规定每个 pass 起始 viewport=(0,0,w,h)、depthRange=0..1、scissor=全附件且使能。GL 不随 FBO 切换复位，继承上通道的小视口会表现成「只渲染一角、其余清屏色」。

实现：begin 时若通道有宿主纹理，reset_pass_state(w,h) 先落镜像再下发；默认帧缓冲通道没有尺寸来源，**不复位**（已登记缺口，上层每通道显式 set_viewport 即可）。

## 5. 清除语义与守卫带

- 标准路径：LoadOp::Clear 清整张附件、不受裁剪影响。清前临时 disable(SCISSOR_TEST)、color_mask 全开、depth_mask 全开、桌面关 FRAMEBUFFER_SRGB；清后按镜像期望逐位归还。
- **CLEAR_SCISSORED（默认开，暂留实验口）**：清屏推迟到本通道首个 draw（或通道结束），按当时的裁剪框清子区域。由来：上层「每元素一 pass + Clear」时，全附件清除会让同目标内容互相覆盖 → 闪。产品实测按绘制框清后不闪。
  - 这是**非标准语义**，偏离 wgpu「Clear 与裁剪无关」；仅此口子使用，勿外推。
- 纯深度格式上给 stencil_ops：降级为忽略模板 + 每 label 一次 warn（不刷屏、不崩）。
- store=Discard：GL 单缓冲无语义，log 留痕后 noop。

## 6. RTT：形状键 + 行驻留 + 附件差分

- FBO 键 = 形状（色/深格式哈希 × samples × GL 目标），**宿主纹理名不入键**。逐帧重建同形状贴图时行稳定。
- 行驻留：FBO/VAO 计数归零不删，ensure 沿名复用；跨帧同形状=同对象同附着。
- ROW_ATT 维护「键 → 附件签名 (色名,target,深名,target,深槽)」；begin 时：
  - 签名不变 → 只 bind；
  - 变了 → 先裸槽（framebuffer_texture_2d None），再重挂，check_framebuffer_status 必须 COMPLETE。
- 纹理删除时 GL 会自动把附件摘下，故 delete_texture 同步 row_att_drop_tex。
- 选架：同签架优先 → 空架 → 备用架线性探测（8 路），读回路只认同签架。
- 交错占用：同一形状键被在世通道占用时，用混合键探测备用架，避免两个 pass 画进同一 FBO。

## 7. 组合 VAO：一次整备，draw 只绑不重下

- 键 = 布局哈希 × 按槽号升序的 (缓冲名, 偏移 + 实例 base×stride)。
- 首次整备 pointer/enable/divisor（整型格式走 vertex_attrib_pointer_i32）；此后同组合只 bindVertexArray。
- 索引绑定写进 VAO（ELEMENT_ARRAY_BUFFER 是 VAO 状态）；键里含索引缓冲名，变了才重绑。
- 容量 256；超限按 LRU 淘汰（目标 7/8 水位，本帧使用过的不淘汰）；草稿键（每布局一具）不计容量；淘汰时真删并整清组合表。
- 为什么有效：实例槽偏移逐批变化时，旧实现每 draw 重下十余条 vertexAttribPointer。

## 8. 绑定：三族槽位 + 管线自持

- set 0..3；每管线创建时 alloc_slots，把布局条目按域紧凑编号：
  - Uniform → ubo[i]；Sampler → smp[i]；Texture → tex[i]；i 即 GL 绑定点/单元号。
- 槽表值 = (set<<16 | binding) 打包键；**每管线自持一份**（出码与查槽同源）。设备级槽表只是「最近一次分配」查询面——若绘制期读它，后建管线会把编号覆盖掉 = 绑定错配。
- 组合采样器：naga ES300 把 sampler2D 合并成一个纹理全局；出码期从 IR 收集 alias，应用期采样器对象绑到配对纹理单元。
- 动态偏移：计数必须等于声明数；按声明序消耗；UBO 带偏移必须声明 size（bindBufferRange 需要合法 size）。
- 指位持久化：uniform1i / glUniformBlockBinding 是程序对象持久态，建管线时一次发；此后随 useProgram 携带，切换不重发（旧实现每切程序重发，产品 trace 双发根因）。

## 9. 着色器与坐标

- WGSL / GLSL 按阶段单源 → naga parse + validate → glsl-out（#version 300 es）。
- WriterFlags::ADJUST_COORDINATE_SPACE 与 wgpu GL 后端同款：顶点末尾 gl_Position.y 取反、z 从 [0,1] 映到 [-1,1]。
- 连带三处：
  1. 正面绕序映射反向（否则正面被当背面剔除）；
  2. 上屏 blit 用反向源矩形翻正；
  3. 拷贝路径不再做行翻。
- entry_point=None → 该 stage 唯一入口自动选；缺失/歧义 = Err(NoEntry)，管线创建期短码断言。
- 反射：BindingKind {UniformBuffer, Texture, Sampler} + (group,binding)；应用期按块索引/槽位寻址，不以名字符串查询。

## 10. 上传与读回

- 缓冲分配目标按 usage：INDEX → ELEMENT_ARRAY_BUFFER，否则 ARRAY_BUFFER。WebGL 合同：曾绑非 ELEMENT 目标的缓冲永不能再作索引绑定（ANGLE 实测）。
- write_buffer 同判据走 ELEMENT/ARRAY；Web 上 COPY 写会把缓冲锁成非索引用途，故不用 COPY_WRITE_BUFFER 做索引写。
- 纹理上传使用专用单元 UPLOAD_TEX_UNIT=24（业务槽 0..23 之外），避免扰动绘制绑定镜像。
- 行距：bytes_per_row 0=紧凑；压缩纹理按块行判据 ceil(w/block_w)*block_bytes 逐块行喂（WebGL2 忽略 UNPACK_ROW_LENGTH）。
- mipmap：全量上传后整链重生成；无「只刷一级」路径（成本合同）。
- 读回：copy_texture_to_buffer 内部 PBO + glFinish，结果进缓冲存储；get_sub_data 同步读。上游 map_async 族不移植（登记差异）。
- 纹→纹拷贝用 CPU 中转两程（不依赖 blit 扩展语义，性能换正确）。

## 11. 呈现：离屏屏幕纹理 + 翻转 blit

- configure 建「屏幕纹理」（尺寸/格式随配置），所有渲染落在它上面；帧视图是它的真实纹理视图。
- present：确保一个读 FBO 挂屏幕纹理 → blit_framebuffer(源矩形 y 反向) 到默认帧缓冲 → 桌面 swap_buffers（Web 空操作）→ bump_frame。
- 不直画默认帧缓冲：自 blit 重叠=未定义；glViewport 负高在 ES 非法。
- 重配置只更新配置并换屏幕纹理，不重建平台表面（Windows 禁止对同一 DC 重复 SetPixelFormat）。
- suboptimal 按真实语义报 false；旧「恒 true」会让上层每帧重新 configure → 屏幕纹理逐帧重建 → RTT 闪黑。

## 12. 线程与上下文

- 全库唯一 GL 入口 = context_inner::with_gl；一次调用 = claim_and_attach → f → release_one_layer。
- 同线程重入计层走快路；异线程等待上限 5s，超时 trace 持锁线程后断言。
- 搬运：make_current / unmake_current（桌面/安卓 EGL）；Web 无 EGL，GL 入口由首个画布面注入。
- Windows 事实约束：隐窗 DC 属于创建线程；该线程须存活到进程末，否则跨线程重绑被地基拒绝。
- single_thread feature：锁步与搬运整体退化为零开销。

## 13. 实机症状 → 根因 → 处理

| 症状 | 根因 | 处理 |
|---|---|---|
| 每帧 INVALID_ENUM、帧率跌到个位数 | 用 getParameter 查能力开关 | 改 isEnabled |
| Polygon mode other than FILL panic | polygon 常量错位（Fill→LINE） | 修正映射；Web 不下发 polygonMode |
| 清屏 INVALID_VALUE | scissor(0,0,i32::MAX,i32::MAX) Web f32 越界 | 删除该调用 |
| location is not from associated program | 程序未绑时 uniform1i | 指位随建管线（程序持久态）下发 |
| useProgram / bindTexture 每秒双发 | 直发路未回写水位 | 直发后同步 water |
| 桌面绑定永久悬空 | 死名水位别名 | 死名一律丢期望 |
| delete/createFramebuffer 洪水 | release 归零即物理删 | FBO/VAO 驻留（归零不删） |
| 每 draw 重下十余条 vertexAttribPointer | 实例槽偏移逐批变化 | 组合 VAO 一次整备 |
| 压缩纹理上传错位 | 行距按 width 估算（BC 误估 4 倍） | 块行判据 + 逐行喂 |
| 画面闪、元素互相覆盖 | 每元素一 pass + 每 pass Clear 全附件 | CLEAR_SCISSORED 延后按裁剪框清（非标准口） |
| 只渲染一角 | pass 继承上通道小视口 | 每 pass reset_pass_state |
| 上层每帧重配表面、纹理逐帧重建 | suboptimal 恒 true | 按真实语义报 false |
| 二维数组/立方体上传错位 | 面/层路由缺失 | formats::target_of + 视图目标覆盖 + 上传层路由 |
| 索引缓冲在 Web 上 set_index 恒拒 | 曾在 COPY_WRITE_BUFFER 分配 | 分配/写入均按 usage 走 ELEMENT |
| 创建纹理时扫 16 槽 bindTexture null | 在 active 业务单元直绑 + 整表刷洗 | 专用 UPLOAD_TEX_UNIT=24，只恢复 active 镜像 |

## 14. 诊断口

| 探针 | 含义 |
|---|---|
| frame churn configure/tex_new/tex_del | 逐帧资源重建来源 |
| rack 重挂 fk/old/new | FBO 附件签名变化原因（每帧一条） |
| 组合 VAO 表满 | 降级到草稿键（每帧一条） |
| probe(0/1/2)（test 构建） | useProgram 次数 / 整组重发 / 回读段执行 |
| CLEAR_SCISSORED | 编译期默认开；实验语义标记 |
| PI_GL_* / glcheck | 环境变量门控的错误探针（历史收口） |
