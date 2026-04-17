//! 游戏常量
//!
//! 技术限制和默认值，游戏性配置请使用 Lua 配置

use bevy::prelude::Color;

// 内部标识（硬编码，不可配置）
pub const PLAYER_ID: &str = "player";

// ===== 物理碰撞尺寸（与模型尺寸强关联） =====
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.35;
pub const PLAYER_COLLIDER_HEIGHT: f32 = 0.9;

// ===== 相机技术限制（防止穿墙/翻转） =====
pub const CAMERA_DISTANCE_MIN: f32 = 5.0;
pub const CAMERA_DISTANCE_MAX: f32 = 40.0;
pub const CAMERA_PITCH_MIN: f32 = 10.0f32.to_radians();
pub const CAMERA_PITCH_MAX: f32 = 80.0f32.to_radians();

// ===== 输入响应（引擎级手感） =====
pub const CAMERA_ZOOM_SPEED: f32 = 2.0;
pub const CAMERA_MOUSE_SENSITIVITY: f32 = 0.001;

// ===== 默认值（当 Lua 配置缺失时使用） =====
// 玩家默认
pub const PLAYER_SPEED: f32 = 5.0;
pub const PLAYER_MODEL_SCENE: &str = "models/fox-eared_game_endfield/scene.gltf#Scene0";
pub const PLAYER_MODEL_YAW_OFFSET: f32 = 0.0;
pub const PLAYER_MODEL_SCALE: f32 = 1.0;
pub const PLAYER_BASE_HEIGHT: f32 = 1.0;

// 相机默认（仅用于默认值，实际值从 Lua 读取）
pub const CAMERA_DISTANCE_DEFAULT: f32 = 20.0;
pub const CAMERA_SMOOTH_FACTOR_DEFAULT: f32 = 0.1;

// 动画默认
pub const WALK_BOB_AMPLITUDE: f32 = 0.08;
pub const WALK_BOB_SPEED: f32 = 10.0;
pub const WALK_BOB_RECOVER_SPEED: f32 = 8.0;
pub const ROTATION_SPEED: f32 = 10.0;

// ===== 场景（后续迁移到 Lua 关卡配置） =====
pub const GROUND_SIZE: f32 = 20.0;
pub const WALL_SIZE: f32 = 2.0;
pub const ROOF_SIZE: f32 = 2.2;
pub const TREE_SIZE: f32 = 1.5;

pub const WALL_COLOR: Color = Color::rgb(0.86, 0.24, 0.18);
pub const ROOF_COLOR: Color = Color::rgb(0.9, 0.8, 0.1);
pub const TREE_COLOR: Color = Color::rgb(0.2, 0.6, 0.2);
pub const GROUND_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

// 热重载
pub const HOT_RELOAD_INTERVAL: f32 = 0.5;
