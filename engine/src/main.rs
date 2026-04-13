//! 明朝修仙 RPG - 主入口
//!
//! 核心原则：
//! 1. 简单明确 - 每个模块只做一件事
//! 2. 错误透明 - 用 Result 传播错误，不 panic
//! 3. 可测试 - 核心业务逻辑独立可测

use bevy::log::LogPlugin;
use bevy::prelude::*;
use tracing::info;

mod core;
mod lua_api;

use lua_api::LuaRuntime;

fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,ming_rpg=debug")
        .init();

    info!("启动 明朝修仙 RPG...");

    // 初始化 Lua 运行时
    let lua = LuaRuntime::new()?;
    info!("Lua 运行时初始化完成");

    // 加载主脚本
    lua.load_main_script("game/main.lua")?;
    info!("主脚本加载完成");

    // 启动 Bevy 应用
    // 注意：禁用 LogPlugin，因为我们已经在上面手动初始化了 tracing_subscriber
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
        .add_systems(Startup, setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    info!("Bevy 初始化完成");

    // 相机
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 光源
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // 地面（方块占位）
    commands.spawn(PbrBundle {
        mesh: default(), // 后续用实际 mesh
        material: default(),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });

    info!("基础场景创建完成");
}
