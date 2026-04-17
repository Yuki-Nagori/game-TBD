//! Bevy 插件系统
//!
//! 按功能拆分的插件，便于管理和扩展

pub mod camera_plugin;
pub mod lua_command_plugin;
pub mod player_plugin;
pub mod scene_plugin;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use camera_plugin::CameraPlugin;
use lua_command_plugin::LuaCommandPlugin;
use player_plugin::PlayerPlugin;
use scene_plugin::ScenePlugin;

/// 游戏主插件：注册所有子插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // 物理引擎
            RapierPhysicsPlugin::<NoUserData>::default(),
            // 调试渲染（开发时启用）
            // RapierDebugRenderPlugin::default(),
            // 游戏功能插件
            PlayerPlugin,
            CameraPlugin,
            ScenePlugin,
            LuaCommandPlugin,
        ));
    }
}
