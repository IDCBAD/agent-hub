# Agent Hub 完整实施方案

## 0. 文档状态

- 状态：MVP 已实现，进入本地验收
- 首批目标平台：Windows、macOS、Linux
- 首批 Agent：Claude Code、Codex、Hermes
- 产品形态：本地优先的 Tauri 桌面应用
- MVP 核心结果：为本机 Agent 建立一份持久化、可解释、可导航的资源地图

### 0.1 架构修订：Runtime 与 Configuration 分离

MVP v2 将 Agent Instance 拆为两个独立观测对象：

- Runtime：CLI 是否安装、命令名、可执行文件路径、CLI 自报版本、解析来源（PATH/默认路径）和安装方式（npm/Python/Agent 自带/原生/未知）。
- Configuration：配置根、配置文件、可读/有效状态和 Resource 列表。
- Health：由 Runtime 与 Configuration 综合计算为 `healthy`、`runtime_only`、`config_only`、`degraded`、`changed`、`missing` 或 `disabled`。

SQLite 使用 `agent_runtimes` 与 `agent_configurations` 一对一关联 `agent_instances`，`resources` 归属于 Configuration。旧 v1 扁平字段仅作为兼容迁移来源，不再作为查询事实来源。

手动路径属于用户登记意图，使用 `manual_agent_locations` 独立持久化。移除手动 Agent 只删除 Hub 数据库中的登记与级联索引，禁止删除 Agent 原始目录。

本文档用于指导从空仓库到 MVP 发布的完整实施过程。任何改变数据所有权、安全边界或 Agent 抽象的实现，都应先更新本文档或新增 ADR。

---

## 1. 产品目标与边界

### 1.1 用户问题

本地 AI Agent 的安装目录、身份说明、Prompt、Skill、MCP、配置和工作目录分散在不同位置。用户需要反复进入隐藏目录查找文件，无法形成完整的 Agent 视图，也无法判断资源之间的关系和当前状态。

### 1.2 产品定位

Agent Hub 是 Agent 的管理层，不是 Agent 的替代运行时。

它负责：

- 发现
- 建模
- 索引
- 导航
- 对账
- 后续的安全配置编排

它不负责：

- 替代 Agent 完成推理
- 接管模型请求
- 在 MVP 中修改 Agent 原生配置
- 上传本地配置或工作区内容
- 保存认证文件正文

### 1.3 MVP 用户闭环

```text
启动 Agent Hub
→ 自动扫描
→ 查看发现的 Agent
→ 选择一个 Agent
→ 理解其安装、配置与资源结构
→ 打开目标目录或资源
→ 外部修改后重新扫描并看到状态更新
```

如果这一闭环不能稳定工作，不进入 Prompt、Skill、MCP 的统一写入阶段。

---

## 2. 核心领域模型

### 2.1 五层抽象

| 概念 | 示例 | 生命周期 |
|---|---|---|
| Agent Type | Claude Code | 随 Adapter 发布 |
| Agent Instance | 本机 `~/.claude` 实例 | 发现或手动添加 |
| Agent Profile | 工作配置、个人配置 | Phase 3 |
| Resource | `settings.json`、Skill 目录 | 扫描与文件变化 |
| Workspace | 本地代码项目 | 用户添加或历史发现 |

UI 可以将 Agent Instance 简称为 Agent，但 Rust 与数据库不得混淆 Agent Type 和 Agent Instance。

### 2.2 Resource 与 Artifact 的区别

- **Resource**：Agent 当前磁盘上使用的文件或目录。
- **Artifact**：Agent Hub 管理的可复用 Prompt、Skill 或 MCP 逻辑资产。
- **Binding**：Artifact 部署到 Agent/Profile 的关系。

MVP 只实现 Resource。Artifact 与 Binding 在 Phase 2 引入。

### 2.3 数据所有权

采用双事实来源但职责不重叠的模型：

- SQLite 是管理信息的事实来源。
- Agent 原生文件是运行配置内容的事实来源。

SQLite 保存身份、关系、路径、哈希、扫描结果和用户元数据；不默认复制完整配置正文。

---

## 3. 总体架构

```text
React Presentation
        │
Typed IPC Client
        │
Tauri Commands
        │
Application Services
   ┌────┴─────────┐
Domain Model   Adapter Registry
   │                │
SQLite         Agent Adapters
                    │
          Filesystem / Process / OS
```

### 3.1 前端职责

- Agent 列表、搜索、筛选和选中状态
- 资源清单、发现依据和错误展示
- 发起扫描、打开目录、打开资源等用例
- 编辑名称、标签和备注
- 对高风险操作展示预览和确认

前端不解析真实配置，不拼接系统路径，不直接访问 SQLite，不执行任意命令。

### 3.2 Rust 职责

- 跨平台路径解析
- Agent 自动发现与验证
- 可执行文件定位和版本检测
- 资源盘点与结构解析
- 路径规范化、权限检查和符号链接处理
- SQLite 事务与 migration
- 系统文件管理器和编辑器集成
- 敏感信息识别和脱敏

### 3.3 Adapter 接口

每个 Agent Adapter 提供：

```text
descriptor()
candidate_locations(context)
detect(candidate)
inventory(instance)
inspect(resource)
capabilities()
```

返回值必须使用统一领域对象：

- `DetectionCandidate`
- `DetectionEvidence`
- `AgentInstanceDraft`
- `ResourceObservation`
- `CapabilitySet`

Adapter 可以包含 Agent 特有 metadata，但核心查询不能依赖任意 JSON 字段完成。

---

## 4. 推荐代码结构

```text
src/
├── app/
├── features/
│   ├── agents/
│   ├── discovery/
│   ├── resources/
│   ├── workspaces/
│   └── settings/
├── shared/
│   ├── api/
│   ├── components/
│   ├── schemas/
│   └── styles/
└── main.tsx

src-tauri/src/
├── commands/
├── application/
│   ├── discover_agents.rs
│   ├── get_agent_overview.rs
│   ├── inspect_resource.rs
│   └── open_resource.rs
├── domain/
│   ├── agent.rs
│   ├── resource.rs
│   ├── workspace.rs
│   └── discovery.rs
├── adapters/
│   ├── claude/
│   ├── codex/
│   └── hermes/
├── infrastructure/
│   ├── database/
│   ├── filesystem/
│   ├── process/
│   └── platform/
├── security/
└── error.rs
```

依赖方向必须由外向内：

```text
commands → application → domain
adapters → domain
infrastructure → domain ports
```

Domain 不依赖 Tauri、SQLite 或某个具体 Agent。

---

## 5. SQLite 数据模型

### 5.1 `agent_types`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PK | 稳定标识，如 `claude-code` |
| display_name | TEXT | 展示名 |
| adapter_version | INTEGER | Adapter 数据协议版本 |
| icon_key | TEXT | 图标标识 |
| capabilities_json | TEXT | 能力快照 |
| created_at | INTEGER | 创建时间 |
| updated_at | INTEGER | 更新时间 |

### 5.2 `agent_instances`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PK | UUID |
| agent_type_id | TEXT FK | Agent Type |
| display_name | TEXT | 用户可修改 |
| executable_path | TEXT NULL | 规范化路径 |
| config_root | TEXT | 规范化路径 |
| detected_version | TEXT NULL | 版本 |
| discovery_source | TEXT | default/env/path/manual |
| status | TEXT | ready/config_only/changed/missing/invalid/disabled |
| confidence | TEXT | high/medium/low |
| metadata_json | TEXT | Adapter 扩展字段 |
| last_seen_at | INTEGER | 最近发现 |
| created_at | INTEGER | 创建时间 |
| updated_at | INTEGER | 更新时间 |

唯一约束建议使用：

```text
(agent_type_id, normalized_config_root)
```

不能只按 Agent Type 去重，因为未来可能有多个 Instance。

### 5.3 `resources`

| 字段 | 类型 | 说明 |
|---|---|---|
| id | TEXT PK | UUID |
| agent_instance_id | TEXT FK | 所属 Agent |
| kind | TEXT | config/prompt/skill/mcp/identity/memory/other |
| logical_key | TEXT | Adapter 内稳定键 |
| path | TEXT | 原始展示路径 |
| normalized_path | TEXT | 比较和安全校验路径 |
| format | TEXT | json/jsonc/toml/yaml/markdown/directory |
| scope | TEXT | global/workspace/profile |
| is_sensitive | INTEGER | 敏感标记 |
| exists_flag | INTEGER | 是否存在 |
| writable_flag | INTEGER | 是否可写 |
| content_hash | TEXT NULL | SHA-256 |
| modified_at | INTEGER NULL | 文件时间 |
| size_bytes | INTEGER NULL | 文件大小 |
| structure_json | TEXT NULL | 脱敏结构摘要 |
| last_observed_at | INTEGER | 最近观察 |

唯一约束：

```text
(agent_instance_id, logical_key, normalized_path)
```

### 5.4 `workspaces`

保存名称、规范化路径、状态、Git 元数据和最后使用时间。

### 5.5 `agent_workspace_links`

保存 Agent 与 Workspace 的多对多关系、角色和默认标记。

### 5.6 `discovery_runs`

保存扫描开始与结束时间、状态、发现数量、Adapter 版本和错误摘要。

### 5.7 `discovery_evidence`

| 字段 | 说明 |
|---|---|
| agent_instance_id | Agent |
| evidence_type | executable/config_root/signature/env/manual |
| source | PATH、环境变量名或默认规则 |
| observed_value | 脱敏后的值 |
| success | 是否验证成功 |
| message | 用户可理解的解释 |
| observed_at | 观察时间 |

### 5.8 `app_settings`

Key-value 仅用于非敏感设备设置。密钥和 Token 不进入该表。

### 5.9 数据库规则

- 使用 `rusqlite` bundled
- 开启 foreign keys
- 设置 busy timeout
- migration 使用单调版本号
- 扫描对账使用事务
- 删除 Agent 默认软删除或状态标记
- 数据库备份不包含 Agent 原始文件

---

## 6. 自动发现与对账

### 6.1 发现来源优先级

1. 用户配置的覆盖路径
2. Agent 专用环境变量
3. 平台默认配置目录
4. PATH 中的可执行文件
5. 常见包管理器路径
6. 手动添加
7. WSL 和远程位置，MVP 后半程评估

不要递归扫描整个用户主目录。

### 6.2 检测步骤

```text
生成候选路径
→ 规范化
→ 检查目录存在性
→ 查找 Agent 特征文件
→ 定位可执行文件
→ 获取版本
→ 生成证据和置信度
→ 盘点 Resources
→ 与数据库对账
```

### 6.3 对账规则

- 新发现：创建 Instance。
- 路径及类型相同：更新观察信息，不覆盖用户名称和标签。
- 资源哈希变化：状态设为 changed。
- 已记录路径消失：状态设为 missing，不立即删除。
- 只找到配置：保留为 config_only。
- 疑似路径迁移：给出合并建议，不自动合并。
- Adapter 升级：重新扫描其管理的 Instance。

### 6.4 扫描性能

- 扫描目录使用白名单和最大深度。
- 文件结构解析设置大小上限。
- 哈希只对已识别且大小合理的文件执行。
- 慢速版本检测设置超时。
- 扫描在后台任务执行，通过事件报告进度。

---

## 7. IPC 合约

MVP 命令：

```text
discover_agents(options)
list_agents(filter)
get_agent_overview(agent_id)
get_agent_resources(agent_id)
get_discovery_evidence(agent_id)
inspect_resource(resource_id)
open_agent_directory(agent_id)
open_resource(resource_id)
update_agent_metadata(agent_id, patch)
rescan_agent(agent_id)
add_manual_location(request)
```

禁止暴露：

```text
read_file(path)
write_file(path, content)
execute(command)
```

打开操作必须接收数据库 ID，由 Rust 查询并重新验证路径。

错误统一包含：

```text
code
message
recoverable
suggested_action
context_id
```

前端显示用户消息，详细错误写入已脱敏日志。

---

## 8. 安全与隐私

### 8.1 路径安全

- 所有路径在 Rust 层规范化。
- 验证目标是否属于已确认的 Agent 根目录或手动授权目录。
- 符号链接分别记录展示路径和 canonical path。
- 拒绝 `..` 路径穿越。
- 不通过 Shell 字符串打开文件。

### 8.2 敏感数据

敏感资源包括但不限于：

- `auth.json`
- `.env`
- API Key 字段
- Token、Cookie、Authorization
- 私钥和证书

MVP 默认行为：

- 显示文件存在和结构
- 不显示敏感值
- 不将正文写入 SQLite
- 日志只保存字段名和错误类型

### 8.3 Tauri 权限

- 使用最小 capability
- 不允许前端任意 filesystem scope
- 不允许任意 shell execution
- 系统打开操作封装为窄命令

### 8.4 未来写入协议

任何配置写入必须执行：

```text
读取当前内容和哈希
→ 解析
→ 生成变更计划
→ 用户确认
→ 创建备份
→ 写入临时文件
→ flush / sync
→ 原子替换
→ 重新解析
→ 更新数据库
→ 失败时回滚
```

---

## 9. 前端信息架构

所有界面遵循 [`docs/design-system.md`](design-system.md)：

- Notion 风格的暖纸背景、白色表面和单一蓝色结构强调色
- `NotionInter / Inter / 系统无衬线` 字体体系
- 默认简体中文
- Agent、Skill、MCP、SQLite、PATH 等必要技术名词可以保留英文
- 内部状态码与用户展示文案分离，为后续多语言预留 i18n key

### 9.1 MVP 导航

```text
Agent 管理
资源索引
设置
```

Workspaces 可以显示为预览入口，但不在第一里程碑实现完整管理。

### 9.2 Agent 管理工作台

左侧：

- 搜索
- 状态筛选
- Agent 列表
- 手动添加

右侧：

- Agent 状态与操作
- 资源关系
- 资源清单
- 发现依据
- 基本信息

### 9.3 资源索引

提供跨 Agent 的只读资源索引：

- 按 Agent、Kind、Scope、状态筛选
- 显示路径、修改时间、敏感标记
- 打开资源

### 9.4 设置

- 扫描路径
- 忽略规则
- 默认编辑器
- 扫描超时
- 数据目录
- 诊断日志入口

---

## 10. 测试策略

### 10.1 Domain 单元测试

- 状态转换
- 置信度计算
- 资源分类
- 对账决策

### 10.2 Adapter fixture 测试

为每个 Agent 保存无敏感信息的目录 fixture：

```text
missing
config-only
valid
invalid-format
custom-home
external-change
```

测试只使用临时目录，不读取开发者真实主目录。

### 10.3 数据库测试

- 空库初始化
- 每个 migration
- 重复扫描幂等
- 事务回滚
- 外键约束

### 10.4 IPC 测试

- 非法 ID
- 已删除资源
- 路径越界
- 不可访问目录
- 敏感数据脱敏

### 10.5 前端测试

- 列表和状态筛选
- Agent 选择保持
- 扫描加载和错误状态
- 空状态
- 键盘操作

### 10.6 跨平台验证

- Windows 原生路径
- macOS/Linux home
- 大小写差异
- 空格与非 ASCII 路径
- 符号链接
- Windows WSL 路径作为扩展测试

---

## 11. 里程碑与实施顺序

## Milestone 0：架构基线

交付：

- 产品边界
- 领域词汇表
- Adapter 接口草案
- 数据模型
- IPC 安全边界
- 可交互原型

退出条件：

- Agent Type 与 Instance 不再混淆
- 数据所有权确定
- MVP 明确为只读

## Milestone 1：工程骨架

预计 4–5 个工作日。

工作：

1. 创建 Tauri 2 + React + TypeScript 工程。
2. 配置格式化、lint、typecheck 和单元测试。
3. 建立 Rust 分层目录。
4. 初始化 SQLite 和 migration。
5. 定义共享 DTO 与错误结构。
6. 完成空状态 UI。

退出条件：

- 三个平台至少能构建
- 数据库可以创建、迁移和重新打开
- 前端能通过 typed IPC 获取空 Agent 列表

## Milestone 2：发现系统

预计 5–7 个工作日。

工作：

1. 实现 Adapter Registry。
2. 实现 Claude Code Adapter。
3. 实现 Codex Adapter。
4. 实现 Hermes Adapter。
5. 实现扫描任务和进度事件。
6. 实现手动路径。
7. 实现发现证据。
8. 实现对账事务。

退出条件：

- 能发现默认路径与环境变量覆盖路径
- 重复扫描幂等
- 配置存在但无可执行文件时正确显示 config_only
- 路径消失后显示 missing

## Milestone 3：资源地图

预计 4–5 个工作日。

工作：

1. Resource inventory。
2. JSON/JSONC/TOML/YAML/Markdown 格式识别。
3. 脱敏结构摘要。
4. 资源关系与资源表。
5. 打开目录和资源。
6. Agent 元数据编辑。

退出条件：

- 用户能从 Agent 进入任何已识别的非敏感资源
- 敏感配置不会显示正文
- 外部修改会在重新扫描后呈现 changed

## Milestone 4：可靠性与 MVP 发布

预计 4–6 个工作日。

工作：

1. Adapter fixture 和数据库 migration 测试。
2. 路径安全测试。
3. 空状态、错误和恢复动作。
4. Windows/macOS/Linux 构建。
5. 安装包和升级策略。
6. 诊断导出。
7. MVP 使用说明。

退出条件：

- MVP 验收标准全部通过
- 没有任意文件访问或任意命令执行 IPC
- 日志抽查不包含敏感信息

总计：单人约 3–4 周。

---

## 12. MVP 验收标准

- 自动发现 Claude Code、Codex 和 Hermes。
- 自动发现失败时允许手动添加。
- 重启应用后 Agent 和用户元数据仍然存在。
- 展示配置根、可执行文件、版本、状态和发现证据。
- 展示配置、Prompt、Skill、MCP 等 Resource。
- 正确打开 Agent 目录和已验证资源。
- 文件移动、删除或外部修改后状态可被重新扫描更新。
- 数据库和日志不保存密钥、Token 或认证正文。
- MVP 不修改 Agent 原生文件。
- 20 个 Agent 时列表仍可快速搜索和浏览。

---

## 13. 后续演进

### Phase 2：Artifact Library

- Prompt、Skill、MCP 统一资产库
- 版本与来源
- 导入现有 Resource
- Artifact Binding
- copy、symlink、generated 部署模式

### Phase 3：Profile

- 身份、Prompt、Skill、MCP 和环境组合
- 变更计划
- 备份、原子应用、验证和回滚
- 激活历史

### Phase 4：Runtime Observability

- 进程状态
- PID、版本、启动时间
- 当前工作目录
- Session 索引
- 文件系统 watcher
- 系统托盘

### Phase 5：Workflow

- 选择 Workspace
- 应用 Profile
- 启动 Agent
- 收集退出状态和产物
- 声明式步骤与白名单操作

Agent Hub 不统一不同 Agent 的内部推理协议，工作流只编排可验证的本地操作。

---

## 14. 决策门与需要确认的事项

以下问题不阻塞 MVP 架构，但应在对应阶段前确认：

| 时间点 | 决策 |
|---|---|
| Milestone 1 前 | 开源许可证 |
| Milestone 1 前 | 最低 Windows/macOS/Linux 版本 |
| Milestone 2 前 | WSL 是否纳入 MVP |
| Milestone 3 前 | 默认编辑器策略 |
| Milestone 4 前 | 自动更新与代码签名方案 |
| Phase 2 前 | Hub 管理资源的默认部署方式 |
| Phase 3 前 | Profile 切换是否允许修改认证配置 |

建议默认：

- MVP 不含 WSL 深度扫描，但允许手动添加 WSL 路径。
- 默认使用系统关联程序打开文件，同时允许用户选择编辑器。
- 认证配置不进入第一版 Profile。

---

## 15. 首次发布后的工程节奏

每个里程碑采用：

```text
理解
→ ADR / Issue
→ 小范围实现
→ Fixture 与自动化测试
→ 原型或截图验收
→ 合并
→ 更新实施文档
```

Pull Request 必须说明：

- 改了什么
- 为什么
- 数据流和边界如何工作
- 安全影响
- 验证方式
- 是否改变 Adapter 或数据库协议
