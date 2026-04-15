//! 明朝修仙 RPG - 主入口
//!
//! 核心原则：
//! 1. 简单明确 - 每个模块只做一件事
//! 2. 错误透明 - 用 Result 传播错误，不 panic
//! 3. 可测试 - 核心业务逻辑独立可测

use bevy::log::LogPlugin;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use tracing::{error, info, warn};

mod core;
mod lua_api;

use lua_api::{LuaCommand, LuaRuntime};

const PLAYER_SPEED: f32 = 260.0;
const PLAYER_ID: &str = "player";

#[derive(Component)]
struct Player;

#[derive(Default, Resource)]
struct EntityRegistry {
    by_id: HashMap<String, Entity>,
    components: HashMap<String, HashMap<String, serde_json::Value>>,
}

#[derive(Resource)]
struct ScriptHotReload {
    script_path: String,
    last_modified: SystemTime,
    check_timer: Timer,
}

impl ScriptHotReload {
    fn new(script_path: &str) -> Self {
        let last_modified = get_last_modified(script_path).unwrap_or(SystemTime::UNIX_EPOCH);
        Self {
            script_path: script_path.to_string(),
            last_modified,
            check_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GameConfig {
    version: String,
    start_year: i32,
    time_scale: f32,
}

fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,ming_rpg=debug")
        .init();

    info!("启动 明朝修仙 RPG...");

    // 加载配置资源
    let config = load_game_config("game/config.toml")?;
    info!(
        "配置加载完成 version={} start_year={} time_scale={}",
        config.version, config.start_year, config.time_scale
    );

    // 初始化 Lua 运行时
    let lua = LuaRuntime::new()?;
    info!("Lua 运行时初始化完成");

    // 加载主脚本
    lua.load_main_script("game/main.lua")?;
    info!("主脚本加载完成");

    // 启动 Bevy 应用
    // 注意：禁用 LogPlugin，因为我们已经在上面手动初始化了 tracing_subscriber
    App::new()
        .insert_non_send_resource(lua)
        .insert_resource(EntityRegistry::default())
        .insert_resource(ScriptHotReload::new("game/main.lua"))
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "ming-rpg Phase 1".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_input_system,
                lua_update_system,
                apply_lua_commands_system,
                sync_entity_positions_to_lua_system,
                hot_reload_lua_script_system,
            ),
        )
        .run();

    Ok(())
}

fn load_game_config<P: AsRef<Path>>(path: P) -> anyhow::Result<GameConfig> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("读取配置文件 {:?} 失败: {}", path, e))?;
    toml::from_str(&content).map_err(|e| anyhow::anyhow!("解析配置文件 {:?} 失败: {}", path, e))
}

fn setup(mut commands: Commands, mut registry: ResMut<EntityRegistry>, lua: NonSend<LuaRuntime>) {
    info!("Bevy 初始化完成");

    commands.spawn(Camera2dBundle::default());
    let player = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.86, 0.24, 0.18),
                    custom_size: Some(Vec2::new(64.0, 64.0)),
                    ..default()
                },
                ..default()
            },
            Player,
        ))
        .id();

    registry.by_id.insert(PLAYER_ID.to_string(), player);
    lua.update_entity_position(PLAYER_ID, Vec3::ZERO);

    info!("2D Sprite 场景创建完成");
}

fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction == Vec2::ZERO {
        return;
    }

    let delta = direction.normalize_or_zero() * PLAYER_SPEED * time.delta_seconds();
    for mut transform in &mut query {
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
    }
}

fn lua_update_system(lua: NonSend<LuaRuntime>, time: Res<Time>) {
    if let Err(err) = lua.call_function::<_, ()>("update", time.delta_seconds()) {
        error!("Lua update 调用失败: {}", err);
    }
}

fn apply_lua_commands_system(
    mut commands: Commands,
    lua: NonSend<LuaRuntime>,
    mut registry: ResMut<EntityRegistry>,
    mut query: Query<&mut Transform>,
) {
    for command in lua.drain_commands() {
        match command {
            LuaCommand::CreateEntity { id, entity_type } => {
                let color = match entity_type.as_str() {
                    "npc" => Color::rgb(0.2, 0.5, 0.9),
                    "effect" => Color::rgb(0.9, 0.8, 0.1),
                    _ => Color::rgb(0.4, 0.4, 0.4),
                };

                let entity = commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::new(42.0, 42.0)),
                            ..default()
                        },
                        ..default()
                    })
                    .id();

                registry.by_id.insert(id.clone(), entity);
                lua.update_entity_position(&id, Vec3::ZERO);
                info!("Lua 创建实体成功: {}", id);
            }
            LuaCommand::DestroyEntity { id } => {
                lua.remove_entity_position(&id);
                if let Some(entity) = registry.by_id.remove(&id) {
                    commands.entity(entity).despawn_recursive();
                    registry.components.remove(&id);
                    info!("Lua 销毁实体成功: {}", id);
                } else {
                    warn!("Lua 请求销毁未知实体: {}", id);
                }
            }
            LuaCommand::SetPosition { id, x, y, z } => {
                if let Some(entity) = registry.by_id.get(&id)
                    && let Ok(mut transform) = query.get_mut(*entity)
                {
                    transform.translation = Vec3::new(x, y, z);
                }
            }
            LuaCommand::AddComponent { id, name, value } => {
                registry
                    .components
                    .entry(id)
                    .or_default()
                    .insert(name, value);
            }
            LuaCommand::RemoveComponent { id, name } => {
                if let Some(components) = registry.components.get_mut(&id) {
                    components.remove(&name);
                }
            }
        }
    }
}

fn sync_entity_positions_to_lua_system(
    lua: NonSend<LuaRuntime>,
    registry: Res<EntityRegistry>,
    query: Query<&Transform>,
) {
    for (id, entity) in &registry.by_id {
        if let Ok(transform) = query.get(*entity) {
            lua.update_entity_position(id, transform.translation);
        }
    }
}

fn hot_reload_lua_script_system(
    time: Res<Time>,
    lua: NonSend<LuaRuntime>,
    mut hot_reload: ResMut<ScriptHotReload>,
) {
    if !hot_reload.check_timer.tick(time.delta()).just_finished() {
        return;
    }

    let Some(current_modified) = get_last_modified(&hot_reload.script_path) else {
        return;
    };

    if current_modified <= hot_reload.last_modified {
        return;
    }

    match lua.load_main_script(&hot_reload.script_path) {
        Ok(()) => {
            hot_reload.last_modified = current_modified;
            info!("Lua 脚本已热重载: {}", hot_reload.script_path);
        }
        Err(err) => {
            error!("Lua 热重载失败: {}", err);
        }
    }
}

fn get_last_modified(path: &str) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
