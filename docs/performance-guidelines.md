# 性能编码规范

> 「先写对，再写快；但写的时候要知道哪里会慢。」

本文档是 Phase 2.8 性能优化的经验沉淀，用于指导后续 Rust/Lua 代码开发，避免重复踩坑。

---

## Rust 规范

### 0. 错误处理

#### 0.1 用 `Result` 传播错误，不 panic

```rust
// ❌ 坏：公共 API 中 panic
pub fn load_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path).unwrap(); // 禁止！
    parse_config(&content)
}

// ✅ 好：返回 Result，让调用者决定
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::Io(path.to_string(), e))?;
    parse_config(&content)
}
```

**为什么**：游戏引擎在运行时不应因单个文件加载失败而崩溃。`Result` 允许优雅降级（如使用默认配置）。

#### 0.2 错误类型使用 `thiserror` 派生

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("文件未找到: {0}")]
    NotFound(String),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("解析失败: {0}")]
    Parse(String),
}
```

**原则**：
- 每个模块定义自己的错误枚举
- 使用 `#[from]` 自动转换底层错误
- 错误消息面向开发者，包含上下文（文件名、行号等）

#### 0.3 panic 策略

| 场景 | 处理方式 |
|:---|:---|
| 文件不存在 | `Result::Err` |
| 配置格式错误 | `Result::Err` |
| 资源加载失败 | `Result::Err` |
| 内部状态不一致（bug） | `expect("不变式说明")` |
| 除以零 / 数组越界（不应发生） | `expect("理由")` 或 `debug_assert!` |

**禁止裸 `unwrap()`**。如果必须用，使用 `expect("说明为什么这里不会失败")`。

```rust
// ✅ 好： unwrap 附带理由
let config = load_config(path).expect("主配置必须在启动时存在");

// ❌ 坏：裸 unwrap
let config = load_config(path).unwrap();
```

### 0.4 并发与线程安全

#### Actor 模式替代共享可变状态

```rust
// ✅ 好：LuaRuntime 使用 Actor 模式，内部线程持有 Lua 状态
pub struct LuaRuntime {
    sender: Sender<LuaRequest>,      // Send + Sync
    positions: Arc<Mutex<HashMap<...>>>, // 线程安全共享
}

// ❌ 坏：直接暴露非 Send 类型
pub struct BadRuntime {
    lua: Lua, // !Send，不能在 Bevy 系统中使用
}
```

**原则**：
- `Resource` 必须是 `Send + Sync`
- 非 `Send` 类型（如 `mlua::Lua`）必须包装在独立线程中，通过通道通信
- `Mutex` 优先使用 `std::sync::Mutex`（而非 `parking_lot`），保持标准库一致性
- 锁持有时间最小化：只保护临界区，不要在锁内做 IO 或复杂计算

#### 锁使用规范

```rust
// ✅ 好：锁持有时间最短
let commands = {
    let mut queue = self.command_queue.lock().unwrap();
    std::mem::take(&mut *queue)
};
// 锁已释放，后续处理不阻塞其他线程

// ❌ 坏：锁内做复杂操作
let mut queue = self.command_queue.lock().unwrap();
for cmd in &commands {
    process(cmd); // 阻塞其他线程！
}
```

---

### 1. ECS 查询

#### 1.1 使用 `With<T>` / `Without<T>` 过滤，避免全表扫描

```rust
// ❌ 坏：遍历所有实体再 match
for (entity, maybe_player) in &query {
    if maybe_player.is_some() { /* ... */ }
}

// ✅ 好：查询时过滤
fn system(query: Query<Entity, With<Player>>) { /* ... */ }
```

**为什么**：Bevy 的 Query archetype 匹配在编译期确定，过滤后只遍历目标 archetype，跳过无关实体。

#### 1.2 组合查询替代多个独立 Query

```rust
// ❌ 坏：5 个独立 Query，每次都要遍历
fn bad(q1: Query<&A>, q2: Query<&B>, q3: Query<&C>) {}

// ✅ 好：1 个组合查询，一次遍历
fn good(q: Query<(Entity, Option<&A>, Option<&B>, Option<&C>)>) {}
```

#### 1.3 低频系统内部降频，不要挂 `FixedUpdate`

```rust
// ✅ 好：在 Update 中内部计数器降频
fn check_scene_switch(mut frame_counter: Local<u8>) {
    *frame_counter = frame_counter.wrapping_add(1);
    if !frame_counter.is_multiple_of(30) {
        return;
    }
    // 实际逻辑...
}
```

**为什么**：`FixedUpdate` 会增加调度复杂度，内部降频更灵活，且仍可享受 `Update` 的并行调度。

#### 1.4 使用 `SystemSet` 分组，但不要过度串行化

```rust
// ✅ 好：定义集合但仅在需要时 chain
app.configure_sets(Update, (InputSet, LogicSet, RenderSet).chain());
app.add_systems(Update, (a, b).in_set(InputSet)); // a, b 可并行
```

**为什么**：`chain()` 会让集合内系统串行执行。只在有数据依赖时才 `chain()`，无关系统应并行。

---

### 2. 内存分配

#### 2.1 预分配 Vec/VecDeque/String 容量

```rust
// ❌ 坏：运行时反复扩容
let mut history = VecDeque::new();
for _ in 0..120 {
    history.push_back(value); // 扩容 5-6 次
}

// ✅ 好：预分配
let mut history = VecDeque::with_capacity(120);
```

**适用场景**：PerformanceMonitor 的历史缓冲区、日志队列、批量生成实体列表。

#### 2.2 循环内避免重复 `to_lowercase()` / `format!`

```rust
// ❌ 坏：每实体都分配
for entity in &entities {
    if id_str.to_lowercase().contains(&filter.to_lowercase()) {}
}

// ✅ 好：提取到循环外
let filter_lower = filter.to_lowercase();
let filter_empty = filter_lower.is_empty();
for entity in &entities {
    if !filter_empty && !id_str.to_lowercase().contains(&filter_lower) {}
}
```

#### 2.3 日志使用 `tracing` 级别过滤，避免字符串构造

```rust
// ❌ 坏：每次都要 format，即使日志级别被过滤
einfo!("entity {:?} position {:?}", entity, pos);

// ✅ 好：使用 tracing 的懒求值（已经自动处理）
tracing::info!("entity {:?} position {:?}", entity, pos);
```

**注意**：`tracing` crate 会在级别不匹配时跳过格式构造，无需手动 `if log_enabled!()`。

---

### 3. 资源加载

#### 3.1 AssetManager 仅轮询 `Loading` 状态资源

```rust
// ✅ 好：只收集需要轮询的路径
let paths: Vec<String> = self.states
    .iter()
    .filter(|(_, s)| matches!(s, AssetLoadState::Loading { .. }))
    .map(|(k, _)| k.clone())
    .collect();
```

**为什么**：避免每次轮询都 clone 所有状态的 key，Ready/Failed 状态无需再检查。

#### 3.2 资源轮询系统降低频率

```rust
// ✅ 好：每 4 帧轮询一次
const POLL_INTERVAL: u8 = 4;
fn asset_manager_poll_system(mut counter: Local<u8>) {
    *counter = counter.wrapping_add(1);
    if !counter.is_multiple_of(POLL_INTERVAL) { return; }
    // poll...
}
```

---

### 4. 渲染

#### 4.1 场景对象批量 spawn

```rust
// ❌ 坏：逐个 spawn，每次都要命令队列分配
for pos in positions {
    commands.spawn((Mesh3d(mesh.clone()), ...));
}

// ✅ 好：使用 spawn_batch（Bevy 0.18+）
commands.spawn_batch(positions.into_iter().map(|pos| {
    (Mesh3d(mesh.clone()), Transform::from_translation(pos), ...)
}));
```

#### 4.2 阴影与后处理

- **阴影**：Bevy 0.18 `DirectionalLight` 不支持直接配置 `shadow_map_resolution`，如需调整需通过 `RenderPlugin` 的 `render_creation` 配置
- **MSAA**：Low Poly 风格默认关闭，如需开启优先 `2x` 而非 `4x/8x`
- **后处理**：不要默认开启 SSAO / Bloom / Tonemapping，除非美术需要

---

### 5. 物理（Rapier3D）

#### 5.1 静态碰撞体使用 `Collider::cuboid`，避免 MeshCollider

```rust
// ✅ 好：AABB 碰撞体计算极快
collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0)

// ❌ 坏：三角形网格碰撞体计算开销高
Collider::trimesh_from_mesh(mesh) // 仅在复杂形状且必须精确碰撞时使用
```

#### 5.2 物理模拟频率根据场景调整

```rust
// 默认 60Hz，非战斗场景可降至 45Hz
app.insert_resource(TimestepMode::Fixed {
    dt: 1.0 / 45.0,
    substeps: 1,
});
```

**注意**：战斗场景需恢复 60Hz 以保证手感。通过 `Resource<TimestepMode>` 动态切换。

#### 5.3 远离玩家的物体进入睡眠

```rust
// 后续添加 NPC/动态物体时启用
commands.spawn((
    RigidBody::Dynamic,
    Sleeping::disabled(), // 默认即可，Rapier 会自动睡眠
    // 或通过距离判断手动设置
));
```

---

### 6. Lua 跨边界调用

#### 6.1 Lua Actor 缓存函数引用

```rust
// ✅ 好：首次获取后缓存到 mlua RegistryKey
struct LuaActor {
    function_cache: HashMap<String, mlua::RegistryKey>,
}

fn call_cached(&mut self, name: &str, arg: f32) {
    let func: mlua::Function = if let Some(key) = self.function_cache.get(name) {
        self.lua.registry_value(key)?
    } else {
        let func: mlua::Function = self.lua.globals().get(name)?;
        let key = self.lua.create_registry_value(&func)?;
        self.function_cache.insert(name.to_string(), key);
        func
    };
    func.call(arg)
}
```

**为什么**：`globals().get::<Function>` 每次都要查全局表（hash 查找），缓存后直接从注册表取。

#### 6.2 低频系统降低 Lua 调用频率

```rust
// ✅ 好：lua_update 每 2 帧调用，位置同步每 6 帧
fn lua_update_system(mut frame_counter: Local<u8>) {
    *frame_counter = frame_counter.wrapping_add(1);
    if !frame_counter.is_multiple_of(2) { return; }
    lua.call_function("update", dt)?;
}
```

**为什么**：每次 `call_function` 都是跨线程通道通信（Send + Recv），有显著开销。

#### 6.3 批量同步实体位置

```rust
// ✅ 好：一次遍历所有实体，而非每实体一调
for (id, entity) in &registry.by_id {
    if let Ok(transform) = query.get(*entity) {
        lua.update_entity_position(id, transform.translation);
    }
}
```

---

## Lua 规范

### 0. 错误处理与日志

#### 0.1 Lua 脚本不 panic

```lua
-- ❌ 坏：脚本错误导致引擎崩溃
function process_event(event)
    if not event.id then
        error("event 缺少 id") -- 禁止！会传播到 Rust 层导致 panic
    end
end

-- ✅ 好：错误通过日志报告，返回降级结果
function process_event(event)
    if not event.id then
        log_error("process_event: event 缺少 id，跳过处理")
        return nil
    end
    -- 正常处理...
end
```

**原则**：
- Lua 脚本禁止调用 `error()` 或 `assert(false)` 抛出不可恢复错误
- 所有错误通过 `log_error` / `log_warn` 报告
- 函数返回 `nil` 或默认值表示降级处理

#### 0.2 日志级别使用规范

```lua
-- trace：最详细的调试信息（每帧调用，生产环境关闭）
log_debug("帧更新: dt=" .. dt)

-- debug：开发调试信息（场景加载、状态变化）
log_debug("场景切换: " .. scene_name)

-- info：正常运行信息（玩家操作、事件触发）
log_info("玩家进入区域: " .. area_id)

-- warn：非致命异常（资源缺失、配置回退）
log_warn("贴图未找到，使用默认: " .. texture_path)

-- error：功能失效（脚本语法错误、必要数据缺失）
log_error("任务数据加载失败: " .. quest_id)
```

**性能注意**：`log_debug` 在 release 模式下会被 Rust 层过滤，但字符串构造仍发生。高频日志使用条件编译：

```lua
-- ✅ 好：高频日志加条件
if DEBUG_MODE then
    log_debug("每帧位置: " .. tostring(pos))
end
```

---

### 1. GC 调优

#### 1.1 初始化时设置 GC 参数

```lua
-- game/main.lua 或 config 中执行一次
-- setpause=150: 内存增长 50% 后才触发新一轮 GC（默认 200=100%）
-- setstepmul=200: 每步回收速度加倍，单次停顿更短（默认 200）
collectgarbage("setpause", 150)
collectgarbage("setstepmul", 200)
```

**为什么**：游戏运行时 Lua 表增长缓慢（主要是配置和状态），降低 GC 频率可减少帧时间抖动。

#### 1.2 大表预分配容量

```lua
-- ❌ 坏：运行时反复 rehash
local t = {}
for i = 1, 1000 do
    t[i] = { ... }
end

-- ✅ 好：预分配
local t = {}
t[1000] = nil  -- 预分配数组部分
for i = 1, 1000 do
    t[i] = { ... }
end
```

**注意**：Lua 5.5 支持 `table.move` 和 `table.pack`，但预分配仍推荐通过 `t[n] = nil` 提示容量。

---

### 2. 配置加载

#### 2.1 配置表扁平化，减少嵌套深度

```lua
-- ❌ 坏：深层嵌套导致 serde 反序列化开销高
PLAYER_CONFIG = {
    model = {
        scene = "...",
        transform = {
            scale = 1.0,
            rotation = { x = 0, y = 0, z = 0 }
        }
    }
}

-- ✅ 好：扁平化，Rust 端直接对应 struct
PLAYER_CONFIG = {
    model_scene = "...",
    scale = 1.0,
    yaw_offset = 0.0,
}
```

**为什么**：每多一层嵌套，mlua + serde 的反序列化就要多一次递归和临时表分配。

#### 2.2 配置加载后缓存到全局，不要重复 require

```lua
-- ✅ 好：main.lua 加载一次，全局使用
local player_config = require("config/player")
PLAYER_CONFIG = {
    model_scene = player_config.model.scene,
    scale = player_config.model.scale,
}

-- ❌ 坏：每个系统都 require
function some_system()
    local cfg = require("config/player")  -- 重复查找 package.loaded
end
```

---

### 3. 事件与回调

#### 3.1 避免每帧触发大量 Lua 事件

```lua
-- ❌ 坏：Rust 每帧回调 Lua
for _, npc in ipairs(npcs) do
    Event.trigger("npc_update", { id = npc.id })  -- 100 个 NPC = 100 次跨边界
end

-- ✅ 好：批量回调
Event.trigger("npc_batch_update", npcs)  -- 1 次跨边界
```

#### 3.2 使用局部变量缓存全局函数

```lua
-- ✅ 好：函数开头缓存
local insert = table.insert
local format = string.format

function process_items(items)
    for _, item in ipairs(items) do
        insert(result, format("item_%d", item.id))
    end
end
```

---

## 模块组织与 API 设计

### 1. 模块边界

- **每个 `.rs` 文件只负责一个概念域**。文件超过 300 行考虑拆分。
- **公共 API 最小化**：`pub` 只暴露必要接口，内部实现用 `pub(crate)` 或 `pub(super)`。
- **禁止循环依赖**：`core/` 不依赖 `plugins/`，`plugins/` 通过 `Resource` 和 `Event` 通信。

```rust
// ✅ 好：模块层次清晰
engine/src/
├── lib.rs           # 公共 API 门面
├── core/            # 纯逻辑（无 Bevy 依赖或仅依赖 bevy::math）
├── components/      # ECS 组件定义
├── resources/       # 全局状态
├── lua_api/         # Lua 运行时与 API
├── asset_manager.rs # 资源管理
├── font_center.rs   # UI 字体基础设施
├── utils.rs         # 纯工具函数
└── plugins/         # Bevy 插件
    ├── mod.rs       # GamePlugin 汇总
    ├── player_plugin.rs
    ├── camera_plugin.rs
    └── ...
```

### 2. 向后兼容

- **Lua API 变更必须保留旧接口至少一个版本周期**。
- **公共 Resource 的字段增改**：新增字段用 `Option<T>`，旧代码无感。
- **枚举新增变体**：非 exhaustive 枚举对外暴露时需处理 `Unknown` 情况。

```rust
// ✅ 好：新增字段不破坏旧代码
#[derive(Resource)]
pub struct GameConfig {
    pub player_speed: f32,
    // Phase 3 新增，旧配置兼容
    pub cultivation_enabled: Option<bool>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player_speed: 5.0,
            cultivation_enabled: Some(false), // 默认关闭，旧存档安全
        }
    }
}
```

### 3. Component 设计

- **Component 必须是纯数据**，禁止包含业务逻辑方法。
- **允许的方法**：构造函数 `new()`、`Default` 实现、简单的 getter/setter。
- **复杂计算放在 System 中**，不要在 Component 内持有 `Query` 或 `Commands`。

```rust
// ✅ 好：纯数据 + 简单构造
#[derive(Component)]
pub struct CharacterMotion {
    pub is_moving: bool,
    pub facing_yaw: f32,
}

impl CharacterMotion {
    pub fn new() -> Self {
        Self { is_moving: false, facing_yaw: 0.0 }
    }
}

// ❌ 坏：Component 持有业务逻辑
#[derive(Component)]
pub struct BadCharacter {
    // ...
}

impl BadCharacter {
    pub fn attack(&mut self, target: Entity, commands: &mut Commands) {
        // 业务逻辑不应在 Component 中！
    }
}
```

---

## 文档注释规范

### 1. Rust 文档（rustdoc）

- 所有 `pub` 项必须有 `///` 文档。
- 文档包含：一句话描述、详细说明（如需要）、参数、返回值、错误、示例。

```rust
/// 推进游戏内时间
///
/// 处理小时、日、月、年的进位。每月固定 30 天。
///
/// # 参数
/// - `hours`: 推进的小时数，可为负数（回退时间）
///
/// # 示例
/// ```
/// let mut time = GameTime::default();
/// time.advance(25.0); // 推进 25 小时 = 1 天 1 小时
/// assert_eq!(time.day, 2);
/// ```
pub fn advance(&mut self, hours: f32) {
    // ...
}
```

### 2. Lua 文档（ldoc）

- 所有导出的 Lua 函数/表必须有 ldoc 注释。
- 配置文件头部说明用途和字段类型。

```lua
--- 玩家配置
-- @table PLAYER_CONFIG
-- @field model_scene string 模型文件路径（相对于 assets/models/）
-- @field scale number 模型缩放比例
-- @field speed number 移动速度（单位/秒）
PLAYER_CONFIG = {
    model_scene = "player.gltf",
    scale = 1.0,
    speed = 5.0,
}
```

---

## 检查清单（Code Review 用）

提交 PR 前自检：

- [ ] Rust 中无循环内 `format!` / `to_lowercase()` / `to_string()` 重复分配
- [ ] Query 使用了 `With<T>` / `Without<T>` 过滤
- [ ] Vec/VecDeque 有 `with_capacity`
- [ ] Lua 函数调用有缓存（非每帧 `globals().get`）
- [ ] 低频系统有内部降频（`frame_counter`）
- [ ] 静态碰撞体使用 `cuboid` 而非 `trimesh`
- [ ] 资源轮询有间隔（非每帧）
- [ ] Lua 大表有预分配
- [ ] clippy 零警告 + luacheck 零警告
- [ ] 无裸 `unwrap()` / `unwrap_unchecked()`（有 `expect` 理由）
- [ ] 公共 API 有 `///` 文档注释
- [ ] Component 是纯数据，无业务逻辑方法
- [ ] Lua 脚本无 `error()` / `assert(false)` 不可恢复错误
- [ ] 新增字段使用 `Option<T>` 保证向后兼容（如适用）

---

*「性能是功能的一部分。」*
