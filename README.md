# Agent Hub

Agent Hub 是一个本地优先的 AI Agent 桌面管理入口。它不会替代 Claude Code、Codex、Hermes Agent 或 Kimi Code，而是发现本机实例、建立只读资源索引，并把分散的配置、Prompt、Skill、MCP 与身份文件整理为可解释、可导航的资源地图。

当前仓库已包含可运行的 MVP：

- Tauri 2 + React 19 + TypeScript + Vite
- Rust 分层后端与窄 IPC 命令
- SQLite 初始化、migration、事务对账与持久化
- Claude Code、Codex、Hermes Agent、Kimi Code Adapter
- Runtime 与 Configuration 分离检测
- PATH、npm 与默认安装路径中的 CLI Runtime 发现和版本检测
- Agent 列表、搜索、状态筛选与详情
- 白名单 Resource 扫描、敏感标记与跨 Agent 索引
- 打开 Agent 目录与已验证资源
- 手动添加 Agent 配置路径
- 加载、空状态、错误和恢复动作

## 安全边界

MVP 是只读管理层：

- 不修改 Agent 原生配置。
- 不上传本地配置或工作区内容。
- 不把配置正文、API Key、Token 或认证文件内容写入 SQLite。
- 不向前端暴露任意文件读取、任意文件写入或任意命令执行接口。
- 打开操作只接收数据库 ID，由 Rust 查询并再次验证目标路径属于已确认的 Agent 根目录。

## 开发环境

需要：

- Node.js 20 或更高版本
- npm 10 或更高版本
- Rust stable（通过 rustup 安装）
- 对应平台的 [Tauri 系统依赖](https://v2.tauri.app/start/prerequisites/)

安装依赖：

```bash
npm install
```

启动浏览器 UI 预览：

```bash
npm run dev
```

浏览器预览不访问本地文件系统；扫描、手动添加和打开操作只有在 Tauri 桌面运行时可用。

启动桌面应用：

```bash
npm run tauri dev
```

## 质量检查

前端：

```bash
npm run typecheck
npm run lint
npm run test
npm run build
```

Rust：

```bash
cd src-tauri
cargo fmt --all -- --check
cargo check --all-targets
cargo test --lib
cargo clippy --all-targets -- -D warnings
```

生成可独立运行但不打包的桌面程序：

```bash
npm run desktop:build:binary
```

生成 Windows Release 安装包：

```bash
npm run desktop:build
```

安装包输出到：

```text
src-tauri/target/release/bundle/nsis/
src-tauri/target/release/bundle/msi/
```

## 架构

```text
React Presentation
        │
Typed IPC Client
        │
Tauri Commands
        │
Application Service
   ┌────┴──────────┐
Domain Model   Adapter Registry
   │                 ├── Runtime Detection
   │                 └── Configuration Detection
   │                            │
SQLite v2                 Filesystem / PATH / OS
   │
   ├── Agent Runtime
   ├── Agent Configuration
   └── Resource Index
```

每个 Agent Instance 由三部分组成：Runtime 记录 CLI 是否安装、命令名、可执行文件、CLI 自报版本、解析来源和安装方式；Configuration 记录配置根、配置文件及 Resource；Health 由两者综合计算，不作为 Adapter 内的混合检测结果。

手动添加路径作为独立的用户登记保存在 `manual_agent_locations`。从 Hub 移除手动 Agent 只删除本地索引和登记，不会删除 Agent 原始配置目录。

关键目录：

```text
src/
├── app/
├── features/
│   ├── agents/
│   ├── resources/
│   └── settings/
└── shared/
    ├── api/
    ├── components/
    ├── i18n/
    ├── lib/
    └── styles/

src-tauri/src/
├── commands/
├── application/
├── domain/
├── adapters/
├── infrastructure/
└── error.rs
```

数据流与职责边界详见 [完整实施方案](docs/implementation-plan.md)，UI 规范见 [设计系统](docs/design-system.md)。

## Adapter 扫描范围

- Claude Code：`~/.claude`、`CLAUDE_CONFIG_DIR`，以及 PATH/npm/默认安装位置中的 `claude`
- Codex：`~/.codex`、`CODEX_HOME`，以及 PATH/npm/默认安装位置中的 `codex`
- Hermes：`~/.hermes`、`HERMES_HOME` / `HERMES_CONFIG_DIR`，以及 PATH/npm/默认安装位置中的 `hermes`
- Kimi Code：`~/.kimi-code`、`KIMI_HOME` / `KIMI_CONFIG_DIR`，以及 PATH 或 `~/.kimi-code/bin` 中的 `kimi`

扫描只进入每个 Adapter 声明的资源目录，最大深度为 3、单实例最多 250 个资源；不会递归扫描整个用户主目录。
