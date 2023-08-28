- [pi\_wgpu](#pi_wgpu)
  - [1. Why](#1-why)
  - [2. 设计](#2-设计)
  - [3. 用 `WebGL` 模拟 `WebGPU` 的 要点](#3-用-webgl-模拟-webgpu-的-要点)
    - [3.1. 和 `WebGL2` 相比，缺失 的 功能](#31-和-webgl2-相比缺失-的-功能)
    - [3.2. 微信小游戏: `iOS` 的 `WebGL` / `WebGL2` 移植](#32-微信小游戏-ios-的-webgl--webgl2-移植)
  - [4. 限制](#4-限制)
    - [4.1. 不支持](#41-不支持)
    - [4.2. `Device`](#42-device)
    - [4.3. `Queue`](#43-queue)
    - [4.4. `CommandEncoder`](#44-commandencoder)
    - [4.5. `RenderPass`](#45-renderpass)
    - [4.6. `Texture`](#46-texture)
    - [4.7. `Buffer`](#47-buffer)
    - [4.8. `BufferSlice`](#48-bufferslice)

# pi_wgpu

WebGPU 的 Rust 实现，目前只实现 GL 后端；

接口 基于 [wgpu-rs](https://github.com/gfx-rs/wgpu) 的 0.15 版本 为基准 进行修改；

## 1. Why

为什么不用 [wgpu-rs](https://github.com/gfx-rs/wgpu)，因为 实际测试中，这个库目前 WebGL2 Backend 由于 为了实现通用的目的，而性能不满足本司游戏的性能的需求；

Vulkan / WebGPU 因为渲染驱动兼容性问题，近期内 不能广泛 在如下平台中广泛使用：

+ Android / PC的Android模拟器
+ 浏览器
+ 微信小游戏

所以 需要 简化版本的 GL 后端实现 过渡 一段时间。

## 2. 设计

+ 接口上 [wgpu-rs](https://github.com/gfx-rs/wgpu) 的 0.15 版本 兼容；
+ 实现后端 & 支持平台：
    - Windows: OpenGL 3.3
    - Android: GLES 3.0
    - 浏览器 / 微信小游戏: WebGL 2.0
+ 仅 实现 单线程 版本；不考虑多线程的安全性；
+ 接口上可以创建多个 CommandEncoder，但是实际上 都返回 同一个；
+ 录制 即是 提交，所谓的提交是空实现，为了以后的兼容性；
+ 性能优化：底层设置 状态机，做 OpenGL 的 全状态比较；所以 GL-Backend 的指令数量 预期 会比 [wgpu-rs](https://github.com/gfx-rs/wgpu) 少；

## 3. 用 `WebGL` 模拟 `WebGPU` 的 要点

### 3.1. 和 `WebGL2` 相比，缺失 的 功能

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

### 3.2. [微信小游戏: `iOS` 的 `WebGL` / `WebGL2` 移植](https://developers.weixin.qq.com/minigame/dev/guide/performance/perf-high-performance.html#%E7%AE%80%E4%BB%8B)

+ `WebGL`: 如果发现 iOS 14 的 帧率提升不上去，试试 在 canvas.getContext 接口 将 antialias: true
+ `WebGL`: iOS 版本 14.0 ~ iOS 15.3, 多个 drawcall 使用不同偏移 来 共享 `VB`/`IB`，性能非常糟糕 ！
+ `WebGL2`: 至少 iOS 版本 >= 15.5 才能开启 ！

注释：

+ **注1**: 怀疑 微信小游戏 高性能模式，就是将 js 和 渲染 在 Safari 坏境跑，但仅仅是怀疑，没有证据；
+ **注2**: 微信小游戏运行时有问题，不完全等于 `Safari` / `WkWebview` 有问题；
+ **注3**: 要稳妥的话，**建议** 请 负责iOS的同事 进行 `Safari` / `WkWebview` 测试；

## 4. 限制

**不支持** 意味着：调用相关函数时，运行时 会 panic

### 4.1. 不支持

+ 非 GLSL 格式的 Shader
+ `ComputePipeline`
+ `ComputePass`
+ `RenderBundle`
+ `QuerySet`
+ `hal::Fence`

### 4.2. `Device`

| 函数                           | 支持 | 说明 |
| ------------------------------ | ---- | ---- |
| `features`                     | ✔    |      |
| `limits`                       | ✔    |      |
| `create_shader_module`         | ✔    |      |
| `create_command_encoder`       | ✔    |      |
| `create_bind_group_layout`     | ✔    |      |
| `create_bind_group`            | ✔    |      |
| `create_pipeline_layout`       | ✔    |      |
| `create_render_pipeline`       | ✔    |      |
| `create_buffer`                | ✔    |      |
| `create_texture`               | ✔    |      |
| `create_sampler`               | ✔    |      |
| poll                           | ×    |      |
| create_shader_module_unchecked | ×    |      |
| create_shader_module_spirv     | ×    |      |
| create_render_bundle_encoder   | ×    |      |
| create_compute_pipeline        | ×    |      |
| create_texture_from_hal        | ×    |      |
| create_query_set               | ×    |      |
| on_uncaptured_error            | ×    |      |
| push_error_scope               | ×    |      |
| pop_error_scope                | ×    |      |
| start_capture                  | ×    |      |
| stop_capture                   | ×    |      |
| as_hal                         | ×    |      |

### 4.3. `Queue`

| 函数                           | 支持 | 说明 |
| ------------------------------ | ---- | ---- |
| `write_buffer`                 | ✔    |      |
| `write_texture`                | ✔    |      |
| `submit`                       | ✔    |      |
| on_submitted_work_done         | ×    |      |
| write_buffer_with              | ×    |      |
| copy_external_image_to_texture | ×    |      |
| get_timestamp_period           | ×    |      |

### 4.4. `CommandEncoder`

| 函数                    | 支持 | 说明 |
| ----------------------- | ---- | ---- |
| `finish`                | ✔    |      |
| `begin_render_pass`     | ✔    |      |
| `clear_texture`         | ✔    |      |
| `clear_buffer`          | ✔    |      |
| begin_compute_pass      | ×    |      |
| copy_buffer_to_buffer   | ×    |      |
| copy_buffer_to_texture  | ×    |      |
| copy_texture_to_buffer  | ×    |      |
| copy_texture_to_texture | ×    |      |
| insert_debug_marker     | ×    |      |
| push_debug_group        | ×    |      |
| pop_debug_group         | ×    |      |
| write_timestamp         | ×    |      |
| resolve_query_set       | ×    |      |

### 4.5. `RenderPass`

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

### 4.6. `Texture`

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

### 4.7. `Buffer`

| 函数                       | 支持 | 说明 |
| -------------------------- | ---- | ---- |
| `size`                     | ✔    |      |
| `usage`                    | ✔    |      |
| `as_entire_binding`        | ✔    |      |
| `as_entire_buffer_binding` | ✔    |      |
| slice                      | ×    |      |
| unmap                      | ×    |      |
| destroy                    | ×    |      |

### 4.8. `BufferSlice`

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |      |
| get_mapped_range     | ×    |      |
| get_mapped_range_mut | ×    |      |
