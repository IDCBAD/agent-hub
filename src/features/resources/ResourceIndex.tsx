import {
  ArrowSquareOutIcon,
  MagnifyingGlassIcon,
  ShieldWarningIcon,
} from "@phosphor-icons/react";
import { useMemo, useState } from "react";
import type { Resource, ResourceKind } from "../../shared/api/types";
import {
  copy,
  resourceKindLabel,
  resourceScopeLabel,
} from "../../shared/i18n/zh-CN";
import { basename, formatTimestamp } from "../../shared/lib/format";

interface ResourceIndexProps {
  resources: Resource[];
  isLoading: boolean;
  onOpenResource: (resourceId: string) => void;
}

export function ResourceIndex({
  resources,
  isLoading,
  onOpenResource,
}: ResourceIndexProps) {
  const [search, setSearch] = useState("");
  const [kind, setKind] = useState<ResourceKind | "all">("all");

  const filtered = useMemo(() => {
    const term = search.trim().toLocaleLowerCase();
    return resources.filter(
      (resource) =>
        (kind === "all" || resource.kind === kind) &&
        (!term ||
          resource.path.toLocaleLowerCase().includes(term) ||
          resource.agentDisplayName.toLocaleLowerCase().includes(term)),
    );
  }, [kind, resources, search]);

  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>{copy.resources.title}</h1>
          <p>{copy.resources.subtitle}</p>
        </div>
      </header>

      <div className="resource-index-card">
        <div className="index-toolbar">
          <label className="search-field">
            <span className="sr-only">{copy.resources.searchPlaceholder}</span>
            <MagnifyingGlassIcon size={17} />
            <input
              value={search}
              placeholder={copy.resources.searchPlaceholder}
              onChange={(event) => setSearch(event.target.value)}
            />
          </label>
          <label className="select-field">
            <span className="sr-only">按资源类型筛选</span>
            <select
              value={kind}
              onChange={(event) =>
                setKind(event.target.value as ResourceKind | "all")
              }
            >
              <option value="all">{copy.resources.allKinds}</option>
              {Object.entries(resourceKindLabel).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
        </div>

        {isLoading ? (
          <div className="table-loading" role="status">
            {copy.common.loading}
          </div>
        ) : filtered.length === 0 ? (
          <div className="page-empty">
            <strong>{copy.resources.emptyTitle}</strong>
            <p>{copy.resources.emptyBody}</p>
          </div>
        ) : (
          <div className="resource-table-wrap">
            <table className="resource-table index-table">
              <thead>
                <tr>
                  <th>{copy.resources.resource}</th>
                  <th>{copy.resources.agent}</th>
                  <th>{copy.resources.kind}</th>
                  <th>{copy.resources.scope}</th>
                  <th>{copy.resources.modified}</th>
                  <th>
                    <span className="sr-only">{copy.resources.action}</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((resource) => (
                  <tr key={resource.id}>
                    <td>
                      <div className="resource-name">
                        <span className="mono">{basename(resource.path)}</span>
                        {resource.isSensitive && (
                          <ShieldWarningIcon
                            size={14}
                            aria-label={copy.resources.sensitive}
                          />
                        )}
                      </div>
                      <span className="resource-path mono">{resource.path}</span>
                    </td>
                    <td>{resource.agentDisplayName}</td>
                    <td>{resourceKindLabel[resource.kind]}</td>
                    <td>{resourceScopeLabel[resource.scope]}</td>
                    <td>{formatTimestamp(resource.modifiedAt)}</td>
                    <td>
                      <button
                        className="icon-button"
                        type="button"
                        aria-label={`打开 ${basename(resource.path)}`}
                        onClick={() => onOpenResource(resource.id)}
                      >
                        <ArrowSquareOutIcon size={17} />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
