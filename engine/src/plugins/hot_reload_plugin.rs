//! 热重载插件
//!
//! 开发模式下监听Lua脚本和配置文件变化，自动重载
//! 无需重启游戏即可看到修改效果

use bevy::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "hot-reload")]
use crate::lua_api::LuaRuntime;
#[cfg(feature = "hot-reload")]
use crate::resources::EntityRegistry;

/// 热重载事件
#[derive(Event)]
pub enum HotReloadEvent {
    /// Lua脚本变化
    LuaScriptChanged(PathBuf),
    /// 配置文件变化
    ConfigChanged(PathBuf),
    /// 资源文件变化
    AssetChanged(PathBuf),
    /// 手动触发重载（F5键）
    ManualReload,
}

/// 热重载状态
#[derive(Resource)]
pub struct HotReloadState {
    /// 是否启用热重载
    pub enabled: bool,
    /// 上次重载时间（防抖）
    pub last_reload: Duration,
    /// 重载冷却时间（毫秒）
    pub cooldown_ms: u64,
    /// 文件变化接收器（线程安全包装）
    #[cfg(feature = "hot-reload")]
    pub file_receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<notify::Event>>>,
}

#[cfg(feature = "hot-reload")]
use std::sync::{
    Arc, Mutex,
    mpsc::{Sender, channel},
};

#[cfg(feature = "hot-reload")]
impl HotReloadState {
    /// 创建新的热重载状态
    pub fn new() -> (Self, Sender<notify::Event>) {
        let (tx, rx) = channel();
        (
            Self {
                enabled: true,
                last_reload: Duration::from_secs(0),
                cooldown_ms: 500, // 500ms防抖
                file_receiver: Arc::new(Mutex::new(rx)),
            },
            tx,
        )
    }
}

#[cfg(not(feature = "hot-reload"))]
impl HotReloadState {
    /// 创建新的热重载状态（热重载未启用）
    pub fn new() -> Self {
        Self {
            enabled: false,
            last_reload: Duration::from_secs(0),
            cooldown_ms: 500,
        }
    }
}

impl Default for HotReloadState {
    fn default() -> Self {
        #[cfg(feature = "hot-reload")]
        {
            let (state, _) = Self::new();
            state
        }
        #[cfg(not(feature = "hot-reload"))]
        {
            Self::new()
        }
    }
}

/// 热重载插件
pub struct HotReloadPlugin;

impl Plugin for HotReloadPlugin {
    fn build(&self, app: &mut App) {
        // 检查环境变量是否启用热重载
        let enabled = std::env::var("MING_RPG_HOT_RELOAD")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if !enabled {
            info!("热重载已禁用（设置 MING_RPG_HOT_RELOAD=1 启用）");
            return;
        }

        #[cfg(feature = "hot-reload")]
        {
            use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

            let (mut state, tx) = HotReloadState::new();
            state.enabled = true;

            // 创建文件监听器
            let watcher = RecommendedWatcher::new(
                move |res: Result<notify::Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = tx.send(event);
                    }
                },
                Config::default(),
            );

            match watcher {
                Ok(mut watcher) => {
                    // 监听 game/ 目录
                    let paths = ["game/", "assets/"];
                    for path in &paths {
                        if let Err(e) = watcher.watch(path.as_ref(), RecursiveMode::Recursive) {
                            warn!("无法监听目录 {}: {}", path, e);
                        } else {
                            info!("正在监听目录: {}", path);
                        }
                    }

                    // 保存watcher防止被drop
                    app.insert_resource(HotReloadWatcher(watcher));
                    app.insert_resource(state);
                    app.add_event::<HotReloadEvent>();
                    app.add_systems(Update, (check_file_changes, handle_hot_reload).chain());
                    info!("热重载系统已启动");
                }
                Err(e) => {
                    error!("无法创建文件监听器: {}", e);
                    warn!("热重载功能不可用");
                }
            }
        }

        #[cfg(not(feature = "hot-reload"))]
        {
            warn!("热重载功能未编译（使用 --features hot-reload 启用）");
        }

        // 无论是否启用文件监听，都添加手动重载支持
        app.add_event::<HotReloadEvent>();
        app.add_systems(Update, manual_reload_system);
    }
}

/// 保存watcher防止被drop
#[cfg(feature = "hot-reload")]
#[derive(Resource)]
struct HotReloadWatcher(#[allow(dead_code)] RecommendedWatcher);

#[cfg(feature = "hot-reload")]
use notify::RecommendedWatcher;

/// 检查文件变化
#[cfg(feature = "hot-reload")]
fn check_file_changes(
    mut state: ResMut<HotReloadState>,
    mut events: EventWriter<HotReloadEvent>,
    time: Res<Time>,
) {
    if !state.enabled {
        return;
    }

    // 防抖检查
    let elapsed = time.elapsed();
    if elapsed.saturating_sub(state.last_reload).as_millis() < state.cooldown_ms as u128 {
        return;
    }

    // 接收文件变化事件
    let receiver = state.file_receiver.clone();
    if let Ok(receiver) = receiver.lock() {
        while let Ok(event) = receiver.try_recv() {
            for path in &event.paths {
                let path_str = path.to_string_lossy();

                // 只处理.lua和.toml文件
                if path_str.ends_with(".lua") {
                    info!("检测到Lua脚本变化: {}", path.display());
                    events.send(HotReloadEvent::LuaScriptChanged(path.clone()));
                    state.last_reload = elapsed;
                } else if path_str.ends_with(".toml") {
                    info!("检测到配置文件变化: {}", path.display());
                    events.send(HotReloadEvent::ConfigChanged(path.clone()));
                    state.last_reload = elapsed;
                } else if path_str.ends_with(".png")
                    || path_str.ends_with(".jpg")
                    || path_str.ends_with(".gltf")
                    || path_str.ends_with(".glb")
                {
                    info!("检测到资源文件变化: {}", path.display());
                    events.send(HotReloadEvent::AssetChanged(path.clone()));
                    state.last_reload = elapsed;
                }
            }
        }
    }
}

/// 手动触发重载（F5键）
fn manual_reload_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<HotReloadEvent>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        info!("手动触发重载 (F5)");
        events.send(HotReloadEvent::ManualReload);
    }
}

/// 处理热重载事件
#[cfg(feature = "hot-reload")]
fn handle_hot_reload(
    mut events: EventReader<HotReloadEvent>,
    lua: Res<LuaRuntime>,
    mut asset_manager: ResMut<crate::asset_manager::AssetManager>,
    asset_server: Res<AssetServer>,
    _entity_registry: ResMut<EntityRegistry>,
) {
    for event in events.read() {
        match event {
            HotReloadEvent::LuaScriptChanged(path) => {
                info!("重载Lua脚本: {}", path.display());

                // 重新加载主脚本
                if path.to_string_lossy().contains("main.lua") {
                    if let Err(e) = lua.load_main_script("game/main.lua") {
                        error!("重载主脚本失败: {}", e);
                    } else {
                        info!("✓ 主脚本重载成功");
                    }
                }

                // 重新加载配置
                if path.to_string_lossy().contains("config/") {
                    if let Err(e) = reload_configs(&lua) {
                        error!("重载配置失败: {}", e);
                    } else {
                        info!("✓ 配置重载成功");
                    }
                }
            }
            HotReloadEvent::ConfigChanged(path) => {
                info!("重载配置: {}", path.display());
                if let Err(e) = reload_configs(&lua) {
                    error!("重载配置失败: {}", e);
                } else {
                    info!("✓ 配置重载成功");
                }
            }
            HotReloadEvent::AssetChanged(path) => {
                let relative = path
                    .strip_prefix(std::env::current_dir().unwrap().join("assets"))
                    .unwrap_or(path);
                let path_str = relative.to_string_lossy().to_string();
                info!("重载资源: {}", path.display());
                asset_manager.reload(&path_str, &asset_server);
                info!("✓ 资源已重新加载: {}", path.display());
            }
            HotReloadEvent::ManualReload => {
                info!("执行手动重载...");
                if let Err(e) = lua.load_main_script("game/main.lua") {
                    error!("重载主脚本失败: {}", e);
                } else {
                    info!("✓ 主脚本重载成功");
                }
                if let Err(e) = reload_configs(&lua) {
                    error!("重载配置失败: {}", e);
                } else {
                    info!("✓ 配置重载成功");
                }
            }
        }
    }
}

/// 重新加载所有配置
#[cfg(feature = "hot-reload")]
fn reload_configs(lua: &LuaRuntime) -> anyhow::Result<()> {
    // 触发Lua重新加载配置
    lua.execute(
        r#"
        -- 重新加载所有配置模块
        package.loaded["config/game"] = nil
        package.loaded["config/player"] = nil
        package.loaded["config/camera"] = nil
        package.loaded["config/colors"] = nil
        package.loaded["config/scenes"] = nil

        -- 重新加载
        local game_config = require("config/game")
        local player_config = require("config/player")
        local camera_config = require("config/camera")
        local colors_config = require("config/colors")
        local scenes_config = require("config/scenes")

        -- 更新全局变量
        GAME_CONFIG = game_config
        PLAYER_CONFIG = {
            model_scene = player_config.model.scene,
            scale = player_config.model.scale,
            base_height = player_config.model.base_height,
            yaw_offset = player_config.model.yaw_offset,
        }
        PLAYER_MOVEMENT = player_config.movement
        WALK_ANIMATION = player_config.animation
        CAMERA_CONFIG = camera_config
        SCENE_COLORS = colors_config
        SCENE_CONFIG = scenes_config

        log_info("配置已热重载")
    "#,
    )?;

    Ok(())
}
