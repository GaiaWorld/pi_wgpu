# 业务流程图（L1 · 按需）

```mermaid
flowchart TD
  A[create_pipeline_layout] --> B{push 范围非空?}
  B -- 是 --> E1[Err 短码: 推送恒拒止]
  B -- 否 --> C[建布局：set 0-3 校验照旧]
  D[create_render_pipeline] --> F{layout 给值?}
  F -- None --> E2[断言短码含 label: ErrNoLayout]
  F -- Some --> EP{entry_point 给值?}
  EP -- None --> E4[断言短码含 label: ErrEntryPoint]
  EP -- Some --> CO{编译选项默认?}
  CO -- 否 --> E5[断言短码含 label: ErrCompileOpts]
  CO -- 是 --> G{multiview / cache 给值?}
  G -- 是 --> E3[断言短码: ErrMultiview / ErrCache]
  G -- 否 --> I[转译+建管线：现行为不变]
```

## 源表（图是视图，表是源）
| 入口 | 判序 | 违约行为 | 合法路径 |
|---|---|---|---|
| create_pipeline_layout | 推送范围非空 → set 归属 | 短码失败（未建布局） | 现行为 |
| create_render_pipeline | layout 缺省 → entry_point → 编译选项 → 多视角 → 缓存 → 其余现行校验（与芯 create_rp_impl 断言序一致） | 断言含 label+短码（无半建管线） | 转译+创建照旧 |
