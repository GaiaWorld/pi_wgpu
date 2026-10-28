# 从 wgpu-rs 25 迁移到 pi_wgpu（v0.4.1）

> 本文只列「会改变行为/编译结果」的差异。类型词汇已对齐 wgpu-types 25（types 直接 pub use），旧 v0.3.7 的收窄/更名清单大多失效。

## 1. 词汇面

- types 模块：pub use wgpu_types::*；描述符加别名（BufferDescriptor<'a>、TextureDescriptor<'a>、SamplerDescriptor<'a>、DeviceDescriptor<'a>、TextureViewDescriptor<'a>、SurfaceConfiguration、TexelCopyTextureInfo<'a> 等）。
- 保留旧名 ImageCopyTexture（= TexelCopyTextureInfo 别名，deprecated 注记），便于 pi 系调用面平移。
- 自建类型（wgpu 无同名）：
  - BufferBinding / BindingResource / BindGroupEntry / BindGroupDescriptor
  - BindGroupLayoutDescriptor / PipelineLayoutDescriptor
  - RenderPassColorAttachment / RenderPassDepthStencilAttachment / RenderPassTimestampWrites / RenderPassDescriptor
  - VertexBufferLayout / VertexState / FragmentState / PipelineCompilationOptions / RenderPipelineDescriptor
  - ShaderSource（Wgsl / Glsl{shader, stage, defines}）
  - TexelCopyBufferInfo（把布局与 Buffer 引用合在一起）
  - CullMode = Option<Face>
- 域外变体：ShaderSource::SpirV / Naga 不存在（枚举 non_exhaustive，保留扩充位）。
- 整数宽度：Color 为 f32 四通道；BufferSize 为 NonZeroU32 包装（兼容子集）。

## 2. 行为差异（按影响排序）

### 2.1 录制与提交

| 项 | wgpu | pi_wgpu v4 |
|---|---|---|
| begin_render_pass / set_* / draw | 录制到命令缓冲 | 录制到内部命令表（不触 GL） |
| Queue::submit | 提交到 GPU 队列 | 按 submit 顺序回放命令表（触 GL） |
| Queue::write_buffer / write_texture | 进入队列，提交后生效 | **调用点即时生效** |
| copy_*（encoder 方法） | 提交后执行 | 录制，随 submit 回放（内部同步完成） |
| finish | 产出可提交命令 | 只交出命令表 |

迁移：不要依赖「write 在 submit 后才可见」；也不要假设 submit 是空操作。跨编码器顺序=submit 列表顺序。

### 2.2 描述符合法域

| 项 | wgpu | pi_wgpu |
|---|---|---|
| RenderPipelineDescriptor.layout | Option，None=auto layout | **必须 Some**；None=ErrNoLayout 断言 |
| compilation_options | 可给常量/覆盖 | 必须默认（constants 空且 zero_initialize=true） |
| multiview | Option | 必须 None（Some=ErrMultiview） |
| cache | Option | 必须 None（PipelineCache 无构造面） |
| push_constant_ranges | 支持 | 必须空；非空=ErrPushRange |
| set_push_constants | 支持 | 恒拒止（断言） |
| 双源混合因子（Src1 系） | 支持 | 创建期拒止（ErrBlendDual） |
| 顶点格式 | 45 值 | 核心共有子集；Float64 系 / Unorm8x4Bgra（桌面专有）拒止 |
| sample_count | 任意支持值 | 仅 1 或 4；MSAA 仅 D2 单层无 mip |

### 2.3 绑定与着色器

| 项 | wgpu | pi_wgpu |
|---|---|---|
| set 域 | 0..N | 仅 0..3；每管线最多 4 个 BGL，成员不重复 |
| 槽上限 | 由设备 Limits | UBO 24、纹理 16、采样器 16（槽表宽度 24）；越限断言 |
| storage buffer / storage texture | 支持 | 不支持（WebGL2 无；按 UBO/采样纹理建模） |
| 动态偏移 | 按 layout 声明 | 个数必须等于 has_dynamic_offset 条目数，按声明序；带偏移必须声明 size |
| 组合采样器 | 分离 texture/sampler | naga ES300 合并为 sampler2D；库按 IR 配对自动改道 |
| entry_point=None | 由实现决定 | 该 stage 唯一入口自动选；多入口=断言 |
| 着色器源 | WGSL/GLSL/SPIR-V/Naga | WGSL / GLSL 单源；SPIR-V/Naga 不存在 |
| 着色器报错 | 错误作用域/Validation | log 短码 + 空产物；管线创建期出码失败=断言 |

### 2.4 纹理与拷贝

| 项 | wgpu | pi_wgpu |
|---|---|---|
| 格式表 | ~75 | 22 个显式条目 + ASTC 参数族（见 §8），表外=断言 |
| mipmap | 可逐级写 | 全量上传后整链重生成；无「只刷一级」 |
| 纹理视图切片 | base_mip/base_array_layer 生效 | 当前取整张（切片暂不生效） |
| copy_texture_to_texture | GPU blit | CPU 中转两程（性能换正确；双方单层同宽、非深度族） |
| copy_texture_to_buffer | 队列拷贝 | PBO + glFinish 同步读；限制：aspect=All、非深度族/数组、mip 域内 |
| Buffer::map_async/get_mapped_range | 支持 | **不存在**；用 Buffer::get_sub_data(offset,size) 同步读 |
| Texture::destroy | 延迟 | 即刻删 + 死名登记 |
| Web 外部图源 | copy_external_image_to_texture | 同形；仅单层 2D、主线程；origin/flip_y 非默认=留痕后整帧发 |

### 2.5 表面与呈现

| 项 | wgpu | pi_wgpu |
|---|---|---|
| Surface::configure | 宽高/格式/呈现模式多选 | 宽高>0、usage 非空、present=Fifo、alpha ∈ {Auto, PreMultiplied, Opaque}；失败 log 且不写配置 |
| get_current_texture | 交换链帧 | 离屏屏幕纹理租约；suboptimal=false |
| 帧内容来源 | 交换链图像 | render 到屏幕纹理，present 翻转 blit 上屏 |
| present | 提交呈现 | blit + swap（桌面）/空 swap（Web）+ 推进帧号 |
| SurfaceTexture::view | 无此方法 | 有（颜色附件视图） |
| 未 present 即 Drop | 丢弃 | 丢弃（debug log） |
| get_capabilities | 支持 | 支持（桌面 4 格式，Web 2 格式） |
| OffscreenCanvas | 支持 | 构造可达；create_surface 暂拒 |

### 2.6 通道语义

| 项 | wgpu | pi_wgpu |
|---|---|---|
| 颜色附件数 | 多目标 | Some ≤1（0=纯深度通道） |
| timestamp_writes / occlusion_query_set | 支持 | Some 恒拒 |
| LoadOp::Clear | 清整张附件 | 默认 CLEAR_SCISSORED=true：推迟到首个 draw，按当时裁剪框清 |
| StoreOp::Discard | 丢弃 | noop（log 留痕） |
| 每 pass 默认 viewport/scissor | 全附件 | 有宿主纹理时复位；默认帧缓冲通道不复位 |
| MSAA resolve | 精确解析 | readPixels 滤波采样 → texSubImage 中转（uniform-clear 场景同色） |
| base_instance | 支持 | 实例区间起点必须 0 |
| draw_indexed base_vertex | 任意 | 必须 0 |

## 3. 资源生命周期（使用方合同）

1. 资源必须在该资源最后一次 draw 之后销毁；本库不延迟回收、不校验。
2. RenderPass 绑定的资源必须等通道结束（Drop）后销毁。
3. 被 BindGroup 持有的资源与 BindGroup 同弃（BindGroup 持共享计数，用户先 drop 不会删驱动对象；整组 drop 才末引用删）。
4. 违约后果=画面错误/未定义，库不兜底。

## 4. 线程

- 录制（编码器）单线程；创建/销毁/写入可跨线程（context 锁 + make/unmake current）。
- 提交（submit 回放）在其调用线程执行；状态 epoch 保证跨线程水位不误用。
- single_thread feature：锁与搬运全部退化，要求全程同线程（零开销）。
- Windows：首次建立附着的线程必须存活到进程末（隐窗 DC 生命周期）。

## 5. 能力与上限

- Limits：静态保底表（Limits::default），不逐项探测；request_device 按 36 字段对照，超出即 Err。
- Features：按探测投影，实际可能出现的位：DEPTH32FLOAT_STENCIL8、TEXTURE_COMPRESSION_BC、TEXTURE_COMPRESSION_ETC2、TEXTURE_COMPRESSION_ASTC、FLOAT32_FILTERABLE、TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES。
- 特判：Qualcomm/Adreno 清 BC/ASTC 能力位；iOS18/Safari 清 ASTC。请求被清掉的能力=request_device 拒绝。

## 6. 错误面

- 仅三处 Result：request_device / create_surface / get_current_texture。
- 其余创建违约=断言（短码 + label；断言前无驱动调用）。
- write_* 违约=log::error!(UploadErr)；debug 不下发，release 留痕后下发（不谎报）。
- on_uncaptured_error / push/pop_error_scope 已接；缺省未捕获=log 降级（上游 panic）。

## 7. 迁移清单

- [ ] 去掉 map_async/get_mapped_range，改 get_sub_data。
- [ ] 所有 create_render_pipeline 传 Some(layout)。
- [ ] 去掉 push constants / multiview / pipeline cache / 编译选项覆盖。
- [ ] 确认 set 只有 0..3，每管线 BGL ≤4 且不重复。
- [ ] 检查动态偏移：个数=声明数，带偏移的 UBO 必须给 size。
- [ ] 若使用多颜色目标/存储缓冲/间接绘制 → 改设计或留在 wgpu。
- [ ] 检查纹理格式在 §8 表内；mipmap 依赖整链重生成。
- [ ] Surface 配置改 Fifo + Auto/PreMultiplied/Opaque；每帧只 present 一次。
- [ ] 默认帧缓冲通道显式 set_viewport/set_scissor（不复位）。
- [ ] 若依赖「Clear 与裁剪无关」→ 注意 CLEAR_SCISSORED 默认开，必要时上层自行全屏清。
- [ ] 资源销毁遵守三条硬规则。
- [ ] Web：先 create_surface 再 request_adapter；外部图源仅主线程。

## 8. 格式表（当前实现）

显式条目 22 项：

| 族 | 条目 |
|---|---|
| 单/双通道 | R8Unorm、Rg8Unorm |
| 半浮点 | R16Float、Rg16Float、Rgba16Float |
| 浮点 | Rgba32Float |
| 8 位颜色 | Rgba8Unorm、Rgba8UnormSrgb、Bgra8Unorm、Bgra8UnormSrgb |
| 深度 | Depth16Unorm、Depth24Plus、Depth24PlusStencil8、Depth32Float、Depth32FloatStencil8 |
| BC/S3TC | Bc1RgbaUnorm、Bc2RgbaUnorm、Bc3RgbaUnorm |
| ETC2 | Etc2Rgb8Unorm、Etc2Rgb8UnormSrgb、Etc2Rgba8Unorm、Etc2Rgba8UnormSrgb |

ASTC 参数族（TextureFormat::Astc { block, channel }）：
- 块尺寸全 14 种：4x4、5x4、5x5、6x5、6x6、8x5、8x6、8x8、10x5、10x6、10x8、10x10、12x10、12x12；
- 通道仅 Unorm / UnormSrgb（Hdr 表外→断言）；
- 门控：caps 的 ASTC LDR 位缺失时拒止。

另：Bgra8UnormSrgb 在 GLES/WebGL2 形态拒止（无核心组合）；Depth24PlusStencil8 在 ES 形态拒止（无核心组合，用 Depth32FloatStencil8 替代）。
