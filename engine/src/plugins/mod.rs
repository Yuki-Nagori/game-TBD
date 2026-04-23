//! Bevy 插件系统
//!
//! 按功能拆分的插件，便于管理和扩展

pub mod camera_plugin;
pub mod hot_reload_plugin;
pub mod lua_command_plugin;
pub mod player_plugin;
pub mod scene_plugin;

#[cfg(feature = "hot-reload")]
pub mod debug_console_plugin;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::asset_manager::{AssetManager, asset_manager_poll_system};
use crate::font_center::FontCenterPlugin;

use camera_plugin::CameraPlugin;
use hot_reload_plugin::HotReloadPlugin;
use lua_command_plugin::LuaCommandPlugin;
use player_plugin::PlayerPlugin;
use scene_plugin::ScenePlugin;

#[cfg(feature = "hot-reload")]
use debug_console_plugin::DebugConsolePlugin;

/// 游戏主插件：注册所有子插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManager>()
            .add_systems(Update, asset_manager_poll_system)
            .add_plugins((
                FontCenterPlugin,
                RapierPhysicsPlugin::<NoUserData>::default(),
                PlayerPlugin,
                CameraPlugin,
                ScenePlugin,
                LuaCommandPlugin,
                HotReloadPlugin,
            ));

        #[cfg(feature = "hot-reload")]
        {
            app.add_plugins(DebugConsolePlugin);
        }
    }
}
