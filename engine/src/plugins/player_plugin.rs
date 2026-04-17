//! 玩家插件
//!
//! 处理玩家输入、移动、朝向和动画

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{CharacterMotion, PlaceholderWalkAnimation, Player};
use crate::constants::*;
use crate::lua_api::LuaRuntime;
use crate::resources::EntityRegistry;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_input_system, player_animation_system));
    }
}

/// 生成玩家实体
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<EntityRegistry>,
    lua: NonSend<LuaRuntime>,
) {
    let player_scene: Handle<Scene> = asset_server.load(PLAYER_MODEL_SCENE);
    let player = commands
        .spawn(SceneBundle {
            scene: player_scene,
            transform: Transform::from_xyz(0.0, PLAYER_BASE_HEIGHT, 0.0)
                .with_scale(Vec3::splat(PLAYER_MODEL_SCALE)),
            ..default()
        })
        .insert(Player)
        .insert(CharacterMotion::default())
        .insert(PlaceholderWalkAnimation::new(PLAYER_BASE_HEIGHT))
        // 物理组件
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule_y(0.9, 0.35))
        .insert(KinematicCharacterController::default())
        .id();

    registry.by_id.insert(PLAYER_ID.to_string(), player);
    lua.update_entity_position(PLAYER_ID, Vec3::new(0.0, PLAYER_BASE_HEIGHT, 0.0));

    info!("玩家实体创建完成（含物理碰撞）");
}

/// 玩家输入处理系统
///
/// - WASD/方向键：移动（带碰撞检测）
/// - 移动方向决定人物朝向（独立于相机）
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera_query: Query<&Transform, (With<crate::components::ThirdPersonCamera>, Without<Player>)>,
    mut query: Query<
        (
            &mut Transform,
            &mut CharacterMotion,
            &mut KinematicCharacterController,
        ),
        With<Player>,
    >,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    // 读取输入
    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    // 使用相机朝向映射输入：上前下后、左右平移
    let camera_forward = {
        let f = camera_transform.rotation * -Vec3::Z;
        Vec3::new(f.x, 0.0, f.z).normalize_or_zero()
    };
    let camera_right = {
        let r = camera_transform.rotation * Vec3::X;
        Vec3::new(r.x, 0.0, r.z).normalize_or_zero()
    };
    let direction = (camera_forward * input.y - camera_right * input.x).normalize_or_zero();

    let is_moving = direction != Vec3::ZERO;

    for (mut transform, mut motion, mut controller) in &mut query {
        motion.is_moving = is_moving;

        if is_moving {
            // 计算移动速度
            let velocity = direction * PLAYER_SPEED;
            controller.translation = Some(velocity * time.delta_seconds());

            // 更新人物朝向为移动方向（独立于相机）
            motion.facing_yaw = direction.x.atan2(direction.z) + PLAYER_MODEL_YAW_OFFSET;
        } else {
            controller.translation = None;
        }

        // 始终应用人物朝向（平滑旋转）
        let target_rotation = Quat::from_rotation_y(motion.facing_yaw);
        transform.rotation = transform
            .rotation
            .slerp(target_rotation, ROTATION_SPEED * time.delta_seconds());
    }
}

/// 玩家行走动画系统（占位实现）
///
/// 移动时产生上下起伏，停止时平滑恢复
fn player_animation_system(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &CharacterMotion,
            &mut PlaceholderWalkAnimation,
        ),
        With<Player>,
    >,
) {
    for (mut transform, motion, mut walk_anim) in &mut query {
        if motion.is_moving {
            walk_anim.phase += time.delta_seconds() * WALK_BOB_SPEED;
            transform.translation.y =
                walk_anim.base_height + walk_anim.phase.sin() * WALK_BOB_AMPLITUDE;
        } else {
            walk_anim.phase = 0.0;
            let recover = (WALK_BOB_RECOVER_SPEED * time.delta_seconds()).min(1.0);
            transform.translation.y += (walk_anim.base_height - transform.translation.y) * recover;
        }
    }
}
