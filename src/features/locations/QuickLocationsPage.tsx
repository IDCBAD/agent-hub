import {
  ArrowDownIcon,
  ArrowSquareOutIcon,
  ArrowUpIcon,
  FolderOpenIcon,
  PencilSimpleIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import type { QuickLocation } from "../../shared/api/types";
import { copy } from "../../shared/i18n/zh-CN";
import { formatTimestamp } from "../../shared/lib/format";

interface QuickLocationsPageProps {
  locations: QuickLocation[];
  isLoading: boolean;
  isMutating: boolean;
  onAdd: () => void;
  onEdit: (location: QuickLocation) => void;
  onOpen: (locationId: string) => void;
  onRemove: (location: QuickLocation) => void;
  onToggleTray: (location: QuickLocation) => void;
  onMove: (index: number, direction: -1 | 1) => void;
}

export function QuickLocationsPage({
  locations,
  isLoading,
  isMutating,
  onAdd,
  onEdit,
  onOpen,
  onRemove,
  onToggleTray,
  onMove,
}: QuickLocationsPageProps) {
  return (
    <section className="page">
      <header className="page-header">
        <div>
          <h1>{copy.quickLocations.title}</h1>
          <p>{copy.quickLocations.subtitle}</p>
        </div>
        <button type="button" className="primary-button" onClick={onAdd}>
          <PlusIcon size={17} />
          {copy.quickLocations.add}
        </button>
      </header>

      <div className="quick-location-card">
        <div className="quick-location-intro">
          <FolderOpenIcon size={20} />
          <div>
            <strong>{copy.quickLocations.trayTitle}</strong>
            <p>{copy.quickLocations.trayBody}</p>
          </div>
        </div>

        {isLoading ? (
          <div className="table-loading" role="status">
            {copy.common.loading}
          </div>
        ) : locations.length === 0 ? (
          <div className="page-empty">
            <strong>{copy.quickLocations.emptyTitle}</strong>
            <p>{copy.quickLocations.emptyBody}</p>
            <button type="button" className="secondary-button" onClick={onAdd}>
              <PlusIcon size={17} />
              {copy.quickLocations.add}
            </button>
          </div>
        ) : (
          <div className="quick-location-list">
            {locations.map((location, index) => (
              <article key={location.id}>
                <div className="quick-location-main">
                  <span className="quick-location-icon" aria-hidden="true">
                    <FolderOpenIcon size={19} />
                  </span>
                  <div>
                    <strong>{location.name}</strong>
                    <span className="mono" title={location.path}>
                      {location.path}
                    </span>
                    <small>
                      {location.lastOpenedAt
                        ? copy.quickLocations.lastOpened(
                            formatTimestamp(location.lastOpenedAt),
                          )
                        : copy.quickLocations.neverOpened}
                    </small>
                  </div>
                </div>

                <label className="tray-toggle">
                  <input
                    type="checkbox"
                    checked={location.showInTray}
                    disabled={isMutating}
                    onChange={() => onToggleTray(location)}
                  />
                  <span>{copy.quickLocations.trayShort}</span>
                </label>

                <div className="quick-location-actions">
                  <button
                    type="button"
                    className="icon-button"
                    aria-label={`上移 ${location.name}`}
                    disabled={index === 0 || isMutating}
                    onClick={() => onMove(index, -1)}
                  >
                    <ArrowUpIcon size={16} />
                  </button>
                  <button
                    type="button"
                    className="icon-button"
                    aria-label={`下移 ${location.name}`}
                    disabled={index === locations.length - 1 || isMutating}
                    onClick={() => onMove(index, 1)}
                  >
                    <ArrowDownIcon size={16} />
                  </button>
                  <button
                    type="button"
                    className="icon-button"
                    aria-label={`编辑 ${location.name}`}
                    onClick={() => onEdit(location)}
                  >
                    <PencilSimpleIcon size={16} />
                  </button>
                  <button
                    type="button"
                    className="icon-button danger-icon-button"
                    aria-label={`移除 ${location.name}`}
                    disabled={isMutating}
                    onClick={() => onRemove(location)}
                  >
                    <TrashIcon size={16} />
                  </button>
                  <button
                    type="button"
                    className="secondary-button compact-button"
                    onClick={() => onOpen(location.id)}
                  >
                    <ArrowSquareOutIcon size={16} />
                    {copy.actions.openDirectory}
                  </button>
                </div>
              </article>
            ))}
          </div>
        )}
      </div>
    </section>
  );
}
