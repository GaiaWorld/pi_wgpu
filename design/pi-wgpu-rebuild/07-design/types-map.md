# types 词汇全量对表（B14 · 对 wgpu-rs 25.x 裁剪登记）

**载荷声明**：本册=api.md 顶注「词汇/描述符 85 项全量对表」的落地。口径=**类型粒度**（字段级差归挂接件）。凡收窄/变形/更名/自加/缺失均显式登记，**禁静默**。
**对照版本**：wgpu 25.0.2（wgpu-types 25.0.0），本地 path `E:\Rust\PI_PI\wgpu`（B13 同源）。

## 0. 计数对账

- 声明 85（L7 自记）=「兼容面词汇结构」口径；`src/types/mod.rs` 实测=结构 55 + 枚举 34 + 别名 5（合计 94 词汇结构）+ const 14 值 =108。
- 口径差（85↔94/108）=别名与 const 不计入「词汇结构」+口径演进；**以本册实测为准，无静默排除**。
- 链上 25 结构=设计链提及；其余为兼容面载体。

## 1. 同名同形（类型粒度直对，零收窄）

| 本库项 | wgpu 25.x 对应 | 备注 |
|---|---|---|
| Label / BufferAddress / DynamicOffset | 同 | 别名同形（DynamicOffset=u32） |
| Extent3d / Origin3d | 同 | |
| TextureDimension（D1/D2/D3）/ TextureViewDimension（D2/D2Array/Cube/CubeArray/D1/D3）/ TextureAspect | 同 | 纹理切片：两套维度全量创建/上传/视图已支持（已非「词汇保留」） |
| IndexFormat / VertexStepMode | 同 | |
| SamplerBindingType | 同 | |
| DeviceType | 同 | |
| FilterMode / SamplerBorderColor / CompareFunction | 同 | |
| FrontFace / PolygonMode / PrimitiveTopology | 同 | |
| CullMode | 无同名类型（wgpu: `PrimitiveState.cull_mode: Option<Face>`） | 本库 `type CullMode = Option<Face>`，**同形** |
| BlendOperation / BlendComponent / BlendState | 同 | |
| DepthBiasState / StencilOperation / StencilFaceState / StencilState | 同 | |
| MultisampleState / PrimitiveState | 同 | |
| TextureDataOrder | 同 | |
| RequestAdapterOptions / CommandEncoderDescriptor / InstanceDescriptor | 同（类型粒度；字段级裁剪归挂接件） | |
| ImageCopyTexture / ImageDataLayout | 同（**wgpu 已更名** TexelCopyTextureInfo / TexelCopyBufferLayout，旧名 deprecated 别名） | 本库沿用旧名（见 §5） |
| SurfaceCapabilities | 同 | **源=src/surface/mod.rs（非本册计量源）**；缺 usages 字段=挂接件 |

## 2. 收窄 / 变形 / 更名（显式登记）

| 本库项 | wgpu 25.x | 差异 | 处置 |
|---|---|---|---|
| Backends | 6 位（NOOP/VULKAN/GL/METAL/DX12/BROWSER_WEBGPU） | 仅 GL | 余=OUT；自加 EMPTY |
| Limits | 36 字段 | 4 | 挂接件补对表（TODO-DECL types:43） |
| TextureFormat | ~75 | 7 | 压缩族（BC/ETC2/ASTC）未落（TODO-DECL types:48） |
| VertexFormat | ~45 | 6 | 扩充归后续批 |
| BufferUsages | 12 位 | 4 | 无 STORAGE/MAP/COPY_SRC |
| TextureUsages | 6 位 | 3 | 无 STORAGE/COPY_SRC |
| ShaderStages | 7 位（NONE/VERTEX/FRAGMENT/COMPUTE/VERTEX_FRAGMENT/TASK/MESH） | 2（VERTEX/FRAGMENT） | COMPUTE/TASK/MESH=OUT |
| BufferBindingType | Uniform/Storage{read_only} | 仅 Uniform | storage buffer 无 |
| TextureSampleType | Float{filterable}/Sint/Uint/**Depth** | 3（缺 Depth） | 深度采样未收 |
| BlendFactor | 17 值（含 Src/Dst/OneMinus*/Constant/…） | 6（Zero/One/SrcAlpha/OneMinusSrcAlpha/DstAlpha/OneMinusDstAlpha） | 裁剪 |
| ColorWrites | 具名常量（RED/GREEN/BLUE/ALPHA/COLOR/ALL） | 裸 u8 位集（无具名常量） | 缺常量面 |
| AddressMode | ClampToEdge/Repeat/**MirrorRepeat** | ClampToEdge/Repeat/**MirroredRepeat** | **更名** |
| Face | {Front,Back} | {**Undef**,Front,Back} | **自加 Undef 哨兵** |
| PresentMode | 6（AutoVsync/AutoNoVsync/Fifo/FifoRelaxed/Immediate/Mailbox） | 3（Fifo/Mailbox/Immediate） | 余值缺；configure 合法面仅 Fifo |
| CompositeAlphaMode | 5（Auto/Opaque/PreMultiplied/PostMultiplied/Inherit） | 2（Opaque/PreMultiplied） | configure 合法面仅 Opaque |
| PowerPreference | None/LowPower/HighPerformance | Default/LowPower/HighPerformance | None→Default 更名；单卡不选路 |
| MemoryHints | {Performance(默认), MemoryUsage, Manual{..}} | {Performance, Memory, Auto} | MemoryUsage→Memory 更名；缺 Manual；默认 Auto≠Performance |
| Features | 命名位集 | u64 投影 | 收窄到能力探测集 |
| DeviceDescriptor | label/required_features/required_limits/memory_hints/trace | 4（无 trace） | trace 语义=仅记录 |
| Color | Color（f64 域） | f32 四通道 | 精度收窄 |
| BufferSize | BufferAddress(u64) | NonZeroU32 包装 | 兼容子集差异 |
| LoadOp / StoreOp | LoadOp<V>（泛型）/ StoreOp | LoadOp 非泛型（值内嵌 Color）/ StoreOp 同形 | 泛型收窄 |

## 3. 自加（wgpu 无，本库专有）

| 项 | 说明 |
|---|---|
| Backends::EMPTY | 不选后端哨兵 |
| Face::Undef | 朝向哨兵（§2 Face 行） |
| COPY_BYTES_PER_ROW_ALIGNMENT / COPY_BUFFER_ALIGNMENT / VERTEX_STRIDE_ALIGNMENT | 校验常量 |
| GlslVersion/GlslFragment/GlslVertex/GlslSource/WgslSource | 源包装（wgpu 用 naga 类型） |
| trait PiWgpuAdapter（lib.rs） | 伞面统一双参 request_device |

## 4. 缺失面（wgpu 25.x 有，本库 OUT）

- compute 管线/Storage 缓冲/Storage 纹理、多后端枚举、GPU 资源延迟回收、trace 落盘、Surface 多格式族、Mesh/Task 阶段、深层采样（Depth 采样类型）。
- 均可由 §2/§3 推得；进迁移清单（problem.md OUT）。

## 5. 显式断裂（进迁移清单，禁静默）

- 挂接件逐项补对表清单：① Limits 全字段；② TextureFormat 压缩族；③ VertexFormat 全量；④ Buffer/TextureUsages 全位；⑤ ShaderStages COMPUTE/TASK/MESH；⑥ BufferBindingType Storage；⑦ PresentMode 余值；⑧ CompositeAlphaMode 余值；⑨ MemoryHints Manual+默认口径；⑩ DeviceDescriptor.trace；⑪ TextureSampleType Depth；⑫ BlendFactor 余值；⑬ ColorWrites 具名常量。
- 更名待决：AddressMode.MirroredRepeat、PowerPreference.None→Default、MemoryHints.MemoryUsage→Memory、ImageDataLayout/ImageCopyTexture（wgpu→TexelCopy*）——挂接件决定是否随更（见 §1/§2）。
- request_device 的 trace_path=丢弃（lib_inner）；DeviceDescriptor.trace 无字段。