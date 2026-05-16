# 性能编码规范

> 「先写对，再写快；但写的时候要知道哪里会慢。」

本文档是 Phase 2.8 性能优化的经验沉淀，用于指导后续 Rust/Lua 代码开发，避免重复踩坑。

---

## Rust 规范

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

---

*「性能是功能的一部分。」*
