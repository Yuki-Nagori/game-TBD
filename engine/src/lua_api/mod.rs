//! Lua API 模块
//! 
//! 负责 Rust 与 Lua 的交互，暴露游戏核心功能给脚本层

use mlua::{Lua, Result as LuaResult};
use tracing::{info, error};
use std::path::Path;

/// Lua 运行时封装
pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    /// 创建新的 Lua 运行时
    pub fn new() -> anyhow::Result<Self> {
        let lua = Lua::new();
        
        // 设置 Lua 标准库
        // TODO: 限制标准库，沙箱化
        
        // 注册 Rust 函数到 Lua 全局
        Self::register_globals(&lua)?;
        
        Ok(Self { lua })
    }

    /// 加载并执行 Lua 脚本
    pub fn load_main_script<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();
        let script = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("无法读取脚本 {:?}: {}", path, e))?;
        
        self.lua.load(&script).exec()
            .map_err(|e| anyhow::anyhow!("脚本执行错误: {}", e))?;
        
        info!("脚本 {:?} 加载成功", path);
        Ok(())
    }

    /// 注册全局函数
    fn register_globals(lua: &Lua) -> LuaResult<()> {
        // Log 函数
        let log_info = lua.create_function(|_, msg: String| {
            info!("[Lua] {}", msg);
            Ok(())
        })?;
        lua.globals().set("log_info", log_info)?;

        // 更多 API 后续添加...
        
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
}
