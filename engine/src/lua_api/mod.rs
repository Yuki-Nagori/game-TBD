//! Lua API 模块
//!
//! 负责 Rust 与 Lua 的交互，暴露游戏核心功能给脚本层
//! 设计目标：为创意工坊 Mod 系统预留接口

use mlua::{Lua, Result as LuaResult, Table, Value};
use std::path::Path;
use tracing::{error, info, warn};

/// Lua 运行时封装
pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    /// 创建新的 Lua 运行时
    pub fn new() -> anyhow::Result<Self> {
        let lua = Lua::new();

        // 设置 Lua 标准库（限制版，为安全考虑）
        // TODO: 进一步限制，沙箱化

        // 注册核心 API
        Self::register_core_api(&lua)?;

        // 注册 Mod API（为创意工坊预留）
        Self::register_mod_api(&lua)?;

        Ok(Self { lua })
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
    fn register_mod_api(lua: &Lua) -> LuaResult<()> {
        // Entity API
        let entity = lua.create_table()?;
        entity.set(
            "create",
            lua.create_function(|lua, type_name: String| {
                info!("[Mod API] 创建实体: {}", type_name);
                // TODO: 实现实体创建
                let table = lua.create_table()?;
                table.set("type", type_name)?;
                table.set("id", format!("entity_{}", uuid::Uuid::new_v4()))?;
                Ok(table)
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
        let runtime = LuaRuntime::new().unwrap();
        runtime.lua.load("log_info('测试日志')").exec().unwrap();
    }
}
