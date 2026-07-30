import { WarningCircleIcon, XIcon } from "@phosphor-icons/react";
import { copy } from "../i18n/zh-CN";
import { toAppError } from "../lib/format";

interface ErrorNoticeProps {
  error: unknown;
  onDismiss?: () => void;
  onRetry?: () => void;
}

export function ErrorNotice({
  error,
  onDismiss,
  onRetry,
}: ErrorNoticeProps) {
  const normalized = toAppError(error);

  return (
    <div className="error-notice" role="alert">
      <WarningCircleIcon size={20} weight="fill" />
      <div>
        <strong>{copy.common.errorTitle}</strong>
        <p>{normalized.message}</p>
        {normalized.suggestedAction && (
          <small>{normalized.suggestedAction}</small>
        )}
      </div>
      {onRetry && (
        <button className="text-button" type="button" onClick={onRetry}>
          {copy.actions.retry}
        </button>
      )}
      {onDismiss && (
        <button
          className="icon-button"
          type="button"
          aria-label="关闭错误提示"
          onClick={onDismiss}
        >
          <XIcon size={16} />
        </button>
      )}
    </div>
  );
}
