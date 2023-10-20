# pi_wgpu

**注意：** 运行前，需要将 bin/两个dll 拷贝 到 运行目录，或者 exe所在目录

`WebGPU`接口子集的`GL`后端，基于`Rust`实现。

+ 接口: 基于 [wgpu-rs](https://github.com/gfx-rs/wgpu) `v0.16` 进行修改；
+ 功能: 仅仅对应[WebGL2](https://www.khronos.org/files/webgl20-reference-guide.pdf)的子集
    - 加`压缩纹理`扩展：`DDS` / `ASTC`
+ 平台:
    - `Windows`: `OpenGL 3.3`
    - `Android` / PC-Android 模拟器: `GLES 3.0`
    - 浏览器 / 微信小游戏: [WebGL2](https://www.khronos.org/files/webgl20-reference-guide.pdf)

# 1. 使用

## 1.1. 集成 wgpu-rs

为了兼容 wgpu-rs，pi_wgpu 也 包含了 wgpu

如果不加 feature，默认就是用 pi-wgpu 的实现

如果 想用 wgpu-rs 实现，在 feature加入 `use_wgpu` 即可，例子：

``` toml
[dependencies]
pi_wgpu = {version = "0.1", default-features = false, features = ["use_wgpu"]}
```

## 1.2. 运行 Windows 平台

执行 `cargo run --example triangle` 命令运行 triangle example

## 1.3. 运行 [Web 平台](https://rustwasm.github.io/docs/wasm-bindgen/contributing/testing.html)

+ 执行`wasm-pack test  --chrome --example triangle`命令，构建wasm以及测试环境，并开启服务器监听在`8000`端口 
+  在浏览器中访问`http://127.0.0.1:8000`地址，即可运行测试

## 1.4. 运行 Android 平台

+ 打开Linux Shell, 执行 `cargo apk run --example triangle --lib` 编译 example为apk
+ 链接手机 在 `target\debug\apk\examples` 中打开cmd， 并执行 `adb install triangle.apk` 来安装apk

## 1.5. 需要调试的 Demo

+ GUI: pi_ui_render, bevy 分支
    - blend_mode.rs
    - shadow.rs
    - compress_texture
+ 3D:
    - pi_3d
        * 最简单的渲染 simple / dispose.rs
        * 动画，buffer更新：anime / 01.rs
        * 粒子 particle / render_stretched.rs
    - 后处理 pi_post_process / main.rs

# 2. 设计

+ `性能优化`：设置状态机，做GL的全状态比较；所以GL指令数量会比[wgpu-rs](https://github.com/gfx-rs/wgpu)少；

**注1**：为什么不直接用 [wgpu-rs](https://github.com/gfx-rs/wgpu)

+ 实际测试，库 WebGL2 Backend性能不满足游戏需求；
+ `Vulkan` / `WebGPU` 因兼容性问题，近期内不能广泛使用于各个平台；

**注2**：根据微信小游戏的开发者文档，在 iPhone / iPad 上，`WebGL2` 至少要 `iOS`版本 >= 15.5 才能开启

**注3**：详细的 设计笔记，[见这里](documents/design.md)

## 2.01. 限制

+ 销毁资源：可以在录制指令时 创建 / 销毁 资源
+ 线程安全：
    - 录制指令 是 `单线程`，录制即为提交，所谓的队列提交是空实现；
    - 创建，销毁 资源，Exe / Apk `多线程`；
    - `注`：使用者自己确保，创建/释放 和 录制指令 的 线程安全性；
+ GLSL:
    - 功能: 仅支持 [gles-300 / std140-布局](https://www.khronos.org/files/webgl20-reference-guide.pdf)；
    - 语法: 仅支持 [GLSL 450 语法](https://www.khronos.org/files/webgl20-reference-guide.pdf)；

## 2.02. 销毁资源

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

## 2.03. **不** 支持

| 函数            | 支持 | 说明            |
| --------------- | ---- | --------------- |
| QuerySet        | ×    | WebGL2 本身支持 |
| hal::Fence      | ×    | WebGL2 本身支持 |
| RenderBundle    | ×    |                 |
| ComputePipeline | ×    |                 |
| ComputePass     | ×    |                 |
| hal::Barrier    | ×    |                 |

## 2.04. wgpu::util

| 函数                       | 支持 | 说明                        |
| -------------------------- | ---- | --------------------------- |
| `backend_bits_from_env`    | ✔    | 永远返回 Some(Backends::GL) |
| `create_texture_with_data` | ✔    |                             |
| `create_buffer_init`       | ✔    |                             |

## 2.05. `Instance`

| 函数               | 支持 | 说明 |
| ------------------ | ---- | ---- |
| `new`              | ✔    |      |
| `request_adapter`  | ✔    |      |
| `create_surface`   | ✔    |      |
| enumerate_adapters | ×    |      |

## 2.06. `Adapter`

| 函数                          | 支持 | 说明 |
| ----------------------------- | ---- | ---- |
| `request_device`              | ✔    |      |
| `features`                    | ✔    |      |
| `limits`                      | ✔    |      |
| `get_info`                    | ✔    |      |
| `get_downlevel_capabilities`  | ✔    |      |
| `is_surface_supported`        | ✔    |      |
| `get_texture_format_features` | ✔    |      |

## 2.07. `Surface`

| 函数                  | 支持 | 说明                                                        |
| --------------------- | ---- | ----------------------------------------------------------- |
| `configure`           | ✔    | `present_mode`: 必须是`Fifo`; `alpha_mode`: 必须是 `Opaque` |
| `get_current_texture` | ✔    |                                                             |
| get_default_config    | ×    |                                                             |
| get_capabilities      | ×    |                                                             |

## 2.08. `SurfaceTexture`

| 函数      | 支持 | 说明 |
| --------- | ---- | ---- |
| `present` | ✔    |      |

## 2.09. `Device`

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

## 2.10. `Queue`

| 函数                           | 支持 | 说明   |
| ------------------------------ | ---- | ------ |
| `write_buffer`                 | ✔    |        |
| `write_texture`                | ✔    |        |
| `submit`                       | ✔    | 空实现 |
| on_submitted_work_done         | ×    |        |
| write_buffer_with              | ×    |        |
| copy_external_image_to_texture | ×    |        |
| get_timestamp_period           | ×    |        |

## 2.11. `CommandEncoder`

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

## 2.12. `CommandBuffer` 空内容，空实现

## 2.13. `RenderPass`

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

## 2.14. `Sampler`

**注**：其生命周期还受到 `BindGroup` 的影响，见`BindGroup`

## 2.15. `Texture`

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

## 2.16. `Buffer`

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

## 2.17. `BufferSlice` 空实现

| 函数                 | 支持 | 说明 |
| -------------------- | ---- | ---- |
| map_async            | ×    |      |
| get_mapped_range     | ×    |      |
| get_mapped_range_mut | ×    |      |

## 2.18. `BindGroup`

`BindGroup` 会握住它使用的`Buffer`, `Texture`, `Sampler` 对象，使其不会被销毁。

如果想要对应的资源被销毁，必须同时扔掉`BindGroup`和对应的资源。

## 2.19. `ShaderModule`

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
    - `几何实例化`
    - 多目标渲染 / drawBuffers clearBuffer
    - 多重采样
    - FeedBack

## 3.2. [微信小游戏: `iOS` 的 `WebGL`](https://developers.weixin.qq.com/minigame/dev/guide/performance/perf-high-performance.html#%E7%AE%80%E4%BB%8B)

+ `WebGL`: 如果发现 iOS 14 的 帧率提升不上去，试试 在 canvas.getContext 接口 将 antialias: true
+ `WebGL`: iOS 版本 14.0 ~ iOS 15.3, 多个 drawcall 使用不同偏移 来 共享 `VB`/`IB`，性能非常糟糕 ！
