import {
  FolderOpenIcon,
  GearSixIcon,
  RobotIcon,
  SquaresFourIcon,
} from "@phosphor-icons/react";
import { copy } from "../i18n/zh-CN";

export type AppSection = "agents" | "resources" | "settings";

interface AppShellProps {
  section: AppSection;
  agentCount: number;
  resourceCount: number;
  children: React.ReactNode;
  onSectionChange: (section: AppSection) => void;
}

const navigation = [
  { id: "agents" as const, label: copy.nav.agents, icon: RobotIcon },
  {
    id: "resources" as const,
    label: copy.nav.resources,
    icon: FolderOpenIcon,
  },
  { id: "settings" as const, label: copy.nav.settings, icon: GearSixIcon },
];

export function AppShell({
  section,
  agentCount,
  resourceCount,
  children,
  onSectionChange,
}: AppShellProps) {
  return (
    <div className="app-shell">
      <header className="titlebar">
        <div className="app-mark" aria-hidden="true">
          <SquaresFourIcon weight="fill" />
        </div>
        <strong>{copy.productName}</strong>
        <span>{copy.productCaption}</span>
      </header>

      <div className="app-body">
        <aside className="sidebar" aria-label="主导航">
          <div className="sidebar-brand">
            <span className="eyebrow">LOCAL FIRST</span>
            <p>本机 Agent 的只读资源地图</p>
          </div>

          <nav className="nav-list">
            {navigation.map((item) => {
              const Icon = item.icon;
              const count =
                item.id === "agents"
                  ? agentCount
                  : item.id === "resources"
                    ? resourceCount
                    : null;

              return (
                <button
                  key={item.id}
                  type="button"
                  className={section === item.id ? "active" : undefined}
                  aria-label={item.label}
                  aria-current={section === item.id ? "page" : undefined}
                  onClick={() => onSectionChange(item.id)}
                >
                  <Icon size={18} weight={section === item.id ? "fill" : "regular"} />
                  <span>{item.label}</span>
                  {count !== null && <span className="nav-count">{count}</span>}
                </button>
              );
            })}
          </nav>

          <div className="privacy-note">
            <strong>只读安全边界</strong>
            <p>不修改原生配置，不上传本地文件，不保存认证正文。</p>
            <div className="local-status">
              <span aria-hidden="true" />
              本地服务
            </div>
          </div>
        </aside>

        <main className="main-canvas">{children}</main>
      </div>
    </div>
  );
}
