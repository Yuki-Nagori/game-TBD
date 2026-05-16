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

/// Lua 命令类型
///
/// Lua 脚本通过这些命令与游戏引擎交互
#[derive(Debug, Clone, PartialEq)]
pub enum LuaCommand {
    /// 创建实体
    CreateEntity {
        /// 实体唯一标识
        id: String,
        /// 实体类型（如 "npc", "player"）
        entity_type: String,
    },
    /// 销毁实体
    DestroyEntity {
        /// 实体唯一标识
        id: String,
    },
    /// 设置实体位置
    SetPosition {
        /// 实体唯一标识
        id: String,
        /// X 坐标
        x: f32,
        /// Y 坐标
        y: f32,
        /// Z 坐标
        z: f32,
    },
    /// 添加组件
    AddComponent {
        /// 实体唯一标识
        id: String,
        /// 组件名称
        name: String,
        /// 组件值（JSON 格式）
        value: serde_json::Value,
    },
    /// 移除组件
    RemoveComponent {
        /// 实体唯一标识
        id: String,
        /// 组件名称
        name: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_command_clone() {
        let cmd1 = LuaCommand::CreateEntity {
            id: "e1".to_string(),
            entity_type: "npc".to_string(),
        };
        let cmd2 = cmd1.clone();
        match (&cmd1, &cmd2) {
            (
                LuaCommand::CreateEntity { id: a, entity_type: b },
                LuaCommand::CreateEntity { id: c, entity_type: d },
            ) => {
                assert_eq!(a, c);
                assert_eq!(b, d);
            }
            _ => panic!("Clone 后变体应相同"),
        }
    }

    #[test]
    fn test_lua_command_debug_format() {
        let cmd = LuaCommand::SetPosition {
            id: "test".to_string(),
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let debug = format!("{:?}", cmd);
        assert!(debug.contains("SetPosition"), "Debug 应包含变体名");
        assert!(debug.contains("test"), "Debug 应包含 id");
    }

    #[test]
    fn test_lua_command_all_variants() {
        let cmds = vec![
            LuaCommand::CreateEntity {
                id: "a".to_string(),
                entity_type: "player".to_string(),
            },
            LuaCommand::DestroyEntity { id: "b".to_string() },
            LuaCommand::SetPosition {
                id: "c".to_string(),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            LuaCommand::AddComponent {
                id: "d".to_string(),
                name: "Health".to_string(),
                value: serde_json::json!(100),
            },
            LuaCommand::RemoveComponent {
                id: "e".to_string(),
                name: "Health".to_string(),
            },
        ];
        assert_eq!(cmds.len(), 5, "LuaCommand 应有 5 个变体");
    }

    #[test]
    fn test_lua_command_equality_by_variant() {
        let create1 = LuaCommand::CreateEntity {
            id: "e1".to_string(),
            entity_type: "npc".to_string(),
        };
        let create2 = LuaCommand::CreateEntity {
            id: "e1".to_string(),
            entity_type: "npc".to_string(),
        };
        let destroy = LuaCommand::DestroyEntity { id: "e1".to_string() };
        assert_eq!(create1, create2, "相同数据应相等");
        assert_ne!(
            std::mem::discriminant(&create1),
            std::mem::discriminant(&destroy),
            "不同变体不应相等"
        );
    }
}
