# Agent Hub v0.1.0 发布包

## 发布摘要

- 范围：Agent Hub 本地优先 MVP 首次发布。
- 平台：Windows x64。
- 用户可见能力：发现本机 AI Agent 命令行程序、配置与只读资源；查看版本、健康状态和安装来源；打开已验证目录或资源；手动登记和安全移除实例。
- 影响面：本机应用、用户级 SQLite 数据库和系统默认文件打开程序。
- 外部服务依赖：无。Agent Hub 不上传本地配置。

## 依赖与构建

- Node.js 20+
- npm 10+
- Rust stable MSVC toolchain
- Microsoft Edge WebView2 Runtime
- Tauri 2 系统构建依赖

发布门禁命令：

```powershell
npm run typecheck
npm run lint
npm test
npm run build
cd src-tauri
cargo fmt --all -- --check
cargo check --all-targets
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd ..
npm run desktop:build
```

## 风险

### Medium：SQLite Schema v5 自动迁移

- 影响：已有数据库启动时原地升级。
- 缓解：迁移仅增加 Runtime 版本探测字段并统一历史显示名；已通过 v1 到 v5 测试和本机真实数据库升级。
- 监控：启动后确认 Agent 列表、版本和资源数量存在。

### Medium：Windows 安装包未签名

- 影响：公开下载时可能出现 SmartScreen 警告。
- 缓解：内部或本机安装可通过 SHA-256 校验产物；公开分发前配置可信代码签名证书。

### Low：同一产品可有多个本地实例

- 影响：Hermes Agent 等产品可能因多个配置根显示多行。
- 缓解：列表明确显示“产品种类”和“本地实例”数量。

## Migration

- Type: SQLite schema migration v1-v4 → v5
- Affected Surface: `%APPDATA%\io.agenthub.desktop\agent-hub.db`
- Run Before Rollout: 否；应用首次启动时自动执行
- Rollback Safe: 应用数据可删除后重新扫描重建；原始 Agent 配置不受影响
- Operator Warning: 回退旧版本前建议备份或删除 Hub 自身数据库，避免旧程序读取新字段时出现兼容问题

## 发布步骤

1. 确认 Git 工作树只包含 v0.1.0 范围内文件。
2. 执行全部发布门禁。
3. 构建 NSIS 和 MSI Release 安装包。
4. 生成并记录安装包 SHA-256。
5. 在干净用户环境安装并启动。
6. 确认无 Vite 服务时应用正常显示。
7. 扫描本机并验证至少一个 Runtime、Configuration 和 Resource。
8. 建立 `v0.1.0` Git 标签。

## 回滚

触发条件：

- 应用无法冷启动；
- SQLite 迁移失败或 Agent/Resource 大量丢失；
- 扫描修改了 Agent 原始配置；
- 打开操作越过已确认配置根目录。

回滚步骤：

1. 卸载 v0.1.0。
2. 保留所有 Agent 原始配置目录。
3. 删除或备份 `%APPDATA%\io.agenthub.desktop\agent-hub.db`。
4. 重新安装上一可用版本，或运行上一独立构建。
5. 重新扫描生成 Hub 本地索引。

## 沟通

- 工程维护者：发布时记录提交、标签、安装包路径和 SHA-256；出现迁移或启动问题立即停止分发。
- 验收用户：升级无需处理 Agent 配置；首次启动会自动扫描，时间取决于各 CLI 的版本响应速度。
- 支持说明：版本探测失败会显示具体状态；优先收集程序路径、探测状态和错误说明，不要求用户提供认证文件正文。

## 发布决策

`GO WITH CONDITIONS`

条件：

- 全部门禁已通过：Rust 19 项测试、前端 5 项测试、类型、Lint、Clippy 和生产构建均成功；
- NSIS/MSI 安装包已成功生成并完成 SHA-256 校验，校验值见 `docs/checksums-v0.1.0.sha256`；
- Release 程序已在 `localhost:1420` 无监听时冷启动，并显示实例级统计与本机 Agent 版本；
- 对外公开分发前必须补充 Windows 代码签名。
