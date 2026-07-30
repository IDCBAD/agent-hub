# Changelog

本项目遵循语义化版本号。发布日期使用北京时间。

## 0.1.2 - 2026-07-30

### Changed

- 更新 Agent Hub 品牌标识，并同步应用标题栏与全平台图标资产。
- Windows EXE、NSIS 安装器/卸载器、MSI 产品信息与快捷方式统一使用新版品牌图标。
- Windows 窗口持续失焦 15 秒后，WebView2 自动进入低内存模式；重新激活窗口时立即恢复正常内存目标。

## 0.1.1 - 2026-07-30

### Fixed

- Windows 启动和扫描时，CLI 版本探测子进程不再弹出可见的 CMD/控制台窗口。

## 0.1.0 - 2026-07-30

### Added

- Tauri 2、React 19、TypeScript 与 Rust 分层桌面应用。
- SQLite 本地数据初始化与 v1-v5 自动迁移。
- Claude Code、Codex、Hermes Agent、Kimi Code Adapter。
- Runtime、Configuration、Health Status 分离建模。
- Windows PATH、npm、Python、自带目录与默认路径中的 Runtime 发现。
- CLI 程序路径、命令、版本、安装来源和版本探测诊断。
- Agent 列表、搜索、状态筛选、详情与实例级统计。
- 配置文件、Prompt、Skill、MCP、身份和记忆资源的只读索引。
- 打开 Agent 目录与已验证资源。
- 手动添加配置路径及从 Hub 安全移除。
- Windows NSIS 与 MSI 安装包配置。

### Security

- Agent 原始配置保持只读。
- SQLite 不保存认证正文、Token 或 API Key 内容。
- 打开操作只接受数据库 ID，并在 Rust 层重新验证授权根目录。
- 敏感资源仅保存元数据，不向前端暴露文件正文。

### Known limitations

- 每个配置根目录作为独立 Agent 实例展示；同一产品可以出现多个实例。
- 资源目录索引最多展示 250 个后代条目，并以 `250+` 标记截断。
- 0.1.0 Windows 安装包默认未签名，公开分发时可能触发 SmartScreen 提示。
