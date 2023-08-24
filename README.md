# pi_wgpu

WebGPU 的 Rust 实现，目前只实现 GL 后端；

接口 基于 [wgpu-rs](https://github.com/gfx-rs/wgpu) 的 0.15 版本 为基准 进行修改；

## Why

为什么不用 [wgpu-rs](https://github.com/gfx-rs/wgpu)，因为 实际测试中，这个库目前 WebGL2 Backend 由于 为了实现通用的目的，而性能不满足本司游戏的性能的需求；

Vulkan / WebGPU 因为渲染驱动兼容性问题，近期内 不能广泛 在如下平台中广泛使用：

+ Android / PC的Android模拟器
+ 浏览器
+ 微信小游戏

所以 需要 简化版本的 GL 后端实现 过渡 一段时间。

## 设计

+ 接口上 [wgpu-rs](https://github.com/gfx-rs/wgpu) 的 0.15 版本 兼容；
+ 实现后端 & 支持平台：
    - Windows: OpenGL 3.3
    - Android: GLES 3.0
    - 浏览器 / 微信小游戏: WebGL 2.0
+ 仅 实现 单线程 版本；不考虑多线程的安全性；
+ 接口上可以创建多个 CommandEncoder，但是实际上 都返回 同一个；
+ 录制 即是 提交，所谓的提交是空实现，为了以后的兼容性；
+ 性能优化：底层设置 状态机，做 OpenGL 的 全状态比较；所以 GL-Backend 的指令数量 预期 会比 [wgpu-rs](https://github.com/gfx-rs/wgpu) 少；

## 限制

**不支持** 意味着：调用相关函数时，运行时 会 panic

### 不支持

+ 非 GLSL 格式的 Shader
+ `ComputePipeline`
+ `ComputePass`
+ `RenderBundle`
+ `QuerySet`
+ `hal::Fence`

## `Device`

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

## `Queue`

| 函数                           | 支持 | 说明 |
| ------------------------------ | ---- | ---- |
| `write_buffer`                 | ✔    |
| `write_texture`                | ✔    |
| `submit`                       | ✔    |
| on_submitted_work_done         | ×    |
| write_buffer_with              | ×    |
| copy_external_image_to_texture | ×    |
| get_timestamp_period           | ×    |

### `CommandEncoder`

| 函数                    | 支持 | 说明 |
| ----------------------- | ---- | ---- |
| `finish`                | ✔    |      |
| `begin_render_pass`     | ✔    |      |
| `clear_texture`         | ✔    |
| `clear_buffer`          | ✔    |
| begin_compute_pass      | ×    |      |
| copy_buffer_to_buffer   | ×    |
| copy_buffer_to_texture  | ×    |
| copy_texture_to_buffer  | ×    |
| copy_texture_to_texture | ×    |
| insert_debug_marker     | ×    |
| push_debug_group        | ×    |
| pop_debug_group         | ×    |
| write_timestamp         | ×    |
| resolve_query_set       | ×    |

### `RenderPass`

| 函数                              | 支持 | 说明                                                   |
| --------------------------------- | ---- | ------------------------------------------------------ |
| `set_bind_group`                  | ✔    |
| `set_pipeline`                    | ✔    |
| `set_blend_constant`              | ✔    |
| `set_index_buffer`                | ✔    |
| `set_vertex_buffer`               | ✔    |
| `set_scissor_rect`                | ✔    |
| `set_viewport`                    | ✔    |
| `set_stencil_reference`           | ✔    |
| `draw`                            | ✔    | 参数 `first_instance` 必须为 0; `base_vertex` 必须为 0 |
| `draw_indexed`                    | ✔    |
| `set_push_constants`              | ×    |
| insert_debug_marker               | ×    |
| push_debug_group                  | ×    |
| pop_debug_group                   | ×    |
| draw_indirect                     | ×    |
| draw_indexed_indirect             | ×    |
| execute_bundles                   | ×    |
| multi_draw_indirect               | ×    |
| multi_draw_indexed_indirect       | ×    |
| multi_draw_indirect_count         | ×    |
| multi_draw_indexed_indirect_count | ×    |
| write_timestamp                   | ×    |
| begin_pipeline_statistics_query   | ×    |
| end_pipeline_statistics_query     | ×    |

### `Buffer`

| 函数                       | 支持 | 说明 |
| -------------------------- | ---- | ---- |
| `size`                     | ✔    |      |
| `usage`                    | ✔    |      |
| `as_entire_binding`        | ✔    |      |
| `as_entire_buffer_binding` | ✔    |      |
| slice                      | ×    |
| unmap                      | ×    |
| destroy                    | ×    |

### `BufferSlice`

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |
| get_mapped_range     | ×    |
| get_mapped_range_mut | ×    |

### `Texture`

| 函数                    | 支持 | 说明 |
| ----------------------- | ---- | ---- |
| `create_view`           | ✔    |      |
| `size`                  | ✔    |      |
| `width`                 | ✔    |      |
| `height`                | ✔    |      |
| `depth_or_array_layers` | ✔    |      |
| `mip_level_count`       | ✔    |      |
| `sample_count`          | ✔    |      |
| `dimension`             | ✔    |      |
| `format`                | ✔    |      |
| `usage`                 | ✔    |      |
| as_image_copy           | ×    |      |
| as_hal                  | ×    |      |
| destroy                 | ×    |      |