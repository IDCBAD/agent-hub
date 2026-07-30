import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { QuickLocationDialog } from "./QuickLocationDialog";

describe("QuickLocationDialog", () => {
  it("从所选目录推导名称并提交托盘设置", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    render(
      <QuickLocationDialog
        open
        selectedPath={"C:\\Users\\demo\\prompts"}
        isSubmitting={false}
        onClose={vi.fn()}
        onSubmit={onSubmit}
      />,
    );

    expect(screen.getByLabelText("显示名称")).toHaveValue("prompts");
    await user.clear(screen.getByLabelText("显示名称"));
    await user.type(screen.getByLabelText("显示名称"), "常用 Prompts");
    await user.click(screen.getByRole("button", { name: "完成绑定" }));

    expect(onSubmit).toHaveBeenCalledWith({
      name: "常用 Prompts",
      path: "C:\\Users\\demo\\prompts",
      showInTray: true,
    });
  });
});
