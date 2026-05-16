//! 工具函数

use std::time::SystemTime;

/// 获取文件最后修改时间
pub fn get_last_modified(path: &str) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// 解析资源根目录（支持多种开发环境）
pub fn resolve_asset_root() -> String {
    use std::path::PathBuf;

    let candidates = [
        PathBuf::from("assets"),
        PathBuf::from("..").join("assets"),
        PathBuf::from("..").join("..").join("assets"),
        PathBuf::from("..").join("..").join("..").join("assets"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("assets"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return candidate
                .canonicalize()
                .unwrap_or(candidate)
                .to_string_lossy()
                .to_string();
        }
    }

    "assets".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_last_modified_existing_file() {
        let result = get_last_modified("Cargo.toml");
        assert!(result.is_some(), "Cargo.toml 应存在且有修改时间");
    }

    #[test]
    fn test_get_last_modified_nonexistent_file() {
        let result = get_last_modified("/this/path/does/not/exist.txt");
        assert!(result.is_none(), "不存在的文件应返回 None");
    }

    #[test]
    fn test_resolve_asset_root_returns_existing_path() {
        let root = resolve_asset_root();
        assert!(!root.is_empty(), "资源根目录不应为空");
        let path = std::path::Path::new(&root);
        assert!(path.exists(), "解析出的资源根目录应存在: {}", root);
    }

    #[test]
    fn test_resolve_asset_root_contains_assets() {
        let root = resolve_asset_root();
        assert!(
            root.contains("assets") || root.ends_with("assets"),
            "资源根目录应包含 'assets': {}",
            root
        );
    }
}
