import { useEffect, useMemo, useState } from "react";
import { ArrowClockwiseIcon, PlusIcon } from "@phosphor-icons/react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { agentHubApi } from "../shared/api/client";
import type {
  AppSettings,
  CreateQuickLocationRequest,
  HealthStatus,
  ManualLocationRequest,
  QuickLocation,
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
import { QuickLocationDialog } from "../features/locations/QuickLocationDialog";
import { QuickLocationsPage } from "../features/locations/QuickLocationsPage";

export function App() {
  const queryClient = useQueryClient();
  const [section, setSection] = useState<AppSection>("agents");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [status, setStatus] = useState<HealthStatus | "all">("all");
  const [manualOpen, setManualOpen] = useState(false);
  const [removeOpen, setRemoveOpen] = useState(false);
  const [quickLocationDraft, setQuickLocationDraft] = useState<{
    selectedPath?: string;
    location?: QuickLocation;
  } | null>(null);
  const [operationError, setOperationError] = useState<unknown>(null);

  const agentsQuery = useQuery({
    queryKey: ["agents"],
    queryFn: () => agentHubApi.listAgents(),
  });
  const resourcesIndexQuery = useQuery({
    queryKey: ["resources"],
    queryFn: () => agentHubApi.listResources(),
  });
  const quickLocationsQuery = useQuery({
    queryKey: ["quick-locations"],
    queryFn: () => agentHubApi.listQuickLocations(),
  });
  const settingsQuery = useQuery({
    queryKey: ["app-settings"],
    queryFn: () => agentHubApi.getAppSettings(),
    enabled: section === "settings",
  });
  const appInfoQuery = useQuery({
    queryKey: ["app-info"],
    queryFn: () => agentHubApi.getAppInfo(),
    enabled: section === "settings",
    staleTime: Number.POSITIVE_INFINITY,
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
      typeof window === "undefined" ||
      !window.__TAURI_INTERNALS__
    ) {
      return;
    }
    let disposed = false;
    let stopListening: (() => void) | undefined;
    void listen("agent-hub-data-changed", () => {
      void queryClient.invalidateQueries();
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
      } else {
        stopListening = unlisten;
      }
    });
    return () => {
      disposed = true;
      stopListening?.();
    };
  }, [queryClient]);

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
      type: "directory" | "resource" | "quick";
      id: string;
    }) =>
      type === "directory"
        ? agentHubApi.openAgentDirectory(id)
        : type === "resource"
          ? agentHubApi.openResource(id)
          : agentHubApi.openQuickLocation(id),
    onSuccess: async (_, variables) => {
      setOperationError(null);
      if (variables.type === "quick") {
        await queryClient.invalidateQueries({ queryKey: ["quick-locations"] });
      }
    },
    onError: setOperationError,
  });

  const saveQuickLocationMutation = useMutation({
    mutationFn: ({
      request,
      location,
    }: {
      request: CreateQuickLocationRequest;
      location?: QuickLocation;
    }) =>
      location
        ? agentHubApi.updateQuickLocation({
            id: location.id,
            name: request.name,
            showInTray: request.showInTray,
          })
        : agentHubApi.createQuickLocation(request),
    onSuccess: async () => {
      setQuickLocationDraft(null);
      setOperationError(null);
      await queryClient.invalidateQueries({ queryKey: ["quick-locations"] });
    },
  });

  const removeQuickLocationMutation = useMutation({
    mutationFn: (locationId: string) =>
      agentHubApi.removeQuickLocation(locationId),
    onSuccess: async () => {
      setOperationError(null);
      await queryClient.invalidateQueries({ queryKey: ["quick-locations"] });
    },
    onError: setOperationError,
  });

  const reorderQuickLocationsMutation = useMutation({
    mutationFn: (ids: string[]) => agentHubApi.reorderQuickLocations(ids),
    onSuccess: async () => {
      setOperationError(null);
      await queryClient.invalidateQueries({ queryKey: ["quick-locations"] });
    },
    onError: setOperationError,
  });

  const updateQuickLocationMutation = useMutation({
    mutationFn: (location: QuickLocation) =>
      agentHubApi.updateQuickLocation({
        id: location.id,
        name: location.name,
        showInTray: !location.showInTray,
      }),
    onSuccess: async () => {
      setOperationError(null);
      await queryClient.invalidateQueries({ queryKey: ["quick-locations"] });
    },
    onError: setOperationError,
  });

  const updateSettingsMutation = useMutation({
    mutationFn: (settings: AppSettings) =>
      agentHubApi.updateAppSettings(settings),
    onSuccess: (settings) => {
      setOperationError(null);
      queryClient.setQueryData(["app-settings"], settings);
    },
    onError: setOperationError,
  });

  const rebuildIndexMutation = useMutation({
    mutationFn: () => agentHubApi.rebuildAgentIndex(),
    onSuccess: async () => {
      setSelectedId(null);
      setOperationError(null);
      await refreshData();
    },
    onError: setOperationError,
  });

  const settingsActionMutation = useMutation({
    mutationFn: (action: "data" | "project" | "releases") =>
      action === "data"
        ? agentHubApi.openAppDataDirectory()
        : action === "project"
          ? agentHubApi.openProjectPage()
          : agentHubApi.openReleasesPage(),
    onSuccess: () => setOperationError(null),
    onError: setOperationError,
  });

  const chooseQuickLocation = async () => {
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: copy.quickLocations.addTitle,
      });
      if (typeof selected === "string") {
        setQuickLocationDraft({ selectedPath: selected });
      }
    } catch (error) {
      setOperationError(error);
    }
  };

  const queryError =
    agentsQuery.error ||
    resourcesIndexQuery.error ||
    overviewQuery.error ||
    agentResourcesQuery.error ||
    evidenceQuery.error ||
    quickLocationsQuery.error ||
    settingsQuery.error ||
    appInfoQuery.error;

  return (
    <AppShell
      section={section}
      agentCount={agentsQuery.data?.length ?? 0}
      quickLocationCount={quickLocationsQuery.data?.length ?? 0}
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

      {section === "locations" && (
        <QuickLocationsPage
          locations={quickLocationsQuery.data ?? []}
          isLoading={quickLocationsQuery.isLoading}
          isMutating={
            removeQuickLocationMutation.isPending ||
            reorderQuickLocationsMutation.isPending ||
            updateQuickLocationMutation.isPending
          }
          onAdd={() => void chooseQuickLocation()}
          onEdit={(location) => setQuickLocationDraft({ location })}
          onOpen={(id) => openMutation.mutate({ type: "quick", id })}
          onRemove={(location) => {
            if (
              window.confirm(
                `从 Agent Hub 移除“${location.name}”？本机目录及文件不会被删除。`,
              )
            ) {
              removeQuickLocationMutation.mutate(location.id);
            }
          }}
          onToggleTray={(location) =>
            updateQuickLocationMutation.mutate(location)
          }
          onMove={(index, direction) => {
            const ordered = [...(quickLocationsQuery.data ?? [])];
            const target = index + direction;
            if (target < 0 || target >= ordered.length) return;
            [ordered[index], ordered[target]] = [
              ordered[target],
              ordered[index],
            ];
            reorderQuickLocationsMutation.mutate(
              ordered.map((location) => location.id),
            );
          }}
        />
      )}

      {section === "settings" && (
        <SettingsPage
          settings={settingsQuery.data}
          info={appInfoQuery.data}
          isLoading={settingsQuery.isLoading || appInfoQuery.isLoading}
          isSaving={updateSettingsMutation.isPending}
          isRebuilding={rebuildIndexMutation.isPending}
          onChange={(settings) => updateSettingsMutation.mutate(settings)}
          onOpenDataDirectory={() => settingsActionMutation.mutate("data")}
          onRebuildIndex={() => rebuildIndexMutation.mutate()}
          onOpenProjectPage={() => settingsActionMutation.mutate("project")}
          onOpenReleasesPage={() => settingsActionMutation.mutate("releases")}
        />
      )}

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
      <QuickLocationDialog
        open={!!quickLocationDraft}
        location={quickLocationDraft?.location}
        selectedPath={quickLocationDraft?.selectedPath}
        isSubmitting={saveQuickLocationMutation.isPending}
        errorMessage={
          saveQuickLocationMutation.error
            ? toAppError(saveQuickLocationMutation.error).message
            : undefined
        }
        onClose={() => {
          if (!saveQuickLocationMutation.isPending) {
            saveQuickLocationMutation.reset();
            setQuickLocationDraft(null);
          }
        }}
        onSubmit={(request) =>
          saveQuickLocationMutation.mutate({
            request,
            location: quickLocationDraft?.location,
          })
        }
      />
    </AppShell>
  );
}
