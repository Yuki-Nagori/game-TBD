//! 游戏资源定义
//!
//! 全局状态和配置资源

use bevy::prelude::*;
use std::collections::HashMap;
use std::time::SystemTime;

use crate::constants::HOT_RELOAD_INTERVAL;

/// 实体注册表：管理所有游戏实体的 ID 映射
#[derive(Default, Resource)]
pub struct EntityRegistry {
    pub by_id: HashMap<String, Entity>,
    pub components: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// 相机状态：存储第三人称相机的球面坐标参数
#[derive(Resource)]
pub struct CameraState {
    /// 偏航角（左右旋转）
    pub yaw: f32,
    /// 俯仰角（上下旋转）
    pub pitch: f32,
    /// 当前相机距离（动态调整）
    pub distance: f32,
    /// 平滑因子
    pub smooth_factor: f32,
    /// 鼠标是否锁定（陀螺仪模式）
    pub mouse_locked: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 20.0f32.to_radians(),
            distance: 20.0,
            smooth_factor: 0.1,
            mouse_locked: true,
        }
    }
}

/// Lua 脚本热重载状态
#[derive(Resource)]
pub struct ScriptHotReload {
    pub script_path: String,
    pub last_modified: SystemTime,
    pub check_timer: Timer,
}

impl ScriptHotReload {
    pub fn new(script_path: &str) -> Self {
        let last_modified =
            crate::utils::get_last_modified(script_path).unwrap_or(SystemTime::UNIX_EPOCH);
        Self {
            script_path: script_path.to_string(),
            last_modified,
            check_timer: Timer::from_seconds(HOT_RELOAD_INTERVAL, TimerMode::Repeating),
        }
    }
}
