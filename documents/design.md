- [设计笔记](#设计笔记)
- [1. BindGroup / RenderPipeline / Program](#1-bindgroup--renderpipeline--program)
- [2. WebGL2 的 Buffer 和 Sampler 槽 的 重用问题](#2-webgl2-的-buffer-和-sampler-槽-的-重用问题)
- [3. BindGroup 的 DynamicOffset](#3-bindgroup-的-dynamicoffset)
- [4. opengl 多线程](#4-opengl-多线程)

# 设计笔记

# 1. BindGroup / RenderPipeline / Program

``` rust
// 这里的 BG 布局，用到了 GLSL 的 2，5，8, 10
// 2, 10 是 UBO，在 vs 中用
// 5. 8 是 Texture，在 fs中用
// 仅有 10 的 Buffer带了动态索引
let bg1_layout = BindGroupLayoutDescriptor {
    label: "bg1"
    entries: &[BindGroupLayoutEntry {
        binding: 2, // 对应shader的 layout(set = XXX, binding = 2) uniform`
        visibility: ShaderStages::Vertex,
        count: None,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform, // WebGL2 不支持 Storage
            min_binding_size: None,
            has_dynamic_offset: false,
        },
    }, BindGroupLayoutEntry {
        binding: 8, // 对应shader的 layout(set = XXX, binding = 8) uniform`
        visibility: ShaderStages::Fragment,
        count: None,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float {
                filterable： true,
            },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }, BindGroupLayoutEntry {
        binding: 5, // 对应shader的 layout(set = XXX, binding = 5) uniform`
        visibility: ShaderStages::Fragment,
        count: None,
        ty: BindingType::Sampler(SamplerBindingType::Filtering),
    }, BindGroupLayoutEntry {
        binding: 10, // 对应shader的 layout(set = XXX, binding = 2) uniform`
        visibility: ShaderStages::Vertex,
        count: None,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform, // WebGL2 不支持 Storage
            min_binding_size: None,
            has_dynamic_offset: true,
        },
    }
}
```

Pipeline：

``` rust
// Pipeline 的 Layout 是 &[BindGroupLayout]

let pipeline_layout = PipelineLayoutDescriptor {
    // 每个槽位对应shader中的 set，硬件限制，约定set的设置是 0-3；
    // bg_layout[0] 对应 layout(set = 0, binding = XXX)
    // bg_layout[1] 对应 layout(set = 1, binding = XXX)
    // bg_layout[2] 对应 layout(set = 2, binding = XXX)
    // bg_layout[3] 对应 layout(set = 3, binding = XXX)
    bg_layout: &[BindGroupLayout],
}
```

这里的灵活性在于：

+ 一个 Pipeline 对应 一个 VS，PS，还有自己的 PipelineLayout；
+ 不同的pipeline 可以 指定 同一个 （VS，PS），但有自己的 Layout布局
    - 可以认为 不同的 &[BindGroupLayout], 是 对 同一个 shader 的 binding 的 重新排列；
    - 实现上，为每个 RenderPipeline 准备了 一个 reorder数组，将  program的 binding 数组重新映射到 BindGroupLayout 去；

# 2. WebGL2 的 Buffer 和 Sampler 槽 的 重用问题

因为 WebGL2 的模型是 针对 整个 Buffer 和 UBO 来排列

可以认为，整个GL状态机，有两个槽数组（这里的槽位，跨program的）

+ UBO Slots: [在 联想拯救者笔记本 的 Chrome，是 12个],
+ Sampler Slots: [在 联想拯救者笔记本，是 16个],

naga 编译 shader的时候，会处理 同一个 texture 和 sampler 的 映射，我能知道他们的映射关系；这里不讨论texture，可以认为texture和sampler 一个整体；

我这边的映射关系是：

+ 准备一个 全局 映射关系，将 (set, binding, Buffer or Sampler) --> 上面的分配的槽位
+ 使用时，将 相同buffer或sampler的东西，尽量映射到 同一个 layout(set = X, binding = Y) 去；
+ 例子：相机和投影矩阵，整体作为一个 binding，所有的 program，都帮到 (set=0, binding=13) 上去。shader的名字可以不一样）；同理的还有场景数据（比如  阴影图，光照图）

# 3. BindGroup 的 DynamicOffset

到 set_bind_group(set, bg, &[DynamicOffset]), 这里的 DynamicOffset，仅仅针对 bg1_layout 中，设置了 has_dynamic_offset = true 的 Buffer（按顺序排列），就上面的  bg1_layout 有四个，但因为 has_dynamic_offset 的 只有一个，所以 &[DynamicOffset] 只有一个元素；依次类推；

# 4. opengl 多线程

其实，传统搞法是准备一个指令队列，所有的gl操作放到一个独立的线程异步处理，但那样录制成本未知，而且常见函数要么加壳要么就异步，不符合webgpu接口；所以，下面的搞法其实是 迫不得已：

+ egl 有个make_current，作用是把 三元组（）绑定到 该线程的线程局部存储上；
+ 传统的opengl：gl 初始化的时候，要做一次 egl.make_current(draw_surface read_surface, gl_context);
+ 这样 每个gl函数调用，glClear()，内部做两件事：
    - 到 线程局部存储 找 gl_context
    - 调用 gl_context.clear()
+ 如果想要 gl 函数 不定期的做多线程，在每个gl函数调用的 wgpu 函数，要做：
    - mutex.lock()
    - egl.make_current(draw_surface read_surface, gl_context);
    - 用 gl 函数
    - make_current(egl, None, None, None); // 这样，gl_context 会从 当前线程的线程局部存储拿走，之后就可以绑定到别的线程。如果不调用，在别的线程重复绑定同一个gl-context，就会崩溃；
    - mutex.unlock()
+ 性能：因为 make_current做了两件事：绑定线程局部存储，同时将 环境和两个表面做某种内核态的关联，所以会比普通函数非一些。
+ wgpu-rs 和 serva，这些全平台的封装也是这么搞的。目的是为了让gl平台和别的api一样可以多线程使用。