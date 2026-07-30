# Agent Hub v0.1.2 发布包

## 发布摘要

- Windows WebView2 后台内存优化。
- Agent Hub 窗口持续失焦 15 秒后，将 WebView2 内存目标切换为 `Low`。
- 窗口重新聚焦时立即恢复 `Normal`。
- 短暂失焦会取消过期的延迟任务，避免打开系统对话框或快速切换窗口时反复回收内存。
- 同步发布新版 Agent Hub 品牌标识，并覆盖 EXE、NSIS 安装器/卸载器、MSI 产品信息、Windows 快捷方式、Web favicon 和全平台图标。

## 运行机制

该策略使用 WebView2 `MemoryUsageTargetLevel`，不会暂停 JavaScript、重新加载页面或清除 React 状态。Agent Runtime Detection、Resource 扫描和 SQLite 数据处理继续由 Rust 后端正常执行。

Windows 之外的平台使用空实现，不引入 WebView2 COM 依赖。

## 验证结果

- 前端：TypeScript、ESLint、5 个单元测试和 Vite 生产构建通过。
- Rust：格式、全目标检查、21 个单元测试和 Clippy 通过。
- Windows Release、NSIS 和 MSI 构建成功。
- Release EXE、NSIS 安装器和 MSI 安装后程序提取出的图标像素一致，均为新版 Agent Hub 品牌标识。
- 失焦 10 秒时尚未触发 Low，符合 15 秒延迟策略。
- 失焦 18 秒后，WebView2 工作集从约 440.7 MB 降至 338.1 MB，下降约 102.6 MB（23.3%）。
- 重新聚焦后约 75 ms 内可重新获取完整 UI 并定位“扫描本机”按钮。

内存回收是 WebView2 的 best-effort 行为，实际数值会受窗口尺寸、DPI、GPU 驱动和系统内存压力影响。

## 升级与回滚

- 数据迁移：无。
- 可直接覆盖安装 v0.1.1。
- 如遇兼容性问题，可回滚 v0.1.1；Agent 配置和 Hub SQLite 数据无需迁移。

## 发布条件

发布决策：`GO WITH CONDITIONS`

- 本地安装与功能验收条件已满足。
- 安装包尚未进行 Windows Authenticode 签名；公开分发前建议补充可信代码签名。
- SHA-256 校验值见 `docs/checksums-v0.1.2.sha256`。
