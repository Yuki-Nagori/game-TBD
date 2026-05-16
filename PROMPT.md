# 项目工作提示词 — 明朝修仙 RPG

> 本文件是本项目通用的工作约束。每次执行开发任务前，请先阅读相关章节。
> 核心原则：**简单明确、错误透明、可测试、性能有意识。**

---

## 1. 项目定位

- **代号**：TBD（待命名）
- **类型**：3D 历史玄幻 RPG
- **技术栈**：Rust (Bevy 0.18) + Lua 5.5 + xmake
- **架构**：引擎（Rust/Bevy）与剧本（Lua）分离，支持 Mod
- **平台**：Windows / Linux / macOS

---

## 2. 核心架构原则

### 2.1 ECS 优先

- **所有游戏状态必须是 Component 或 Resource**。禁止在系统中持有可变状态。
- **系统即纯函数**：输入 `Query` + `Res` / `ResMut`，输出副作用（`Commands`、`EventWriter`）。
- **数据驱动**：配置优先从 Lua 加载，技术限制留在 Rust 常量。

```rust
// ✅ 好：系统是纯函数，状态在组件中
fn move_player(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &PlayerInput), With<Player>>,
) { /* ... */ }

// ❌ 坏：系统在内部维护状态
fn bad_system(mut state: Local<Vec3>) { /* ... */ }
```

### 2.2 引擎与脚本分离

| 层级 | 语言 | 职责 | 修改频率 |
|:---|:---|:---|:---|
| 引擎 | Rust | 渲染、物理、ECS、资源管理、Lua 运行时 | 低 |
| API | Lua C API / mlua | 暴露给脚本的接口 | 中 |
| 剧本 | Lua | 游戏逻辑、剧情、配置 | 高 |

- **Rust 不直接实现游戏玩法规则**。玩法规则通过 Lua API 暴露给脚本层。
- **Lua 不直接操作 Bevy 内部**。Lua 通过 `LuaCommand` 队列向引擎发指令。

### 2.3 插件化组织

每个功能域一个 Bevy Plugin：

```rust
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerConfig>()
           .add_systems(Update, (handle_input, update_movement).in_set(GameSystemSet::Input));
    }
}
```

- Plugin 之间通过 `Resource` 和 `Event` 通信，禁止直接调用。
- 公共逻辑提取到 `utils.rs` 或独立模块，不要在 Plugin 间复制代码。

---

## 3. 代码规范

### 3.1 Rust 规范

#### 错误处理

- **用 `Result` 传播错误，不 panic**。公共 API 返回 `Result`，内部可用 `expect` 但需附理由。
- **错误类型使用 `thiserror` 派生**，提供清晰的错误链。
- **禁止裸 `unwrap()` 和 `unwrap_unchecked()`**。如果必须 unwrap，使用 `expect("理由")`。

```rust
// ✅ 好
pub fn load_manifest(path: &Path) -> Result<AssetManifest, ManifestError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ManifestError::Io(e))?;
    toml::from_str(&content)
        .map_err(|e| ManifestError::Parse(e))
}

// ❌ 坏
let content = std::fs::read_to_string(path).unwrap();
```

#### unsafe 代码

- **原则上禁止 `unsafe`**。如必须使用，需满足：
  1. 附有详细的安全注释（`// SAFETY: ...`）
  2. 经过代码审查
  3. 有对应的单元测试验证不变式

#### 类型与命名

- 结构体/枚举使用 `PascalCase`，函数/变量使用 `snake_case`，常量使用 `SCREAMING_SNAKE_CASE`。
- 布尔变量用 `is_` / `has_` / `should_` 前缀。
- 生命周期参数单字母：`'a`, `'b`。复杂场景用描述性名称。

```rust
// ✅ 好
pub struct AssetManager { /* ... */ }
pub fn load_texture(path: &str) -> Result<Handle<Image>, AssetError> { /* ... */ }
pub const MAX_CACHE_SIZE: usize = 64;
```

### 3.2 Lua 规范

- 使用 `snake_case` 命名变量和函数，全局配置表使用 `UPPER_SNAKE_CASE`。
- 模块返回表，不污染全局环境。
- 配置加载后缓存到全局，禁止重复 `require`。

```lua
-- ✅ 好：模块返回表
local M = {}
function M.process_event(event)
    -- ...
end
return M

-- ✅ 好：main.lua 中缓存配置
local player_config = require("config/player")
PLAYER_CONFIG = {
    model_scene = player_config.model.scene,
    scale = player_config.model.scale,
}
```

---

## 4. 性能约束（必读）

**所有代码必须遵守 `docs/performance-guidelines.md`**。以下是最关键的摘要：

### 4.1 ECS 查询

- 使用 `With<T>` / `Without<T>` 过滤，禁止全表扫描。
- 组合查询替代多个独立 `Query`。
- 低频系统内部降频，不要挂 `FixedUpdate`。

### 4.2 内存分配

- 循环内禁止重复 `format!` / `to_lowercase()` / `to_string()`。
- `Vec` / `VecDeque` / `String` 预分配容量。
- `tracing` 日志级别自动过滤，无需手动 `if log_enabled!()`。

### 4.3 资源与加载

- `AssetManager` 仅轮询 `Loading` 状态资源。
- 资源轮询系统每 4 帧执行一次。

### 4.4 渲染与物理

- 批量 `spawn_batch` 替代逐个 `spawn`。
- 静态碰撞体使用 `Collider::cuboid`，禁止 `trimesh_from_mesh`。
- 物理模拟频率非战斗场景 45Hz。

### 4.5 Lua 跨边界

- `LuaActor` 缓存函数引用到 `RegistryKey`。
- `lua_update` 每 2 帧调用，位置同步每 6 帧。
- 批量事件替代每帧单个事件。

---

## 5. 测试与质量

### 5.1 测试策略

| 类型 | 位置 | 范围 | 要求 |
|:---|:---|:---|:---|
| 单元测试 | 源码文件底部 `#[cfg(test)]` | 单个函数/模块 | 新增功能必须配套 |
| 集成测试 | `engine/tests/*.rs` | 跨模块链路 | 修复 Bug 必须配套回归测试 |
| 基准测试 | `engine/benches/*.rs` | 性能敏感路径 | 优化前必须建立基线 |

### 5.2 测试规范

- **命名**：`test_{模块}_{场景}_{预期结果}`
- **断言**：禁止裸 `assert!` 无消息；错误测试必须验证具体错误类型；浮点比较用 `approx_eq`。
- **夹具**：复杂数据放 `tests/fixtures/`，禁止测试内硬编码长 Lua 脚本。

```rust
#[test]
fn test_asset_manager_reload_clears_old_state() {
    let mut manager = AssetManager::new();
    // ... setup ...
    manager.reload("test.png", &asset_server);
    assert!(
        manager.get_state("test.png").is_none(),
        "reload 后旧状态应被清除"
    );
}
```

### 5.3 覆盖率门禁

- 整体行覆盖率 ≥ 60%（`cargo tarpaulin --fail-under 60`）
- PR Patch 覆盖率 ≥ 70%
- 豁免：Bevy `App` 构建、EGUI 绘制回调、`main.rs` 入口、纯 `Component` 派生

---

## 6. 文档规范

### 6.1 Rust 文档

- 所有 `pub` 项必须有 `///` 文档注释。
- 文档包含：功能描述、参数说明、返回值、错误情况、示例（如适用）。
- `lib.rs` 顶部启用 `#![warn(missing_docs)]`。

```rust
/// 加载并验证资产清单
///
/// # 参数
/// - `path`: 清单文件路径（TOML 格式）
///
/// # 返回
/// 验证通过的 `AssetManifest`，或 `ManifestError`
///
/// # 示例
/// ```
/// let manifest = manager.load_manifest("assets/manifest.toml")?;
/// ```
pub fn load_manifest<P: AsRef<Path>>(&mut self, path: P) -> Result<AssetManifest, ManifestError> {
    // ...
}
```

### 6.2 Lua 文档

- 使用 ldoc 格式：`--- 函数描述`，`-- @param name 描述`。
- 配置文件头部说明用途和字段含义。

```lua
--- 处理历史事件
-- @param event_id 事件唯一标识
-- @param handlers 处理函数表 { on_trigger = fn, on_complete = fn }
-- @return boolean 是否成功注册
function History.on(event_id, handlers)
    -- ...
end
```

---

## 7. 工作流与提交

### 7.1 分支与提交

```
feat/xxx   -> 新功能
fix/xxx    -> Bug 修复
docs/xxx   -> 文档更新
perf/xxx   -> 性能优化
test/xxx   -> 测试补充
refactor/xxx -> 重构
```

- 提交信息格式：`<type>: <subject>`
- 一次提交只做一件事，禁止混合不相关改动。

### 7.2 提交前检查

```bash
xmake format      # 格式化
xmake check       # clippy + test + luacheck
```

- **clippy 零警告** (`-D warnings`)
- **luacheck 零警告**
- **cargo test 全部通过**

### 7.3 PR 规范

- 描述中包含：改动动机、主要变更、测试覆盖情况、性能影响评估。
- 关联相关 Issue：`Closes #123` / `Refs #456`。
- PR 模板见 `.github/PULL_REQUEST_TEMPLATE.md`。

---

## 8. 模块级速查表

开发特定模块时，额外关注：

| 模块 | 额外约束 |
|:---|:---|
| `plugins/` | Plugin 只注册系统和资源，禁止在 `build` 中做 IO |
| `lua_api/` | API 变更需同步更新 `docs/lua-api.md`，向后兼容优先 |
| `asset_manager.rs` | 所有加载操作异步非阻塞，状态机必须完整 |
| `components/` | Component 必须是纯数据，禁止方法（除 `Default`/`new`） |
| `core/` | 业务逻辑必须可脱离 Bevy App 测试 |
| `resources/` | Resource 初始化必须安全（`Default` 或 `init_resource`） |
| `game/` (Lua) | 脚本不 panic，错误通过 `log_error` 报告 |

---

## 9. 禁止事项（红线条款）

以下行为 **禁止** 出现在代码库中：

- [ ] 使用 `as any` / `@ts-ignore` / `@ts-expect-error` 等类型安全绕过手段
- [ ] 空的 `catch` 块：`catch(e) {}`
- [ ] 裸 `unwrap()` 或 `unwrap_unchecked()`（无 `expect` 理由）
- [ ] 循环内无节制分配（`format!` / `to_lowercase()` / `collect` 等）
- [ ] `trimesh_from_mesh` 用于静态碰撞体
- [ ] 每帧调用 `globals().get::<Function>`（Lua 函数未缓存）
- [ ] 无测试的新功能代码
- [ ] 删除失败测试以"通过"CI
- [ ] 无文档的 `pub` 公共 API

---

## 10. 上下文引用

| 文档 | 用途 |
|:---|:---|
| `PLAN.md` | 开发阶段、里程碑、技术决策记录 |
| `docs/performance-guidelines.md` | 性能编码规范详细版 |
| `docs/engine-design.md` | 引擎架构设计 |
| `docs/lua-api.md` | Lua API 接口规范 |
| `CONTRIBUTING.md` | 贡献指南、提交规范 |
| `README.md` | 快速开始、项目结构 |

---

*「代码是写给人看的，顺便给机器执行。」*
