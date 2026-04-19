//! Lua 命令处理插件
//!
//! 将 Lua 命令处理从 ScenePlugin 中分离，独立管理
//! 职责：
//! - 处理 Lua 发出的命令（创建/销毁实体、设置位置等）
//! - 同步实体位置到 Lua
//! - Lua 脚本热重载

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::constants::PLAYER_ID;
use crate::lua_api::{LuaCommand, LuaRuntime};
use crate::resources::{EntityRegistry, ScriptHotReload};
use crate::utils::get_last_modified;

pub struct LuaCommandPlugin;

impl Plugin for LuaCommandPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScriptHotReload::new("game/main.lua"))
            .add_systems(
                Update,
                (
                    lua_update_system,
                    apply_lua_commands_system,
                    sync_entity_positions_to_lua_system,
                    hot_reload_lua_script_system,
                )
                    .chain(),
            );
    }
}

/// Lua update 系统
/// 每帧调用 Lua 的 update 函数
fn lua_update_system(lua: Res<LuaRuntime>, time: Res<Time>) {
    use tracing::error;

    if let Err(err) = lua.call_function("update", time.delta_seconds()) {
        error!("Lua update 调用失败: {}", err);
    }
}

/// 应用 Lua 命令系统
/// 处理 Lua 发出的所有命令
fn apply_lua_commands_system(
    mut commands: Commands,
    lua: Res<LuaRuntime>,
    mut registry: ResMut<EntityRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<&mut Transform>,
) {
    use tracing::{info, warn};

    for command in lua.drain_commands() {
        match command {
            LuaCommand::CreateEntity { id, entity_type } => {
                // 3D 实体创建（替换原有的 2D SpriteBundle）
                let (color, size) = match entity_type.as_str() {
                    "npc" => (Color::srgb(0.2, 0.5, 0.9), 1.0),
                    "effect" => (Color::srgb(0.9, 0.8, 0.1), 0.5),
                    _ => (Color::srgb(0.4, 0.4, 0.4), 0.8),
                };

                let mesh = meshes.add(Cuboid::new(size, size, size));
                let material = materials.add(color);

                let entity = commands
                    .spawn((
                        PbrBundle {
                            mesh,
                            material,
                            transform: Transform::from_translation(Vec3::ZERO),
                            ..default()
                        },
                        // 添加碰撞体
                        Collider::cuboid(size / 2.0, size / 2.0, size / 2.0),
                        RigidBody::Fixed,
                    ))
                    .id();

                registry.by_id.insert(id.clone(), entity);
                lua.update_entity_position(&id, Vec3::ZERO);
                info!("Lua 创建 3D 实体成功: {} (类型: {})", id, entity_type);
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

/// 同步实体位置到 Lua
/// 将游戏中所有实体位置同步给 Lua
fn sync_entity_positions_to_lua_system(
    lua: Res<LuaRuntime>,
    registry: Res<EntityRegistry>,
    query: Query<&Transform>,
) {
    for (id, entity) in &registry.by_id {
        if let Ok(transform) = query.get(*entity) {
            lua.update_entity_position(id, transform.translation);
        }
    }
}

/// Lua 脚本热重载系统
/// 监控脚本文件变化，自动重新加载
fn hot_reload_lua_script_system(
    time: Res<Time>,
    mut commands: Commands,
    mut registry: ResMut<EntityRegistry>,
    lua: Res<LuaRuntime>,
    mut hot_reload: ResMut<ScriptHotReload>,
) {
    use tracing::{error, info};

    if !hot_reload.check_timer.tick(time.delta()).just_finished() {
        return;
    }

    let Some(current_modified) = get_last_modified(&hot_reload.script_path) else {
        return;
    };

    if current_modified <= hot_reload.last_modified {
        return;
    }

    // 清理脚本管理的实体（保留玩家）
    let ids_to_remove: Vec<String> = registry
        .by_id
        .keys()
        .filter(|id| id.as_str() != PLAYER_ID)
        .cloned()
        .collect();

    for id in ids_to_remove {
        if let Some(entity) = registry.by_id.remove(&id) {
            commands.entity(entity).despawn_recursive();
            registry.components.remove(&id);
        }
        lua.remove_entity_position(&id);
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
