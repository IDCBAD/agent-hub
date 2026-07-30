import { useEffect, useState } from "react";
import { XIcon } from "@phosphor-icons/react";
import type {
  CreateQuickLocationRequest,
  QuickLocation,
} from "../../shared/api/types";
import { copy } from "../../shared/i18n/zh-CN";

interface QuickLocationDialogProps {
  open: boolean;
  location?: QuickLocation;
  selectedPath?: string;
  isSubmitting: boolean;
  errorMessage?: string;
  onClose: () => void;
  onSubmit: (request: CreateQuickLocationRequest) => void;
}

export function QuickLocationDialog({
  open,
  location,
  selectedPath,
  isSubmitting,
  errorMessage,
  onClose,
  onSubmit,
}: QuickLocationDialogProps) {
  const [name, setName] = useState("");
  const [showInTray, setShowInTray] = useState(true);
  const [validation, setValidation] = useState("");
  const path = location?.path ?? selectedPath ?? "";

  useEffect(() => {
    if (open) {
      setName(location?.name ?? defaultName(selectedPath ?? ""));
      setShowInTray(location?.showInTray ?? true);
      setValidation("");
    }
  }, [location, open, selectedPath]);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="quick-location-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="modal-header">
          <div>
            <h2 id="quick-location-title">
              {location
                ? copy.quickLocations.editTitle
                : copy.quickLocations.addTitle}
            </h2>
            <p>{copy.quickLocations.dialogBody}</p>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-label="关闭"
            onClick={onClose}
          >
            <XIcon size={18} />
          </button>
        </div>

        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (!name.trim()) {
              setValidation(copy.quickLocations.nameRequired);
              return;
            }
            onSubmit({
              name: name.trim(),
              path,
              showInTray,
            });
          }}
        >
          <label className="form-block">
            <span>{copy.quickLocations.nameLabel}</span>
            <input
              autoFocus
              value={name}
              maxLength={60}
              className={validation || errorMessage ? "invalid" : undefined}
              onChange={(event) => {
                setName(event.target.value);
                setValidation("");
              }}
            />
            {(validation || errorMessage) && (
              <em role="alert">{validation || errorMessage}</em>
            )}
          </label>

          <label className="form-block">
            <span>{copy.quickLocations.pathLabel}</span>
            <input value={path} readOnly className="mono" />
          </label>

          <label className="checkbox-row">
            <input
              type="checkbox"
              checked={showInTray}
              onChange={(event) => setShowInTray(event.target.checked)}
            />
            <span>
              <strong>{copy.quickLocations.showInTray}</strong>
              <small>{copy.quickLocations.showInTrayHint}</small>
            </span>
          </label>

          <div className="modal-actions">
            <button
              type="button"
              className="secondary-button"
              onClick={onClose}
            >
              {copy.actions.cancel}
            </button>
            <button
              type="submit"
              className="primary-button"
              disabled={isSubmitting}
            >
              {isSubmitting
                ? copy.common.loading
                : location
                  ? copy.actions.save
                  : copy.quickLocations.bind}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function defaultName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.at(-1) ?? path;
}
