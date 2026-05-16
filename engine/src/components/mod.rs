//! ECS 组件定义
//!
//! 所有游戏组件集中定义，便于管理和查询

use bevy::prelude::*;

/// 玩家标记组件
#[derive(Component)]
pub struct Player;

/// 第三人称相机标记组件
#[derive(Component)]
pub struct ThirdPersonCamera;

/// 角色运动状态组件
#[derive(Component, Default)]
pub struct CharacterMotion {
    /// 是否正在移动
    pub is_moving: bool,
    /// 人物独立的朝向（弧度，不受相机影响）
    pub facing_yaw: f32,
}

/// 行走动画占位组件（后续替换为真实动画系统）
#[derive(Component)]
pub struct PlaceholderWalkAnimation {
    /// 基础高度
    pub base_height: f32,
    /// 动画相位
    pub phase: f32,
}

impl PlaceholderWalkAnimation {
    /// 创建新的行走动画占位组件
    pub fn new(base_height: f32) -> Self {
        Self { base_height, phase: 0.0 }
    }
}

/// 编辑器放置的物体标记组件
#[derive(Component)]
pub struct EditorPlaced;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_walk_animation_new() {
        let anim = PlaceholderWalkAnimation::new(1.5);
        assert!(
            (anim.base_height - 1.5).abs() < f32::EPSILON,
            "base_height 应等于传入值"
        );
        assert_eq!(anim.phase, 0.0, "新创建时 phase 应为 0");
    }

    #[test]
    fn test_placeholder_walk_animation_default() {
        let anim = PlaceholderWalkAnimation::new(0.0);
        assert_eq!(anim.base_height, 0.0);
        assert_eq!(anim.phase, 0.0);
    }

    #[test]
    fn test_character_motion_default() {
        let motion = CharacterMotion::default();
        assert!(!motion.is_moving, "默认不应在移动");
        assert_eq!(motion.facing_yaw, 0.0, "默认朝向应为 0");
    }

    #[test]
    fn test_player_component_exists() {
        let _player = Player;
    }

    #[test]
    fn test_third_person_camera_component_exists() {
        let _cam = ThirdPersonCamera;
    }

    #[test]
    fn test_editor_placed_component_exists() {
        let _placed = EditorPlaced;
    }
}
