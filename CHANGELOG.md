# Changelog

本项目遵循语义化版本号。发布日期使用北京时间。

## 0.2.1 - 2026-07-30

### Added

- 设置页新增开机后台启动、关闭后托盘驻留和延迟自动扫描偏好。
- 新增本地数据目录入口、Agent 扫描索引安全重建与版本信息。
- 开机启动使用 `--background` 模式，不创建 WebView 主窗口。

### Changed

- 设置页从 MVP 静态说明升级为可持久化的最小偏好设置。
- 重建索引只清理可再生成的 Agent、Runtime、Configuration、Resource 与发现证据缓存，保留手动路径、快捷目录、设置和原始文件。

## 0.2.0 - 2026-07-30

### Added

- 新增快捷目录模块，可绑定、重命名、排序、打开和安全移除常用目录。
- 新增原生系统托盘菜单，可快速打开 Agent 配置目录和用户置顶目录。
- 新增单实例保护，后台驻留时再次启动会恢复现有主窗口。

### Changed

- 关闭主窗口后销毁 WebView2，仅保留轻量 Rust 托盘进程；从托盘或再次启动应用时按需重建窗口。
- 启动时只读取 SQLite 缓存，不再自动执行 CLI Runtime 与 Resource 扫描。
- 产品定位收敛为 Agent 配置目录与常用目录的快捷入口。

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
