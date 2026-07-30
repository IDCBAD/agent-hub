import { useEffect, useMemo, useRef, useState } from "react";
import { ArrowClockwiseIcon, PlusIcon } from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { agentHubApi } from "../shared/api/client";
import type {
  HealthStatus,
  ManualLocationRequest,
} from "../shared/api/types";
import {
  AppShell,
  type AppSection,
} from "../shared/components/AppShell";
import { ErrorNotice } from "../shared/components/ErrorNotice";
import { copy } from "../shared/i18n/zh-CN";
import { toAppError } from "../shared/lib/format";
import { AgentDetail } from "../features/agents/AgentDetail";
import { AgentList } from "../features/agents/AgentList";
import { ManualAgentDialog } from "../features/agents/ManualAgentDialog";
import { RemoveAgentDialog } from "../features/agents/RemoveAgentDialog";
import { ResourceIndex } from "../features/resources/ResourceIndex";
import { SettingsPage } from "../features/settings/SettingsPage";

export function App() {
  const queryClient = useQueryClient();
  const autoScanStarted = useRef(false);
  const [section, setSection] = useState<AppSection>("agents");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState<HealthStatus | "all">("all");
  const [manualOpen, setManualOpen] = useState(false);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [operationError, setOperationError] = useState<unknown>(null);

  const agentsQuery = useQuery({
    queryKey: ["agents"],
    queryFn: () => agentHubApi.listAgents(),
  });
  const resourcesIndexQuery = useQuery({
    queryKey: ["resources"],
    queryFn: () => agentHubApi.listResources(),
  });

  const scanMutation = useMutation({
    mutationFn: () => agentHubApi.discoverAgents(),
    onSuccess: async () => {
      setOperationError(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ["agents"] }),
        queryClient.invalidateQueries({ queryKey: ["resources"] }),
      ]);
    },
    onError: setOperationError,
  });

  useEffect(() => {
    if (
      !autoScanStarted.current &&
      typeof window !== "undefined" &&
      window.__TAURI_INTERNALS__
    ) {
      autoScanStarted.current = true;
      scanMutation.mutate();
    }
  }, [scanMutation]);

  const filteredAgents = useMemo(() => {
    const term = search.trim().toLocaleLowerCase();
    return (agentsQuery.data ?? []).filter(
      (agent) =>
        (status === "all" || agent.health === status) &&
        (!term ||
          agent.displayName.toLocaleLowerCase().includes(term) ||
          agent.configuration.rootPath.toLocaleLowerCase().includes(term)),
    );
  }, [agentsQuery.data, search, status]);

  useEffect(() => {
    const allAgents = agentsQuery.data ?? [];
    if (allAgents.length === 0) {
      setSelectedId(null);
    } else if (
      !selectedId ||
      !allAgents.some((agent) => agent.id === selectedId)
    ) {
      setSelectedId(allAgents[0].id);
    }
  }, [agentsQuery.data, selectedId]);

  const overviewQuery = useQuery({
    queryKey: ["agent", selectedId, "overview"],
    queryFn: () => agentHubApi.getAgentOverview(selectedId!),
    enabled: !!selectedId,
  });
  const agentResourcesQuery = useQuery({
    queryKey: ["agent", selectedId, "resources"],
    queryFn: () => agentHubApi.getAgentResources(selectedId!),
    enabled: !!selectedId,
  });
  const evidenceQuery = useQuery({
    queryKey: ["agent", selectedId, "evidence"],
    queryFn: () => agentHubApi.getDiscoveryEvidence(selectedId!),
    enabled: !!selectedId,
  });

  const refreshData = async (agentId?: string) => {
    const tasks = [
      queryClient.invalidateQueries({ queryKey: ["agents"] }),
      queryClient.invalidateQueries({ queryKey: ["resources"] }),
    ];
    if (agentId) {
      tasks.push(queryClient.invalidateQueries({ queryKey: ["agent", agentId] }));
    }
    await Promise.all(tasks);
  };

  const rescanMutation = useMutation({
    mutationFn: (agentId: string) => agentHubApi.rescanAgent(agentId),
    onSuccess: async (_, agentId) => {
      setOperationError(null);
      await refreshData(agentId);
    },
    onError: setOperationError,
  });

  const manualMutation = useMutation({
    mutationFn: (request: ManualLocationRequest) =>
      agentHubApi.addManualLocation(request),
    onSuccess: async (agent) => {
      setManualOpen(false);
      setSelectedId(agent.id);
      setOperationError(null);
      await refreshData(agent.id);
    },
  });

  const removeMutation = useMutation({
    mutationFn: (agentId: string) => agentHubApi.removeManualAgent(agentId),
    onSuccess: async () => {
      setRemoveOpen(false);
      setSelectedId(null);
      setOperationError(null);
      await refreshData();
    },
    onError: setOperationError,
  });

  const openMutation = useMutation({
    mutationFn: ({
      type,
      id,
    }: {
      type: "directory" | "resource";
      id: string;
    }) =>
      type === "directory"
        ? agentHubApi.openAgentDirectory(id)
        : agentHubApi.openResource(id),
    onError: setOperationError,
  });

  const queryError =
    agentsQuery.error ||
    resourcesIndexQuery.error ||
    overviewQuery.error ||
    agentResourcesQuery.error ||
    evidenceQuery.error;

  return (
    <AppShell
      section={section}
      agentCount={agentsQuery.data?.length ?? 0}
      resourceCount={resourcesIndexQuery.data?.length ?? 0}
      onSectionChange={setSection}
    >
      {(operationError || queryError) && (
        <div className="global-notice">
          <ErrorNotice
            error={operationError || queryError}
            onDismiss={() => setOperationError(null)}
          />
        </div>
      )}

      {section === "agents" && (
        <section className="agents-page">
          <header className="page-header agent-page-header">
            <div>
              <h1>{copy.agents.title}</h1>
              <p>{copy.agents.subtitle}</p>
            </div>
            <div className="page-actions">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setManualOpen(true)}
              >
                <PlusIcon size={17} />
                {copy.actions.addManual}
              </button>
              <button
                type="button"
                className="primary-button"
                disabled={scanMutation.isPending}
                onClick={() => scanMutation.mutate()}
              >
                <ArrowClockwiseIcon
                  size={17}
                  className={scanMutation.isPending ? "spin" : undefined}
                />
                {scanMutation.isPending
                  ? copy.actions.scanning
                  : copy.actions.scan}
              </button>
            </div>
          </header>

          <div className="agents-workbench">
            <AgentList
              agents={filteredAgents}
              selectedId={selectedId}
              search={search}
              status={status}
              isLoading={agentsQuery.isLoading}
              onSearchChange={setSearch}
              onStatusChange={setStatus}
              onSelect={setSelectedId}
              onAddManual={() => setManualOpen(true)}
              onClearFilters={() => {
                setSearch("");
                setStatus("all");
              }}
            />
            <AgentDetail
              agent={overviewQuery.data}
              resources={agentResourcesQuery.data ?? []}
              evidence={evidenceQuery.data ?? []}
              isLoading={
                !!selectedId &&
                (overviewQuery.isLoading ||
                  agentResourcesQuery.isLoading ||
                  evidenceQuery.isLoading)
              }
              isRescanning={rescanMutation.isPending}
              isRemoving={removeMutation.isPending}
              onOpenDirectory={() =>
                selectedId &&
                openMutation.mutate({ type: "directory", id: selectedId })
              }
              onOpenResource={(id) =>
                openMutation.mutate({ type: "resource", id })
              }
              onRescan={() =>
                selectedId && rescanMutation.mutate(selectedId)
              }
              onRemove={() => setRemoveOpen(true)}
            />
          </div>
        </section>
      )}

      {section === "resources" && (
        <ResourceIndex
          resources={resourcesIndexQuery.data ?? []}
          isLoading={resourcesIndexQuery.isLoading}
          onOpenResource={(id) =>
            openMutation.mutate({ type: "resource", id })
          }
        />
      )}

      {section === "settings" && <SettingsPage />}

      <ManualAgentDialog
        open={manualOpen}
        isSubmitting={manualMutation.isPending}
        errorMessage={
          manualMutation.error
            ? toAppError(manualMutation.error).message
            : undefined
        }
        onClose={() => {
          manualMutation.reset();
          setManualOpen(false);
        }}
        onSubmit={(request) => manualMutation.mutate(request)}
      />
      <RemoveAgentDialog
        open={removeOpen}
        agent={overviewQuery.data}
        isRemoving={removeMutation.isPending}
        onClose={() => {
          if (!removeMutation.isPending) setRemoveOpen(false);
        }}
        onConfirm={() =>
          selectedId && removeMutation.mutate(selectedId)
        }
      />
    </AppShell>
  );
}
