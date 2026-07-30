import { invoke } from "@tauri-apps/api/core";
import type {
  AgentFilter,
  AgentOverview,
  AgentSummary,
  DiscoveryEvidence,
  DiscoveryResult,
  ManualLocationRequest,
  CreateQuickLocationRequest,
  QuickLocation,
  Resource,
  ResourceFilter,
  UpdateQuickLocationRequest,
} from "./types";

type Invoke = <T>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

const desktopInvoke: Invoke = (command, args) => invoke(command, args);

function requireDesktop<T>(operation: string): Promise<T> {
  return Promise.reject({
    code: "desktop_runtime_required",
    message: `${operation}需要在 Agent Hub 桌面应用中运行。`,
    recoverable: true,
    suggestedAction: "请使用 npm run tauri dev 启动桌面应用。",
    contextId: "browser-preview",
  });
}

export function createAgentHubApi(call: Invoke = desktopInvoke) {
  const isDesktop = () => typeof window !== "undefined" && !!window.__TAURI_INTERNALS__;
  const read = <T>(command: string, args?: Record<string, unknown>) =>
    isDesktop() ? call<T>(command, args) : Promise.resolve([] as T);
  const mutate = <T>(
    command: string,
    operation: string,
    args?: Record<string, unknown>,
  ) => (isDesktop() ? call<T>(command, args) : requireDesktop<T>(operation));

  return {
    listAgents: (filter?: AgentFilter) =>
      read<AgentSummary[]>("list_agents", { filter: filter ?? null }),
    getAgentOverview: (agentId: string) =>
      read<AgentOverview>("get_agent_overview", { agentId }),
    getAgentResources: (agentId: string) =>
      read<Resource[]>("get_agent_resources", { agentId }),
    listResources: (filter?: ResourceFilter) =>
      read<Resource[]>("list_resources", { filter: filter ?? null }),
    getDiscoveryEvidence: (agentId: string) =>
      read<DiscoveryEvidence[]>("get_discovery_evidence", { agentId }),
    discoverAgents: () =>
      mutate<DiscoveryResult>("discover_agents", "扫描本机 Agent"),
    rescanAgent: (agentId: string) =>
      mutate<DiscoveryResult>("rescan_agent", "重新扫描 Agent", { agentId }),
    addManualLocation: (request: ManualLocationRequest) =>
      mutate<AgentSummary>("add_manual_location", "手动添加 Agent", {
        request,
      }),
    removeManualAgent: (agentId: string) =>
      mutate<void>("remove_manual_agent", "从 Agent Hub 移除手动记录", {
        agentId,
      }),
    openAgentDirectory: (agentId: string) =>
      mutate<void>("open_agent_directory", "打开 Agent 目录", { agentId }),
    openResource: (resourceId: string) =>
      mutate<void>("open_resource", "打开资源", { resourceId }),
    listQuickLocations: () =>
      read<QuickLocation[]>("list_quick_locations"),
    createQuickLocation: (request: CreateQuickLocationRequest) =>
      mutate<QuickLocation>("create_quick_location", "绑定快捷目录", {
        request,
      }),
    updateQuickLocation: (request: UpdateQuickLocationRequest) =>
      mutate<QuickLocation>("update_quick_location", "更新快捷目录", {
        request,
      }),
    reorderQuickLocations: (ids: string[]) =>
      mutate<void>("reorder_quick_locations", "调整快捷目录顺序", {
        request: { ids },
      }),
    removeQuickLocation: (locationId: string) =>
      mutate<void>("remove_quick_location", "移除快捷目录", {
        locationId,
      }),
    openQuickLocation: (locationId: string) =>
      mutate<void>("open_quick_location", "打开快捷目录", {
        locationId,
      }),
  };
}

export const agentHubApi = createAgentHubApi();
