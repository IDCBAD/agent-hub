use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn open_agent_directory(path: &Path) -> Result<(), AppError> {
    let verified = verify_existing(path)?;
    if !verified.is_dir() {
        return Err(AppError::invalid_path("Agent 配置路径不是目录。"));
    }
    open::that_detached(&verified).map_err(|error| {
        AppError::new(
            "open_failed",
            format!("系统无法打开 Agent 目录：{error}"),
            true,
            Some("请检查目录权限和系统文件管理器设置。"),
        )
    })
}

pub fn open_resource(path: &Path, authorized_root: &Path) -> Result<(), AppError> {
    let verified = verify_existing(path)?;
    let root = verify_existing(authorized_root)?;
    if !verified.starts_with(&root) {
        return Err(AppError::new(
            "path_outside_agent_root",
            "资源路径不在已授权的 Agent 配置目录内。",
            false,
            Option::<String>::None,
        ));
    }
    open::that_detached(&verified).map_err(|error| {
        AppError::new(
            "open_failed",
            format!("系统无法打开资源：{error}"),
            true,
            Some("请检查文件是否仍然存在以及系统关联程序设置。"),
        )
    })
}

fn verify_existing(path: &Path) -> Result<PathBuf, AppError> {
    if !path.exists() {
        return Err(AppError::invalid_path(format!(
            "路径已不存在：{}",
            path.display()
        )));
    }
    std::fs::canonicalize(path).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_path_must_stay_inside_authorized_root() {
        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::NamedTempFile::new().expect("outside");
        let verified_root = verify_existing(root.path()).expect("root path");
        let verified_file = verify_existing(outside.path()).expect("file path");
        assert!(!verified_file.starts_with(verified_root));
    }
}
