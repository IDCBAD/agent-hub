import {
  ArrowClockwiseIcon,
  ArrowSquareOutIcon,
  CheckCircleIcon,
  FolderOpenIcon,
  HardDrivesIcon,
  ShieldWarningIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import type {
  AgentOverview,
  DiscoveryEvidence,
  Resource,
} from "../../shared/api/types";
import {
  confidenceLabel,
  copy,
  resourceKindLabel,
  resourceScopeLabel,
  runtimeDistributionLabel,
  runtimeSourceLabel,
  versionProbeStatusLabel,
} from "../../shared/i18n/zh-CN";
import {
  basename,
  formatBytes,
  formatTimestamp,
} from "../../shared/lib/format";
import { LoadingState } from "../../shared/components/LoadingState";
import { StatusBadge } from "../../shared/components/StatusBadge";

interface AgentDetailProps {
  agent: AgentOverview | undefined;
  resources: Resource[];
  evidence: DiscoveryEvidence[];
  isLoading: boolean;
  isRescanning: boolean;
  isRemoving: boolean;
  onOpenDirectory: () => void;
  onOpenResource: (resourceId: string) => void;
  onRescan: () => void;
  onRemove: () => void;
}

export function AgentDetail({
  agent,
  resources,
  evidence,
  isLoading,
  isRescanning,
  isRemoving,
  onOpenDirectory,
  onOpenResource,
  onRescan,
  onRemove,
}: AgentDetailProps) {
  if (isLoading) {
    return (
      <section className="agent-detail">
        <LoadingState label={copy.common.loading} />
      </section>
    );
  }

  if (!agent) {
    return (
      <section className="agent-detail detail-empty">
        <HardDrivesIcon size={36} weight="light" />
        <p>{copy.agents.detailPlaceholder}</p>
      </section>
    );
  }

  return (
    <section className="agent-detail">
      <header className="detail-header">
        <div>
          <div className="detail-title-row">
            <h2>{agent.displayName}</h2>
            <StatusBadge status={agent.health} />
          </div>
          <p className="mono">{agent.configuration.rootPath}</p>
        </div>
        <div className="detail-actions">
          {agent.configuration.manuallyAdded && (
            <button
              type="button"
              className="danger-button"
              disabled={isRemoving}
              onClick={onRemove}
            >
              <TrashIcon size={17} />
              {copy.actions.remove}
            </button>
          )}
          <button
            type="button"
            className="secondary-button"
            onClick={onOpenDirectory}
          >
            <FolderOpenIcon size={17} />
            {copy.actions.openDirectory}
          </button>
          <button
            type="button"
            className="primary-button"
            disabled={isRescanning}
            onClick={onRescan}
          >
            <ArrowClockwiseIcon
              size={17}
              className={isRescanning ? "spin" : undefined}
            />
            {isRescanning ? copy.actions.scanning : copy.actions.rescan}
          </button>
        </div>
      </header>

      <div className="detail-scroll">
        <section className="content-section relation-section">
          <div className="section-heading">
            <div>
              <span className="eyebrow">{copy.detail.resourceMap}</span>
              <h3>从程序到本地资源</h3>
            </div>
          </div>
          <div className="relation-map">
            <RelationNode
              label={copy.detail.program}
              value={
                agent.runtime.installed
                  ? (agent.runtime.version ??
                    versionProbeStatusLabel[agent.runtime.versionProbeStatus])
                  : copy.detail.programMissing
              }
              hint={
                agent.runtime.executablePath ?? copy.detail.programMissingHint
              }
              healthy={agent.runtime.installed}
            />
            <span className="relation-line" aria-hidden="true" />
            <RelationNode
              label={copy.detail.configRoot}
              value={
                agent.configuration.readable
                  ? copy.detail.configFileCount(
                      agent.configuration.configFiles.length,
                    )
                  : copy.detail.unavailable
              }
              hint={agent.configuration.rootPath}
              healthy={
                agent.configuration.exists &&
                agent.configuration.readable &&
                agent.configuration.valid
              }
            />
            <span className="relation-line" aria-hidden="true" />
            <RelationNode
              label={copy.detail.resources}
              value={copy.detail.indexed(resources.length)}
              healthy={resources.every((resource) => resource.exists)}
            />
          </div>
        </section>

        <section className="content-section">
          <div className="section-heading">
            <div>
              <span className="eyebrow">{copy.detail.resourceList}</span>
              <h3>{copy.detail.resourceSummary(resources.length)}</h3>
            </div>
          </div>
          {resources.length === 0 ? (
            <p className="section-empty">{copy.detail.noResources}</p>
          ) : (
            <div className="resource-table-wrap">
              <table className="resource-table">
                <thead>
                  <tr>
                    <th>{copy.resources.resource}</th>
                    <th>{copy.resources.kind}</th>
                    <th>{copy.resources.scope}</th>
                    <th>{copy.resources.contentSize}</th>
                    <th>
                      <span className="sr-only">{copy.resources.action}</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {resources.map((resource) => (
                    <tr key={resource.id}>
                      <td>
                        <div className="resource-name">
                          <span className="mono">{basename(resource.path)}</span>
                          {resource.isSensitive && (
                            <span className="sensitive-tag">
                              <ShieldWarningIcon size={13} />
                              {copy.resources.sensitive}
                            </span>
                          )}
                        </div>
                        <span className="resource-path mono">
                          {resource.path}
                        </span>
                      </td>
                      <td>{resourceKindLabel[resource.kind]}</td>
                      <td>{resourceScopeLabel[resource.scope]}</td>
                      <td>
                        {resource.entryCount == null
                          ? formatBytes(resource.sizeBytes)
                          : `${resource.entryCount}${resource.scanTruncated ? "+" : ""} 项`}
                      </td>
                      <td>
                        <button
                          type="button"
                          className="icon-button"
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
        </section>

        <div className="detail-columns">
          <section className="content-section">
            <div className="section-heading">
              <div>
                <span className="eyebrow">{copy.detail.evidence}</span>
                <h3>为什么识别为 {agent.agentTypeName}</h3>
              </div>
            </div>
            {evidence.length === 0 ? (
              <p className="section-empty">{copy.detail.noEvidence}</p>
            ) : (
              <div className="evidence-list">
                {evidence.map((item) => (
                  <div className="evidence-item" key={item.id}>
                    {item.success ? (
                      <CheckCircleIcon
                        size={18}
                        weight="fill"
                        className="success-icon"
                      />
                    ) : (
                      <ShieldWarningIcon
                        size={18}
                        weight="fill"
                        className="warning-icon"
                      />
                    )}
                    <div>
                      <strong>{item.message}</strong>
                      <span className="mono">{item.observedValue}</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>

          <section className="content-section">
            <div className="section-heading">
              <div>
                <span className="eyebrow">{copy.detail.metadata}</span>
                <h3>发现与运行信息</h3>
              </div>
            </div>
            <dl className="metadata-grid">
              <dt>{copy.detail.executable}</dt>
              <dd className="mono">
                {agent.runtime.executablePath ?? copy.detail.unavailable}
              </dd>
              <dt>{copy.detail.runtime}</dt>
              <dd>{copy.detail.runtimeType}</dd>
              <dt>{copy.detail.command}</dt>
              <dd className="mono">{agent.runtime.commandName}</dd>
              <dt>{copy.detail.version}</dt>
              <dd>{agent.runtime.version ?? copy.common.unknown}</dd>
              <dt>{copy.detail.versionProbe}</dt>
              <dd>
                {versionProbeStatusLabel[agent.runtime.versionProbeStatus]}
                {agent.runtime.versionProbeError
                  ? `：${agent.runtime.versionProbeError}`
                  : ""}
              </dd>
              <dt>{copy.detail.runtimeSource}</dt>
              <dd>
                {runtimeSourceLabel[agent.runtime.resolutionSource]}
              </dd>
              <dt>{copy.detail.distribution}</dt>
              <dd>
                {runtimeDistributionLabel[agent.runtime.distribution]}
              </dd>
              <dt>{copy.detail.configurationSource}</dt>
              <dd>{agent.configuration.detectionSource}</dd>
              <dt>{copy.detail.configFiles}</dt>
              <dd>
                {copy.detail.configFileCount(
                  agent.configuration.configFiles.length,
                )}
              </dd>
              <dt>{copy.detail.confidence}</dt>
              <dd>{confidenceLabel[agent.confidence]}</dd>
              <dt>{copy.detail.adapterVersion}</dt>
              <dd>{agent.adapterVersion}</dd>
              <dt>{copy.agents.lastScan}</dt>
              <dd>{formatTimestamp(agent.lastSeenAt)}</dd>
            </dl>
          </section>
        </div>
      </div>
    </section>
  );
}

function RelationNode({
  label,
  value,
  hint,
  healthy,
}: {
  label: string;
  value: string;
  hint?: string;
  healthy: boolean;
}) {
  return (
    <div className={`relation-node${healthy ? "" : " unhealthy"}`}>
      <span className="relation-dot" aria-hidden="true" />
      <strong>{label}</strong>
      <small>{value}</small>
      {hint && <em title={hint}>{hint}</em>}
    </div>
  );
}
