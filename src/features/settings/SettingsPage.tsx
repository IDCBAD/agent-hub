import {
  ArrowClockwiseIcon,
  ArrowSquareOutIcon,
  DatabaseIcon,
  FolderOpenIcon,
} from "@phosphor-icons/react";
import type { AppInfo, AppSettings } from "../../shared/api/types";
import { copy } from "../../shared/i18n/zh-CN";

interface SettingsPageProps {
  settings?: AppSettings;
  info?: AppInfo;
  isLoading: boolean;
  isSaving: boolean;
  isRebuilding: boolean;
  onChange: (settings: AppSettings) => void;
  onOpenDataDirectory: () => void;
  onRebuildIndex: () => void;
  onOpenProjectPage: () => void;
  onOpenReleasesPage: () => void;
}

interface PreferenceRowProps {
  title: string;
  description: string;
  checked: boolean;
  disabled: boolean;
  onChange: (checked: boolean) => void;
}

function PreferenceRow({
  title,
  description,
  checked,
  disabled,
  onChange,
}: PreferenceRowProps) {
  return (
    <div className="setting-row">
      <div className="setting-copy">
        <strong>{title}</strong>
        <p>{description}</p>
      </div>
      <button
        type="button"
        className={`switch-control ${checked ? "is-on" : ""}`}
        role="switch"
        aria-label={title}
        aria-checked={checked}
        disabled={disabled}
        onClick={() => onChange(!checked)}
      >
        <span />
      </button>
    </div>
  );
}

export function SettingsPage({
  settings,
  info,
  isLoading,
  isSaving,
  isRebuilding,
  onChange,
  onOpenDataDirectory,
  onRebuildIndex,
  onOpenProjectPage,
  onOpenReleasesPage,
}: SettingsPageProps) {
  const update = (patch: Partial<AppSettings>) => {
    if (settings) onChange({ ...settings, ...patch });
  };

  return (
    <section className="page settings-page">
      <header className="page-header">
        <div>
          <h1>{copy.settings.title}</h1>
          <p>{copy.settings.subtitle}</p>
        </div>
        {isSaving && <span className="settings-saving">{copy.settings.saving}</span>}
      </header>

      {isLoading || !settings ? (
        <div className="settings-panel loading-state">
          <div className="skeleton skeleton-title" />
          <div className="skeleton skeleton-line" />
          <div className="skeleton skeleton-line short" />
        </div>
      ) : (
        <div className="settings-stack">
          <section className="settings-panel">
            <header className="settings-section-header">
              <h2>{copy.settings.generalTitle}</h2>
              <p>{copy.settings.generalBody}</p>
            </header>
            <PreferenceRow
              title={copy.settings.launchTitle}
              description={copy.settings.launchBody}
              checked={settings.launchAtLogin}
              disabled={isSaving}
              onChange={(launchAtLogin) => update({ launchAtLogin })}
            />
            <PreferenceRow
              title={copy.settings.trayTitle}
              description={copy.settings.trayBody}
              checked={settings.keepRunningInTray}
              disabled={isSaving}
              onChange={(keepRunningInTray) => update({ keepRunningInTray })}
            />
            <PreferenceRow
              title={copy.settings.scanTitle}
              description={copy.settings.scanBody}
              checked={settings.scanOnLaunch}
              disabled={isSaving}
              onChange={(scanOnLaunch) => update({ scanOnLaunch })}
            />
          </section>

          <section className="settings-panel">
            <header className="settings-section-header">
              <h2>{copy.settings.dataTitle}</h2>
              <p>{copy.settings.dataBody}</p>
            </header>
            <div className="setting-action-row">
              <FolderOpenIcon size={20} />
              <div className="setting-copy">
                <strong>{copy.settings.dataDirectoryTitle}</strong>
                <p className="mono" title={info?.dataDirectory}>
                  {info?.dataDirectory ?? copy.common.unknown}
                </p>
              </div>
              <button
                type="button"
                className="secondary-button"
                onClick={onOpenDataDirectory}
              >
                {copy.settings.openDirectory}
              </button>
            </div>
            <div className="setting-action-row">
              <DatabaseIcon size={20} />
              <div className="setting-copy">
                <strong>{copy.settings.indexTitle}</strong>
                <p>{copy.settings.indexBody}</p>
              </div>
              <button
                type="button"
                className="secondary-button"
                disabled={isRebuilding}
                onClick={() => {
                  if (window.confirm(copy.settings.rebuildConfirm)) {
                    onRebuildIndex();
                  }
                }}
              >
                <ArrowClockwiseIcon
                  size={16}
                  className={isRebuilding ? "spin" : undefined}
                />
                {isRebuilding
                  ? copy.settings.rebuilding
                  : copy.settings.rebuild}
              </button>
            </div>
          </section>

          <section className="settings-panel">
            <header className="settings-section-header">
              <h2>{copy.settings.aboutTitle}</h2>
            </header>
            <dl className="about-grid">
              <div>
                <dt>{copy.settings.versionLabel}</dt>
                <dd>{info?.version ?? copy.common.unknown}</dd>
              </div>
              <div>
                <dt>{copy.settings.schemaLabel}</dt>
                <dd>{info ? `SQLite Schema ${info.schemaVersion}` : copy.common.unknown}</dd>
              </div>
            </dl>
            <div className="settings-link-row">
              <button
                type="button"
                className="secondary-button"
                onClick={onOpenProjectPage}
              >
                {copy.settings.projectPage}
                <ArrowSquareOutIcon size={15} />
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={onOpenReleasesPage}
              >
                {copy.settings.releasesPage}
                <ArrowSquareOutIcon size={15} />
              </button>
            </div>
          </section>
        </div>
      )}
    </section>
  );
}
