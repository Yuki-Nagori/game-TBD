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
#[allow(dead_code)]
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
