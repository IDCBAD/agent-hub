import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { QuickLocation } from "../../shared/api/types";
import { QuickLocationsPage } from "./QuickLocationsPage";

const locations: QuickLocation[] = [
  {
    id: "prompts",
    name: "常用 Prompts",
    path: "C:\\Users\\demo\\prompts",
    showInTray: true,
    sortOrder: 0,
    lastOpenedAt: null,
    createdAt: 1,
    updatedAt: 1,
  },
  {
    id: "projects",
    name: "Agent 项目",
    path: "C:\\Users\\demo\\projects",
    showInTray: false,
    sortOrder: 1,
    lastOpenedAt: 1_700_000_000,
    createdAt: 2,
    updatedAt: 2,
  },
];

describe("QuickLocationsPage", () => {
  it("展示目录并支持托盘、排序和打开操作", async () => {
    const user = userEvent.setup();
    const onToggleTray = vi.fn();
    const onMove = vi.fn();
    const onOpen = vi.fn();

    render(
      <QuickLocationsPage
        locations={locations}
        isLoading={false}
        isMutating={false}
        onAdd={vi.fn()}
        onEdit={vi.fn()}
        onOpen={onOpen}
        onRemove={vi.fn()}
        onToggleTray={onToggleTray}
        onMove={onMove}
      />,
    );

    expect(screen.getByText("常用 Prompts")).toBeInTheDocument();
    expect(screen.getByText("Agent 项目")).toBeInTheDocument();

    const toggles = screen.getAllByRole("checkbox");
    await user.click(toggles[1]);
    expect(onToggleTray).toHaveBeenCalledWith(locations[1]);

    await user.click(screen.getByRole("button", { name: "下移 常用 Prompts" }));
    expect(onMove).toHaveBeenCalledWith(0, 1);

    const openButtons = screen.getAllByRole("button", { name: "打开目录" });
    await user.click(openButtons[0]);
    expect(onOpen).toHaveBeenCalledWith("prompts");
  });
});
