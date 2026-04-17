//! Lua API 模块
//!
//! 负责 Rust 与 Lua 的交互，暴露游戏核心功能给脚本层
//! 设计目标：为创意工坊 Mod 系统预留接口
//!
//! # 线程安全说明
//!
//! 本模块使用 Actor 模式实现线程安全：
//! - `LuaRuntime` 是 `Send + Sync`，可在任何 Bevy 系统使用
//! - 实际的 Lua 状态运行在独立的后台线程
//! - 通过通道进行异步通信

mod runtime;
pub use runtime::LuaRuntime;

#[cfg(test)]
use bevy::prelude::Vec3;

/// Lua 命令类型
///
/// Lua 脚本通过这些命令与游戏引擎交互
#[derive(Debug, Clone)]
pub enum LuaCommand {
    CreateEntity {
        id: String,
        entity_type: String,
    },
    DestroyEntity {
        id: String,
    },
    SetPosition {
        id: String,
        x: f32,
        y: f32,
        z: f32,
    },
    AddComponent {
        id: String,
        name: String,
        value: serde_json::Value,
    },
    RemoveComponent {
        id: String,
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
