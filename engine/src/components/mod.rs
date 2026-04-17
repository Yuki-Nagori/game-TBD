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
    pub is_moving: bool,
    /// 人物独立的朝向（弧度，不受相机影响）
    pub facing_yaw: f32,
}

/// 行走动画占位组件（后续替换为真实动画系统）
#[derive(Component)]
pub struct PlaceholderWalkAnimation {
    pub base_height: f32,
    pub phase: f32,
}

impl PlaceholderWalkAnimation {
    pub fn new(base_height: f32) -> Self {
        Self { base_height, phase: 0.0 }
    }
}
