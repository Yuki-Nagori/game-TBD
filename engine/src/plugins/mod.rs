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

/// 游戏系统集合
///
/// 按功能分组系统，明确调度边界，便于插件插入和性能分析。
/// 同一集合内的系统仍由 Bevy 调度器自动并行化（无数据依赖时）。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    /// 玩家输入处理（WASD、鼠标事件）
    Input,
    /// 相机控制（跟随、旋转、缩放）
    Camera,
    /// 物理模拟
    Physics,
    /// Lua 脚本与命令处理
    Lua,
    /// 场景管理（切换检测、对象生成）
    Scene,
    /// 资源加载与缓存
    Asset,
    /// 调试工具（控制台、性能监控、热重载）
    Debug,
}

/// 游戏主插件：注册所有子插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetManager>()
            .add_systems(
                Update,
                asset_manager_poll_system.in_set(GameSystemSet::Asset),
            )
            .insert_resource(TimestepMode::Fixed { dt: 1.0 / 45.0, substeps: 1 })
            .configure_sets(
                Update,
                (
                    GameSystemSet::Input,
                    GameSystemSet::Camera,
                    GameSystemSet::Physics,
                    GameSystemSet::Lua,
                    GameSystemSet::Scene,
                    GameSystemSet::Asset,
                    GameSystemSet::Debug,
                )
                    .chain(),
            )
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
