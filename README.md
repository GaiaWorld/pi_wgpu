- [pi\_wgpu](#pi_wgpu)
  - [1. Why](#1-why)
  - [2. 设计](#2-设计)
  - [3. 限制](#3-限制)
    - [3.1. **不** 支持](#31-不-支持)
    - [3.2. `Instance`](#32-instance)
    - [3.3. `Adapter`](#33-adapter)
    - [3.4. `Surface`](#34-surface)
    - [3.5. `SurfaceTexture`](#35-surfacetexture)
    - [3.6. `Device`](#36-device)
    - [3.7. `Queue`](#37-queue)
    - [3.8. `CommandEncoder`](#38-commandencoder)
    - [3.9. `CommandBuffer` 空内容，空实现](#39-commandbuffer-空内容空实现)
    - [3.10. `RenderPass`](#310-renderpass)
    - [3.11. `Sampler`](#311-sampler)
    - [3.12. `Texture`](#312-texture)
    - [3.13. `Buffer`](#313-buffer)
    - [3.14. `BufferSlice` 空实现](#314-bufferslice-空实现)
    - [3.15. `BindGroup`](#315-bindgroup)
    - [3.16. `ShaderModule`](#316-shadermodule)
  - [4. 附录：（无计划）用 `WebGL` 模拟 `WebGPU` 的 要点](#4-附录无计划用-webgl-模拟-webgpu-的-要点)
    - [4.1. 和 `WebGL2` 相比，缺失 的 功能](#41-和-webgl2-相比缺失-的-功能)
    - [4.2. 微信小游戏: `iOS` 的 `WebGL`](#42-微信小游戏-ios-的-webgl)

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

**注**：根据微信小游戏的开发者文档，在 iPhone / iPad 上，`WebGL2` 至少要 `iOS`版本 >= 15.5 才能开启

+ 接口 是 [wgpu-rs](https://github.com/gfx-rs/wgpu) 的 0.15 版本 的子集；
+ 实现后端 & 支持平台：
    - Windows: OpenGL 3.3
    - Android: GLES 3.0
    - 浏览器 / 微信小游戏: WebGL 2.0
+ 仅 实现 单线程 版本；
+ 接口上可以创建多个 `CommandEncoder`，但是实际上 都返回 同一个；
+ 录制 即是 提交，所谓的提交是空实现，为了以后的兼容性；
+ 性能优化：设置状态机，做 OpenGL 的 全状态比较；所以 GL 指令数量会比[wgpu-rs](https://github.com/gfx-rs/wgpu)少；

## 3. 限制

只支持 GLSL 格式的 Shader，glsl 450

### 3.1. **不** 支持

+ `ComputePipeline`
+ `ComputePass`
+ `RenderBundle`
+ `QuerySet`
+ `hal::Fence`

### 3.2. `Instance`

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| `new`                | ✔    |      |
| `enumerate_adapters` | ✔    |      |
| `request_adapter`    | ✔    |      |
| `create_surface`     | ✔    |      |

### 3.3. `Adapter`

| 函数                          | 支持 | 说明 |
| ----------------------------- | ---- | ---- |
| `request_device`              | ✔    |      |
| `features`                    | ✔    |      |
| `limits`                      | ✔    |      |
| `get_info`                    | ✔    |      |
| `get_downlevel_capabilities`  | ✔    |      |
| `is_surface_supported`        | ✔    |      |
| `get_texture_format_features` | ✔    |      |

### 3.4. `Surface`

| 函数                  | 支持 | 说明 |
| --------------------- | ---- | ---- |
| `get_capabilities`    | ✔    |      |
| `get_default_config`  | ✔    |      |
| `configure`           | ✔    |      |
| `get_current_texture` | ✔    |      |

### 3.5. `SurfaceTexture`

| 函数      | 支持 | 说明 |
| --------- | ---- | ---- |
| `present` | ✔    |      |

### 3.6. `Device`

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
| create_shader_module_unchecked | ×    |      |
| create_shader_module_spirv     | ×    |      |
| create_render_bundle_encoder   | ×    |      |
| create_compute_pipeline        | ×    |      |
| create_texture_from_hal        | ×    |      |
| create_query_set               | ×    |      |
| poll                           | ×    |      |
| on_uncaptured_error            | ×    |      |
| push_error_scope               | ×    |      |
| pop_error_scope                | ×    |      |
| start_capture                  | ×    |      |
| stop_capture                   | ×    |      |
| as_hal                         | ×    |      |

### 3.7. `Queue`

| 函数                           | 支持 | 说明   |
| ------------------------------ | ---- | ------ |
| `write_buffer`                 | ✔    |        |
| `write_texture`                | ✔    |        |
| `submit`                       | ✔    | 空实现 |
| on_submitted_work_done         | ×    |        |
| write_buffer_with              | ×    |        |
| copy_external_image_to_texture | ×    |        |
| get_timestamp_period           | ×    |        |

### 3.8. `CommandEncoder`

| 函数                    | 支持 | 说明   |
| ----------------------- | ---- | ------ |
| `finish`                | ✔    | 空实现 |
| `begin_render_pass`     | ✔    |        |
| clear_texture           | ×    |        |
| clear_buffer            | ×    |        |
| begin_compute_pass      | ×    |        |
| copy_buffer_to_buffer   | ×    |        |
| copy_buffer_to_texture  | ×    |        |
| copy_texture_to_buffer  | ×    |        |
| copy_texture_to_texture | ×    |        |
| insert_debug_marker     | ×    |        |
| push_debug_group        | ×    |        |
| pop_debug_group         | ×    |        |
| write_timestamp         | ×    |        |
| resolve_query_set       | ×    |        |

### 3.9. `CommandBuffer` 空内容，空实现

### 3.10. `RenderPass`

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

### 3.11. `Sampler`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

### 3.12. `Texture`

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

### 3.13. `Buffer`

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

### 3.14. `BufferSlice` 空实现

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |      |
| get_mapped_range     | ×    |      |
| get_mapped_range_mut | ×    |      |

### 3.15. `BindGroup`

`BindGroup` 会握住它使用的`Buffer`, `Texture`, `Sampler` 对象，使其不会被销毁。

如果想要对应的资源被销毁，必须同时扔掉`BindGroup`和对应的资源。

### 3.16. `ShaderModule`

+ 仅支持 Naga 编译过后，版本为 glsl 3.0 的 无 define 宏 的 glsl
+ 扩展：带上一个字段，用于说明 这个shader原始的 set-binding 和 uniform 的 名字

## 4. 附录：（无计划）用 `WebGL` 模拟 `WebGPU` 的 要点

### 4.1. 和 `WebGL2` 相比，缺失 的 功能

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

### 4.2. [微信小游戏: `iOS` 的 `WebGL`](https://developers.weixin.qq.com/minigame/dev/guide/performance/perf-high-performance.html#%E7%AE%80%E4%BB%8B)

+ `WebGL`: 如果发现 iOS 14 的 帧率提升不上去，试试 在 canvas.getContext 接口 将 antialias: true
+ `WebGL`: iOS 版本 14.0 ~ iOS 15.3, 多个 drawcall 使用不同偏移 来 共享 `VB`/`IB`，性能非常糟糕 ！