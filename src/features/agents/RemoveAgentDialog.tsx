import { ShieldCheckIcon, XIcon } from "@phosphor-icons/react";
import type { AgentOverview } from "../../shared/api/types";
import { copy } from "../../shared/i18n/zh-CN";

interface RemoveAgentDialogProps {
  agent: AgentOverview | undefined;
  open: boolean;
  isRemoving: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export function RemoveAgentDialog({
  agent,
  open,
  isRemoving,
  onClose,
  onConfirm,
}: RemoveAgentDialogProps) {
  if (!open || !agent) return null;

  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div
        className="modal remove-agent-modal"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="remove-agent-title"
        aria-describedby="remove-agent-description"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="modal-header">
          <div>
            <h2 id="remove-agent-title">{copy.removeManual.title}</h2>
            <p id="remove-agent-description">
              {copy.removeManual.body(agent.displayName)}
            </p>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-label="关闭"
            disabled={isRemoving}
            onClick={onClose}
          >
            <XIcon size={18} />
          </button>
        </div>

        <div className="remove-agent-content">
          <div className="safety-note">
            <ShieldCheckIcon size={19} weight="fill" />
            <strong>{copy.removeManual.safety}</strong>
          </div>
          <p>{copy.removeManual.rediscovery}</p>
          <span className="mono">{agent.configuration.rootPath}</span>
        </div>

        <div className="modal-actions remove-agent-actions">
          <button
            type="button"
            className="secondary-button"
            disabled={isRemoving}
            onClick={onClose}
          >
            {copy.actions.cancel}
          </button>
          <button
            type="button"
            className="danger-button"
            disabled={isRemoving}
            onClick={onConfirm}
          >
            {isRemoving ? copy.actions.removing : copy.actions.remove}
          </button>
        </div>
      </div>
    </div>
  );
}
