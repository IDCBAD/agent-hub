# Agent Hub v0.1.1 发布包

## 发布摘要

- 类型：Windows 体验修复版本。
- 修复：启动自动扫描和手动扫描期间，CLI Runtime 版本探测不再弹出 CMD/控制台窗口。
- 影响范围：仅 Windows 下的 Runtime `--version` 子进程启动方式。
- 不变项：CLI 探测参数与输出解析、Agent 配置扫描、Resource 索引、SQLite Schema 和只读安全边界。

## 实现原理

Agent Hub 会分别执行 Claude Code、Codex、Hermes Agent 和 Kimi Code 的 CLI `--version` 命令。Windows 原先会为这些控制台子进程创建可见窗口；v0.1.1 在 Windows 条件编译下为版本探测进程添加 `CREATE_NO_WINDOW`，保留标准输出、错误输出和超时控制，因此版本识别能力不变。

## 验证结果

- Rust：`cargo fmt --check`、`cargo check --all-targets`、19 个单元测试、`cargo clippy -D warnings` 通过。
- 前端：TypeScript、ESLint、5 个单元测试和 Vite 生产构建通过。
- Runtime 专项：Claude Code 与 Codex 的 Windows npm `.cmd` shim 版本探测测试通过。
- 启动扫描：Release 程序连续监控 45 秒，新增可见 CLI/CMD/控制台窗口为 0。
- 手动扫描：自动调用“扫描本机”后连续监控 30 秒，新增可见 CLI/CMD/控制台窗口为 0。
- 安装器：NSIS 与 MSI 均成功生成；MSI 产品版本为 `0.1.1`。

## 升级与回滚

- 数据迁移：无。
- 升级：可直接使用 v0.1.1 安装包覆盖 v0.1.0。
- 回滚：如发现异常，可卸载 v0.1.1 并重新安装 v0.1.0；Agent 原始配置不会被 Agent Hub 修改。
- 本地 Hub 数据库与 v0.1.0 兼容，无需删除或重建。

## 风险与发布决策

发布决策：`GO WITH CONDITIONS`

- 本地安装与验收条件已满足。
- 安装包当前未进行 Windows Authenticode 签名；公开分发前仍建议配置可信代码签名证书，以降低 SmartScreen 警告概率。
- SHA-256 校验值见 `docs/checksums-v0.1.1.sha256`。
