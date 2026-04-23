//! 明朝修仙 RPG - 引擎库
//!
//! 核心原则：
//! 1. 简单明确 - 每个模块只做一件事
//! 2. 错误透明 - 用 Result 传播错误，不 panic
//! 3. 可测试 - 核心业务逻辑独立可测

#![warn(missing_docs)]

/// 异步资产加载与管理
pub mod asset_manager;
/// ECS 组件定义
pub mod components;
/// 游戏常量
pub mod constants;
/// 核心逻辑（时间、功法）
pub mod core;
/// UI 字体与主题管理
pub mod font_center;
/// Lua 运行时与 API
pub mod lua_api;
/// Bevy 插件系统
pub mod plugins;
/// 全局状态资源
pub mod resources;
/// 工具函数
pub mod utils;

// 重导出常用类型
pub use lua_api::LuaRuntime;
