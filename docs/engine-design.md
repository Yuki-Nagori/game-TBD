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

# 存档系统
rusqlite = { version = "0.32", features = ["bundled"] }  # SQLite
fjall = "2.0"           # LSM-Tree KV（缓存/日志）
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

## 存档系统

### 混合架构

| 数据库 | 用途 | 场景 |
|:---|:---|:---|
| **SQLite** | 主数据库 | 装备、技能、成就、背包、NPC 关系 |
| **Fjall** | 缓存/日志 | 高性能读写、事件流、临时数据 |

### SQLite 表结构

```sql
-- 存档元数据
CREATE TABLE save_meta (
    slot INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    play_time INTEGER DEFAULT 0,  -- 秒
    game_year INTEGER,
    game_month INTEGER,
    thumbnail BLOB                -- 截图缩略图
);

-- 蝴蝶效应变量
CREATE TABLE butterfly_vars (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,          -- JSON
    updated_at INTEGER
);

-- 角色状态
CREATE TABLE characters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT,                    -- player / npc
    data TEXT NOT NULL,           -- JSON
    location TEXT,
    updated_at INTEGER
);

-- 装备
CREATE TABLE equipment (
    id TEXT PRIMARY KEY,
    owner_id TEXT,                -- 角色ID
    template_id TEXT NOT NULL,    -- 装备模板
    enhance_level INTEGER DEFAULT 0,
    affixes TEXT,                 -- JSON 随机词缀
    FOREIGN KEY (owner_id) REFERENCES characters(id)
);

-- 技能
CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    template_id TEXT NOT NULL,
    level INTEGER DEFAULT 1,
    exp INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT 1,
    FOREIGN KEY (owner_id) REFERENCES characters(id)
);

-- 成就
CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    unlocked_at INTEGER,
    progress INTEGER DEFAULT 0
);
```

### Fjall 使用场景

```rust
// 高性能事件日志
let event_log = keyspace.open_partition("events", ...)?;
event_log.insert(timestamp, event_data).await?;

// 临时缓存（如战斗中的伤害统计）
let combat_cache = keyspace.open_partition("combat", ...)?;
```

### 存档目录

```
save/
├── slot1.db              # SQLite 主存档
├── slot1/
│   └── events/           # Fjall 事件日志
├── slot2.db
├── slot2/
│   └── events/
└── auto/                 # 自动存档
    ├── auto_001.db
    └── auto_001/
        └── events/
```

---

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
// tests/save_system.rs — 测试存档读写
```

---

*「引擎是舞台，代码是幕布」*
