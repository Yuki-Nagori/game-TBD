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
#[derive(Debug, Clone)]
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
