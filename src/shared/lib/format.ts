export function formatTimestamp(timestamp: number | null | undefined): string {
  if (!timestamp) return "—";
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(timestamp * 1000));
}

export function formatBytes(bytes: number | null): string {
  if (bytes == null) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function basename(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.at(-1) ?? path;
}

export function toAppError(error: unknown): {
  code: string;
  message: string;
  suggestedAction: string | null;
} {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return {
      code:
        "code" in error && typeof error.code === "string"
          ? error.code
          : "unexpected_error",
      message: error.message,
      suggestedAction:
        "suggestedAction" in error &&
        typeof error.suggestedAction === "string"
          ? error.suggestedAction
          : null,
    };
  }
  return {
    code: "unexpected_error",
    message: "发生了未预期的错误。",
    suggestedAction: "请重试；如果问题持续，请检查目标路径的访问权限。",
  };
}
