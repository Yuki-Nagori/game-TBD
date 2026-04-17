//! 玩家插件
//!
//! 处理玩家输入、移动、朝向和动画

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::components::{CharacterMotion, PlaceholderWalkAnimation, Player};
use crate::constants::*;
use crate::lua_api::LuaRuntime;
use crate::resources::EntityRegistry;

/// 玩家配置（从 Lua 读取）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlayerConfig {
    #[serde(rename = "model_scene")]
    pub model_scene: String,
    #[serde(rename = "scale")]
    pub scale: f32,
    #[serde(rename = "base_height")]
    pub base_height: f32,
    #[serde(rename = "yaw_offset")]
    pub yaw_offset: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            model_scene: PLAYER_MODEL_SCENE.to_string(),
            scale: PLAYER_MODEL_SCALE,
            base_height: PLAYER_BASE_HEIGHT,
            yaw_offset: PLAYER_MODEL_YAW_OFFSET,
        }
    }
}

/// 玩家移动配置（从 Lua 读取）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlayerMovementConfig {
    #[serde(rename = "speed")]
    pub speed: f32,
    #[serde(rename = "rotation_speed")]
    pub rotation_speed: f32,
}

impl Default for PlayerMovementConfig {
    fn default() -> Self {
        Self {
            speed: PLAYER_SPEED,
            rotation_speed: ROTATION_SPEED,
        }
    }
}

/// 行走动画配置（从 Lua 读取）
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WalkAnimationConfig {
    #[serde(rename = "bob_amplitude")]
    pub bob_amplitude: f32,
    #[serde(rename = "bob_speed")]
    pub bob_speed: f32,
    #[serde(rename = "recover_speed")]
    pub recover_speed: f32,
}

impl Default for WalkAnimationConfig {
    fn default() -> Self {
        Self {
            bob_amplitude: WALK_BOB_AMPLITUDE,
            bob_speed: WALK_BOB_SPEED,
            recover_speed: WALK_BOB_RECOVER_SPEED,
        }
    }
}

/// 运行时配置资源
#[derive(Default, Resource)]
pub struct PlayerRuntimeConfig {
    pub player: PlayerConfig,
    pub movement: PlayerMovementConfig,
    pub animation: WalkAnimationConfig,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRuntimeConfig>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_input_system, player_animation_system));
    }
}

/// 生成玩家实体
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<EntityRegistry>,
    mut runtime_config: ResMut<PlayerRuntimeConfig>,
    lua: NonSend<LuaRuntime>,
) {
    // 尝试从 Lua 读取配置
    let player_config: PlayerConfig = lua.get_config("PLAYER_CONFIG").unwrap_or_else(|| {
        info!("使用默认玩家配置");
        PlayerConfig::default()
    });
    let movement_config: PlayerMovementConfig =
        lua.get_config("PLAYER_MOVEMENT").unwrap_or_else(|| {
            info!("使用默认移动配置");
            PlayerMovementConfig::default()
        });
    let animation_config: WalkAnimationConfig =
        lua.get_config("WALK_ANIMATION").unwrap_or_else(|| {
            info!("使用默认动画配置");
            WalkAnimationConfig::default()
        });

    info!("玩家配置: {:?}", player_config);

    // 保存到运行时配置
    runtime_config.player = player_config.clone();
    runtime_config.movement = movement_config;
    runtime_config.animation = animation_config.clone();

    let player_scene: Handle<Scene> = asset_server.load(&player_config.model_scene);
    let player = commands
        .spawn(SceneBundle {
            scene: player_scene,
            transform: Transform::from_xyz(0.0, player_config.base_height, 0.0)
                .with_scale(Vec3::splat(player_config.scale)),
            ..default()
        })
        .insert(Player)
        .insert(CharacterMotion::default())
        .insert(PlaceholderWalkAnimation::new(player_config.base_height))
        // 物理组件
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule_y(
            PLAYER_COLLIDER_HEIGHT,
            PLAYER_COLLIDER_RADIUS,
        ))
        .insert(KinematicCharacterController::default())
        .id();

    registry.by_id.insert(PLAYER_ID.to_string(), player);
    lua.update_entity_position(PLAYER_ID, Vec3::new(0.0, player_config.base_height, 0.0));

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
    runtime_config: Res<PlayerRuntimeConfig>,
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
    let direction = (camera_forward * input.y + camera_right * input.x).normalize_or_zero();

    let is_moving = direction != Vec3::ZERO;

    for (mut transform, mut motion, mut controller) in &mut query {
        motion.is_moving = is_moving;

        if is_moving {
            // 计算移动速度（使用 Lua 配置）
            let velocity = direction * runtime_config.movement.speed;
            controller.translation = Some(velocity * time.delta_seconds());

            // 更新人物朝向为移动方向（使用 Lua 配置的 yaw_offset）
            motion.facing_yaw = direction.x.atan2(direction.z) + runtime_config.player.yaw_offset;
        } else {
            controller.translation = None;
        }

        // 始终应用人物朝向（平滑旋转）
        let target_rotation = Quat::from_rotation_y(motion.facing_yaw);
        transform.rotation = transform.rotation.slerp(
            target_rotation,
            runtime_config.movement.rotation_speed * time.delta_seconds(),
        );
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
    runtime_config: Res<PlayerRuntimeConfig>,
) {
    for (mut transform, motion, mut walk_anim) in &mut query {
        if motion.is_moving {
            walk_anim.phase += time.delta_seconds() * runtime_config.animation.bob_speed;
            transform.translation.y = walk_anim.base_height
                + walk_anim.phase.sin() * runtime_config.animation.bob_amplitude;
        } else {
            walk_anim.phase = 0.0;
            let recover = (runtime_config.animation.recover_speed * time.delta_seconds()).min(1.0);
            transform.translation.y += (walk_anim.base_height - transform.translation.y) * recover;
        }
    }
}
