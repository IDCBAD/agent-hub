import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { AgentOverview } from "../../shared/api/types";
import { RemoveAgentDialog } from "./RemoveAgentDialog";

const agent: AgentOverview = {
  id: "manual-agent",
  agentTypeId: "kimi-cli",
  agentTypeName: "Kimi Code",
  displayName: "Kimi Code",
  runtime: {
    commandName: "kimi",
    executablePath: "C:\\Users\\demo\\.kimi-code\\bin\\kimi.exe",
    installed: true,
    version: "0.28.1",
    versionProbeStatus: "detected",
    versionProbeError: null,
    resolutionSource: "path",
    distribution: "bundled",
  },
  configuration: {
    rootPath: "C:\\Users\\demo\\.kimi-code",
    configFiles: ["C:\\Users\\demo\\.kimi-code\\config.toml"],
    exists: true,
    readable: true,
    valid: true,
    detectionSource: "manual",
    resourceCount: 1,
    manuallyAdded: true,
  },
  health: "healthy",
  confidence: "high",
  lastSeenAt: 1_700_000_000,
  adapterVersion: 4,
  metadata: {},
};

describe("RemoveAgentDialog", () => {
  it("明确只移除 Hub 数据并执行确认回调", async () => {
    const onConfirm = vi.fn();
    render(
      <RemoveAgentDialog
        agent={agent}
        open
        isRemoving={false}
        onClose={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    expect(
      screen.getByText("本机配置目录和其中的任何文件都不会被删除。"),
    ).toBeInTheDocument();
    await userEvent.click(
      screen.getByRole("button", { name: "从 Hub 移除" }),
    );
    expect(onConfirm).toHaveBeenCalledOnce();
  });
});
