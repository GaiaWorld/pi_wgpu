# pi_wgpu

WebGPU 的 Rust 实现，目前只实现 GLES 3.0 / WebGL 2.0 后端；

接口 基于 [wgpu-rs](https://github.com/gfx-rs/wgpu-rs) 的 0.15 版本 为基准 进行修改；

## Why

为什么不用 [wgpu-rs](https://github.com/gfx-rs/wgpu-rs)，因为 实际测试中，这个库目前 gles-backend 由于 为了实现通用的目的，而性能不满足游戏行业的需求；

而 Vulkan / WebGPU 因为 底层驱动 不够好，而不能广泛使用；

所以需要一个 简化版本的 GLES 3.0 / WebGL 2.0 后端实现 过渡 一段时间。

## 设计

+ 仅 实现 单线程 版本；不考虑多线程的安全性；
+ 接口上可以创建多个 CommandEncoder，但是实际上 都返回同一个；
+ 录制 即是 提交，所谓的提交是空实现，为了以后的兼容性；
+ 性能优化：底层设置 状态机，做 OpenGL 的 全状态比较；所以 GL-Backend 的指令数量 预期 会比 [wgpu-rs](https://github.com/gfx-rs/wgpu-rs) 少很多；