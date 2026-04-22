//! 资源管理器
//!
//! 封装 Bevy AssetServer，提供带状态跟踪的资源加载管理器。

use bevy::prelude::*;
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 资源加载状态
#[derive(Debug, Clone)]
pub enum AssetLoadState {
    /// 加载中
    Loading {
        /// 资源路径
        path: String,
        /// 跟踪 ID
        handle_id: u64,
    },
    /// 加载完成
    Ready {
        /// 资源路径
        path: String,
        /// 跟踪 ID
        handle_id: u64,
    },
    /// 加载失败
    Failed {
        /// 资源路径
        path: String,
        /// 错误信息
        error: String,
    },
}

/// 缓存的资源条目
#[derive(Debug, Clone)]
pub struct CachedAsset {
    /// 跟踪 ID
    pub handle_id: u64,
    /// 加载时间
    pub loaded_at: Instant,
    /// 访问次数
    pub access_count: u64,
}

/// 资源加载计数器，用于生成唯一跟踪 ID
static LOAD_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    LOAD_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// 资源管理器
#[derive(Resource)]
pub struct AssetManager {
    /// 各资源的加载状态
    states: HashMap<String, AssetLoadState>,
    /// 待处理数量
    pending_count: usize,
    /// 已完成数量
    completed_count: usize,
    /// LRU 缓存
    cache: LruCache<String, CachedAsset>,
    /// 缓存命中次数
    cache_hits: u64,
    /// 缓存未命中次数
    cache_misses: u64,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    /// 创建新的资源管理器
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            pending_count: 0,
            completed_count: 0,
            cache: LruCache::new(NonZeroUsize::new(64).unwrap()),
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    /// 启动资源加载，返回跟踪 ID
    pub fn load(&mut self, path: &str) -> u64 {
        // 优先检查缓存命中（peek 不更新访问顺序）
        if let Some(cached) = self.cache.peek(path) {
            self.cache_hits += 1;
            return cached.handle_id;
        }

        let id = next_id();
        self.states.insert(
            path.to_string(),
            AssetLoadState::Loading {
                path: path.to_string(),
                handle_id: id,
            },
        );
        self.pending_count += 1;
        self.cache_misses += 1;
        id
    }

    /// 轮询所有资源的状态
    pub fn poll(&mut self, asset_server: &AssetServer) {
        let paths: Vec<String> = self.states.keys().cloned().collect();
        for path in paths {
            let handle_id = if let Some(state) = self.states.get(&path) {
                match state {
                    AssetLoadState::Loading { handle_id, .. } => *handle_id,
                    _ => continue,
                }
            } else {
                continue;
            };

            // 尝试获取加载状态
            let load_state = asset_server
                .get_handle_untyped(&path)
                .and_then(|h| asset_server.get_load_state(&h));
            match load_state {
                Some(bevy::asset::LoadState::Loaded) => {
                    self.states.insert(
                        path.clone(),
                        AssetLoadState::Ready { path: path.clone(), handle_id },
                    );
                    self.pending_count -= 1;
                    self.completed_count += 1;
                    // 写入缓存
                    self.cache.put(
                        path.clone(),
                        CachedAsset {
                            handle_id,
                            loaded_at: Instant::now(),
                            access_count: 1,
                        },
                    );
                }
                Some(bevy::asset::LoadState::Failed(_)) => {
                    self.states.insert(
                        path.clone(),
                        AssetLoadState::Failed {
                            path: path.clone(),
                            error: "Asset load failed".to_string(),
                        },
                    );
                    self.pending_count -= 1;
                    self.completed_count += 1;
                }
                _ => {}
            }
        }
    }

    /// 获取加载进度 (已完成, 总数)
    pub fn progress(&self) -> (usize, usize) {
        (self.completed_count, self.states.len())
    }

    /// 检查资源是否加载完成
    pub fn is_ready(&self, path: &str) -> bool {
        matches!(self.states.get(path), Some(AssetLoadState::Ready { .. }))
    }

    /// 获取资源的加载状态
    pub fn get_state(&self, path: &str) -> Option<&AssetLoadState> {
        self.states.get(path)
    }

    /// 获取待处理数量
    pub fn pending_count(&self) -> usize {
        self.pending_count
    }

    /// 获取缓存中的资源
    pub fn get_cached(&self, path: &str) -> Option<&CachedAsset> {
        self.cache.peek(path)
    }

    /// 使缓存失效
    pub fn invalidate(&mut self, path: &str) {
        self.cache.pop(path);
        self.states.remove(path);
    }

    /// 清理超期未使用的缓存条目
    pub fn clear_unused_older_than(&mut self, duration: Duration) {
        let now = Instant::now();
        let keys_to_remove: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, v)| now.duration_since(v.loaded_at) > duration)
            .map(|(k, _)| k.clone())
            .collect();
        for key in keys_to_remove {
            self.cache.pop(&key);
        }
    }

    /// 获取缓存统计 (hits, misses)
    pub fn cache_stats(&self) -> (u64, u64) {
        (self.cache_hits, self.cache_misses)
    }

    /// 重新加载资源（失效缓存 + 重新发起加载）
    pub fn reload(&mut self, path: &str) -> u64 {
        self.invalidate(path);
        self.load(path)
    }
}

/// 资源管理器轮询系统
pub fn asset_manager_poll_system(
    mut asset_manager: ResMut<AssetManager>,
    asset_server: Res<AssetServer>,
) {
    if asset_manager.pending_count > 0 {
        asset_manager.poll(&asset_server);
    }
}

// =============================================================================
// 资产清单与验证（B4）
// =============================================================================

/// 资产清单条目
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AssetEntry {
    /// 资源路径（相对于 assets/ 目录）
    pub path: String,
    /// 资源类型
    pub r#type: String,
}

/// 资产清单
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AssetManifest {
    /// Mod 名称
    pub name: String,
    /// 版本号（遵循 SemVer）
    pub version: String,
    /// 资产列表
    pub assets: Vec<AssetEntry>,
}

/// 清单验证错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    /// 缺少必填字段
    #[error("缺少必填字段: {0}")]
    MissingField(String),
    /// 非法路径
    #[error("非法路径: {0}")]
    InvalidPath(String),
    /// 不支持的文件格式
    #[error("不支持的文件格式: {0}")]
    UnsupportedFormat(String),
    /// 重复路径
    #[error("重复路径: {0}")]
    DuplicatePath(String),
    /// 文件不存在
    #[error("文件不存在: {0}")]
    FileNotFound(String),
    /// 版本格式错误
    #[error("版本格式错误: {0}")]
    InvalidVersion(String),
}

impl AssetManifest {
    /// 验证清单格式与内容
    pub fn validate(&self) -> Result<(), ValidationError> {
        // 必填字段检查
        if self.name.is_empty() {
            return Err(ValidationError::MissingField("name".to_string()));
        }
        if self.version.is_empty() {
            return Err(ValidationError::MissingField("version".to_string()));
        }
        if self.assets.is_empty() {
            return Err(ValidationError::MissingField("assets".to_string()));
        }

        // 版本格式检查（简单 SemVer）
        if !self.version.contains('.') {
            return Err(ValidationError::InvalidVersion(self.version.clone()));
        }

        let mut seen_paths = std::collections::HashSet::new();
        let allowed_exts = ["png", "jpg", "webp", "gltf", "glb"];

        for entry in &self.assets {
            // 路径格式检查
            if entry.path.contains("..") {
                return Err(ValidationError::InvalidPath(entry.path.clone()));
            }

            // 扩展名白名单
            let ext = std::path::Path::new(&entry.path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !allowed_exts.contains(&ext) {
                return Err(ValidationError::UnsupportedFormat(ext.to_string()));
            }

            // 路径唯一性检查
            if !seen_paths.insert(entry.path.clone()) {
                return Err(ValidationError::DuplicatePath(entry.path.clone()));
            }

            // 文件存在性检查
            if !std::path::Path::new(&entry.path).exists() {
                return Err(ValidationError::FileNotFound(entry.path.clone()));
            }
        }

        Ok(())
    }

    /// 检查与另一版本是否兼容（主版本号相同）
    pub fn is_compatible_with(&self, other: &str) -> bool {
        let self_major = self.version.split('.').next();
        let other_major = other.split('.').next();
        self_major == other_major
    }
}

impl AssetManager {
    /// 从文件加载并验证资产清单
    pub fn load_manifest<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> anyhow::Result<AssetManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: AssetManifest = toml::from_str(&content)?;
        manifest.validate()?;
        Ok(manifest)
    }
}
