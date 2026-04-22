//! 集成测试
//!
//! 测试 Rust ↔ Lua 完整交互链

use ming_rpg::lua_api::{LuaCommand, LuaRuntime};

#[test]
fn test_lua_runtime_lifecycle() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
    runtime.execute("x = 1 + 1").unwrap();
    let result = runtime.execute_with_return("return x").unwrap();
    assert_eq!(result, "2");
    // Drop 时不应 panic
}

#[test]
fn test_config_loading_from_fixtures() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    let script_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_config.lua");
    runtime.load_main_script(&script_path).unwrap();

    let config: std::collections::HashMap<String, serde_json::Value> = runtime
        .get_config("TEST_CONFIG")
        .expect("should get config");
    assert_eq!(config["speed"], 10.5);
    assert_eq!(config["name"], "test");
}

#[test]
fn test_execute_with_return_types() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    assert_eq!(runtime.execute_with_return("return nil").unwrap(), "nil");
    assert_eq!(runtime.execute_with_return("return true").unwrap(), "true");
    assert_eq!(runtime.execute_with_return("return 42").unwrap(), "42");
    assert_eq!(runtime.execute_with_return("return 3.14").unwrap(), "3.14");
    assert_eq!(
        runtime.execute_with_return("return 'hello'").unwrap(),
        "hello"
    );

    let table_result = runtime.execute_with_return("return {a = 1}").unwrap();
    assert!(
        table_result.contains("\"a\":1") || table_result.contains("{\"a\":1}"),
        "Table result should contain a=1: {}",
        table_result
    );
}

#[test]
fn test_entity_api_full_pipeline() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    // 加载实体夹具
    let script_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/test_entities.lua");
    runtime.load_main_script(&script_path).unwrap();

    let commands = runtime.drain_commands();
    // 3 个 create + 3 个 set_position
    assert_eq!(commands.len(), 6);

    let mut create_count = 0;
    let mut set_position_count = 0;
    for cmd in &commands {
        match cmd {
            LuaCommand::CreateEntity { .. } => create_count += 1,
            LuaCommand::SetPosition { .. } => set_position_count += 1,
            _ => {}
        }
    }
    assert_eq!(create_count, 3);
    assert_eq!(set_position_count, 3);
}

#[test]
fn test_lua_error_from_fixture() {
    let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");

    let script_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test_errors.lua");
    let result = runtime.load_main_script(&script_path);
    assert!(result.is_err(), "Loading invalid syntax should fail");
}
