import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { AgentSummary } from "../../shared/api/types";
import { AgentList } from "./AgentList";

const agents: AgentSummary[] = [
  {
    id: "agent-1",
    agentTypeId: "codex",
    agentTypeName: "Codex",
    displayName: "Codex",
    runtime: {
      commandName: "codex",
      executablePath: "C:\\bin\\codex.exe",
      installed: true,
      version: "1.2.3",
      versionProbeStatus: "detected",
      versionProbeError: null,
      resolutionSource: "path",
      distribution: "native",
    },
    configuration: {
      rootPath: "C:\\Users\\demo\\.codex",
      configFiles: ["C:\\Users\\demo\\.codex\\config.toml"],
      exists: true,
      readable: true,
      valid: true,
      detectionSource: "default",
      resourceCount: 3,
      manuallyAdded: false,
    },
    health: "healthy",
    confidence: "high",
    lastSeenAt: 1_700_000_000,
  },
];

describe("AgentList", () => {
  it("展示 Agent 状态并保持选择交互", async () => {
    const onSelect = vi.fn();
    render(
      <AgentList
        agents={agents}
        selectedId={null}
        search=""
        status="all"
        isLoading={false}
        onSearchChange={vi.fn()}
        onStatusChange={vi.fn()}
        onSelect={onSelect}
        onAddManual={vi.fn()}
        onClearFilters={vi.fn()}
      />,
    );

    expect(screen.getAllByText("健康")).toHaveLength(2);
    expect(screen.getByText("1 种 Agent · 1 个本地实例")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Codex/ }));
    expect(onSelect).toHaveBeenCalledWith("agent-1");
  });

  it("筛选为空时提供恢复动作", () => {
    render(
      <AgentList
        agents={[]}
        selectedId={null}
        search="missing"
        status="all"
        isLoading={false}
        onSearchChange={vi.fn()}
        onStatusChange={vi.fn()}
        onSelect={vi.fn()}
        onAddManual={vi.fn()}
        onClearFilters={vi.fn()}
      />,
    );

    expect(screen.getByText("没有符合条件的 Agent")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "清除筛选" })).toBeEnabled();
  });
});
