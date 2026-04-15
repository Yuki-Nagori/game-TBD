//! Lua API 模块
//!
//! 负责 Rust 与 Lua 的交互，暴露游戏核心功能给脚本层
//! 设计目标：为创意工坊 Mod 系统预留接口

use bevy::prelude::Vec3;
use mlua::{FromLuaMulti, IntoLuaMulti, Lua, LuaSerdeExt, Result as LuaResult, Table, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

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

#[derive(Default)]
struct SharedState {
    commands: Mutex<Vec<LuaCommand>>,
    positions: Mutex<HashMap<String, [f32; 3]>>,
}

/// Lua 运行时封装
pub struct LuaRuntime {
    lua: Lua,
    shared: Arc<SharedState>,
}

impl LuaRuntime {
    /// 创建新的 Lua 运行时
    pub fn new() -> anyhow::Result<Self> {
        let lua = Lua::new();
        let shared = Arc::new(SharedState::default());

        // 设置 Lua 标准库（限制版，为安全考虑）
        // TODO: 进一步限制，沙箱化

        // 注册核心 API
        Self::register_core_api(&lua)?;

        // 注册 Mod API（为创意工坊预留）
        Self::register_mod_api(&lua, Arc::clone(&shared))?;

        Ok(Self { lua, shared })
    }

    /// 加载并执行 Lua 脚本
    pub fn load_main_script<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        let script = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("无法读取脚本 {:?}: {}", path, e))?;

        self.lua
            .load(&script)
            .set_name(path.to_string_lossy())
            .exec()
            .map_err(|e| anyhow::anyhow!("脚本执行错误: {}", e))?;

        info!("脚本 {:?} 加载成功", path);
        Ok(())
    }

    /// 调用 Lua 全局函数
    pub fn call_function<A, R>(&self, function_name: &str, args: A) -> anyhow::Result<R>
    where
        A: for<'lua> IntoLuaMulti<'lua>,
        R: for<'lua> FromLuaMulti<'lua>,
    {
        let function: mlua::Function = self
            .lua
            .globals()
            .get(function_name)
            .map_err(|e| anyhow::anyhow!("获取 Lua 函数 {} 失败: {}", function_name, e))?;

        function
            .call(args)
            .map_err(|e| anyhow::anyhow!("调用 Lua 函数 {} 失败: {}", function_name, e))
    }

    pub fn drain_commands(&self) -> Vec<LuaCommand> {
        self.shared
            .commands
            .lock()
            .map(|mut queue| std::mem::take(&mut *queue))
            .unwrap_or_default()
    }

    pub fn update_entity_position(&self, id: &str, position: Vec3) {
        if let Ok(mut positions) = self.shared.positions.lock() {
            positions.insert(id.to_string(), [position.x, position.y, position.z]);
        }
    }

    pub fn remove_entity_position(&self, id: &str) {
        if let Ok(mut positions) = self.shared.positions.lock() {
            positions.remove(id);
        }
    }

    /// 加载 Mod（为创意工坊预留）
    #[allow(dead_code)]
    pub fn load_mod<P: AsRef<Path>>(&self, mod_path: P) -> anyhow::Result<()> {
        let mod_path = mod_path.as_ref();

        // 检查 mod.json
        let manifest_path = mod_path.join("mod.json");
        if !manifest_path.exists() {
            anyhow::bail!("Mod 缺少 mod.json: {:?}", mod_path);
        }

        // 加载 init.lua
        let init_path = mod_path.join("init.lua");
        if init_path.exists() {
            self.load_main_script(&init_path)?;
        }

        info!("Mod {:?} 加载成功", mod_path);
        Ok(())
    }

    /// 注册核心 API（游戏内部使用）
    fn register_core_api(lua: &Lua) -> LuaResult<()> {
        // 日志函数（全局）
        lua.globals().set(
            "log_info",
            lua.create_function(|_, msg: String| {
                info!("[Lua] {}", msg);
                Ok(())
            })?,
        )?;

        lua.globals().set(
            "log_debug",
            lua.create_function(|_, msg: String| {
                info!("[Lua:debug] {}", msg);
                Ok(())
            })?,
        )?;

        lua.globals().set(
            "log_warn",
            lua.create_function(|_, msg: String| {
                warn!("[Lua] {}", msg);
                Ok(())
            })?,
        )?;

        lua.globals().set(
            "log_error",
            lua.create_function(|_, msg: String| {
                error!("[Lua] {}", msg);
                Ok(())
            })?,
        )?;

        // Core 表（保留用于其他核心功能）
        let core = lua.create_table()?;
        core.set("version", "0.1.0")?;
        lua.globals().set("Core", core)?;

        Ok(())
    }

    /// 注册 Mod API（为创意工坊预留）
    fn register_mod_api(lua: &Lua, shared: Arc<SharedState>) -> LuaResult<()> {
        // Entity API
        let entity = lua.create_table()?;

        let create_shared = Arc::clone(&shared);
        entity.set(
            "create",
            lua.create_function(move |lua, entity_type: String| {
                let id = format!("entity_{}", uuid::Uuid::new_v4());
                let command = LuaCommand::CreateEntity {
                    id: id.clone(),
                    entity_type: entity_type.clone(),
                };

                if let Ok(mut queue) = create_shared.commands.lock() {
                    queue.push(command);
                }

                info!("[Mod API] 创建实体: {} ({})", entity_type, id);
                let table = lua.create_table()?;
                table.set("type", entity_type)?;
                table.set("id", id)?;
                Ok(table)
            })?,
        )?;

        let destroy_shared = Arc::clone(&shared);
        entity.set(
            "destroy",
            lua.create_function(move |_, id: String| {
                if let Ok(mut queue) = destroy_shared.commands.lock() {
                    queue.push(LuaCommand::DestroyEntity { id });
                }
                Ok(())
            })?,
        )?;

        let set_pos_shared = Arc::clone(&shared);
        entity.set(
            "set_position",
            lua.create_function(move |_, (id, x, y, z): (String, f32, f32, f32)| {
                if let Ok(mut queue) = set_pos_shared.commands.lock() {
                    queue.push(LuaCommand::SetPosition { id, x, y, z });
                }
                Ok(())
            })?,
        )?;

        let get_pos_shared = Arc::clone(&shared);
        entity.set(
            "get_position",
            lua.create_function(move |lua, id: String| {
                let table = lua.create_table()?;
                let default = [0.0_f32, 0.0_f32, 0.0_f32];
                let position = get_pos_shared
                    .positions
                    .lock()
                    .ok()
                    .and_then(|positions| positions.get(&id).copied())
                    .unwrap_or(default);

                table.set("x", position[0])?;
                table.set("y", position[1])?;
                table.set("z", position[2])?;
                Ok(table)
            })?,
        )?;

        let add_comp_shared = Arc::clone(&shared);
        entity.set(
            "add_component",
            lua.create_function(move |lua, (id, name, value): (String, String, Value)| {
                let json_value = lua.from_value::<serde_json::Value>(value)?;
                if let Ok(mut queue) = add_comp_shared.commands.lock() {
                    queue.push(LuaCommand::AddComponent { id, name, value: json_value });
                }
                Ok(())
            })?,
        )?;

        let remove_comp_shared = Arc::clone(&shared);
        entity.set(
            "remove_component",
            lua.create_function(move |_, (id, name): (String, String)| {
                if let Ok(mut queue) = remove_comp_shared.commands.lock() {
                    queue.push(LuaCommand::RemoveComponent { id, name });
                }
                Ok(())
            })?,
        )?;

        lua.globals().set("Entity", entity)?;

        // Event API（预留）
        let event = lua.create_table()?;
        event.set(
            "on",
            lua.create_function(|_, (event_name, _callback): (String, Value)| {
                info!("[Mod API] 订阅事件: {}", event_name);
                // TODO: 实现事件订阅
                Ok(())
            })?,
        )?;

        event.set(
            "trigger",
            lua.create_function(|_, (event_name, _data): (String, Table)| {
                info!("[Mod API] 触发事件: {}", event_name);
                // TODO: 实现事件触发
                Ok(())
            })?,
        )?;

        lua.globals().set("Event", event)?;

        // World API（预留）
        let world = lua.create_table()?;
        world.set(
            "get_variable",
            lua.create_function(|_, _key: String| {
                // TODO: 实现世界变量读取
                Ok(Value::Nil)
            })?,
        )?;

        world.set(
            "set_variable",
            lua.create_function(|_, (key, value): (String, Value)| {
                info!("[Mod API] 设置世界变量: {} = {:?}", key, value);
                // TODO: 实现世界变量设置
                Ok(())
            })?,
        )?;

        lua.globals().set("World", world)?;

        // Player API（预留）
        let player = lua.create_table()?;
        player.set(
            "add_item",
            lua.create_function(|_, _item: Table| {
                info!("[Mod API] 添加物品到玩家");
                // TODO: 实现物品添加
                Ok(())
            })?,
        )?;

        lua.globals().set("Player", player)?;

        // History API（预留，用于历史事件干预）
        let history = lua.create_table()?;
        history.set(
            "on",
            lua.create_function(|_, (event_id, _handlers): (String, Table)| {
                info!("[Mod API] 注册历史事件钩子: {}", event_id);
                // TODO: 实现历史事件钩子
                Ok(())
            })?,
        )?;

        lua.globals().set("History", history)?;

        Ok(())
    }
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
    fn test_core_api() {
        let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
        runtime
            .lua
            .load("log_info('测试日志')")
            .exec()
            .expect("lua script should execute");
    }

    #[test]
    fn test_call_lua_function() {
        let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
        runtime
            .lua
            .load(
                r#"
                function add(a, b)
                    return a + b
                end
                "#,
            )
            .exec()
            .expect("lua script should execute");

        let result: i64 = runtime
            .call_function("add", (2_i64, 3_i64))
            .expect("call_function should succeed");
        assert_eq!(result, 5);
    }

    #[test]
    fn test_entity_api_queue_command() {
        let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
        runtime
            .lua
            .load("Entity.set_position('player', 12.0, 8.0, 0.0)")
            .exec()
            .expect("lua script should execute");

        let commands = runtime.drain_commands();
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            LuaCommand::SetPosition { id, x, y, z } => {
                assert_eq!(id, "player");
                assert_eq!((*x, *y, *z), (12.0, 8.0, 0.0));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn test_entity_position_cache_clear() {
        let runtime = LuaRuntime::new().expect("LuaRuntime::new should succeed");
        runtime
            .lua
            .load(
                r#"
                function read_pos(id)
                    local pos = Entity.get_position(id)
                    return pos.x, pos.y, pos.z
                end
                "#,
            )
            .exec()
            .expect("lua script should execute");

        runtime.update_entity_position("temp_entity", Vec3::new(3.0, 4.0, 5.0));
        let before: (f32, f32, f32) = runtime
            .call_function("read_pos", "temp_entity")
            .expect("call_function should succeed");
        assert_eq!(before, (3.0, 4.0, 5.0));

        runtime.remove_entity_position("temp_entity");
        let after: (f32, f32, f32) = runtime
            .call_function("read_pos", "temp_entity")
            .expect("call_function should succeed");
        assert_eq!(after, (0.0, 0.0, 0.0));
    }
}
