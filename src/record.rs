// 设计：pi-wgpu-rebuild · 提交序命令缓冲（B 案：录制≠执行）
//! record · 渲染通道命令的录制与回放。录制期只入表（持有资源强引用），
//! `Queue::submit` 按提交序逐条回放——多线程并行编码因此获得与上游一致的提交序语义。
//! 隐藏决定=命令粒度（通道整体为单位、状态与绘制逐条）。 @internal

use crate::binds::BoundEntry;
use crate::entry::{BindGroup, Buffer, CommandBuffer, CommandEncoder, Queue, RenderPipeline, TextureView};
use crate::types::{Color, DynamicOffset, IndexFormat, Operations};

/// 颜色附件录制形（自带宿主：视图持 Texture 共享引用，命令存活期内资源不被删）。 @internal
#[derive(Debug)]
pub(crate) struct Attachment {
    /// 目标视图（帧视图=surface 占位；宿主视图=纹理）。
    pub(crate) view: TextureView,
    /// MSAA 解析目的视图。
    pub(crate) resolve: Option<TextureView>,
    /// 负载/存储动作。
    pub(crate) ops: Operations<Color>,
}

/// 深度/模板附件录制形。 @internal
#[derive(Debug)]
pub(crate) struct DepthAttachment {
    /// 目标视图。
    pub(crate) view: TextureView,
    /// 深度负载/存储。
    pub(crate) depth_ops: Option<Operations<f32>>,
    /// 模板负载/存储。
    pub(crate) stencil_ops: Option<Operations<u32>>,
}

/// 通道开启录制形。 @internal
#[derive(Debug)]
pub(crate) struct PassBegin {
    /// 通道标签（诊断与错误码用）。
    pub(crate) label: Option<String>,
    /// 颜色附件。
    pub(crate) color: Option<Attachment>,
    /// 深度/模板附件。
    pub(crate) depth: Option<DepthAttachment>,
}

/// 管线持行：只存「部件」而非 RenderPipeline（后者 Drop 会自行释放，双持会双释放）。
/// 生命周期：命令销毁时程序/VAO 计数各 -1。 @internal
#[derive(Debug)]
pub(crate) struct RetainedPipeline {
    /// 程序复用对象（持行）。
    pub(crate) program: crate::objects::Reusable,
    /// VAO 复用对象（持行）。
    pub(crate) vao: crate::objects::Reusable,
    /// 顶点布局。
    pub(crate) vertex: Vec<crate::draw::VertexLayout>,
    /// 图元拓扑。
    pub(crate) topology: crate::types::PrimitiveTopology,
    /// 深度模板定态。
    pub(crate) depth_stencil: Option<crate::types::DepthStencilState>,
    /// 管线混合定态（与管线创建期同源；随程序切换差异由 set_blend 成对下发）。
    pub(crate) blend: Option<crate::types::BlendState>,
    /// 图元固定功能（剔除/绕序/填充）。
    pub(crate) primitive: crate::types::PrimitiveState,
    /// 颜色写掩码（target[0]）。
    pub(crate) color_writes: crate::types::ColorWrites,
    /// 覆盖采样使能。
    pub(crate) alpha_to_coverage: bool,
    /// 槽分配。
    pub(crate) slots: crate::binds::SlotTable,
}
impl RetainedPipeline {
    /// 从管线取持行（程序/VAO 计数各 +1）。 @internal
    pub(crate) fn new(p: &RenderPipeline) -> Self {
        Self {
            program: crate::objects::retain_reuse(&p._program),
            vao: crate::objects::retain_reuse(&p._vao),
            vertex: p._vertex.clone(),
            topology: p._topology,
            depth_stencil: p._depth_stencil.clone(),
            blend: p._blend,
            primitive: p._primitive.clone(),
            color_writes: p._color_writes,
            alpha_to_coverage: p._alpha_to_coverage,
            slots: p._slots.clone(),
        }
    }
}
impl Drop for RetainedPipeline {
    fn drop(&mut self) {
        crate::objects::release_reuse(&self.program);
        crate::objects::release_reuse(&self.vao);
    }
}
/// 绑定组持行：条目里的直建句柄克隆即共享计数，随命令销毁自然归还。 @internal
#[derive(Debug)]
pub(crate) struct RetainedBindGroup {
    /// 条目集（声明序）。
    pub(crate) entries: Vec<BoundEntry>,
}

/// 纹理侧拷贝录制形（持宿主强引用，命令存活期内纹理不被删）。 @internal
#[derive(Debug)]
pub(crate) struct CopyTex {
    /// 宿主纹理。
    pub(crate) texture: crate::entry::Texture,
    /// mip 层。
    pub(crate) mip_level: u32,
    /// 起始纹素。
    pub(crate) origin: crate::types::Origin3d,
    /// 宽高面选择。
    pub(crate) aspect: crate::types::TextureAspect,
}

/// 缓冲侧拷贝录制形。 @internal
#[derive(Debug)]
pub(crate) struct CopyBuf {
    /// 目标/源缓冲。
    pub(crate) buffer: Buffer,
    /// 缓冲侧布局（偏移/行步幅/每图行数）。
    pub(crate) layout: crate::types::TexelCopyBufferLayout,
}

/// 单条录制命令。 @internal
#[derive(Debug)]
pub(crate) enum Command {
    /// 开通道。
    Begin(PassBegin),
    /// 关通道。
    End,
    /// 设管线（持行）。
    SetPipeline(RetainedPipeline),
    /// 设绑定组 + 动态偏移。
    SetBindGroup {
        /// set 域 0-3。
        set: u32,
        /// 条目集持行。
        group: RetainedBindGroup,
        /// 动态偏移（声明序）。
        offsets: Vec<DynamicOffset>,
    },
    /// 设顶点缓冲槽。
    SetVertex {
        /// 槽号。
        slot: u32,
        /// 缓冲持行。
        buffer: Buffer,
        /// 片段起点。
        offset: u64,
        /// 片段长度（None=到末尾）。
        size: Option<u64>,
    },
    /// 设索引缓冲。
    SetIndex {
        /// 索引缓冲持行。
        buffer: Buffer,
        /// 片段起点。
        offset: u64,
        /// 片段长度。
        size: Option<u64>,
        /// 索引宽度。
        format: IndexFormat,
    },
    /// 设视口。
    SetViewport {
        /// x。
        x: f32,
        /// y。
        y: f32,
        /// 宽。
        w: f32,
        /// 高。
        h: f32,
        /// 近平面。
        dn: f32,
        /// 远平面。
        up: f32,
    },
    /// 设裁剪框。
    SetScissor {
        /// x。
        x: u32,
        /// y。
        y: u32,
        /// 宽。
        w: u32,
        /// 高。
        h: u32,
    },
    /// 模板参考值。
    SetStencilRef(u32),
    /// 混合常量色。
    SetBlendConstant(Color),
    /// 数组绘制。
    Draw {
        /// 顶点区间。
        vertices: std::ops::Range<u32>,
        /// 实例区间。
        instances: std::ops::Range<u32>,
    },
    /// 纹理到缓冲读回（编码器命令，随提交序执行）。
    CopyT2B {
        /// 源纹理位。
        src: CopyTex,
        /// 目的缓冲位。
        dst: CopyBuf,
        /// 拷贝尺寸。
        size: crate::types::Extent3d,
    },
    /// 缓冲到纹理上传（编码器命令，随提交序执行）。
    CopyB2T {
        /// 源缓冲位。
        src: CopyBuf,
        /// 目的纹理位。
        dst: CopyTex,
        /// 拷贝尺寸。
        size: crate::types::Extent3d,
    },
    /// 纹理到纹理拷贝（编码器命令，随提交序执行）。
    CopyT2T {
        /// 源纹理位。
        src: CopyTex,
        /// 目的纹理位。
        dst: CopyTex,
        /// 拷贝尺寸。
        size: crate::types::Extent3d,
    },
    /// 索引绘制。
    DrawIndexed {
        /// 索引区间。
        indices: std::ops::Range<u32>,
        /// 顶点基址（本层恒 0）。
        base_vertex: i32,
        /// 实例区间。
        instances: std::ops::Range<u32>,
    },
}

/// 立即回放某编码器已录制命令（测试钩子；不改变提交序合同）。 @internal
pub(crate) fn flush_encoder(e: &mut CommandEncoder) {
    let cmds = std::mem::take(&mut e._commands);
    if cmds.is_empty() {
        return;
    }

    replay_commands(&e._inner, &cmds);
}

/// 提交回放：按提交序逐缓冲、逐通道执行。
/// 现状：跨编码器的顺序=submit 列表顺序（对齐上游）；通道内命令按录制序。 @internal
pub(crate) fn submit_impl(q: &Queue, buffers: &[CommandBuffer]) {
    let Some(dev) = q._dev.as_ref() else {
        return;
    };

    for cb in buffers {
        replay_commands(&dev._inner, &cb._commands);
    }
}

/// 回放一组命令（同一编码器命名空间）。
fn replay_commands(inner: &pi_share::Share<crate::entry::DeviceInner>, cmds: &[Command]) {
    let mut enc = CommandEncoder {
        _opened_pass: false,
        _inner: inner.clone(),
        _commands: Vec::new(),
    };
    let mut it = cmds.iter().peekable();
    while let Some(cmd) = it.next() {
        match cmd {
            Command::Begin(b) => exec_pass(&mut enc, b, &mut it),
            Command::CopyT2B { src, dst, size } => {
                crate::upload_inner::exec_copy_texture_to_buffer(&mut enc, tex_of(src), buf_of(dst), *size)
            }
            Command::CopyB2T { src, dst, size } => {
                crate::upload_inner::exec_copy_buffer_to_texture(buf_of(src), tex_of(dst), *size)
            }
            Command::CopyT2T { src, dst, size } => {
                crate::upload_inner::exec_copy_texture_to_texture(&mut enc, tex_of(src), tex_of(dst), *size)
            }
            _ => panic!("record 断言: 通道外命令（提交序被破坏）"),
        }
    }
}

/// 回放一个通道：把录制形还原成描述符，交执行芯开通道，再按序下发命令。
fn exec_pass<'a, I: Iterator<Item = &'a Command>>(
    enc: &mut CommandEncoder,
    b: &PassBegin,
    it: &mut std::iter::Peekable<I>,
) {
    let ca = b.color.as_ref().map(|a| crate::types::RenderPassColorAttachment {
        view: &a.view,
        resolve_target: a.resolve.as_ref(),
        ops: a.ops,
    });
    let color_attachments = [ca];
    let dsa = b.depth.as_ref().map(|d| crate::types::RenderPassDepthStencilAttachment {
        view: &d.view,
        depth_ops: d.depth_ops,
        stencil_ops: d.stencil_ops,
    });
    let desc = crate::types::RenderPassDescriptor {
        label: b.label.as_deref(),
        color_attachments: &color_attachments,
        depth_stencil_attachment: dsa,
        timestamp_writes: None,
        occlusion_query_set: None,
    };
    let mut pass = crate::draw_inner::exec_begin_impl(enc, &desc);
    let _ = &pass;
    loop {
        match it.peek() {
            None => break,
            Some(Command::End) => {
                it.next();
                break;
            }
            Some(Command::CopyT2B { .. } | Command::CopyB2T { .. } | Command::CopyT2T { .. }) => break,
            Some(_) => {
                let c = it.next().expect("peeked 必有");
                exec_one(&mut pass, c);
            }
        }
    }
    // 延后清屏兜底：通道从未 draw 时也要按 wgpu 语义把 Clear 落地。
    if let Some(pc) = pass._clear.take() {
        crate::draw_inner::run_pending_clear(&pass, &pc);
    }
}

/// 单条命令执行（通道内）。
fn exec_one(p: &mut crate::draw::RenderPass<'_>, c: &Command) {
    match c {
        Command::SetPipeline(pl) => crate::draw_inner::exec_set_pipeline_parts(p, pl),
        Command::SetBindGroup {
            set,
            group,
            offsets,
        } => {
            let g = BindGroup {
                _bound: group.entries.clone(),
            };
            crate::draw_inner::exec_set_bind_group(p, *set, &g, offsets)
        }
        Command::SetVertex {
            slot,
            buffer,
            offset,
            size,
        } => {
            let slice = crate::entry::BufferSlice {
                buffer,
                offset: *offset,
                size: size.map(std::num::NonZeroU64::new).flatten(),
            };
            crate::draw_inner::exec_set_vertex(p, *slot, &slice)
        }
        Command::SetIndex {
            buffer,
            offset,
            size,
            format,
        } => {
            let slice = crate::entry::BufferSlice {
                buffer,
                offset: *offset,
                size: size.map(std::num::NonZeroU64::new).flatten(),
            };
            crate::draw_inner::exec_set_index(p, &slice, *format)
        }
        Command::SetViewport {
            x,
            y,
            w,
            h,
            dn,
            up,
        } => crate::draw_inner::exec_set_viewport(p, *x, *y, *w, *h, *dn, *up),
        Command::SetScissor { x, y, w, h } => crate::draw_inner::exec_set_scissor(p, *x, *y, *w, *h),
        Command::SetStencilRef(v) => crate::draw_inner::exec_set_stencil(p, *v),
        Command::SetBlendConstant(c) => crate::draw_inner::exec_set_blend(p, *c),
        Command::Draw {
            vertices,
            instances,
        } => crate::draw_inner::exec_draw(p, vertices.clone(), instances.clone()),
        Command::DrawIndexed {
            indices,
            base_vertex,
            instances,
        } => crate::draw_inner::exec_draw_idx(p, indices.clone(), *base_vertex, instances.clone()),
        Command::Begin(_) | Command::End => panic!("record 断言: 通道嵌套"),
        Command::CopyT2B { .. } | Command::CopyB2T { .. } | Command::CopyT2T { .. } => {
            panic!("record 断言: 通道内不得有拷贝命令")
        }
    }
}


/// 由录制形还原纹理借用位。
fn tex_of(c: &CopyTex) -> crate::types::TexelCopyTextureInfo<'_> {
    crate::types::TexelCopyTextureInfo {
        texture: &c.texture,
        mip_level: c.mip_level,
        origin: c.origin,
        aspect: c.aspect,
    }
}

/// 由录制形还原缓冲借用位。
fn buf_of(c: &CopyBuf) -> crate::types::TexelCopyBufferInfo<'_> {
    crate::types::TexelCopyBufferInfo {
        buffer: &c.buffer,
        layout: c.layout,
    }
}