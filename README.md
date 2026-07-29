# Agent Hub

Agent Hub 是一个面向个人开发者的本地 AI Agent 控制中心。

它不会替代 Claude Code、Codex、Hermes 等 Agent，而是在它们之上建立统一的管理层：自动发现本地安装与配置目录，整理 Agent、资源和工作空间之间的关系，并提供可解释、可导航的本地资源地图。

## 当前阶段

项目处于架构与 MVP 设计阶段，首个版本聚焦：

- 自动发现 Claude Code、Codex 和 Hermes
- 管理 Agent 的名称、路径、标签和状态
- 展示 Agent 的配置结构与发现证据
- 快速打开配置目录和已识别资源
- 使用 SQLite 持久化 Agent 元数据与资源关系

首个版本不会修改 Agent 原生配置，也不会存储 API Key、Token 或认证文件正文。

## 核心模型

```text
Agent Type
  └── Agent Instance
        ├── Resources
        ├── Workspaces
        └── Discovery Evidence
```

- **Agent Type**：Claude Code、Codex、Hermes 等运行时类型
- **Agent Instance**：本机的一次实际安装或配置实例
- **Resource**：配置、Prompt、Skill、MCP、身份文件和目录
- **Workspace**：Agent 使用的本地项目目录

## 文档与原型

- [完整实施方案](docs/implementation-plan.md)
- [UI 设计规范](docs/design-system.md)
- [原型评审与设计决策](docs/prototype-review.md)
- [交互原型 v2](prototype/agent-hub-prototype-v2.html)

原始原型保留在用户本地，不纳入仓库；v2 是根据架构评审重新组织信息结构后的参考版本。

## 推荐技术栈

- React + TypeScript + Vite
- Tauri 2
- Rust
- SQLite / rusqlite
- TanStack Query
- Radix UI 或 shadcn/ui

## 计划中的仓库结构

```text
agent-hub/
├── docs/
├── prototype/
├── src/                    # React 前端
├── src-tauri/
│   └── src/
│       ├── commands/
│       ├── application/
│       ├── domain/
│       ├── adapters/
│       └── infrastructure/
└── tests/
```

实际工程代码将在 MVP Milestone 1 中初始化，避免在领域边界尚未稳定时提前生成一次性脚手架。

## 项目原则

1. SQLite 管理 Agent 元数据和资源关系，原生文件仍是 Agent 运行配置的事实来源。
2. 不向前端暴露任意文件读写或任意命令执行接口。
3. 新增 Agent 通过 Adapter 扩展，不在数据库中增加新的应用布尔字段。
4. 任何未来的配置写入都必须经过预览、备份、原子写入、验证和回滚。
5. 默认本地优先，不上传用户配置、工作区内容或认证信息。
