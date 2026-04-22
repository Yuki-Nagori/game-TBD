//! Lua API 集成测试
//!
//! 测试 Rust 与 Lua 的交互

use bevy::prelude::Vec3;
use ming_rpg::lua_api::{LuaCommand, LuaRuntime};

#[test]
fn test_lua_runtime_creation() {
    let runtime = LuaRuntime::new();
    assert!(runtime.is_ok());
}

#[test]
fn test_entity_api_queue_command() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
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

#[test]
fn test_lua_config_roundtrip() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    runtime
        .execute(r#"TEST_CONFIG = { speed = 10.5, name = "test", enabled = true }"#)
        .unwrap();

    let config: std::collections::HashMap<String, serde_json::Value> = runtime
        .get_config("TEST_CONFIG")
        .expect("should get config");
    assert_eq!(config["speed"], 10.5);
    assert_eq!(config["name"], "test");
    assert_eq!(config["enabled"], true);
}

#[test]
fn test_entity_creation_pipeline() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    runtime.execute(r#"Entity.create("npc")"#).unwrap();

    let commands = runtime.drain_commands();
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        LuaCommand::CreateEntity { entity_type, .. } => {
            assert_eq!(entity_type, "npc");
        }
        _ => panic!("Expected CreateEntity command"),
    }
}

#[test]
fn test_entity_position_update() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    runtime
        .execute(r#"Entity.set_position("test", 1.0, 2.0, 3.0)"#)
        .unwrap();

    let commands = runtime.drain_commands();
    assert_eq!(commands.len(), 1);
    match &commands[0] {
        LuaCommand::SetPosition { id, x, y, z } => {
            assert_eq!(id, "test");
            assert_eq!(*x, 1.0);
            assert_eq!(*y, 2.0);
            assert_eq!(*z, 3.0);
        }
        _ => panic!("Expected SetPosition command"),
    }
}

#[test]
fn test_lua_error_handling() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    let result = runtime.execute("invalid syntax!!!");
    assert!(result.is_err());
}

#[test]
fn test_command_queue_batching() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    runtime
        .execute(
            r#"
        Entity.create("npc")
        Entity.create("player")
        Entity.set_position("test", 1.0, 2.0, 3.0)
    "#,
        )
        .unwrap();

    let commands = runtime.drain_commands();
    assert_eq!(commands.len(), 3);

    match &commands[0] {
        LuaCommand::CreateEntity { entity_type, .. } => {
            assert_eq!(entity_type, "npc")
        }
        _ => panic!("Expected first command to be CreateEntity(npc)"),
    }
    match &commands[1] {
        LuaCommand::CreateEntity { entity_type, .. } => {
            assert_eq!(entity_type, "player")
        }
        _ => panic!("Expected second command to be CreateEntity(player)"),
    }
    match &commands[2] {
        LuaCommand::SetPosition { id, .. } => assert_eq!(id, "test"),
        _ => panic!("Expected third command to be SetPosition(test)"),
    }
}
