//! 游戏常量
//!
//! 技术限制和默认值，游戏性配置请使用 Lua 配置

/// 玩家实体ID（硬编码，不可配置）
pub const PLAYER_ID: &str = "player";

// ===== 物理碰撞尺寸（与模型尺寸强关联） =====
/// 玩家碰撞体半径
pub const PLAYER_COLLIDER_RADIUS: f32 = 0.35;
/// 玩家碰撞体高度
pub const PLAYER_COLLIDER_HEIGHT: f32 = 0.9;

// ===== 相机技术限制（防止穿墙/翻转） =====
/// 相机最小距离
pub const CAMERA_DISTANCE_MIN: f32 = 5.0;
/// 相机最大距离
pub const CAMERA_DISTANCE_MAX: f32 = 40.0;
/// 相机最小俯仰角
pub const CAMERA_PITCH_MIN: f32 = 10.0f32.to_radians();
/// 相机最大俯仰角
pub const CAMERA_PITCH_MAX: f32 = 80.0f32.to_radians();

// ===== 输入响应（引擎级手感） =====
/// 相机滚轮缩放速度
pub const CAMERA_ZOOM_SPEED: f32 = 2.0;
/// 相机鼠标灵敏度
pub const CAMERA_MOUSE_SENSITIVITY: f32 = 0.001;

// ===== 默认值（当 Lua 配置缺失时使用） =====
/// 玩家默认移动速度
pub const PLAYER_SPEED: f32 = 5.0;
/// 玩家默认模型场景路径
pub const PLAYER_MODEL_SCENE: &str = "models/fox-eared_game_endfield/scene.gltf#Scene0";
/// 玩家默认偏航角偏移
pub const PLAYER_MODEL_YAW_OFFSET: f32 = 0.0;
/// 玩家默认模型缩放
pub const PLAYER_MODEL_SCALE: f32 = 1.0;
/// 玩家默认基础高度
pub const PLAYER_BASE_HEIGHT: f32 = 1.0;

/// 相机默认距离
pub const CAMERA_DISTANCE_DEFAULT: f32 = 20.0;
/// 相机默认平滑因子
pub const CAMERA_SMOOTH_FACTOR_DEFAULT: f32 = 0.1;

/// 行走动画振幅
pub const WALK_BOB_AMPLITUDE: f32 = 0.08;
/// 行走动画速度
pub const WALK_BOB_SPEED: f32 = 10.0;
/// 行走动画恢复速度
pub const WALK_BOB_RECOVER_SPEED: f32 = 8.0;
/// 旋转速度
pub const ROTATION_SPEED: f32 = 10.0;

// ===== 场景（后续迁移到 Lua 关卡配置） =====
/// 墙体尺寸
pub const WALL_SIZE: f32 = 2.0;
/// 屋顶尺寸
pub const ROOF_SIZE: f32 = 2.2;
/// 树木尺寸
pub const TREE_SIZE: f32 = 1.5;

/// 热重载检查间隔（秒）
pub const HOT_RELOAD_INTERVAL: f32 = 0.5;
