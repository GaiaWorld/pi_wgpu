- [pi\_wgpu](#pi_wgpu)
- [1. 设计](#1-设计)
- [2. 限制](#2-限制)
  - [2.0. 销毁资源](#20-销毁资源)
  - [2.1. **不** 支持](#21-不-支持)
  - [2.3. wgpu::util](#23-wgpuutil)
  - [2.2. `Instance`](#22-instance)
  - [2.3. `Adapter`](#23-adapter)
  - [2.4. `Surface`](#24-surface)
  - [2.5. `SurfaceTexture`](#25-surfacetexture)
  - [2.6. `Device`](#26-device)
  - [2.7. `Queue`](#27-queue)
  - [2.8. `CommandEncoder`](#28-commandencoder)
  - [2.9. `CommandBuffer` 空内容，空实现](#29-commandbuffer-空内容空实现)
  - [2.10. `RenderPass`](#210-renderpass)
  - [2.11. `Sampler`](#211-sampler)
  - [2.12. `Texture`](#212-texture)
  - [2.13. `Buffer`](#213-buffer)
  - [2.14. `BufferSlice` 空实现](#214-bufferslice-空实现)
  - [2.15. `BindGroup`](#215-bindgroup)
  - [2.16. `ShaderModule`](#216-shadermodule)
- [3. 附录：（无计划）用 `WebGL` 模拟 `WebGPU` 的 要点](#3-附录无计划用-webgl-模拟-webgpu-的-要点)
  - [3.1. 和 `WebGL2` 相比，缺失 的 功能](#31-和-webgl2-相比缺失-的-功能)
  - [3.2. 微信小游戏: `iOS` 的 `WebGL`](#32-微信小游戏-ios-的-webgl)

# pi_wgpu

`WebGPU`接口子集的`GL`后端，基于`Rust`实现。

+ 接口: 基于 [wgpu-rs](https://github.com/gfx-rs/wgpu) `v0.16` 进行修改；
+ 功能: 仅仅对应[WebGL2](https://www.khronos.org/files/webgl20-reference-guide.pdf)的子集
    - 加`压缩纹理`扩展：`DDS` / `ASTC`
+ 平台:
    - `Windows`: `OpenGL 3.3`
    - `Android` / PC-Android 模拟器: `GLES 3.0`
    - 浏览器 / 微信小游戏: [WebGL2](https://www.khronos.org/files/webgl20-reference-guide.pdf)


| 序号 | 功能                     | 时间  | 说明                               |
| ---- | ------------------------ | ----- | ---------------------------------- |
| 01   | ~~BindGroup~~            | 1天   | **Done**                           |
| 02   | ~~egl：初始化~~          | 1.5天 | **Done**                           |
| 03   | ~~压缩纹理：DDS / ASTC~~ | 0.5天 | **Done**                           |
| 04   | 多线程                   | 1.5天 | **Doing**                          |
| 05   | Shader                   | 0.5天 |                                    |
| 06   | Surface                  | 1天   |                                    |
| 07   | `TODO` EXE 联调          | ?天   | ?                                  |
| 08   | `TODO` WebGL2 实现       | ?天   | ?                                  |
| 08   | `TODO` WebGL2 联调       | ?天   | ?                                  |
| 08   | `TODO` Android 实现      | ?天   | ?                                  |
| 09   | `TODO` Android 联调      | ?天   | ?                                  |
| 10   | `TODO` 多重采样          | ?天   | 待定                               |
| 11   | `TODO` 多目标 渲染       | ?天   | 待定: `drawBuffers` / `clearBufer` |
| 12   | `TODO` 各种 Copy         | ?天   | 待定: `blitFramebuffer`            |
| 13   | `TODO` `QuerySet`        | ?天   | 待定:                              |
| 14   | `TODO` `Fence`           | ?天   | 待定:                              |
| 15   | `TODO` 其他功能          | ?天   | 待定: 如 `framebufferTextureLayer` |
| 16   | `TODO` 其他 Feature      | ?天   | 待定: 如 `PushConst`               |

# 1. 设计

+ `性能优化`：设置状态机，做GL的全状态比较；所以GL指令数量会比[wgpu-rs](https://github.com/gfx-rs/wgpu)少；

**注1**：为什么不直接用 [wgpu-rs](https://github.com/gfx-rs/wgpu)

+ 实际测试，库 WebGL2 Backend性能不满足游戏需求；
+ `Vulkan` / `WebGPU` 因兼容性问题，近期内不能广泛使用于各个平台；

**注2**：根据微信小游戏的开发者文档，在 iPhone / iPad 上，`WebGL2` 至少要 `iOS`版本 >= 15.5 才能开启

**注3**：详细的 设计笔记，[见这里](documents/design.md)

# 2. 限制

+ 销毁资源：可以在录制指令时 创建 / 销毁 资源
+ 线程安全：
    - 录制指令 是 `单线程`，录制即为提交，所谓的队列提交是空实现；
    - **TODO**: 创建，销毁 资源，Exe / Apk `多线程`；
    - `注`：使用者自己确保，创建/释放 和 录制指令 的 线程安全性；
+ GLSL:
    - 功能: 仅支持 [gles-300 / std140-布局](https://www.khronos.org/files/webgl20-reference-guide.pdf)；
    - 语法: 仅支持 [GLSL 450 语法](https://www.khronos.org/files/webgl20-reference-guide.pdf)；

## 2.0. 销毁资源

+ 必须等该资源的物体`draw`之后，才能销毁；
+ `RenderPass` 绑定的 纹理 资源，必须等到 `RenderPass` Drop 方法调用后，才能销毁；
+ 原因：因为 不知道调用顺序，vb / ib / ubo 的调用都是先缓存，等到 draw() 内部 才 统一调用的 gl 函数；

例子：

```rust
pass.set_vertex_buffer(vb1)
// 不能 调用 vb1 的 drop
pass.set_bind_group(bg1)
// 不能 调用 bg2 的 drop
pass.set_render_pipeline(rp1)
// 不能 调用 rp1 的 drop
pass.draw()
// 这里之后，可以扔掉 vb1, bg1, rp1
pass.set_vertex_buffer(vb2)
```

## 2.1. **不** 支持

| 函数            | 支持 | 说明            |
| --------------- | ---- | --------------- |
| QuerySet        | ×    | WebGL2 本身支持 |
| hal::Fence      | ×    | WebGL2 本身支持 |
| RenderBundle    | ×    |                 |
| ComputePipeline | ×    |                 |
| ComputePass     | ×    |                 |
| hal::Barrier    | ×    |                 |

## 2.3. wgpu::util

| 函数                       | 支持 | 说明                        |
| -------------------------- | ---- | --------------------------- |
| `backend_bits_from_env`    | ✔    | 永远返回 Some(Backends::GL) |
| `create_texture_with_data` | ✔    |                             |
| `create_buffer_init`       | ✔    |                             |

## 2.2. `Instance`

| 函数               | 支持 | 说明 |
| ------------------ | ---- | ---- |
| `new`              | ✔    |      |
| `request_adapter`  | ✔    |      |
| `create_surface`   | ✔    |      |
| enumerate_adapters | ×    |      |

## 2.3. `Adapter`

| 函数                          | 支持 | 说明 |
| ----------------------------- | ---- | ---- |
| `request_device`              | ✔    |      |
| `features`                    | ✔    |      |
| `limits`                      | ✔    |      |
| `get_info`                    | ✔    |      |
| `get_downlevel_capabilities`  | ✔    |      |
| `is_surface_supported`        | ✔    |      |
| `get_texture_format_features` | ✔    |      |

## 2.4. `Surface`

| 函数                  | 支持 | 说明 |
| --------------------- | ---- | ---- |
| `get_capabilities`    | ✔    |      |
| `get_default_config`  | ✔    |      |
| `configure`           | ✔    |      |
| `get_current_texture` | ✔    |      |

## 2.5. `SurfaceTexture`

| 函数      | 支持 | 说明 |
| --------- | ---- | ---- |
| `present` | ✔    |      |

## 2.6. `Device`

| 函数                           | 支持 | 说明                                                |
| ------------------------------ | ---- | --------------------------------------------------- |
| `features`                     | ✔    |                                                     |
| `limits`                       | ✔    |                                                     |
| `create_shader_module`         | ✔    |                                                     |
| `create_command_encoder`       | ✔    |                                                     |
| `create_bind_group_layout`     | ✔    |                                                     |
| `create_bind_group`            | ✔    |                                                     |
| `create_pipeline_layout`       | ✔    |                                                     |
| `create_render_pipeline`       | ✔    | 参数 layout: Option<&'a PipelineLayout> 必须 有值！ |
| `create_buffer`                | ✔    |                                                     |
| `create_texture`               | ✔    |                                                     |
| `create_sampler`               | ✔    |                                                     |
| create_shader_module_unchecked | ×    |                                                     |
| create_shader_module_spirv     | ×    |                                                     |
| create_render_bundle_encoder   | ×    |                                                     |
| create_compute_pipeline        | ×    |                                                     |
| create_texture_from_hal        | ×    |                                                     |
| create_query_set               | ×    |                                                     |
| poll                           | ×    |                                                     |
| on_uncaptured_error            | ×    |                                                     |
| push_error_scope               | ×    |                                                     |
| pop_error_scope                | ×    |                                                     |
| start_capture                  | ×    |                                                     |
| stop_capture                   | ×    |                                                     |
| as_hal                         | ×    |                                                     |

## 2.7. `Queue`

| 函数                           | 支持 | 说明   |
| ------------------------------ | ---- | ------ |
| `write_buffer`                 | ✔    |        |
| `write_texture`                | ✔    |        |
| `submit`                       | ✔    | 空实现 |
| on_submitted_work_done         | ×    |        |
| write_buffer_with              | ×    |        |
| copy_external_image_to_texture | ×    |        |
| get_timestamp_period           | ×    |        |

## 2.8. `CommandEncoder`

| 函数                    | 支持 | 说明               |
| ----------------------- | ---- | ------------------ |
| `finish`                | ✔    | 空实现             |
| `begin_render_pass`     | ✔    | 只支持一个渲染目标 |
| clear_texture           | ×    |                    |
| clear_buffer            | ×    |                    |
| begin_compute_pass      | ×    |                    |
| copy_buffer_to_buffer   | ×    |                    |
| copy_buffer_to_texture  | ×    |                    |
| copy_texture_to_buffer  | ×    |                    |
| copy_texture_to_texture | ×    |                    |
| insert_debug_marker     | ×    |                    |
| push_debug_group        | ×    |                    |
| pop_debug_group         | ×    |                    |
| write_timestamp         | ×    |                    |
| resolve_query_set       | ×    |                    |

## 2.9. `CommandBuffer` 空内容，空实现

## 2.10. `RenderPass`

| 函数                              | 支持 | 说明                                                   |
| --------------------------------- | ---- | ------------------------------------------------------ |
| `set_bind_group`                  | ✔    |                                                        |
| `set_pipeline`                    | ✔    |                                                        |
| `set_blend_constant`              | ✔    |                                                        |
| `set_index_buffer`                | ✔    |                                                        |
| `set_vertex_buffer`               | ✔    |                                                        |
| `set_scissor_rect`                | ✔    |                                                        |
| `set_viewport`                    | ✔    |                                                        |
| `set_stencil_reference`           | ✔    |                                                        |
| `draw`                            | ✔    | 参数 `first_instance` 必须为 0; `base_vertex` 必须为 0 |
| `draw_indexed`                    | ✔    | 参数 `first_instance` 必须为 0; `base_vertex` 必须为 0 |
| set_push_constants                | ×    |                                                        |
| insert_debug_marker               | ×    |                                                        |
| push_debug_group                  | ×    |                                                        |
| pop_debug_group                   | ×    |                                                        |
| draw_indirect                     | ×    |                                                        |
| draw_indexed_indirect             | ×    |                                                        |
| execute_bundles                   | ×    |                                                        |
| multi_draw_indirect               | ×    |                                                        |
| multi_draw_indexed_indirect       | ×    |                                                        |
| multi_draw_indirect_count         | ×    |                                                        |
| multi_draw_indexed_indirect_count | ×    |                                                        |
| write_timestamp                   | ×    |                                                        |
| begin_pipeline_statistics_query   | ×    |                                                        |
| end_pipeline_statistics_query     | ×    |                                                        |

## 2.11. `Sampler`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

## 2.12. `Texture`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

| 函数                    | 支持 | 说明 |
| ----------------------- | ---- | ---- |
| `create_view`           | ✔    |      |
| `as_image_copy`         | ✔    |      |
| `size`                  | ✔    |      |
| `width`                 | ✔    |      |
| `height`                | ✔    |      |
| `depth_or_array_layers` | ✔    |      |
| `mip_level_count`       | ✔    |      |
| `sample_count`          | ✔    |      |
| `dimension`             | ✔    |      |
| `format`                | ✔    |      |
| `usage`                 | ✔    |      |
| as_hal                  | ×    |      |
| destroy                 | ×    |      |

## 2.13. `Buffer`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

| 函数                       | 支持 | 说明 |
| -------------------------- | ---- | ---- |
| `size`                     | ✔    |      |
| `usage`                    | ✔    |      |
| `as_entire_binding`        | ✔    |      |
| `as_entire_buffer_binding` | ✔    |      |
| `slice`                    | ✔    |      |
| unmap                      | ×    |      |
| destroy                    | ×    |      |

## 2.14. `BufferSlice` 空实现

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |      |
| get_mapped_range     | ×    |      |
| get_mapped_range_mut | ×    |      |

## 2.15. `BindGroup`

`BindGroup` 会握住它使用的`Buffer`, `Texture`, `Sampler` 对象，使其不会被销毁。

如果想要对应的资源被销毁，必须同时扔掉`BindGroup`和对应的资源。

## 2.16. `ShaderModule`

+ 仅支持 Naga 编译过后，版本为 glsl 3.0 的 无 define 宏 的 glsl
+ 扩展：带上一个字段，用于说明 这个shader原始的 set-binding 和 uniform 的 名字

# 3. 附录：（无计划）用 `WebGL` 模拟 `WebGPU` 的 要点

## 3.1. 和 `WebGL2` 相比，缺失 的 功能

+ 代码 模拟
    - `UBO`：用 uniform 模拟
    - `Sampler`：用 texture 函数 模拟
    - `VS` / `FS`: 不能指定 layout
        - **难点，需要 `可行性测试`**：要找一套 WebGL Shader 的 编译转换工具，`naga`未必支持；
            * 将 UBO 转成 Uniform；
            * 去掉 所有的 layout，同时导出成 json文件，供 rust 设置；
        - rust层，调用 WebGL 的 getXXXLocation 取 Uniform / Attribute 的 location，建立 layout 的 hash-map；
+ 看 **扩展** 有没有
    - 很多纹理格式: 比如 float 纹理，深度 纹理
    - `VAO`
    - 几何实例化
    - 多目标渲染 / drawBuffers
    - 多重采样
    - ClearBuffer
    - FeedBack

## 3.2. [微信小游戏: `iOS` 的 `WebGL`](https://developers.weixin.qq.com/minigame/dev/guide/performance/perf-high-performance.html#%E7%AE%80%E4%BB%8B)

+ `WebGL`: 如果发现 iOS 14 的 帧率提升不上去，试试 在 canvas.getContext 接口 将 antialias: true
+ `WebGL`: iOS 版本 14.0 ~ iOS 15.3, 多个 drawcall 使用不同偏移 来 共享 `VB`/`IB`，性能非常糟糕 ！
