import { MagnifyingGlassIcon, PlusIcon } from "@phosphor-icons/react";
import type { AgentSummary, HealthStatus } from "../../shared/api/types";
import {
  copy,
  statusLabel,
  versionProbeStatusLabel,
} from "../../shared/i18n/zh-CN";
import { formatTimestamp } from "../../shared/lib/format";
import { StatusBadge } from "../../shared/components/StatusBadge";

interface AgentListProps {
  agents: AgentSummary[];
  selectedId: string | null;
  search: string;
  status: HealthStatus | "all";
  isLoading: boolean;
  onSearchChange: (search: string) => void;
  onStatusChange: (status: HealthStatus | "all") => void;
  onSelect: (id: string) => void;
  onAddManual: () => void;
  onClearFilters: () => void;
}

const statusOptions: Array<HealthStatus | "all"> = [
  "all",
  "healthy",
  "changed",
  "runtime_only",
  "config_only",
  "missing",
  "degraded",
  "disabled",
];

export function AgentList({
  agents,
  selectedId,
  search,
  status,
  isLoading,
  onSearchChange,
  onStatusChange,
  onSelect,
  onAddManual,
  onClearFilters,
}: AgentListProps) {
  const productCount = new Set(agents.map((agent) => agent.agentTypeId)).size;
  const hasFilters = !!search || status !== "all";

  return (
    <section className="agent-list-panel" aria-label="Agent 清单">
      <div className="agent-list-tools">
        <label className="search-field">
          <span className="sr-only">{copy.agents.searchPlaceholder}</span>
          <MagnifyingGlassIcon size={17} />
          <input
            value={search}
            placeholder={copy.agents.searchPlaceholder}
            onChange={(event) => onSearchChange(event.target.value)}
          />
        </label>

        <label className="select-field">
          <span className="sr-only">按状态筛选</span>
          <select
            value={status}
            onChange={(event) =>
              onStatusChange(event.target.value as HealthStatus | "all")
            }
          >
            {statusOptions.map((option) => (
              <option key={option} value={option}>
                {option === "all"
                  ? copy.agents.allStatuses
                  : statusLabel[option]}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="list-summary">
        <span>{copy.agents.count(productCount, agents.length)}</span>
        <button type="button" className="text-button" onClick={onAddManual}>
          <PlusIcon size={14} weight="bold" />
          {copy.actions.addManual}
        </button>
      </div>

      <div className="agent-rows" aria-live="polite">
        {isLoading &&
          Array.from({ length: 4 }, (_, index) => (
            <div className="agent-row-skeleton" key={index}>
              <div className="skeleton skeleton-line" />
              <div className="skeleton skeleton-line short" />
            </div>
          ))}

        {!isLoading &&
          agents.map((agent) => (
            <button
              type="button"
              key={agent.id}
              className={`agent-row${selectedId === agent.id ? " selected" : ""}`}
              aria-pressed={selectedId === agent.id}
              onClick={() => onSelect(agent.id)}
            >
              <span className="agent-row-top">
                <strong>{agent.displayName}</strong>
                <StatusBadge status={agent.health} />
              </span>
              <span className="agent-path mono">
                {agent.configuration.rootPath}
              </span>
              <span className="agent-row-meta">
                {agent.runtime.version ??
                  (agent.runtime.installed
                    ? versionProbeStatusLabel[agent.runtime.versionProbeStatus]
                    : copy.detail.notInstalled)}
                <span aria-hidden="true">·</span>
                {formatTimestamp(agent.lastSeenAt)}
              </span>
            </button>
          ))}

        {!isLoading && agents.length === 0 && (
          <div className="compact-empty">
            <strong>
              {hasFilters
                ? copy.agents.filteredEmptyTitle
                : copy.agents.emptyTitle}
            </strong>
            <p>
              {hasFilters
                ? copy.agents.filteredEmptyBody
                : copy.agents.emptyBody}
            </p>
            {hasFilters && (
              <button
                className="secondary-button"
                type="button"
                onClick={onClearFilters}
              >
                {copy.actions.clear}
              </button>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
