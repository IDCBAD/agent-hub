import type { HealthStatus } from "../api/types";
import { statusLabel } from "../i18n/zh-CN";

export function StatusBadge({ status }: { status: HealthStatus }) {
  return (
    <span className={`status-badge status-${status}`}>
      <span aria-hidden="true" />
      {statusLabel[status]}
    </span>
  );
}
