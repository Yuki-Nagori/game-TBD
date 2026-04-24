# Phase 2.7: 全依赖升级计划 (Bevy 0.14 → 0.18.1)

> **目标**：将所有引擎依赖升级至最新稳定版本，消除技术债务，确保项目基于最新生态构建。

---

## 背景

当前引擎基于 Bevy 0.14，而 Bevy 最新稳定版为 **0.18.1**（2026-03-02 发布）。跨越 4 个 major 版本的升级涉及大量 Breaking Changes。同时，其他依赖（mlua、thiserror、bevy_rapier3d、bevy_egui 等）也已大幅更新。

---

## 依赖升级对照表

| 依赖 | 当前版本 | 目标版本 | 兼容性 | 风险等级 |
|:---|:---|:---|:---|:---|
| `bevy` | `0.14` | **`0.18.1`** | - | 🔴 高 |
| `bevy_rapier3d` | `0.27` | **`0.33.0`** | bevy 0.18 | 🔴 高 |
| `bevy_egui` | `0.28` | **`0.39.1`** | bevy 0.18 | 🔴 高 |
| `mlua` | `0.9` (`lua54`) | **`0.11.6` (`lua55`)** | - | 🟡 中 |
| `thiserror` | `1.0` | **`2.0.18`** | - | 🟡 中 |
| `anyhow` | `1.0` | **`1.0.102`** | - | 🟢 低 |
| `serde` | `1.0` | **`1.0.228`** | - | 🟢 低 |
| `serde_json` | `1.0` | **`1.0`** (最新补丁) | - | 🟢 低 |
| `hashbrown` | `0.14` | **`0.16.1`** | - | 🟢 低 |
| `lru` | `0.12` | **`0.16.3`** | hashbrown 0.16 | 🟢 低 |
| `uuid` | `1.7` | **`1.16`** | - | 🟢 低 |
| `notify` | `6.1` | **`8.2.0`** | - | 🟡 中 |
| `toml` | `0.8` | **`0.8`** (最新补丁) | - | 🟢 低 |
| `tracing` | `0.1` | **`0.1`** (最新补丁) | - | 🟢 低 |
| `tracing-subscriber` | `0.3` | **`0.3`** (最新补丁) | - | 🟢 低 |
| `chrono` | `0.4` | **`0.4`** (最新补丁) | - | 🟢 低 |
| `rand` | `0.9` | **`0.9`** | 已最新 | 🟢 低 |
| `bincode` | `1.3` | **`rkyv 0.8.15`** | API 完全不同 | 🔴 高 |
| `pretty_assertions` | `1.4` | **`1.4`** (最新补丁) | - | 🟢 低 |
| `criterion` | `0.5` | **`0.5`** (最新补丁) | - | 🟢 低 |
| `tempfile` | `3.10` | **`3.10`** (最新补丁) | - | 🟢 低 |

---

## Bevy 0.14 → 0.18 关键迁移点

> 参考官方迁移指南：
> - [0.14 → 0.15](https://bevyengine.org/learn/migration-guides/0-14-to-0-15/)
> - [0.15 → 0.16](https://bevyengine.org/learn/migration-guides/0-15-to-0-16/)
> - [0.16 → 0.17](https://bevyengine.org/learn/migration-guides/0-16-to-0-17/)
> - [0.17 → 0.18](https://bevyengine.org/learn/migration-guides/0-17-to-0-18/)

### 1. ECS 系统 (0.14 → 0.15 + 0.15 → 0.16)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| **Required Components** 替代 Bundle | 高 | `SpatialBundle` 等已弃用，改为直接 spawn 核心组件 |
| `Handle<T>` 不再实现 `Component` | 高 | 使用包装组件如 `Mesh3d`、`MeshMaterial3d` |
| `EventWriter::send()` → `write()` | 中 | 全局替换方法名 |
| `Query::many()` / `many_mut()` 弃用 | 低 | 改为 `Query::get_many()` |
| `Parent` → `ChildOf` (关系系统重构) | 高 | 所有父子关系代码需重写 |
| `insert_or_spawn()` 家族弃用 | 中 | 改用 `spawn` + `insert` |
| `World::flush_commands` 私有化 | 低 | 移除直接调用 |
| `Observer` 移除类型参数 | 中 | 更新 observer 定义 |
| `App::observe` → `add_observer` | 低 | 重命名 |
| `Event` trait 拆分为 `Message` + `Event` (0.16→0.17) | 高 | 所有自定义事件需重新分类 |
| `EntityEvent` 变为不可变 (0.17→0.18) | 中 | 使用 `SetEntityEventTarget` trait |
| **Entities API 大改** (0.17→0.18) | 高 | `spawn` 返回 `EntityRows`，批量 spawn 方式改变 |

### 2. 渲染与材质 (0.14 → 0.18)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| `Color::rgb()` → `Color::srgb()` / `Srgba` | 中 | 已在 0.14 迁移时处理，需再次确认 |
| `Mesh` + `Material` → Required Components | 高 | `PbrBundle` 等 Bundle 弃用 |
| `Camera` → `Camera3d` + `Camera` | 高 | 相机初始化方式改变 |
| `RenderTarget` 变为 Component (0.17→0.18) | 中 | 从 `Camera` 字段中移出 |
| `Msaa` 变为 Component | 低 | 调整抗锯齿配置位置 |
| `OrthographicProjection::scale` 移除 | 低 | 使用新 API |
| `bevy_render` 重组 (0.15→0.16) | 中 | 导入路径可能变化 |
| `bevy_gizmos` 渲染拆分 (0.17→0.18) | 低 | 可能影响 debug 绘制 |

### 3. UI 系统 (0.14 → 0.18)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| `UiImage` → `ImageNode` (0.15→0.16) | 中 | 重命名 |
| `TextStyle` 拆分 (0.14→0.15) | 中 | 使用 `TextFont` + `TextColor` + `TextLayout` |
| `BorderRadius` 并入 `Node` (0.17→0.18) | 低 | 不再是独立组件 |
| `LineHeight` 变为独立组件 (0.17→0.18) | 低 | 调整文本布局代码 |
| `TextLayoutInfo` 字段变更 (0.17→0.18) | 低 | `section_rects` → `run_geometry` |

### 4. 输入系统 (0.14 → 0.18)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| `Gamepad` 变为 Entity (0.14→0.15) | 中 | 游戏手柄查询方式改变 |
| `ReceivedCharacter` 移除 (0.14→0.15) | 低 | 改用 `KeyboardInput` |
| 输入源 feature-gated (0.17→0.18) | 低 | 显式启用 `mouse`/`keyboard`/`gamepad` 等 features |
| `KeyboardInput` 新增 `text` 字段 (0.15→0.16) | 低 | 可简化字符输入处理 |

### 5. 资产系统 (0.14 → 0.18)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| `AssetServer::load` API 一致性调整 | 低 | 加载状态检查方式微调 |
| `Handle::weak_from_u128()` 弃用 (0.15→0.16) | 低 | 使用 `Handle::Weak` 或新 API |
| `Assets::insert` 返回 `Result` (0.15→0.16) | 低 | 添加错误处理 |
| `LoadContext::path` 返回 `AssetPath` (0.17→0.18) | 低 | 类型转换 |
| `ron` 不再从 `bevy_scene`/`bevy_asset` re-export (0.17→0.18) | 中 | 添加显式 `ron` 依赖 |

### 6. 其他 (0.14 → 0.18)

| 变更 | 影响 | 迁移操作 |
|:---|:---|:---|
| **Rust Edition 2024** (0.15→0.16) | 低 | 项目已是 2024，无需变更 |
| `Timer::paused/finished` → `is_paused/is_finished` (0.16→0.17) | 低 | 重命名 |
| `bevy_core` 移除 (0.15→0.16) | 低 | 相关类型移到其他 crate |
| `SpatialBundle` 弃用 (0.14→0.15) | 中 | 使用 `Transform` + `Visibility` 等必需组件 |

---

## 第三方插件迁移

### bevy_rapier3d 0.27 → 0.33

- 需要跟踪 `bevy_rapier` 的 [CHANGELOG](https://github.com/dimforge/bevy_rapier/blob/master/CHANGELOG.md)
- 主要变更可能在 `RapierContext` API、`Collider` 构建器、`KinematicCharacterController` 等方面
- 当前使用 `KinematicCharacterController` 进行角色移动，需验证 0.33 的兼容性

### bevy_egui 0.28 → 0.39

- `bevy_egui` 的 [CHANGELOG](https://github.com/vladbat00/bevy_egui/blob/main/CHANGELOG.md)
- 0.39 适配 bevy 0.18
- `EguiContext` API 可能有变化
- `PICKING_ORDER` 常量已移除（0.39.0）， picking 顺序现在动态计算
- `bevy_picking` 支持被 feature-gated（0.34+）

---

## 特别风险项

### ⚠️ bincode → rkyv 迁移

`bincode` 作者于 2025-12-16 宣布停止维护。替换为 **`rkyv 0.8.15`**。

**rkyv 核心特点**：
- 零拷贝反序列化框架，性能极高
- 序列化后数据为 `Archived*` 类型（如 `ArchivedMyStruct`），可直接从字节缓冲区访问而无需分配
- 不基于 serde，使用自身的 `Archive`/`Serialize`/`Deserialize` traits
- 支持 `no_std` 和 `no_alloc`

**迁移要点**：

```rust
// bincode 时代
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct MyData {
    id: u64,
    name: String,
}

let encoded = bincode::serialize(&data)?;
let decoded: MyData = bincode::deserialize(&encoded)?;

// rkyv 时代
use rkyv::{Archive, Serialize, Deserialize};

#[derive(Archive, Serialize, Deserialize)]
struct MyData {
    id: u64,
    name: String,
}

// 序列化
let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&data)?;

// 零拷贝访问（无需反序列化）
let archived = rkyv::access::<MyData, rkyv::rancor::Error>(&bytes)?;
println!("{}", archived.name); // 直接访问 archived 字段

// 完整反序列化回 Rust 类型
let deserialized: MyData = rkyv::deserialize(archived)?;
```

**项目影响**：
- 当前 `bincode` 用于"Lua Actor 通信"的二进制序列化
- 所有使用 bincode 的数据结构需要添加 rkyv 的 derive macro
- rkyv 提供 `uuid-1` feature（兼容 `uuid 1.16`）和 `hashbrown-0_15` feature
- **注意**：rkyv 的 `String`/`Vec` 在 archived 形式中为 `ArchivedString`/`ArchivedVec`，与标准类型不同，若代码期望原生 Rust 类型，需要显式 `deserialize`

**Cargo.toml 配置**：
```toml
[dependencies]
rkyv = { version = "0.8.15", features = ["uuid-1", "hashbrown-0_15"] }
```

### ⚠️ mlua 0.9 → 0.11（`lua54` → `lua55`）

- Lua 版本升级：**`lua54` → `lua55`**（mlua 0.11.6 新增支持）
- `serialize` feature 被新的 `serde` feature 替代（0.11.0-beta.1+）
- `LuaSerdeExt` trait 可能有变化
- 当前使用 `lua54` + `vendored` + `serialize` features
- **迁移**：`lua54` → `lua55`，`serialize` → `serde`，验证 `LuaSerdeExt` 用法

### ⚠️ thiserror 1.0 → 2.0

- Breaking change：使用 `derive(Error)` 的代码必须直接依赖 `thiserror` crate
- 项目已直接依赖，影响较小
- 枚举级 `#[error("...")]` + variant 级 `transparent` 现在支持

### ⚠️ notify 6.1 → 8.0

- 文件监听 API 可能有 breaking changes
- 当前用于 Lua 热重载（`hot-reload` feature）
- 需验证 `RecommendedWatcher` 和事件类型是否变化

---

## 迁移步骤

### Step 1: 非 Bevy 依赖先行升级

1. 升级 `thiserror` → `2.0.18`
2. 升级 `anyhow` → `1.0.102`
3. 升级 `serde` → `1.0.228`
4. 升级 `hashbrown` → `0.16.1`
5. 升级 `lru` → `0.16.3`
6. 升级 `uuid` → `1.16`
7. 升级 `notify` → `8.0`
8. 升级 `mlua` → `0.11.6`（同步调整：`lua54` → `lua55`，`serialize` → `serde`）
9. **替换 `bincode` → `rkyv 0.8.15`**：
   - 添加 `rkyv = { version = "0.8.15", features = ["uuid-1", "hashbrown-0_15"] }`
   - 移除 `bincode` 依赖
   - 将所有 `#[derive(Serialize, Deserialize)]` 的通信数据结构添加 rkyv derive：`#[derive(Archive, Serialize, Deserialize)]`（rkyv 的 traits）
   - 替换 `bincode::serialize` / `bincode::deserialize` 调用为 `rkyv::to_bytes` / `rkyv::access` / `rkyv::deserialize`
10. `cargo check` 验证编译
11. `cargo test` 验证测试

### Step 2: Bevy 0.14 → 0.15

1. `bevy = "0.15"`
2. `bevy_rapier3d = "0.28"`（验证兼容性）
3. `bevy_egui = "0.31"`（bevy 0.15 对应版本）
4. 按 0.14→0.15 迁移指南处理：
   - Required Components（替换所有 Bundle）
   - `Handle<T>` 不再 `Component`
   - `TextStyle` 拆分
   - `SpatialBundle` 弃用
   - `Gamepad` Entity 化
5. `cargo check` → 修复错误 → `cargo test`

### Step 3: Bevy 0.15 → 0.16

1. `bevy = "0.16"`
2. `bevy_rapier3d = "0.30"`
3. `bevy_egui = "0.34"`
4. 按 0.15→0.16 迁移指南处理：
   - `Event` → `Message` / `Event` 拆分
   - `Parent` → `ChildOf`
   - `bevy_render` 重组
   - `EventWriter::send` → `write`
   - 其他 ECS API 变更
5. `cargo check` → 修复 → `cargo test`

### Step 4: Bevy 0.16 → 0.17

1. `bevy = "0.17"`
2. `bevy_rapier3d = "0.32"`
3. `bevy_egui = "0.37"`
4. 按 0.16→0.17 迁移指南处理：
   - `bevy_render` 进一步重组
   - `Message` / `Event` 最终调整
   - 大量重命名（`Timer` 方法、`Picking` API 等）
   - `Window` 拆分为多组件
5. `cargo check` → 修复 → `cargo test`

### Step 5: Bevy 0.17 → 0.18

1. `bevy = "0.18.1"`
2. `bevy_rapier3d = "0.33"`
3. `bevy_egui = "0.39.1"`
4. 按 0.17→0.18 迁移指南处理：
   - `RenderTarget` 变为 Component
   - `Entities` API 大改
   - Feature cleanup（`animation` → `gltf_animation` 等）
   - `BorderRadius` 并入 `Node`
   - `AmbientLight` 拆分
   - `ron` 显式依赖
5. `cargo check` → 修复 → `cargo test`

### Step 6: 最终验证

1. `cargo clippy --features dev-tools -- -D warnings`
2. `cargo test --features dev-tools`
3. `cargo fmt --check`
4. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features dev-tools`
5. 运行游戏验证：
   - 3D 场景加载
   - 角色移动 + 碰撞
   - 相机控制
   - 调试控制台（中文显示）
   - Lua 热重载
   - 场景编辑器

---

## 工作量评估

| 步骤 | 预估工作量 | 说明 |
|:---|:---|:---|
| Step 1（非 Bevy 依赖）| 1-2 天 | mlua 和 notify 可能有 API 变化 |
| Step 2（Bevy 0.15）| 3-5 天 | Required Components 改动量大 |
| Step 3（Bevy 0.16）| 2-3 天 | Event 拆分和关系系统 |
| Step 4（Bevy 0.17）| 2-3 天 | 大量重命名和渲染重组 |
| Step 5（Bevy 0.18）| 2-3 天 | Entities API 和 Feature 调整 |
| Step 6（验证）| 1-2 天 | 全功能回归测试 |
| **总计** | **11-18 天** | 取决于每个版本的实际破坏程度 |

---

## 回滚策略

- 每个 Step 完成后立即提交（commit）
- 为每个 Bevy 版本创建独立分支（`bevy-0.15`、`bevy-0.16` 等）
- 如果某一步升级遇到无法解决的问题，可停留在该版本并记录阻塞项

---

## 参考资源

- [Bevy 0.18 Release Notes](https://bevyengine.org/news/bevy-0-18/)
- [Bevy 0.17 Release Notes](https://bevyengine.org/news/bevy-0-17/)
- [Bevy 0.16 Release Notes](https://bevyengine.org/news/bevy-0-16/)
- [Bevy 0.15 Release Notes](https://bevyengine.org/news/bevy-0-15/)
- [bevy_rapier CHANGELOG](https://github.com/dimforge/bevy_rapier/blob/master/CHANGELOG.md)
- [bevy_egui CHANGELOG](https://github.com/vladbat00/bevy_egui/blob/main/CHANGELOG.md)
- [mlua CHANGELOG](https://github.com/mlua-rs/mlua/blob/main/CHANGELOG.md)

---

*计划创建时间：2026-04-25*
*负责人：待分配*
