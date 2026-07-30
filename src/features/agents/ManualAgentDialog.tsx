import { useEffect, useState } from "react";
import { XIcon } from "@phosphor-icons/react";
import type { ManualLocationRequest } from "../../shared/api/types";
import { copy } from "../../shared/i18n/zh-CN";

interface ManualAgentDialogProps {
  open: boolean;
  isSubmitting: boolean;
  errorMessage?: string;
  onClose: () => void;
  onSubmit: (request: ManualLocationRequest) => void;
}

export function ManualAgentDialog({
  open,
  isSubmitting,
  errorMessage,
  onClose,
  onSubmit,
}: ManualAgentDialogProps) {
  const [agentTypeId, setAgentTypeId] = useState("claude-code");
  const [path, setPath] = useState("");
  const [validation, setValidation] = useState("");

  useEffect(() => {
    if (!open) {
      setPath("");
      setValidation("");
    }
  }, [open]);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div
        className="modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="manual-agent-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="modal-header">
          <div>
            <h2 id="manual-agent-title">{copy.manual.title}</h2>
            <p>{copy.manual.body}</p>
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
            if (!path.trim()) {
              setValidation(copy.manual.pathRequired);
              return;
            }
            onSubmit({ agentTypeId, path: path.trim() });
          }}
        >
          <label className="form-block">
            <span>{copy.manual.typeLabel}</span>
            <select
              value={agentTypeId}
              onChange={(event) => setAgentTypeId(event.target.value)}
            >
              <option value="claude-code">Claude Code</option>
              <option value="codex">Codex</option>
              <option value="hermes">Hermes Agent</option>
              <option value="kimi-cli">Kimi Code</option>
            </select>
          </label>

          <label className="form-block">
            <span>{copy.manual.pathLabel}</span>
            <input
              autoFocus
              value={path}
              className={validation || errorMessage ? "invalid" : undefined}
              placeholder={copy.manual.pathPlaceholder}
              onChange={(event) => {
                setPath(event.target.value);
                setValidation("");
              }}
            />
            <small>{copy.manual.pathHint}</small>
            {(validation || errorMessage) && (
              <em role="alert">{validation || errorMessage}</em>
            )}
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
              {isSubmitting ? copy.common.loading : copy.actions.add}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
