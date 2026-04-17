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
