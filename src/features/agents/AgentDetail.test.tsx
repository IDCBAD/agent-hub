import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type {
  AgentOverview,
  Resource,
} from "../../shared/api/types";
import { AgentDetail } from "./AgentDetail";

const agent: AgentOverview = {
  id: "agent-1",
  agentTypeId: "claude-code",
  agentTypeName: "Claude Code",
  displayName: "Claude Code",
  runtime: {
    commandName: "claude",
    executablePath: "C:\\Users\\demo\\AppData\\Roaming\\npm\\claude.cmd",
    installed: true,
    version: null,
    versionProbeStatus: "failed",
    versionProbeError: "无法启动版本命令。",
    resolutionSource: "path",
    distribution: "npm",
  },
  configuration: {
    rootPath: "C:\\Users\\demo\\.claude",
    configFiles: ["C:\\Users\\demo\\.claude\\settings.json"],
    exists: true,
    readable: true,
    valid: true,
    detectionSource: "default",
    resourceCount: 1,
    manuallyAdded: true,
  },
  health: "healthy",
  confidence: "high",
  lastSeenAt: 1_700_000_000,
  adapterVersion: 3,
  metadata: {},
};

const directoryResource: Resource = {
  id: "resource-1",
  agentInstanceId: "agent-1",
  agentDisplayName: "Claude Code",
  kind: "skill",
  logicalKey: "skills",
  path: "C:\\Users\\demo\\.claude\\skills",
  format: "directory",
  scope: "global",
  isSensitive: false,
  exists: true,
  writable: true,
  contentHash: "fingerprint",
  modifiedAt: 1_700_000_000,
  sizeBytes: null,
  entryCount: 250,
  scanTruncated: true,
};

const defaultProps = {
  resources: [directoryResource],
  evidence: [],
  isLoading: false,
  isRescanning: false,
  isRemoving: false,
  onOpenDirectory: vi.fn(),
  onOpenResource: vi.fn(),
  onRescan: vi.fn(),
  onRemove: vi.fn(),
};

describe("AgentDetail", () => {
  it("区分程序已定位与版本探测失败，并聚合目录资源", () => {
    render(<AgentDetail {...defaultProps} agent={agent} />);

    expect(screen.getAllByText("版本探测失败")).toHaveLength(1);
    expect(
      screen.getByText("版本探测失败：无法启动版本命令。"),
    ).toBeInTheDocument();
    expect(screen.getAllByText(agent.runtime.executablePath!)).toHaveLength(2);
    expect(screen.getByText("npm")).toBeInTheDocument();
    expect(screen.getByText("claude")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "从 Hub 移除" }),
    ).toBeEnabled();
    expect(screen.getByText("250+ 项")).toBeInTheDocument();
    expect(screen.queryByText(/\\\\\?\\/)).not.toBeInTheDocument();
  });

  it("程序未定位时展示可执行的原因提示", () => {
    render(
      <AgentDetail
        {...defaultProps}
        agent={{
          ...agent,
          runtime: {
            commandName: "claude",
            executablePath: null,
            installed: false,
            version: null,
            versionProbeStatus: "not_attempted",
            versionProbeError: null,
            resolutionSource: "not_found",
            distribution: "unknown",
          },
          health: "config_only",
        }}
      />,
    );

    expect(screen.getByText("未找到命令行程序")).toBeInTheDocument();
    expect(
      screen.getByText("已检查 PATH 和该 Agent 的默认程序安装位置。"),
    ).toBeInTheDocument();
  });
});
