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
    /// 按 ID 索引的实体映射
    pub by_id: HashMap<String, Entity>,
    /// 按 ID 索引的组件数据映射
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
    /// 脚本文件路径
    pub script_path: String,
    /// 上次修改时间
    pub last_modified: SystemTime,
    /// 检查定时器
    pub check_timer: Timer,
}

impl ScriptHotReload {
    /// 创建新的热重载状态
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_state_default() {
        let state = CameraState::default();
        assert_eq!(state.yaw, 0.0, "默认偏航角应为 0");
        assert_eq!(state.pitch, 20.0f32.to_radians(), "默认俯仰角应为 20 度");
        assert_eq!(state.distance, 20.0, "默认距离应为 20");
        assert!(
            (state.smooth_factor - 0.1).abs() < f32::EPSILON,
            "默认平滑因子应为 0.1"
        );
        assert!(state.mouse_locked, "默认应锁定鼠标");
    }

    #[test]
    fn test_entity_registry_insert_and_query() {
        let mut registry = EntityRegistry::default();
        let entity = Entity::from_bits(42);
        registry.by_id.insert("test_id".to_string(), entity);

        assert_eq!(
            registry.by_id.get("test_id"),
            Some(&entity),
            "应能查询到插入的实体"
        );
    }

    #[test]
    fn test_entity_registry_overwrite() {
        let mut registry = EntityRegistry::default();
        let entity1 = Entity::from_bits(1);
        let entity2 = Entity::from_bits(2);

        registry.by_id.insert("same_id".to_string(), entity1);
        registry.by_id.insert("same_id".to_string(), entity2);

        assert_eq!(
            registry.by_id.get("same_id"),
            Some(&entity2),
            "覆盖后应返回新实体"
        );
    }

    #[test]
    fn test_entity_registry_components_store() {
        let mut registry = EntityRegistry::default();
        let value = serde_json::json!({"hp": 100});
        let mut comps = HashMap::new();
        comps.insert("Health".to_string(), value.clone());
        registry.components.insert("entity_1".to_string(), comps);

        assert_eq!(
            registry.components.get("entity_1").unwrap().get("Health"),
            Some(&value),
            "应能查询到存储的组件数据"
        );
    }

    #[test]
    fn test_script_hot_reload_new() {
        let hot_reload = ScriptHotReload::new("game/main.lua");
        assert_eq!(hot_reload.script_path, "game/main.lua");
    }

    #[test]
    fn test_entity_registry_default_empty() {
        let registry = EntityRegistry::default();
        assert!(registry.by_id.is_empty(), "默认实体映射应为空");
        assert!(registry.components.is_empty(), "默认组件映射应为空");
    }
}
