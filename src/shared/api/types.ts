export type HealthStatus =
  | "healthy"
  | "runtime_only"
  | "config_only"
  | "changed"
  | "missing"
  | "degraded"
  | "disabled";

export type RuntimeResolutionSource =
  | "path"
  | "default_path"
  | "not_found";

export type RuntimeDistribution =
  | "npm"
  | "python"
  | "bundled"
  | "native"
  | "unknown";

export type VersionProbeStatus =
  | "not_attempted"
  | "detected"
  | "failed"
  | "timed_out"
  | "unsupported";

export type Confidence = "high" | "medium" | "low";

export type ResourceKind =
  | "config"
  | "prompt"
  | "skill"
  | "mcp"
  | "identity"
  | "memory"
  | "other";

export type ResourceScope = "global" | "workspace" | "profile";

export interface AgentSummary {
  id: string;
  agentTypeId: string;
  agentTypeName: string;
  displayName: string;
  runtime: {
    commandName: string;
    executablePath: string | null;
    installed: boolean;
    version: string | null;
    versionProbeStatus: VersionProbeStatus;
    versionProbeError: string | null;
    resolutionSource: RuntimeResolutionSource;
    distribution: RuntimeDistribution;
  };
  configuration: {
    rootPath: string;
    configFiles: string[];
    exists: boolean;
    readable: boolean;
    valid: boolean;
    detectionSource: string;
    resourceCount: number;
    manuallyAdded: boolean;
  };
  health: HealthStatus;
  confidence: Confidence;
  lastSeenAt: number;
}

export interface AgentOverview extends AgentSummary {
  adapterVersion: number;
  metadata: Record<string, unknown>;
}

export interface Resource {
  id: string;
  agentInstanceId: string;
  agentDisplayName: string;
  kind: ResourceKind;
  logicalKey: string;
  path: string;
  format: string;
  scope: ResourceScope;
  isSensitive: boolean;
  exists: boolean;
  writable: boolean;
  contentHash: string | null;
  modifiedAt: number | null;
  sizeBytes: number | null;
  entryCount: number | null;
  scanTruncated: boolean;
}

export interface DiscoveryEvidence {
  id: string;
  agentInstanceId: string;
  evidenceType: string;
  source: string;
  observedValue: string;
  success: boolean;
  message: string;
  observedAt: number;
}

export interface DiscoveryResult {
  runId: string;
  discoveredCount: number;
  changedCount: number;
  missingCount: number;
  finishedAt: number;
}

export interface AgentFilter {
  search?: string;
  statuses?: HealthStatus[];
}

export interface ResourceFilter {
  search?: string;
  agentId?: string;
  kinds?: ResourceKind[];
}

export interface ManualLocationRequest {
  agentTypeId: string;
  path: string;
}

export interface QuickLocation {
  id: string;
  name: string;
  path: string;
  showInTray: boolean;
  sortOrder: number;
  lastOpenedAt: number | null;
  createdAt: number;
  updatedAt: number;
}

export interface CreateQuickLocationRequest {
  name: string;
  path: string;
  showInTray: boolean;
}

export interface UpdateQuickLocationRequest {
  id: string;
  name: string;
  showInTray: boolean;
}

export interface AppErrorShape {
  code: string;
  message: string;
  recoverable: boolean;
  suggestedAction: string | null;
  contextId: string;
}
