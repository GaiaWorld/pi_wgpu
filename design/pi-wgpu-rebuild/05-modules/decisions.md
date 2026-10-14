# pi-wgpu-rebuild · 决定与来源（L5 · 资产出库校准版）

## 决定记录

| # | 决定 | 理由 | 被否方案（为何否） |
|---|---|---|---|
| ① | 按易变决定切 12 模块，不采旧仓 wgc/hal/wgt 三层 | 用户「不依赖原仓结构」；步骤式切分=任何变动动全身 | 继承旧三层（违背拍板）；保留 hal 关联类型多后端抽象（无当前消费者判据 6，指不出用它的地方） |
| ② | 公开对象=同步转发薄壳；无验证层、无命令队列 | 录制即执行定论（L1 轮1）；错误面=调用现场即时 | wgpu-core 语义复刻（成本远超收益）；提交前校验/回滚（已被轮1 否） |
| ③ | types 自建移植词汇表，不复用官方 wgpu-types | 子集需偏离字段（动态偏移声明、count 等），官方面装不下收窄语义 | 直接依赖 wgpu-types 改造（上游类型不可加字段）；完全跟官方命名（偏离项编译即暴露，登记迁移清单更诚实） |
| ④ | 依赖复用清单：pi_egl 0.2.3 / pi_glow 0.13.3 / naga 25.0.1（本地 fork，glsl-in/out）/ pi_share 0.5.2 / pi_hash 0.1.1 + 通用 crates.io 沿用旧清单 | L5 可行性验证全绿（本机版本逐一对、fork 含所需特性、nightly/wasm32/wasm-pack 俱备） | pi_assets / pi_time 移出本库依赖（资产出库校准，用户判生态以 pi_render_asset 为准——其 Cargo.toml 实证：上游用 pi_wgpu+pi_assets 组合） |
| ⑤ | single_thread 维持编译期 feature | 运行时开关的逐调用分支违背零开销（NFR-03） | 运行时检测（分支成本+语义不清） |
| ⑥ | 交集原则：跨形态差异仅经 caps/formats/context-surface/pi_glow 四道闸，交集层禁形态分支 | 散分支=变更点无处隔离（旧仓 cfg 散布方式用户已判不采用）；新增形态约束只动三处（变更点行） | 每模块自写形态分支（维护性差，实测特判散落）；三形态三实现（复制镜像逻辑） |
| ⑦ | 资产管理出本库：纹理/缓冲/采样器 1:1 直建；仅程序/VAO/FBO/RBO 库内键复用；记账/键值/异步/收缩全在上游资产层 | L5 门禁校准（2026-10-13）：pi_render_asset 实证资产面建在本库公开 API 之上；键复用对象=纯性能无语义分歧 | 库内建 AssetMgr 缓存/队列（用户翻转原 FR-10 直译）；连 program/VAO 复用也废（FR-11 判据实测大头存在，消费者=gui_demo） |
| ⑧ | 删除时机=即删+镜像同刷失；库不设兜底队列 | 资产出库后队列无资产可管；合同（三硬规则）唯一化 | 库内延迟队列兜底违约（合同与兜底双标准，文档不可判）；即删不刷（镜像脏读）——刷失是即删的必要配套 |
| ⑩ | feature 开关保留 gl/single_thread/geterror/webgl/webgl_context、**删除 webgpu**；零 src 内当前 cfg 消费者=属相-2 实现批的内部 cfg（geterror 探针=upload wasm 路径、single_thread=context 退化、webgl*=驱动形态分支），非公开 API 扩展点 | R13 复判：开关=构造面声明（实现批即消费者），留痕本行；**webgpu 与 problem OUT「非 GL 系后端」相抵=删** | 全留（被否：webgpu 违判据 6+OUT 冲突） |
| ⑨ | 复用类句柄的删除=全部持有者释放后（引用规则细节 L7 定） | 键复用对象的固有约束；即删语义对 1:1 类不变 | 复用对象也即删（其余持有者悬空） |
| ⑪ | 错误处理：优先上递错误码（`Result<_, ErrCode>`），避免原地 panic 使运行崩溃；不便上递处 `log::error!` 记码后降级/继续；错误信息=短错误码（枚举），禁长文本；断言仅留「绝不该发生」的编程合同违反且用短码 | 运行时稳健性优先（原地 panic=整体崩溃，用户判不可接受）；错误码便于程序化处理与低噪日志 | 全断言（现状，崩溃面大）；anyhow/String 长文本（噪音大、不可程序化） |
| ⑫ | 1:1 资源句柄 DirectHandle 改**共享计数**（Share<T>：Clone 共享同一驱动名，末引用 Drop 才删）；绑定组条目持 clone，用户与绑定组双持互不删（对齐 pi_render `Share<wgpu::BindGroup>` / `Arc<RWBufferRange>` 强引用） | B11 整合暴露：create_bind_group 只拿 &Buffer，持有型句柄无法既入组又不双重删除；共享计数=wgpu/pi_render 既有范式 | 非持有快照 HandleRef{name,generation}（无法实现「资源先于组销毁=断言」，探不到用户释放）；并列反转本册数据册早前「被否 Arc+refcount」条目 |
| ⑬ | 状态镜像整发档改「已下发水位」（water）比对增量下发 | 每帧 60+ 条冗余 GL 调用（帧 2 崩溃排查判为无效负载）；水位=单点比对，非各写入点置位 | 维持恒幂等整发（冗余调用不退）；dirty 位图（仍被否：置位散落各写入点易漏置位） |

## 可行性验证（高风险假设）

| 假设 | 验证方式 | 结果 |
|---|---|---|
| 私有依赖版本与本机可得 | 读 E:\\Rust\\PI_PI\\各 crate 的 Cargo.toml | ✔ semver 全对（pi_assets 验证后已从依赖移除） |
| naga 本地 fork 含 glsl-in/out | 读 ..\\wgpu\\naga\\Cargo.toml | ✔ 25.0.1 含两特性 |
| nightly 工具链可用 | rustup toolchain list | ✔ 默认 nightly-2025-03-04 active |
| wasm 验收链路（L1 轮2 拍板先行平台） | rustup target list / wasm-pack | ✔ wasm32 已装、wasm-pack 0.12.1 |
| 资产层在生态中存在且在本库之上 | 读 pi_render/crates/render_asset（Cargo 依赖 pi_wgpu+pi_assets；src 对其实现 Asset/Size/Garbageer） | ✔ 分层实证——决定⑦的直接依据 |
| registry 在线可解 / 全依赖 cargo check | 未验 | ✔ L7 已收敛（2026-10-13）：path 依赖全解析、cargo check Finished；registry 在线本身仍未测（发布批再验） |

## 来源（买还是造）

| 对象 | 来源 | 理由 | 被否 | 版本锁定/升级路径 |
|---|---|---|---|---|
| GL/GLES/WebGL2 函数入口 | 复用 pi_glow | 三形态一签名=交集原则的物质基础 | 自封装（重复） | 0.13.3；随 wgpu 大版本评估 |
| EGL 绑定 | 复用 pi_egl | 旧环境实证 | 官方 egl 绑定面窄 | 0.2.3；registry 不可解时 path 兜底 |
| 着色器转译 | 复用 naga（本地 fork） | 旧实现同一 fork；glsl-in/out 齐 | 上游 crates.io naga（fork 差异未核） | 25.0.1 锁 path；升级随 wgpu 主版本 |
| 跨线程所有权 | 复用 pi_share | 对象引用链原语 | Rc+unsafe（线程合同破） | 0.5.2 |
| 高速哈希 | 复用 pi_hash/twox-hash | 复用键与镜像表热点 | std HashMap（键控性能实测不足） | 0.1.1 / 1.6 |
| 资产记账/缓存/回收 | 复用生态=上游 pi_render_asset（库外） | 校准决定⑦ | 库内自建（已否） | 随上游 crate |
| 12 内部模块 | 自建 | 核心职责所在 | — | 本设计 L7 |
