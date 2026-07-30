import type {
  HealthStatus,
  Confidence,
  ResourceKind,
  ResourceScope,
  VersionProbeStatus,
} from "../api/types";

export const copy = {
  productName: "Agent Hub",
  productCaption: "本地 Agent 控制中心",
  nav: {
    agents: "Agent 管理",
    resources: "资源索引",
    settings: "设置",
  },
  actions: {
    scan: "扫描本机",
    scanning: "正在扫描",
    rescan: "重新扫描",
    openDirectory: "打开目录",
    openResource: "打开",
    addManual: "手动添加",
    cancel: "取消",
    add: "添加 Agent",
    remove: "从 Hub 移除",
    removing: "正在移除",
    retry: "重试",
    clear: "清除筛选",
  },
  agents: {
    title: "Agent 管理",
    subtitle: "发现、理解并导航本机 AI Agent 的配置与资源。",
    searchPlaceholder: "搜索名称或配置路径",
    allStatuses: "全部状态",
    count: (productCount: number, instanceCount: number) =>
      `${productCount} 种 Agent · ${instanceCount} 个本地实例`,
    lastScan: "最近扫描",
    neverScanned: "尚未扫描",
    emptyTitle: "还没有发现 Agent",
    emptyBody:
      "扫描默认位置与 PATH，或手动添加 Claude Code、Codex、Hermes Agent 或 Kimi Code 配置目录。",
    filteredEmptyTitle: "没有符合条件的 Agent",
    filteredEmptyBody: "调整搜索词或状态筛选后再试。",
    loading: "正在读取 Agent 清单",
    detailPlaceholder: "选择一个 Agent 查看资源与发现依据",
  },
  detail: {
    program: "本地程序",
    configRoot: "配置目录",
    resources: "已发现资源",
    resourceMap: "资源关系",
    resourceList: "资源清单",
    evidence: "发现依据",
    metadata: "基本信息",
    command: "启动命令",
    executable: "程序路径",
    version: "程序版本",
    runtime: "运行入口",
    runtimeType: "命令行（CLI）",
    runtimeSource: "程序发现来源",
    distribution: "安装方式",
    versionProbe: "版本探测",
    configurationSource: "配置发现来源",
    configFiles: "配置文件",
    configFileCount: (count: number) => `${count} 个配置文件`,
    installed: "已安装",
    notInstalled: "未安装",
    programLocated: "已定位，版本待确认",
    programMissing: "未找到命令行程序",
    programMissingHint: "已检查 PATH 和该 Agent 的默认程序安装位置。",
    source: "发现来源",
    confidence: "置信度",
    adapterVersion: "Adapter 版本",
    unavailable: "未发现",
    accessible: "可访问",
    indexed: (count: number) => `已识别 ${count} 组`,
    resourceSummary: (count: number) => `${count} 项顶层资源`,
    noResources: "这个 Agent 暂未发现可展示的资源。",
    noEvidence: "暂无发现依据。",
  },
  resources: {
    title: "资源索引",
    subtitle: "跨 Agent 查看已确认的本地资源；Agent Hub 不保存文件正文。",
    searchPlaceholder: "搜索文件名、路径或 Agent",
    allKinds: "全部类型",
    resource: "资源",
    agent: "所属 Agent",
    kind: "类型",
    scope: "范围",
    modified: "修改时间",
    contentSize: "内容 / 大小",
    action: "操作",
    sensitive: "含敏感信息",
    emptyTitle: "还没有可索引的资源",
    emptyBody: "先扫描 Agent，资源会按已知规则安全地加入索引。",
  },
  settings: {
    title: "设置",
    subtitle: "MVP 使用本地优先、只读的安全边界。",
    dataTitle: "本地数据",
    dataBody:
      "SQLite 仅保存 Agent 身份、路径、哈希与发现结果，不保存配置正文、密钥或 Token。",
    scanTitle: "扫描策略",
    scanBody:
      "只检查默认目录、环境变量、PATH 与手动授权路径，不递归扫描整个用户目录。",
    openTitle: "打开策略",
    openBody:
      "打开操作仅接受数据库资源 ID，并在 Rust 层再次验证路径。",
  },
  manual: {
    title: "手动添加 Agent",
    body: "选择 Agent 类型并填写其配置根目录。目录只会用于只读扫描。",
    typeLabel: "Agent 类型",
    pathLabel: "配置目录",
    pathPlaceholder: "例如 C:\\Users\\name\\.codex",
    pathHint: "支持绝对路径和 ~ 开头的用户目录路径。",
    pathRequired: "请输入配置目录。",
  },
  removeManual: {
    title: "从 Agent Hub 移除？",
    body: (name: string) =>
      `将移除 ${name} 的手动登记、资源索引和发现记录。`,
    safety: "本机配置目录和其中的任何文件都不会被删除。",
    rediscovery:
      "如果该路径同时属于默认扫描位置，后续扫描仍可能重新发现它。",
  },
  common: {
    errorTitle: "操作未完成",
    loading: "正在加载",
    directory: "目录",
    unknown: "未知",
    yes: "是",
    no: "否",
  },
} as const;

export const statusLabel: Record<HealthStatus, string> = {
  healthy: "健康",
  runtime_only: "仅发现程序",
  config_only: "仅发现配置",
  changed: "有变化",
  missing: "未发现",
  degraded: "状态异常",
  disabled: "已停用",
};

export const runtimeSourceLabel = {
  path: "PATH",
  default_path: "默认路径",
  not_found: "未发现",
} as const;

export const runtimeDistributionLabel = {
  npm: "npm",
  python: "Python",
  bundled: "Agent 自带安装目录",
  native: "原生可执行程序",
  unknown: "未知",
} as const;

export const versionProbeStatusLabel: Record<VersionProbeStatus, string> = {
  not_attempted: "未执行版本探测",
  detected: "已获取版本",
  failed: "版本探测失败",
  timed_out: "版本探测超时",
  unsupported: "不支持版本命令",
};

export const confidenceLabel: Record<Confidence, string> = {
  high: "高",
  medium: "中",
  low: "低",
};

export const resourceKindLabel: Record<ResourceKind, string> = {
  config: "配置",
  prompt: "Prompt",
  skill: "Skill",
  mcp: "MCP",
  identity: "身份",
  memory: "记忆",
  other: "其他",
};

export const resourceScopeLabel: Record<ResourceScope, string> = {
  global: "全局",
  workspace: "工作区",
  profile: "Profile",
};
