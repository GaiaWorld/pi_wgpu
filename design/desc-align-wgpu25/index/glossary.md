# 登记分册：词汇表

## 词汇

<!-- 只登记用户词→领域判据翻译后的条目（L1 产出，ddt tableAddRow 建行）。不是术语大全，不追求覆盖。 -->

| 术语 | 定义 | 出处 |
|---|---|---|
| | 无（建行由机器层 tableAddRow 追加） | — |
| 强拉齐 | 公开词汇形态逐项照抄上游，能力域维持收窄；源兼容优先于历史形便 | 用户拍板（本会话） |
| 形宽法窄 | 官方词汇面全表声明，非法值运行期短码拒止，不留未声明的洞 | 本会话拍板 + pi-wgpu-rebuild 先例 |
| 违约 | 违反硬约束的输入：断言失败含 label+短码，不残留半建对象 | pi-wgpu-rebuild 创建族合同 |
| 违约短码（Err* 族） | 拒止断言词汇位：ErrNoLayout/ErrEntryPoint/ErrCompileOpts/ErrMultiview/ErrCache/ErrPushRange/ErrPush；触发点见 07-design/call-chain.md 源表 | L9 审 R8 回改 |
