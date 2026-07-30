use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub suggested_action: Option<String>,
    pub context_id: String,
}

impl AppError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        recoverable: bool,
        suggested_action: Option<impl Into<String>>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            recoverable,
            suggested_action: suggested_action.map(Into::into),
            context_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn not_found(entity: &str) -> Self {
        Self::new(
            "not_found",
            format!("未找到指定的{entity}。"),
            true,
            Some("请刷新列表后重试。"),
        )
    }

    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::new(
            "invalid_path",
            message,
            true,
            Some("请选择一个存在且可访问的目录。"),
        )
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            "internal_error",
            message,
            true,
            Some("请重试；如果问题持续，请检查诊断日志。"),
        )
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::internal(format!("本地数据库操作失败：{error}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::internal(format!("本地文件操作失败：{error}"))
    }
}
