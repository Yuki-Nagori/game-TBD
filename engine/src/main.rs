//! 明朝修仙 RPG - 主入口
//!
//! 核心原则：
//! 1. 简单明确 - 每个模块只做一件事
//! 2. 错误透明 - 用 Result 传播错误，不 panic
//! 3. 可测试 - 核心业务逻辑独立可测
//!
//! 代码结构：
//! - main.rs: 入口，插件注册
//! - plugins/: 功能插件（player, camera, scene）
//! - components/: ECS 组件定义
//! - resources/: 全局资源
//! - constants.rs: 游戏常量
//! - utils.rs: 工具函数
//! - core/: 游戏核心逻辑（时间、功法等）
//! - lua_api/: Lua 运行时与 API

use bevy::log::LogPlugin;
use bevy::prelude::*;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

mod components;
mod constants;
mod core;
mod lua_api;
mod plugins;
mod resources;
mod utils;

use lua_api::LuaRuntime;
use plugins::GamePlugin;
use utils::resolve_asset_root;

/// 游戏配置
#[derive(Debug, Deserialize)]
struct GameConfig {
    version: String,
    start_year: i32,
    time_scale: f32,
}

fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,ming_rpg=debug")
        .init();

    info!("启动 明朝修仙 RPG...");

    // 加载配置
    let config = load_game_config("game/config.toml")?;
    info!(
        "配置加载完成 version={} start_year={} time_scale={}",
        config.version, config.start_year, config.time_scale
    );

    // 初始化 Lua 运行时
    let lua = LuaRuntime::new()?;
    info!("Lua 运行时初始化完成");

    // 加载主脚本
    lua.load_main_script("game/main.lua")?;
    info!("主脚本加载完成");

    // 启动 Bevy 应用
    let asset_root = resolve_asset_root();
    info!("资源目录: {}", asset_root);

    App::new()
        .insert_non_send_resource(lua)
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .set(AssetPlugin {
                    file_path: asset_root,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "明朝修仙 RPG - Phase 2".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();

    Ok(())
}

/// 加载游戏配置
fn load_game_config<P: AsRef<Path>>(path: P) -> anyhow::Result<GameConfig> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("读取配置文件 {:?} 失败: {}", path, e))?;
    toml::from_str(&content).map_err(|e| anyhow::anyhow!("解析配置文件 {:?} 失败: {}", path, e))
}
