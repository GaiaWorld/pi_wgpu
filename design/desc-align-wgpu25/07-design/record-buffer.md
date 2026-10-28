# 07-design · 追加：提交序命令缓冲（record 芯）

## 触发
产品 `instance_color_tilloff` 在多线程并行编码下出现画面闪烁。取证（`PI_GL_CHECK`）显示：
每帧两个默认帧通道（`C`=清屏+场景、`L:11`=覆盖层 load），103 帧中 2 帧顺序反成 `L C`；
且通道会与 `swap` 交错。根因=「录制即执行」：GL 调用在编码点立即发出，
并行编码器之间没有提交序，帧间还会互相覆写同一后缓冲。

## 决定
改为**录制→提交回放**：

1. `CommandEncoder` 持 `_commands: Vec<record::Command>`；`RenderPass` 各方法只入表。
2. `CommandBuffer` = 命令表；`Queue::submit` 按提交列表顺序逐缓冲、逐通道回放。
3. 回放夹具=`record::replay_commands`：造一个内部 `CommandEncoder`（共享 `DeviceInner`），
   `Begin` 调执行芯开通道，其余命令按录制序调 `exec_*`，`End` 由通道 Drop（`_replay=true` 分支）收尾。
4. 资源生命周期：
   - 附件/纹理视图：`TextureView`（含 `Texture` 共享引用）随命令存活；
   - 管线：`RetainedPipeline` 持「部件」（程序/VAO `Reusable` 计数各 +1，Drop 成对 -1；
     **不**克隆 `RenderPipeline` 本体，避免与其 `Drop` 双释放）；
   - 绑定组：`BoundEntry`（`DirectHandle`）克隆即共享计数；
   - 顶点/索引缓冲：`Buffer` 克隆。
5. 编码器内的即时命令（`copy_texture_to_buffer` / `copy_texture_to_texture`）先 `record::flush_encoder`
   再执行，保「先绘制后拷贝」的同编码器顺序（跨编码器顺序仍以 submit 序为准）。
6. 录制期保留描述符合同校验（`validate_begin`：查询位、附件数、深度族挂色槽、resolve 配对），
   拒止在 `begin_render_pass` 调用点发生，不留半通道。

## 验收
- 产品：约 6 秒 343 次默认帧通道 / 172 次换帧，帧内序列 `C L:11` **171/171 全对**（改前 2/103 反序）；
- `glcheck=0`、`glfbo=0`、`panic=0`，连续 5 轮稳定（swap 181–188/轮）；
- `cargo test --lib` 31 通过；`--ignored --test-threads=1` 真 GL 全链通过；examples 全编；use_wgpu 轨 lib check 通过。

## 登记差异
- 校验时机：本层部分参数级断言从「调用点」移到「提交回放」（结构性描述符校验仍在调用点）；
- `CommandEncoder::copy_texture_to_texture` 取 `&mut self`（上游为 `&self`）——为在拷贝前冲掉本编码器录制；
- 队列写（`write_buffer`/`write_texture`）仍即时，语义等价于「上一提交之后、下一提交之前」。