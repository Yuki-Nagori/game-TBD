//! 明朝修仙 RPG - 引擎库
//!
//! 核心原则：
//! 1. 简单明确 - 每个模块只做一件事
//! 2. 错误透明 - 用 Result 传播错误，不 panic
//! 3. 可测试 - 核心业务逻辑独立可测

pub mod components;
pub mod constants;
pub mod core;
pub mod lua_api;
pub mod plugins;
pub mod resources;
pub mod utils;

// 重导出常用类型
pub use lua_api::LuaRuntime;
