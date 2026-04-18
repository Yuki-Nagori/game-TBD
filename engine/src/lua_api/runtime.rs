//! Lua 运行时 - Actor 模式实现
//!
//! 解决 Lua 的 !Send 限制，允许在多线程 ECS 中安全使用
//!
//! 设计：
//! - LuaActorHandle: Send + Sync，可在任何系统使用
//! - LuaActor: 在独立线程运行 Lua 状态
//! - 通过通道进行异步通信

use bevy::prelude::{Resource, Vec3};
use mlua::{Lua, LuaSerdeExt, Result as LuaResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use tracing::{error, info, warn};

use super::LuaCommand;

/// 请求类型
#[derive(Debug)]
enum LuaRequest {
    LoadScript {
        path: String,
        respond_to: Sender<Result<(), String>>,
    },
    CallFunction {
        name: String,
        args: Vec<u8>, // 序列化后的参数
        respond_to: Sender<Result<Vec<u8>, String>>,
    },
    GetConfig {
        table_name: String,
        respond_to: Sender<Option<String>>,
    },
    DrainCommands {
        respond_to: Sender<Vec<LuaCommand>>,
    },
    Execute {
        code: String,
        respond_to: Sender<Result<(), String>>,
    },
    ExecuteWithReturn {
        code: String,
        respond_to: Sender<Result<String, String>>,
    },
}

/// Lua Actor 句柄
///
/// 线程安全，可在 Bevy 系统的任何地方使用
/// 实现了 Resource trait，可作为 Bevy 资源使用
#[derive(Clone, Resource)]
pub struct LuaRuntime {
    sender: Sender<LuaRequest>,
    // 这些字段用于传递给 Actor 线程，通过 Clone 保持引用
    #[allow(dead_code)]
    command_queue: Arc<Mutex<Vec<LuaCommand>>>,
    #[allow(dead_code)]
    positions: Arc<Mutex<HashMap<String, [f32; 3]>>>,
}

impl LuaRuntime {
    /// 创建新的 Lua 运行时
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = channel::<LuaRequest>();
        let command_queue = Arc::new(Mutex::new(Vec::new()));
        let positions = Arc::new(Mutex::new(HashMap::new()));

        let command_queue_clone = Arc::clone(&command_queue);
        let positions_clone = Arc::clone(&positions);

        // 启动 Lua 线程
        let _handle = thread::spawn(move || {
            let mut actor = match LuaActor::new(command_queue_clone, positions_clone) {
                Ok(actor) => actor,
                Err(e) => {
                    error!("Lua Actor 创建失败: {}", e);
                    return;
                }
            };
            actor.run(receiver);
        });

        Ok(Self {
            sender,
            command_queue,
            positions,
        })
    }

    /// 加载并执行 Lua 脚本
    pub fn load_main_script<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref().to_string_lossy().to_string();
        let (tx, rx) = channel();

        self.sender
            .send(LuaRequest::LoadScript { path, respond_to: tx })
            .map_err(|e| anyhow::anyhow!("发送请求失败: {}", e))?;

        rx.recv()
            .map_err(|e| anyhow::anyhow!("接收响应失败: {}", e))?
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// 调用 Lua 全局函数（仅支持 f32 参数和 () 返回值）
    ///
    /// 示例: `runtime.call_function("update", 0.016f32)?;`
    pub fn call_function(&self, function_name: &str, arg: f32) -> anyhow::Result<()> {
        let args_bytes =
            bincode::serialize(&arg).map_err(|e| anyhow::anyhow!("序列化参数失败: {}", e))?;

        let (tx, rx) = channel();

        self.sender
            .send(LuaRequest::CallFunction {
                name: function_name.to_string(),
                args: args_bytes,
                respond_to: tx,
            })
            .map_err(|e| anyhow::anyhow!("发送请求失败: {}", e))?;

        rx.recv()
            .map_err(|e| anyhow::anyhow!("接收响应失败: {}", e))?
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }

    /// 从 Lua 全局表读取配置
    pub fn get_config<T: serde::de::DeserializeOwned>(&self, table_name: &str) -> Option<T> {
        let (tx, rx) = channel();

        self.sender
            .send(LuaRequest::GetConfig {
                table_name: table_name.to_string(),
                respond_to: tx,
            })
            .ok()?;

        let json_str = rx.recv().ok()??;
        serde_json::from_str(&json_str).ok()
    }

    /// 更新实体位置
    pub fn update_entity_position(&self, id: &str, position: Vec3) {
        if let Ok(mut positions) = self.positions.lock() {
            positions.insert(id.to_string(), [position.x, position.y, position.z]);
        }
    }

    /// 移除实体位置
    pub fn remove_entity_position(&self, id: &str) {
        if let Ok(mut positions) = self.positions.lock() {
            positions.remove(id);
        }
    }

    /// 获取并清空命令队列
    pub fn drain_commands(&self) -> Vec<LuaCommand> {
        let (tx, rx) = channel();

        if self
            .sender
            .send(LuaRequest::DrainCommands { respond_to: tx })
            .is_err()
        {
            return Vec::new();
        }

        rx.recv().unwrap_or_default()
    }

    /// 执行 Lua 代码字符串
    pub fn execute(&self, code: &str) -> anyhow::Result<()> {
        let (tx, rx) = channel();

        self.sender
            .send(LuaRequest::Execute {
                code: code.to_string(),
                respond_to: tx,
            })
            .map_err(|e| anyhow::anyhow!("发送请求失败: {}", e))?;

        rx.recv()
            .map_err(|e| anyhow::anyhow!("接收响应失败: {}", e))?
            .map_err(|e: String| anyhow::anyhow!("Lua 执行失败: {}", e))?;
        Ok(())
    }

    /// 执行 Lua 代码字符串并返回结果
    pub fn execute_with_return(&self, code: &str) -> anyhow::Result<String> {
        let (tx, rx) = channel();

        self.sender
            .send(LuaRequest::ExecuteWithReturn {
                code: code.to_string(),
                respond_to: tx,
            })
            .map_err(|e| anyhow::anyhow!("发送请求失败: {}", e))?;

        rx.recv()
            .map_err(|e| anyhow::anyhow!("接收响应失败: {}", e))?
            .map_err(|e: String| anyhow::anyhow!("Lua 执行失败: {}", e))
    }
}

/// 内部 Lua Actor
///
/// 在独立线程中运行实际的 Lua 状态
struct LuaActor {
    lua: Lua,
    command_queue: Arc<Mutex<Vec<LuaCommand>>>,
    // 共享的位置状态，Lua 代码可以通过 API 读取
    #[allow(dead_code)]
    positions: Arc<Mutex<HashMap<String, [f32; 3]>>>,
}

impl LuaActor {
    fn new(
        command_queue: Arc<Mutex<Vec<LuaCommand>>>,
        positions: Arc<Mutex<HashMap<String, [f32; 3]>>>,
    ) -> anyhow::Result<Self> {
        let lua = Lua::new();

        // 注册核心 API
        Self::register_core_api(&lua)?;
        Self::register_mod_api(&lua, Arc::clone(&command_queue), Arc::clone(&positions))?;

        Ok(Self { lua, command_queue, positions })
    }

    fn run(&mut self, receiver: Receiver<LuaRequest>) {
        while let Ok(request) = receiver.recv() {
            match request {
                LuaRequest::LoadScript { path, respond_to } => {
                    let result = self.load_script(&path);
                    let _ = respond_to.send(result);
                }
                LuaRequest::CallFunction { name, args, respond_to } => {
                    let result = self.call_function(&name, &args);
                    let _ = respond_to.send(result);
                }
                LuaRequest::GetConfig { table_name, respond_to } => {
                    let result = self.get_config(&table_name);
                    let _ = respond_to.send(result);
                }
                LuaRequest::DrainCommands { respond_to } => {
                    let commands = if let Ok(mut queue) = self.command_queue.lock() {
                        std::mem::take(&mut *queue)
                    } else {
                        Vec::new()
                    };
                    let _ = respond_to.send(commands);
                }
                LuaRequest::Execute { code, respond_to } => {
                    let result = self.execute_code(&code);
                    let _ = respond_to.send(result);
                }
                LuaRequest::ExecuteWithReturn { code, respond_to } => {
                    let result = self.execute_code_with_return(&code);
                    let _ = respond_to.send(result);
                }
            }
        }

        info!("Lua Actor 线程已关闭");
    }

    fn load_script(&self, path: &str) -> Result<(), String> {
        let path_obj = Path::new(path);

        // 设置 require 搜索路径
        let script_dir = path_obj
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let package: mlua::Table = self
            .lua
            .globals()
            .get("package")
            .map_err(|e| format!("获取 package 失败: {}", e))?;
        let lua_path = format!(
            "{}/?.lua;{}/?/init.lua",
            script_dir.display(),
            script_dir.display()
        );
        package
            .set("path", lua_path)
            .map_err(|e| format!("设置路径失败: {}", e))?;

        let script =
            std::fs::read_to_string(path).map_err(|e| format!("无法读取脚本 {:?}: {}", path, e))?;

        self.lua
            .load(&script)
            .set_name(path)
            .exec()
            .map_err(|e| format!("脚本执行错误: {}", e))?;

        info!("脚本 {:?} 加载成功", path);
        Ok(())
    }

    fn call_function(&self, name: &str, args_bytes: &[u8]) -> Result<Vec<u8>, String> {
        // 反序列化参数为 f32（目前只支持单个 f32 参数）
        let arg: f32 =
            bincode::deserialize(args_bytes).map_err(|e| format!("反序列化参数失败: {}", e))?;

        let function: mlua::Function = self
            .lua
            .globals()
            .get(name)
            .map_err(|e| format!("获取 Lua 函数 {} 失败: {}", name, e))?;

        // 调用 Lua 函数
        let _: () = function
            .call(arg)
            .map_err(|e| format!("调用 Lua 函数 {} 失败: {}", name, e))?;

        // 返回空结果
        bincode::serialize(&()).map_err(|e| format!("序列化结果失败: {}", e))
    }

    fn get_config(&self, table_name: &str) -> Option<String> {
        let globals = self.lua.globals();
        let table: mlua::Table = globals.get(table_name).ok()?;
        let json_value: serde_json::Value = self.lua.from_value(mlua::Value::Table(table)).ok()?;
        serde_json::to_string(&json_value).ok()
    }

    fn execute_code(&self, code: &str) -> Result<(), String> {
        self.lua
            .load(code)
            .exec()
            .map_err(|e| format!("Lua 执行错误: {}", e))
    }

    fn execute_code_with_return(&self, code: &str) -> Result<String, String> {
        let result: mlua::Value = self
            .lua
            .load(code)
            .eval()
            .map_err(|e| format!("Lua 执行错误: {}", e))?;

        // 将返回值转换为字符串
        match result {
            mlua::Value::Nil => Ok("nil".to_string()),
            mlua::Value::Boolean(b) => Ok(b.to_string()),
            mlua::Value::Integer(i) => Ok(i.to_string()),
            mlua::Value::Number(n) => Ok(n.to_string()),
            mlua::Value::String(s) => Ok(s.to_string_lossy().to_string()),
            mlua::Value::Table(t) => {
                // 尝试序列化为 JSON
                let json_value: serde_json::Value = self
                    .lua
                    .from_value(mlua::Value::Table(t))
                    .map_err(|e| format!("序列化表失败: {}", e))?;
                Ok(json_value.to_string())
            }
            _ => Ok(format!("{:?}", result)),
        }
    }

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
                tracing::debug!("[Lua] {}", msg);
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

    fn register_mod_api(
        lua: &Lua,
        command_queue: Arc<Mutex<Vec<LuaCommand>>>,
        positions: Arc<Mutex<HashMap<String, [f32; 3]>>>,
    ) -> LuaResult<()> {
        // Entity API
        let entity = lua.create_table()?;

        let create_queue = Arc::clone(&command_queue);
        entity.set(
            "create",
            lua.create_function(move |lua, entity_type: String| {
                let id = format!("entity_{}", uuid::Uuid::new_v4());
                let command = LuaCommand::CreateEntity {
                    id: id.clone(),
                    entity_type: entity_type.clone(),
                };

                if let Ok(mut queue) = create_queue.lock() {
                    queue.push(command);
                }

                info!("[Mod API] 创建实体: {} ({})", entity_type, id);
                let table = lua.create_table()?;
                table.set("type", entity_type)?;
                table.set("id", id)?;
                Ok(table)
            })?,
        )?;

        let destroy_queue = Arc::clone(&command_queue);
        entity.set(
            "destroy",
            lua.create_function(move |_, id: String| {
                if let Ok(mut queue) = destroy_queue.lock() {
                    queue.push(LuaCommand::DestroyEntity { id });
                }
                Ok(())
            })?,
        )?;

        let set_pos_queue = Arc::clone(&command_queue);
        entity.set(
            "set_position",
            lua.create_function(move |_, (id, x, y, z): (String, f32, f32, f32)| {
                if let Ok(mut queue) = set_pos_queue.lock() {
                    queue.push(LuaCommand::SetPosition { id, x, y, z });
                }
                Ok(())
            })?,
        )?;

        let get_pos_map = Arc::clone(&positions);
        entity.set(
            "get_position",
            lua.create_function(move |lua, id: String| {
                let table = lua.create_table()?;
                let pos = get_pos_map
                    .lock()
                    .ok()
                    .and_then(|m| m.get(&id).copied())
                    .unwrap_or([0.0, 0.0, 0.0]);
                table.set("x", pos[0])?;
                table.set("y", pos[1])?;
                table.set("z", pos[2])?;
                Ok(table)
            })?,
        )?;

        let add_comp_queue = Arc::clone(&command_queue);
        entity.set(
            "add_component",
            lua.create_function(
                move |lua, (id, name, value): (String, String, mlua::Value)| {
                    let json_value = lua.from_value::<serde_json::Value>(value)?;
                    if let Ok(mut queue) = add_comp_queue.lock() {
                        queue.push(LuaCommand::AddComponent { id, name, value: json_value });
                    }
                    Ok(())
                },
            )?,
        )?;

        let remove_comp_queue = Arc::clone(&command_queue);
        entity.set(
            "remove_component",
            lua.create_function(move |_, (id, name): (String, String)| {
                if let Ok(mut queue) = remove_comp_queue.lock() {
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
            lua.create_function(|_, (event_name, _callback): (String, mlua::Value)| {
                info!("[Mod API] 订阅事件: {}", event_name);
                Ok(())
            })?,
        )?;

        event.set(
            "trigger",
            lua.create_function(|_, (event_name, _data): (String, mlua::Table)| {
                info!("[Mod API] 触发事件: {}", event_name);
                Ok(())
            })?,
        )?;

        lua.globals().set("Event", event)?;

        // World API（预留）
        let world = lua.create_table()?;
        world.set(
            "get_variable",
            lua.create_function(|_, _key: String| Ok(mlua::Value::Nil))?,
        )?;

        world.set(
            "set_variable",
            lua.create_function(|_, (key, value): (String, mlua::Value)| {
                info!("[Mod API] 设置世界变量: {} = {:?}", key, value);
                Ok(())
            })?,
        )?;

        lua.globals().set("World", world)?;

        // Player API（预留）
        let player = lua.create_table()?;
        player.set(
            "add_item",
            lua.create_function(|_, _item: mlua::Table| {
                info!("[Mod API] 添加物品到玩家");
                Ok(())
            })?,
        )?;

        lua.globals().set("Player", player)?;

        // History API（预留，用于历史事件干预）
        let history = lua.create_table()?;
        history.set(
            "on",
            lua.create_function(|_, (event_id, _handlers): (String, mlua::Table)| {
                info!("[Mod API] 注册历史事件钩子: {}", event_id);
                Ok(())
            })?,
        )?;

        lua.globals().set("History", history)?;

        Ok(())
    }
}
