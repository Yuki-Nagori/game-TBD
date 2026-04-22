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
    /// 模型场景路径
    #[serde(rename = "model_scene")]
    pub model_scene: String,
    /// 模型缩放
    #[serde(rename = "scale")]
    pub scale: f32,
    /// 基础高度
    #[serde(rename = "base_height")]
    pub base_height: f32,
    /// 偏航角偏移
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
    /// 移动速度
    #[serde(rename = "speed")]
    pub speed: f32,
    /// 旋转速度
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
#[allow(dead_code)]
pub struct WalkAnimationConfig {
    /// 行走振幅
    #[serde(rename = "bob_amplitude")]
    pub bob_amplitude: f32,
    /// 行走速度
    #[serde(rename = "bob_speed")]
    pub bob_speed: f32,
    /// 恢复速度
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
    /// 玩家配置
    pub player: PlayerConfig,
    /// 移动配置
    pub movement: PlayerMovementConfig,
    /// 动画配置
    pub animation: WalkAnimationConfig,
}

/// 玩家插件
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRuntimeConfig>()
            .init_resource::<CachedCameraDirection>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    player_input_system,
                    invalidate_camera_cache_system,
                    player_animation_system,
                ),
            );
    }
}

/// 生成玩家实体
fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<EntityRegistry>,
    mut runtime_config: ResMut<PlayerRuntimeConfig>,
    lua: Res<LuaRuntime>,
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

    // 尝试加载模型，失败时使用降级方案（胶囊体占位）
    let player_entity = spawn_player_with_fallback(&mut commands, &asset_server, &player_config);

    registry.by_id.insert(PLAYER_ID.to_string(), player_entity);

    // 计算实际的出生高度：确保胶囊体底部略高于地面，避免卡住
    // 胶囊体半高为 PLAYER_COLLIDER_HEIGHT，半径为 PLAYER_COLLIDER_RADIUS
    // 底部在 y - (PLAYER_COLLIDER_HEIGHT + PLAYER_COLLIDER_RADIUS)，需要保证 > 0
    let spawn_height = player_config
        .base_height
        .max(PLAYER_COLLIDER_HEIGHT + PLAYER_COLLIDER_RADIUS + 0.1);
    lua.update_entity_position(PLAYER_ID, Vec3::new(0.0, spawn_height, 0.0));

    info!("玩家实体创建完成（位置: y={}）", spawn_height);
}

/// 生成玩家实体，支持模型加载失败的降级方案
fn spawn_player_with_fallback(
    commands: &mut Commands,
    asset_server: &AssetServer,
    config: &PlayerConfig,
) -> Entity {
    // 尝试加载模型资源
    let scene_handle: Handle<Scene> = asset_server.load(&config.model_scene);

    // 计算出生高度：确保胶囊体底部略高于地面，避免卡住
    // 胶囊体半高为 PLAYER_COLLIDER_HEIGHT，半径为 PLAYER_COLLIDER_RADIUS
    // 底部在 y - (PLAYER_COLLIDER_HEIGHT + PLAYER_COLLIDER_RADIUS)，需要保证 > 0
    let spawn_height = config
        .base_height
        .max(PLAYER_COLLIDER_HEIGHT + PLAYER_COLLIDER_RADIUS + 0.1);

    commands
        .spawn(SceneBundle {
            scene: scene_handle,
            transform: Transform::from_xyz(0.0, spawn_height, 0.0)
                .with_scale(Vec3::splat(config.scale)),
            ..default()
        })
        .insert(Player)
        .insert(CharacterMotion::default())
        .insert(PlaceholderWalkAnimation::new(spawn_height))
        // 物理组件：使用 Dynamic 刚体配合速度控制，防止穿墙
        .insert(RigidBody::Dynamic)
        .insert(Collider::capsule_y(
            PLAYER_COLLIDER_HEIGHT,
            PLAYER_COLLIDER_RADIUS,
        ))
        // 锁定旋转，防止玩家翻滚
        .insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z)
        // 质量设置（使用 AdditionalMassProperties）
        .insert(AdditionalMassProperties::Mass(70.0))
        // 阻尼设置，防止滑行过久
        .insert(Damping {
            linear_damping: 5.0,
            angular_damping: 1.0,
        })
        // 使用速度控制移动，而非直接设置位置（防止穿墙）
        .insert(Velocity::default())
        // 启用 CCD（连续碰撞检测），防止高速移动穿墙
        .insert(Ccd::enabled())
        .id()
}

/// 缓存的相机方向资源
/// 用于优化性能，避免每帧重复计算
#[derive(Resource, Default)]
pub struct CachedCameraDirection {
    /// 相机前方向量
    pub forward: Vec3,
    /// 相机右方向量
    pub right: Vec3,
    /// 缓存是否有效
    pub is_valid: bool,
}

/// 玩家输入处理系统
///
/// - WASD/方向键：移动（使用速度控制，带碰撞检测）
/// - 移动方向决定人物朝向（独立于相机）
/// - 性能优化：相机方向在查询外计算并缓存
pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera_query: Query<&Transform, (With<crate::components::ThirdPersonCamera>, Without<Player>)>,
    mut query: Query<(&mut Transform, &mut CharacterMotion, &mut Velocity), With<Player>>,
    runtime_config: Res<PlayerRuntimeConfig>,
    mut cached_direction: ResMut<CachedCameraDirection>,
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

    // 性能优化：计算并缓存相机方向（在查询循环外）
    // 只有当缓存无效或相机变化时才重新计算
    if !cached_direction.is_valid {
        let camera_forward_raw = camera_transform.rotation * -Vec3::Z;
        cached_direction.forward =
            Vec3::new(camera_forward_raw.x, 0.0, camera_forward_raw.z).normalize_or_zero();

        let camera_right_raw = camera_transform.rotation * Vec3::X;
        cached_direction.right =
            Vec3::new(camera_right_raw.x, 0.0, camera_right_raw.z).normalize_or_zero();

        cached_direction.is_valid = true;
    }

    let direction =
        (cached_direction.forward * input.y + cached_direction.right * input.x).normalize_or_zero();

    let is_moving = direction != Vec3::ZERO;

    for (mut transform, mut motion, mut velocity) in &mut query {
        motion.is_moving = is_moving;

        if is_moving {
            // 使用速度控制移动（防止穿墙）
            let target_velocity = direction * runtime_config.movement.speed;
            velocity.linvel.x = target_velocity.x;
            velocity.linvel.z = target_velocity.z;

            // 更新人物朝向为移动方向
            motion.facing_yaw = direction.x.atan2(direction.z) + runtime_config.player.yaw_offset;
        }

        // 始终应用人物朝向（平滑旋转）
        let target_rotation = Quat::from_rotation_y(motion.facing_yaw);
        transform.rotation = transform.rotation.slerp(
            target_rotation,
            runtime_config.movement.rotation_speed * time.delta_seconds(),
        );
    }
}

/// 重置相机方向缓存系统
/// 在相机变化时调用，使缓存失效
pub fn invalidate_camera_cache_system(
    camera_query: Query<
        &Transform,
        (
            With<crate::components::ThirdPersonCamera>,
            Changed<Transform>,
        ),
    >,
    mut cached_direction: ResMut<CachedCameraDirection>,
) {
    if camera_query.get_single().is_ok() {
        cached_direction.is_valid = false;
    }
}

/// 玩家行走动画系统（已禁用 - 和物理引擎冲突）
///
/// 原实现直接修改 Y 轴位置，但 Dynamic 刚体由物理引擎控制位置，
/// 两者冲突会导致角色悬浮或抖动。
/// 后续需要改为视觉层动画（如模型缩放/旋转）而非位置修改。
fn player_animation_system(
    _time: Res<Time>,
    mut _query: Query<
        (
            &mut Transform,
            &CharacterMotion,
            &mut PlaceholderWalkAnimation,
        ),
        With<Player>,
    >,
    _runtime_config: Res<PlayerRuntimeConfig>,
) {
    // 暂时禁用 - 直接修改 Y 位置会和 Dynamic 刚体的物理引擎冲突
    // 角色需要落在地面上，由物理引擎控制位置
}
