//! Lua API 集成测试
//!
//! 测试 Rust 与 Lua 的交互

use bevy::prelude::Vec3;
use ming_rpg::lua_api::LuaRuntime;

#[test]
fn test_lua_runtime_creation() {
    let runtime = LuaRuntime::new();
    assert!(runtime.is_ok());
}

#[test]
fn test_entity_api_queue_command() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    // 通过 Lua 代码添加命令
    // 注意：Actor 模式下需要加载脚本后才能测试
    // 这里仅验证结构

    let commands = runtime.drain_commands();
    // 新创建的 runtime 应该没有命令
    assert!(commands.is_empty());
}

#[test]
fn test_entity_position_cache() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    runtime.update_entity_position("test_entity", Vec3::new(1.0, 2.0, 3.0));
    runtime.remove_entity_position("test_entity");

    // 验证操作不会 panic
}

// TODO: 更多测试
// - test_lua_config_loading: 加载 game/main.lua 并读取配置
// - test_lua_function_call: 测试 Lua 函数调用
// - test_lua_error_handling: 测试错误处理
