import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { AppInfo, AppSettings } from "../../shared/api/types";
import { SettingsPage } from "./SettingsPage";

const settings: AppSettings = {
  launchAtLogin: false,
  keepRunningInTray: true,
  scanOnLaunch: false,
};

const info: AppInfo = {
  version: "0.2.1",
  schemaVersion: 6,
  dataDirectory: "C:\\Users\\demo\\AppData\\Local\\Agent Hub",
};

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("SettingsPage", () => {
  it("展示持久化偏好并提交完整设置", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <SettingsPage
        settings={settings}
        info={info}
        isLoading={false}
        isSaving={false}
        isRebuilding={false}
        onChange={onChange}
        onOpenDataDirectory={vi.fn()}
        onRebuildIndex={vi.fn()}
        onOpenProjectPage={vi.fn()}
        onOpenReleasesPage={vi.fn()}
      />,
    );

    expect(screen.getByText("0.2.1")).toBeInTheDocument();
    expect(screen.getByText("SQLite Schema 6")).toBeInTheDocument();
    expect(
      screen.getByRole("switch", { name: "关闭窗口后继续在托盘运行" }),
    ).toHaveAttribute("aria-checked", "true");

    await user.click(
      screen.getByRole("switch", { name: "开机后自动启动 Agent Hub" }),
    );
    expect(onChange).toHaveBeenCalledWith({
      ...settings,
      launchAtLogin: true,
    });
  });

  it("执行目录、索引和外部页面动作", async () => {
    const user = userEvent.setup();
    const onOpenDataDirectory = vi.fn();
    const onRebuildIndex = vi.fn();
    const onOpenProjectPage = vi.fn();
    const onOpenReleasesPage = vi.fn();
    vi.spyOn(window, "confirm").mockReturnValue(true);

    render(
      <SettingsPage
        settings={settings}
        info={info}
        isLoading={false}
        isSaving={false}
        isRebuilding={false}
        onChange={vi.fn()}
        onOpenDataDirectory={onOpenDataDirectory}
        onRebuildIndex={onRebuildIndex}
        onOpenProjectPage={onOpenProjectPage}
        onOpenReleasesPage={onOpenReleasesPage}
      />,
    );

    await user.click(screen.getByRole("button", { name: "打开目录" }));
    await user.click(screen.getByRole("button", { name: "重建索引" }));
    await user.click(screen.getByRole("button", { name: "项目主页" }));
    await user.click(screen.getByRole("button", { name: "发布版本" }));

    expect(onOpenDataDirectory).toHaveBeenCalledOnce();
    expect(onRebuildIndex).toHaveBeenCalledOnce();
    expect(onOpenProjectPage).toHaveBeenCalledOnce();
    expect(onOpenReleasesPage).toHaveBeenCalledOnce();
  });
});
