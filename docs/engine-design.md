# 引擎设计

> Rust 核心引擎技术实现细节

## 架构概述

```
┌─────────────────────────────────────────┐
│           Bevy 应用层                    │
│  (窗口、渲染、输入、事件循环)              │
├─────────────────────────────────────────┤
│           游戏逻辑层 (ECS)               │
│  (GameTime、Character、Cultivation...)   │
├─────────────────────────────────────────┤
│           Lua 运行时层                   │
│  (mlua、API 注册、热重载)                 │
├─────────────────────────────────────────┤
│           资源管理层                     │
│  (脚本加载、配置解析、存档系统)            │
└─────────────────────────────────────────┘
```

## 目录结构

```
engine/
├── Cargo.toml           # Workspace 根配置
├── src/
│   ├── main.rs          # 主入口
│   ├── core/            # 核心 ECS 组件与系统
│   │   └── mod.rs       # GameTime、Character、Cultivation
│   └── lua_api/         # Lua 绑定与 API 暴露
│       └── mod.rs       # LuaRuntime、API 注册
└── crates/              # Workspace 子 crate（未来拆分）
    └── core/            # 占位：阶段 2 迁移至此
        ├── Cargo.toml
        └── src/lib.rs
```

## Workspace 演进计划

| 阶段 | 状态 | 描述 |
|:---|:---|:---|
| 阶段 1 | 🔨 当前 | 单 crate，代码在 `src/`，`crates/core/` 为占位符 |
| 阶段 2 | 📝 待开始 | 将 `src/core/` 迁移至 `crates/core/`，独立编译 |
| 阶段 3 | 📝 远期 | 拆分为 `core`、`renderer`、`lua_api`、`save_system` |

**拆分好处：**
1. **编译加速** — 修改一个 crate 不影响其他（增量编译缓存）
2. **依赖隔离** — 核心逻辑不依赖渲染/网络库
3. **代码复用** — `renderer` 可作为独立库发布
4. **独立测试** — 每个 crate 可单独 `cargo test`

## 核心模块

### 1. GameTime（游戏时间系统）

```rust
#[derive(Resource)]
pub struct GameTime {
    year: i32,      // 1573 起
    month: u8,      // 1-12
    day: u8,        // 1-30（简化）
    hour: f32,      // 0.0-24.0
    paused: bool,   // 暂停标志
}

// 1 小时现实时间 = 1 年游戏时间
// 战斗/对话时暂停
```

### 2. LuaRuntime（Lua 运行时）

```rust
pub struct LuaRuntime {
    lua: Lua,
}

impl LuaRuntime {
    pub fn new() -> Result<Self>;
    pub fn load_main_script(&self, path: &str) -> Result<()>;
    pub fn register_api(&self) -> Result<()>;
}
```

### 3. 热重载系统（开发模式）

```rust
#[derive(Resource)]
pub struct HotReloadSystem {
    watcher: notify::RecommendedWatcher,
    reload_queue: Vec<PathBuf>,
}

// 监听 scripts/ 目录变化
// 文件修改后下一帧重载
// Release 模式禁用
```

## 依赖清单

```toml
[dependencies]
bevy = "0.13"           # ECS 游戏引擎
mlua = { version = "0.9", features = ["luajit"] }  # Lua 绑定
anyhow = "1.0"          # 错误处理
tracing = "0.1"         # 日志
serde = { version = "1.0", features = ["derive"] }  # 序列化
notify = "6.1"          # 文件监听（热重载）
uuid = { version = "1.7", features = ["v4"] }      # 实体 ID
```

## 构建流程

```
xmake f -m release    # 配置 Release 模式
xmake b               # 构建
    └── 调用 cargo build --release
    └── 复制引擎二进制到 build/

xmake run             # 运行
    └── 启动 Bevy 应用
    └── 初始化 Lua 运行时
    └── 加载 game/main.lua
```

## 与 Lua 的交互

### Rust → Lua

```rust
// 暴露函数给 Lua
lua.globals().set("log_info", lua.create_function(|_, msg: String| {
    info!("[Lua] {}", msg);
    Ok(())
})?)?;
```

### Lua → Rust

```lua
-- Lua 调用 Rust 提供的 API
log_info("游戏启动")
Time.pause()
local year = Time.get_year()
```

## 性能考虑

1. **ECS 查询缓存** — Bevy 自动处理
2. **Lua 调用开销** — 每帧批量处理，避免频繁跨边界
3. **资源异步加载** — Bevy AssetServer
4. **增量编译** — Workspace 拆分后生效

## 测试策略

```rust
// 单元测试
#[cfg(test)]
mod tests {
    #[test]
    fn test_game_time_advance() {
        let mut time = GameTime::default();
        time.advance(48.0);
        assert_eq!(time.day, 3);
    }
}

// 集成测试
// tests/lua_api.rs — 测试 Lua 绑定
```

---

*「引擎是舞台，代码是幕布」*
