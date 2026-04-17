//! 游戏常量
//!
//! 集中管理所有游戏参数，方便调整平衡性

use bevy::prelude::Color;

// 玩家相关
pub const PLAYER_SPEED: f32 = 5.0;
pub const PLAYER_ID: &str = "player";
pub const PLAYER_MODEL_SCENE: &str = "models/zhuang_fangyi__arknights_endfield/scene.gltf#Scene0";
pub const PLAYER_MODEL_YAW_OFFSET: f32 = 0.0;
pub const PLAYER_MODEL_SCALE: f32 = 1.0;
pub const PLAYER_BASE_HEIGHT: f32 = 0.0;

// 玩家碰撞体尺寸
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.35;
pub const PLAYER_COLLIDER_HEIGHT: f32 = 0.9;

// 相机相关
pub const CAMERA_DISTANCE: f32 = 20.0;
pub const CAMERA_SMOOTH_FACTOR: f32 = 0.1;
pub const CAMERA_PITCH_MIN: f32 = 10.0f32.to_radians();
pub const CAMERA_PITCH_MAX: f32 = 80.0f32.to_radians();
pub const CAMERA_MOUSE_FOLLOW_SPEED: f32 = 2.0; // 鼠标跟随旋转速度（弧度/秒）
pub const CAMERA_EDGE_THRESHOLD: f32 = 0.15; // 边缘触发区域比例（15%）

// 动画相关
pub const WALK_BOB_AMPLITUDE: f32 = 0.08;
pub const WALK_BOB_SPEED: f32 = 10.0;
pub const WALK_BOB_RECOVER_SPEED: f32 = 8.0;
pub const ROTATION_SPEED: f32 = 10.0;

// 场景相关
pub const GROUND_SIZE: f32 = 20.0;
pub const WALL_SIZE: f32 = 2.0;
pub const ROOF_SIZE: f32 = 2.2;
pub const TREE_SIZE: f32 = 1.5;

// 颜色
pub const WALL_COLOR: Color = Color::rgb(0.86, 0.24, 0.18);
pub const ROOF_COLOR: Color = Color::rgb(0.9, 0.8, 0.1);
pub const TREE_COLOR: Color = Color::rgb(0.2, 0.6, 0.2);
pub const GROUND_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);

// 热重载
pub const HOT_RELOAD_INTERVAL: f32 = 0.5;
