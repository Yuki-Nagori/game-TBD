//! 场景插件
//!
//! 场景初始化、方块建筑系统、Lua 热重载

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::constants::*;
use crate::lua_api::{LuaCommand, LuaRuntime};
use crate::resources::{EntityRegistry, ScriptHotReload};
use crate::utils::get_last_modified;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityRegistry>()
            .insert_resource(ScriptHotReload::new("game/main.lua"))
            .add_systems(Startup, spawn_scene)
            .add_systems(
                Update,
                (
                    lua_update_system,
                    apply_lua_commands_system,
                    sync_entity_positions_to_lua_system,
                    hot_reload_lua_script_system,
                ),
            );
    }
}

/// 生成场景：地面、光照、方块建筑占位
fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("初始化 3D 场景");

    // 光照
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        ..default()
    });

    // 地面（带碰撞）
    let ground_mesh = meshes.add(Plane3d::default().mesh().size(GROUND_SIZE, GROUND_SIZE));
    commands.spawn((
        PbrBundle {
            mesh: ground_mesh,
            material: materials.add(GROUND_COLOR),
            ..default()
        },
        Collider::cuboid(GROUND_SIZE / 2.0, 0.1, GROUND_SIZE / 2.0),
    ));

    // 方块建筑系统：简单建筑占位
    spawn_building_blocks(&mut commands, &mut meshes, &mut materials);

    info!("场景创建完成");
}

/// 生成方块建筑占位
fn spawn_building_blocks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let wall_mesh = meshes.add(Cuboid::new(WALL_SIZE, WALL_SIZE, WALL_SIZE));
    let roof_mesh = meshes.add(Cuboid::new(ROOF_SIZE, ROOF_SIZE / 2.0, ROOF_SIZE));
    let tree_mesh = meshes.add(Cuboid::new(TREE_SIZE, TREE_SIZE * 2.0, TREE_SIZE));

    // 红色墙面（四个方块围成方形）
    let wall_positions = [
        Vec3::new(8.0, WALL_SIZE / 2.0, 0.0),
        Vec3::new(-8.0, WALL_SIZE / 2.0, 0.0),
        Vec3::new(0.0, WALL_SIZE / 2.0, 8.0),
        Vec3::new(0.0, WALL_SIZE / 2.0, -8.0),
    ];
    for pos in wall_positions {
        commands.spawn((
            PbrBundle {
                mesh: wall_mesh.clone(),
                material: materials.add(WALL_COLOR),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Collider::cuboid(WALL_SIZE / 2.0, WALL_SIZE / 2.0, WALL_SIZE / 2.0),
        ));
    }

    // 黄色屋顶
    commands.spawn((
        PbrBundle {
            mesh: roof_mesh,
            material: materials.add(ROOF_COLOR),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                WALL_SIZE + ROOF_SIZE / 4.0,
                0.0,
            )),
            ..default()
        },
        Collider::cuboid(ROOF_SIZE / 2.0, ROOF_SIZE / 4.0, ROOF_SIZE / 2.0),
    ));

    // 绿色树木
    let tree_positions = [
        Vec3::new(5.0, TREE_SIZE, 5.0),
        Vec3::new(-5.0, TREE_SIZE, -5.0),
        Vec3::new(5.0, TREE_SIZE, -5.0),
    ];
    for pos in tree_positions {
        commands.spawn((
            PbrBundle {
                mesh: tree_mesh.clone(),
                material: materials.add(TREE_COLOR),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Collider::cuboid(TREE_SIZE / 2.0, TREE_SIZE, TREE_SIZE / 2.0),
        ));
    }
}

/// Lua 更新系统
fn lua_update_system(lua: NonSend<LuaRuntime>, time: Res<Time>) {
    use tracing::error;

    if let Err(err) = lua.call_function::<_, ()>("update", time.delta_seconds()) {
        error!("Lua update 调用失败: {}", err);
    }
}

/// 应用 Lua 命令系统
fn apply_lua_commands_system(
    mut commands: Commands,
    lua: NonSend<LuaRuntime>,
    mut registry: ResMut<EntityRegistry>,
    mut query: Query<&mut Transform>,
) {
    use tracing::{info, warn};

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

/// 同步实体位置到 Lua
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

/// Lua 脚本热重载系统
fn hot_reload_lua_script_system(
    time: Res<Time>,
    mut commands: Commands,
    mut registry: ResMut<EntityRegistry>,
    lua: NonSend<LuaRuntime>,
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
