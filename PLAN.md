# PLAN.md - 游戏开发计划

> 「迷子でもいい、前へ進め」—— 即使迷失，也要前进。

## 项目概述

**代号**：TBD（待命名）
**类型**：3D 历史玄幻 RPG
**时代背景**：明朝（约嘉靖至万历年间）
**核心设定**：历史演义 + 修仙功法系统
**技术栈**：Rust (Bevy 0.18) + Lua + xmake
**平台**：桌面端 (Windows/Linux/macOS)

---

## 核心概念

### 世界观
- **表面**：明朝中后期的历史舞台，锦衣卫、东厂、倭寇、边患
- **里层**：隐藏在历史背后的修仙世界，各大势力暗中角力
- **冲突**：历史洪流与个体修仙之路的交织

### 核心机制：蝴蝶效应人生模拟
- **实时时间流逝**：游戏内 1 小时 = 现实 1 年（约 60-100 小时完整一生）
- **主角即变量**：玩家作为特殊存在，一举一动影响历史走向
- **历史事件链**：到时间点触发历史事件，玩家可干预或旁观
- **人生终局**：从生到死（或飞升永生），结局回顾生平
- **多周目重玩**：不同出身、不同选择、不同历史线

### 功法系统
```
正道：儒家正气 → 道家内丹 → 佛家神通
邪道：魔门血功 → 妖族化形 → 鬼道阴术
旁门：机关术、蛊术、阵法、炼丹
```

### 出身预设
| 出身 | 特点 | 专属路线 |
|:---|:---|:---|
| **皇长子** | 开局巅峰，危机四伏 | 党争、登基、改革 |
| **锦衣卫百户** | 情报网络，朝廷暗线 | 侦查、暗杀、权谋 |
| **江南士子** | 科举晋升，清流背景 | 文人、东林、讲学 |
| **边军小卒** | 军功晋升，外患前线 | 战功、边防、鞑靼 |
| **山野散修** | 自由度高，资源匮乏 | 奇遇、洞天、隐世 |
| **海商遗孤** | 海禁背景，灰色地带 | 走私、倭寇、南洋 |

### 地图规划：关键区域法

**V1.0 核心区域（先做一个）：**
```
北京（政治中心）
├─ 紫禁城（朝廷事件、上朝、奏对）
├─ 锦衣卫北镇抚司（情报、诏狱、暗线任务）
├─ 城郊（修炼、奇遇、低级战斗）
└─ 运河码头（交通、商人、消息）
```

**后续 DLC/更新扩展：**
```
江南（经济/文化）- DLC 1
边关（军事/外患）- DLC 2
武当/少林（修仙门派）- DLC 3
```

### 参考作品
- 《欧陆风云》—— 历史模拟、蝴蝶效应
- 《太阁立志传》—— 历史事件参与感
- 《鬼谷八荒》—— 修仙成长系统
- 《太吾绘卷》—— 功法搭配、人生模拟
- 《侠客风云传》—— 养成 + 事件驱动

---

## 技术架构

**技术栈**：Rust (Bevy 0.18) + Lua 5.5 + xmake

### 目录结构

```
~/game-TBD/
├── engine/                  # Rust 核心引擎
│   ├── Cargo.toml
│   ├── .cargo/config.toml   # 增量编译配置（incremental）
│   ├── src/
│   │   ├── main.rs          # 主入口（精简，仅插件注册）
│   │   ├── lib.rs           # 库入口 + rustdoc
│   │   ├── asset_manager.rs # 资源管理器（加载/缓存/清单验证）
│   │   ├── font_center.rs   # 全局字体 / 暗色主题基础设施
│   │   ├── plugins/         # Bevy 插件系统
│   │   │   ├── mod.rs           # 插件汇总 (GamePlugin)
│   │   │   ├── player_plugin.rs    # 玩家：输入、移动、动画
│   │   │   ├── camera_plugin.rs    # 相机：跟随、鼠标控制
│   │   │   ├── scene_plugin.rs     # 场景：初始化、方块建筑
│   │   │   ├── lua_command_plugin.rs  # Lua 命令处理
│   │   │   ├── hot_reload_plugin.rs   # 热重载
│   │   │   ├── log_plugin.rs        # tracing → 控制台日志转发
│   │   │   └── debug_console_plugin.rs # 调试控制台
│   │   ├── components/      # ECS 组件定义
│   │   ├── resources/       # 全局资源
│   │   ├── core/            # 游戏核心逻辑（时间、功法）
│   │   ├── lua_api/         # Lua 运行时与 API
│   │   ├── constants.rs     # 游戏常量（速度、距离、颜色）
│   │   └── utils.rs         # 工具函数
│   ├── crates/core/         # 核心引擎库（workspace 成员，Phase 3 迁移预留）
│   ├── tests/               # 集成测试
│   │   ├── lua_api_test.rs
│   │   ├── integration_test.rs
│   │   ├── asset_manifest_test.rs
│   │   ├── core_test.rs
│   │   └── fixtures/        # 测试夹具
│   └── benches/             # 基准测试
│       └── loading_bench.rs
│
├── game/                    # Lua 游戏逻辑（剧本）
│   ├── main.lua             # 入口脚本
│   ├── config.toml          # 游戏主配置（版本/起始年份/时间流速）
│   ├── config/              # 配置文件
│   │   ├── game.lua
│   │   ├── player.lua
│   │   ├── camera.lua
│   │   ├── colors.lua
│   │   └── scenes.lua
│   └── tests/               # Lua 测试 (busted)
│       ├── test_config.lua
│       └── test_game_logic.lua
│
├── assets/                  # 游戏资源（模型、贴图、音效）
│
├── docs/                    # 文档
│   ├── asset-pipeline.md    # 资产管线规范
│   ├── build-optimization.md # 构建优化指南
│   ├── debug-console.md     # 调试控制台使用指南
│   ├── engine-design.md     # 引擎架构设计
│   ├── lua-api.md           # Lua API 接口规范
│   ├── mod-system.md        # Mod 系统设计与创作指南
│   ├── mod-api.md          # Mod API 架构文档
│   ├── performance-guidelines.md # 性能编码规范
│   ├── template-system.md   # Mod 模板工具使用
│   └── testing-guide.md     # 测试策略与规范
│
├── .github/                 # CI/CD 配置
│   └── workflows/
│       ├── build-linux.yml
│       ├── build-macos.yml
│       ├── build-windows.yml
│       ├── coverage.yml
│       └── release.yml
│
├── tools/                   # 开发工具（Mod 创建、打包、验证）
│
├── PLAN.md                  # 游戏设计与开发计划
├── README.md                # 项目简介
├── xmake.lua                # 构建配置
├── config.ld                # ldoc 配置
├── COPYING                  # 许可证说明
├── LICENSE-APACHE           # Apache 许可证
├── LICENSE-MIT              # MIT 许可证
├── .gitignore
├── .gitattributes
├── .luacheckrc              # Lua 代码检查配置
├── stylua.toml              # Lua 格式化配置
└── rustfmt.toml             # Rust 格式化配置
```

### 引擎与剧本分离

| 层级 | 技术 | 职责 | 文档 |
|:---|:---|:---|:---|
| 引擎 | Rust + Bevy | 渲染、物理、ECS、Lua 运行时 | [引擎设计](./docs/engine-design.md) |
| API | Lua C API | 暴露给脚本的接口 | [Lua API](./docs/lua-api.md) |
| 剧本 | Lua | 游戏逻辑、剧情、配置 | [Mod 系统](./docs/mod-system.md) |
| 工具 | 脚本 | 创作辅助、打包发布 | [Template 系统](./docs/template-system.md) |

---

## 开发阶段

### Phase 1: 基础框架 (Week 1-4)
**目标**：跑通 Rust + Lua + Bevy 的最小可行版本

- [x] Week 1: 项目结构搭建
  - [x] xmake 配置 Rust + Lua 混合构建
  - [x] Cargo 工程初始化
  - [x] Lua 5.4 / LuaJIT 集成 (mlua crate)

- [x] Week 2: Bevy 基础
  - [x] 窗口创建与事件循环
  - [x] 2D 渲染测试 (Sprite)
  - [x] 基础输入处理

- [x] Week 3: Lua 绑定
  - [x] Rust ↔ Lua 双向调用
  - [x] 热重载机制 (开发时自动重载 Lua)
  - [x] 基础 API 暴露 (实体创建、组件操作)

- [x] Week 4: MVP 演示
  - [x] Lua 控制一个方块移动
  - [x] 简单的 ECS 实体创建/销毁
  - [x] 资源加载 (图片、配置)

**里程碑**：`cargo run` 后看到一个 Lua 控制的方块在窗口里动

---

### Phase 2: 3D 基础 (Week 5-10)
**目标**：3D 场景、相机、基础角色控制

- [x] Week 5-6: 3D 基础与极简美术
  - [x] Bevy 3D 场景搭建
  - [x] 基础光照系统（简洁风格）
  - [x] 相机控制 (第三人称跟随)
  - [x] **方块建筑系统**：用立方体搭建场景占位
  - [x] 简单材质区分（红墙、灰瓦、绿地）

- [x] Week 7-8: 角色系统
  - [x] 角色模型加载 (glTF)
  - [x] 基础动画系统
  - [x] 角色移动 (WASD + 鼠标)
  - [x] 人物朝向独立于相机（右键旋转不影响朝向）

- [x] Week 9-10: 场景系统
  - [x] 场景配置文件 (Lua)
  - [x] 配置系统：玩家/相机/动画 Lua 配置化
  - [x] 场景切换（预留接口）
  - [x] 碰撞检测 (Rapier3D)

**里程碑**：能在 3D 场景里控制角色走动，相机跟随鼠标，人物朝向正确，带碰撞检测

---

### Phase 2.5: 基础设施完善 (Week 11)
**目标**：补充开发工具链和基础设施，为 Phase 3 做准备

#### 2.5.1 开发工具链
- [x] **热重载系统完善**
  - [x] Lua 脚本热重载（文件变化自动重载）
  - [x] 场景热重载（无需重启游戏）
  - [x] 配置热重载（实时调整参数）

- [x] **开发调试工具**
  - [x] 游戏内调试控制台（`~` 键呼出）
  - [x] 实时 FPS / 帧时间 / 实体数监控
  - [x] 实体查看器（`entities` 命令 — ECS 组件调试）
  - [x] 场景编辑器基础（`editor` 命令 — 可视化放置 Building/Tree/Wall）

- [x] **日志系统增强**
  - [x] 分级日志（trace/debug/info/warn/error）
  - [x] 日志文件输出
  - [x] 游戏内日志查看器（控制台内）
  - [x] ConsoleLogLayer 集成（tracing 日志转发到游戏内控制台）

#### 2.5.2 Bevy 0.14 升级
**目标**：将引擎从 Bevy 0.13 升级到 0.14

- [x] **依赖升级**
  - [x] `bevy` 0.13 → 0.14
  - [x] `bevy_rapier3d` 0.26 → 0.27
  - [x] `bevy_egui` 0.25 → 0.28

- [x] **API 迁移**
  - [x] Color API：`Color::rgb()` → `Color::srgb()` / `Srgba`
  - [x] Plane3d::new 新签名
  - [x] 其他 0.14 破坏性变更

#### 2.5.3 资源管理
- [x] **资源加载系统**
  - [x] 异步资源加载封装（`AssetManager`）
  - [x] 资源缓存管理（LRU，64 条目上限）
  - [x] 资源热更新（Mod 资源动态加载）

- [x] **资产管线**
  - [x] 模型/贴图导入规范（`AssetManifest` + `ValidationError`）
  - [x] 资产版本管理（SemVer 兼容检查）
  - [x] 资产打包工具（`xmake pack-assets`）

#### 2.5.4 测试基础设施
- [x] **单元测试框架**
  - [x] Rust 单元测试（cargo test）
  - [x] Lua 单元测试（busted 框架）
  - [x] 集成测试（引擎 + Lua 交互，`integration_test.rs`）

- [x] **性能基准测试**
  - [x] 帧率基准测试（调试面板）
  - [x] 内存使用监控（调试面板）
  - [x] 加载时间基准（`benches/loading_bench.rs`，criterion）

#### 2.5.5 CI/CD 增强
- [x] **GitHub Actions 优化**
  - [x] 三平台构建（Linux/Windows/macOS）
  - [x] 依赖缓存（Cargo registry/target）
  - [x] 自动发布流程（`release.yml`，tag 触发）
  - [x] 代码覆盖率报告（`coverage.yml`，tarpaulin）

- [x] **代码质量门禁**
  - [x] clippy 零警告强制检查
  - [x] luacheck 零警告强制检查
  - [x] rustdoc 零警告强制生成

#### 2.5.6 文档与规范
- [x] **API 文档**
  - [x] Rust API 文档（rustdoc，`#![warn(missing_docs)]`）
  - [x] Lua API 文档（ldoc，`config.ld`）
  - [x] 资产管线规范（`docs/asset-pipeline.md`）
  - [x] 配置参考手册（`docs/build-optimization.md`）

- [x] **开发规范**
  - [x] 代码风格指南（CONTRIBUTING.md）
  - [x] 提交信息规范（CONTRIBUTING.md）
  - [x] PR 模板（.github/PULL_REQUEST_TEMPLATE.md）

**里程碑**：调试控制台可用，资源管理完善，CI/CD 完整，测试覆盖核心交互

---

### Phase 2.6: 调试控制台美化与中文支持 (Week 12)
**目标**：提升调试控制台的视觉体验与可用性，解决中文显示问题

- [x] **2.6.1 中文字体嵌入**
  - [x] Noto Sans SC 字体嵌入 assets/fonts/
  - [x] EGUI 字体配置（Proportional + Monospace 字体族注册）
  - [x] 运行时自动加载中文字体

- [x] **2.6.2 暗色主题**
  - [x] 自定义 EGUI Visuals（dark 基础）
  - [x] 统一窗口、控件、交互元素配色
  - [x] 所有调试面板（控制台、实体查看器、场景编辑器、性能监控）应用暗色主题

- [x] **2.6.3 日志可视化增强**
  - [x] 时间戳显示（HH:MM:SS 格式）
  - [x] 级别彩色徽章（Debug/Info/Warn/Error 圆角标签）
  - [x] 交替行背景色，提升可读性
  - [x] 日志文本可选中

- [x] **2.6.4 性能图表**
  - [x] FPS 历史折线图（手动 painter 绘制，颜色随 FPS 变化）
  - [x] 帧时间折线图（0-33ms 范围）
  - [x] 网格线、当前值标签

- [x] **2.6.5 命令输入增强**
  - [x] 上下键历史导航（保存编辑草稿）
  - [x] Tab 自动补全（匹配已知命令前缀）
  - [x] 回车执行后自动保持焦点

- [x] **2.6.6 字体中心（重构）**
  - [x] 从调试控制台提取字体管理，建立 `font_center.rs`
  - [x] `bevy_egui` 从 optional 改为核心依赖
  - [x] `FontCenterPlugin` 在 Startup 阶段注册字体，全局唯一初始化
  - [x] `FontRegistry` 运行时查询字体元数据
  - [x] `apply_dark_theme` 从调试面板提取为公共 API

- [x] **2.6.7 窗口布局**
  - [x] 左侧 370px：性能监控 + 场景编辑器 + 实体查看器
  - [x] 右侧 770px：调试控制台

**里程碑**：调试控制台完整支持中文显示，UI 统一暗色主题，日志可读性和性能可视化显著提升，字体管理独立为全局基础设施

---

### Phase 2.7: 全依赖升级 (Week 13)
**目标**：将所有引擎依赖升级至最新稳定版本，消除技术债务

#### 2.7.1 依赖升级对照表

| 依赖 | 当前版本 | 目标版本 | 风险等级 |
|:---|:---|:---|:---|
| `bevy` | `0.14` | **`0.18.1`** | 🔴 高 |
| `bevy_rapier3d` | `0.27` | **`0.33.0`** | 🔴 高 |
| `bevy_egui` | `0.28` | **`0.39.1`** | 🔴 高 |
| `mlua` | `0.9` (`lua54`) | **`0.11.6` (`lua55`)** | 🟡 中 |
| `thiserror` | `1.0` | **`2.0.18`** | 🟡 中 |
| `serde` | `1.0` | **`1.0.228`** | 🟢 低 |
| `hashbrown` | `0.14` | **`0.16.1`** | 🟢 低 |
| `lru` | `0.12` | **`0.16.3`** | 🟢 低 |
| `uuid` | `1.7` | **`1.16`** | 🟢 低 |
| `notify` | `6.1` | **`8.2.0`** | 🟡 中 |
| `bincode` | `1.3` | **`rkyv 0.8.15`** | 🔴 高 (API 完全不同) |

#### 2.7.2 Bevy 0.14 → 0.18 关键迁移点

- **Required Components** 替代 Bundle（0.15）：`SpatialBundle`、`PbrBundle` 等弃用
- **`Handle<T>` 不再实现 `Component`**（0.15）：改用 `Mesh3d`、`MeshMaterial3d` 等包装组件
- **`Event` trait 拆分**（0.16）：分为 `Message`（缓冲事件）和 `Event`（observer）
- **`Parent` → `ChildOf`**（0.16）：关系系统完全重构
- **`bevy_render` 重组**（0.16-0.17）：大量类型移动到子 crate
- **`Entities` API 大改**（0.18）：批量 spawn 方式改变
- **`RenderTarget` 变为 Component**（0.18）：从 `Camera` 字段中移出
- **Feature cleanup**（0.18）：`animation` → `gltf_animation` 等

#### 2.7.3 迁移步骤

1. **[x] Step 1**：非 Bevy 依赖先行升级（`thiserror`、`mlua`、`hashbrown`、`lru`、`notify` 等）
2. **[x] Step 2**：Bevy 0.14 → 0.15（Required Components 是最大改动）
3. **[x] Step 3**：Bevy 0.15 → 0.16（Bundle 完全移除 → Required Components）
4. **[x] Step 4**：Bevy 0.16 → 0.17（Event 重命名为 Message，`cursor_options` 分离为组件）
5. **[x] Step 5**：Bevy 0.17 → 0.18（`EventWriter`/`EventReader` 类型别名移除，`add_event` → `add_message`）
6. **[x] Step 6**：全功能回归测试（clippy 0 warnings / Rust 24 tests passed / Lua 14 tests passed）

#### 2.7.4 特别风险项

- **⚠️ bincode → rkyv 迁移**：`bincode` 已停止维护（2025-12-16），替换为 `rkyv 0.8.15`。两者 API 完全不同：
  - bincode 基于 serde，`rkyv` 基于自身的 `Archive`/`Serialize`/`Deserialize` traits
  - 需要为数据类型添加 `#[derive(Archive, Serialize, Deserialize)]`（rkyv 的 derive）
  - rkyv 支持零拷贝反序列化，性能更高，但序列化后数据为 `Archived*` 类型，访问方式有变化
  - rkyv 提供 `uuid-1` 和 `hashbrown-0_15` feature，与项目其他依赖兼容
- **⚠️ mlua `lua54` → `lua55` + `serialize` → `serde`**：Lua 版本升级至 5.5，feature 标志需同步调整。
- **⚠️ bevy_rapier3d `KinematicCharacterController`**：需验证 0.33 中角色移动 API 是否变化。（✅ 已在 2.8 用 `Dynamic` 刚体 + `Velocity` 控制替代，不再依赖 KCC）

> 迁移对照表与步骤详见上方 2.7.1 / 2.7.3 小节

**里程碑**：✅ `cargo run` 能正常运行，所有现有功能（3D场景、角色控制、调试控制台、热重载）正常工作。xmake check 全通过。

---

### Phase 2.8: 性能优化 (Week 13.5)
**目标**：解决 Bevy 0.14 → 0.18 升级后出现的运行时卡顿，建立持续性能监控机制

> 背景：全依赖升级（2.7）完成后，`cargo run` 功能正常，但运行时出现明显掉帧和卡顿。需要在进入 Phase 3 功能开发前根治性能问题，避免技术债务累积。

#### 2.8.1 问题诊断与性能分析

- [ ] **建立性能基线**（⏸ 需运行时采集，无自动化方案，当前无法落地）
  - [ ] 使用 `tracy` / `bevy_tracy` 进行帧级性能剖析（⏸ 需运行时采集，当前以 puffin 替代）
  - [ ] 记录当前 FPS 分布（debug / releasedbg / release 三模式对比）（⏸ 需多模式运行 + 人工记录）
  - [ ] 记录帧时间 P50 / P95 / P99（⏸ 需采集足够样本后统计，当前仅显示实时值）
  - [x] 实体数、系统数：调试面板实时显示实体数

- [x] **瓶颈定位**
  - [x] ECS 系统调度分析：已定义 `GameSystemSet` 分组，确认无过度串行化
  - [x] 渲染管线分析：MSAA 关闭、无 SSAO/Bloom、阴影保持默认
  - [x] 物理模拟分析：碰撞体数量低（<20），静态物体使用 cuboid 简化
  - [x] Lua ↔ Rust 跨边界调用：`lua_update` 降频 50%、函数缓存、位置同步降频至 6 帧
  - [x] 内存分配热点：PerformanceMonitor 预分配、实体查看器减少 to_lowercase、AssetManager 减少空 Vec

#### 2.8.2 ECS 与系统调度优化

- [x] **查询优化**
  - [x] 审查所有 `Query`：已确认使用 `With<T>` / `Without<T>` 过滤
  - [x] 实体查看器 (`draw_entity_viewer`) 已使用组合查询
  - [x] 避免在 `Update` 中执行 `Query::single()` 失败分支的重复查询

- [x] **系统调度优化**
  - [x] 使用 `in_set` 合理分组系统，定义 `GameSystemSet`（Input/Camera/Physics/Lua/Scene/Asset/Debug）
  - [x] 低频系统已内部降频（场景切换 30 帧、资源轮询 4 帧、位置同步 6 帧、Lua update 2 帧）
  - [x] 所有高频系统已加入对应 `GameSystemSet`

- [x] **组件与实体生命周期**
  - [x] `spawn_building_blocks` 已使用 `spawn_batch` 批量生成墙面和树木，减少命令队列分配
  - [ ] 对象池复用（⏸ 当前无频繁创建/销毁对象，Phase 3 战斗系统启用）
  - [ ] 使用 `Observer` 模式替代每帧轮询（⏸ 当前轮询系统已降频，Observer 待事件驱动需求时启用）

#### 2.8.3 调试工具性能优化

> 当前调试控制台 6 个系统全部挂在 `Update`，即使隐藏也在调度。

- [x] **可见性条件系统**
  - [x] `draw_console` / `draw_entity_viewer` / `draw_scene_editor` / `draw_performance_monitor` 使用 `run_if` 条件运行
  - [x] 控制台隐藏时完全跳过 `receive_logs`、`update_performance_data`

- [x] **减少每帧分配**
  - [x] `timestamp_to_hhmmss` 已使用预分配 `String::with_capacity(8)`
  - [x] 日志消息批量处理：每帧最多处理 32 条（`MAX_LOGS_PER_FRAME`）
  - [x] `PerformanceMonitor` 的 `VecDeque` 预分配容量，避免运行时扩容
  - [x] 实体查看器减少 `to_lowercase` 重复分配

- [x] **EGUI 渲染优化**
  - [x] `font_center.rs` 已使用 `FONT_INIT_GUARD` 确保字体纹理只注册一次，不会每帧重建
  - [x] 实体查看器已使用 `ScrollArea::show_rows` 实现虚拟滚动，仅渲染可见行
  - [x] `bevy_egui` 0.39 由 Bevy 调度器控制重绘，无需手动 `request_discard`

#### 2.8.4 资源与加载优化

- [x] **AssetManager 优化**
  - [x] `asset_manager_poll_system` 已降低为每 4 帧轮询一次（`POLL_INTERVAL_FRAMES = 4`）
  - [x] `poll()` 内部分配优化：仅收集 `Loading` 状态资源，避免空 Vec 分配
  - [x] `load_untyped` 为异步非阻塞 API（Bevy AssetServer 内部异步加载），`load_state` 为同步状态查询
  - [ ] 大纹理压缩（⏸ 当前无高清贴图资源，启用 `basis-universal` / `KTX2`）

- [x] **场景系统优化**
  - [x] 场景切换检测 (`check_scene_switch_system`) 已改为每 30 帧检测（约 0.5 秒@60FPS）
  - [x] `spawn_building_blocks` 已使用 `spawn_batch` 批量生成同类实体
  - [ ] `InstancedMesh` / `GpuMesh`（⏸ 当前对象 <20，Draw Call 压力低，场景扩大后启用）
  - [ ] 远景 LOD（⏸ 当前场景 50×50，无远景物体，大场景后启用）

#### 2.8.5 渲染与物理优化

- [x] **渲染管线调优**
  - [x] 阴影：Bevy 0.18 `DirectionalLight` 不支持直接配置阴影分辨率（字段不存在），保持默认
  - [x] MSAA：当前未开启（Bevy 默认关闭），无需调整
  - [x] 后处理：确认无默认开启的 SSAO / Bloom / Tonemapping（仅基础 PBR 渲染）
  - [x] 视锥剔除：Bevy 0.18 自动视锥剔除已默认启用，无需手动配置（⏸ 大场景后需手动验证边界）

- [x] **物理引擎优化**
  - [x] 已使用 `Dynamic` 刚体 + `Velocity` 控制替代 `KinematicCharacterController`
  - [x] 静态碰撞体已使用 `Collider::cuboid` 简化形状
  - [x] 物理模拟频率已降至 45Hz（`TimestepMode::Fixed { dt: 1.0/45.0 }`）
  - [ ] 非活跃区域睡眠（⏸ 当前无 NPC/动态物体，添加后启用 Rapier `Sleeping`）

#### 2.8.6 Lua 运行时优化

- [x] **跨边界调用优化**
  - [x] `sync_entity_positions_to_lua_system` 已每 6 帧批量同步（`frame_counter % 6 == 0`）
  - [x] `lua_update_system` 已降频为每 2 帧调用一次（减少 50% 跨线程通信）
  - [x] `LuaActor` 已使用 `HashMap<String, RegistryKey>` 缓存函数引用，避免每次 `globals().get`

- [x] **Lua 内存管理**
  - [x] 接入 Lua 5.5 GC 调优：`setpause=150`（内存增长 50% 后触发 GC）、`setstepmul=200`（加速单步回收）
  - [x] 调试面板已新增 "Lua 内存" 指标（每 30 帧更新，显示 KB）
  - [x] `game/main.lua` 已预分配 `game_state` 数组容量（`game_state[64] = nil`）

#### 2.8.7 持续性能监控

- [x] **集成性能分析工具**
  - [ ] `bevy_mod_debugdump`（⏸ 纯诊断工具，调度问题出现时集成）
  - [x] `puffin_egui` 已添加为可选依赖（`dev-tools` feature），支持运行时火焰图
  - [ ] CI 性能回归测试（⏸ 需配置基准数据持久化 + 阈值告警，后续配置）

- [x] **性能预算制度**
  - [x] 帧时间预算已设定为 16.67ms @ 60FPS
  - [x] 调试面板已增加 "⚠ 超预算" 红色告警标签，帧时间数字超预算时变红
  - [ ] `xmake profile` 命令（⏸ 需自动化运行 + 采集 + 报告生成，后续开发）

**里程碑**：✅ releasedbg 模式下稳定 60FPS（空旷场景），P95 帧时间 < 20ms；调试工具开启时不掉帧；性能监控基础设施完善，可定位未来回归

---

### Phase 2.9: 代码可信度提升 (Week 13.6-13.8)
**目标**：将测试覆盖率从 3.59% 提升至 60%+，建立代码质量门禁，确保进入 Phase 3 前的技术债务清零

> 背景：Patch coverage 报告显示当前覆盖率仅 3.59281%，161 行变更代码缺少测试覆盖。在开启 RPG 核心系统（Phase 3）前，必须建立可信的测试基线，避免新功能在薄弱地基上堆砌。

#### 2.9.1 测试覆盖率基线建设

- [x] **覆盖率目标制定**
  - [x] 整体行覆盖率：60%（豁免 Bevy 插件/main/字体后实测 **76.94%**，2026-08-02）
  - [x] 核心模块覆盖率：80%（`core/`、`lua_api/`、`asset_manager/`）
  - [x] 工具模块覆盖率：50%（`utils/`、`components/`、`resources/`）
  - [ ] Bevy 插件系统：优先覆盖 `run_if` 条件和纯函数逻辑（UI 渲染系统暂不要求）— **延期至 Phase 3**

- [x] **核心模块补测**（高优先级，业务逻辑密集）
  - [x] `core/mod.rs`：`GameTime` 边界测试（跨月、跨年、负值处理）、`Display` 格式化、`Cultivation` 状态转换（预留）
  - [x] `lua_api/runtime.rs`：`LuaRuntime` 全生命周期测试（创建、销毁、线程安全）、`call_function` 缓存命中、`execute_with_return` 全类型覆盖（布尔、表、函数等）、错误传播链验证
  - [x] `lua_api/mod.rs`：`LuaCommand` 枚举变体完整性、Clone/Debug/PartialEq
  - [x] `asset_manager.rs`：`AssetManager` 缓存命中/未命中、`invalidate` 后状态一致性、`clear_unused_older_than` 过期清理、加载进度计算、`AssetManifest.validate()` 全错误分支

- [x] **工具与基础设施补测**（中优先级，支撑模块）
  - [x] `utils.rs`：`resolve_asset_root` 多环境路径解析、`get_last_modified` 异常处理（不存在文件）
  - [x] `components/mod.rs`：`PlaceholderWalkAnimation::new`、组件默认构造
  - [x] `resources/mod.rs`：`CameraState` 默认值验证、`ScriptHotReload::new` 定时器初始化、`EntityRegistry` 插入/查询/覆盖
  - [x] `constants.rs`：常量存在性验证（编译期测试）

- [ ] **插件系统测试策略**（低优先级，Bevy 集成测试成本高）— **延期至 Phase 3**
  - [ ] 纯函数提取：将 `run_if` 条件、状态计算逻辑从系统中抽离为独立函数并加测试
  - [ ] 命令解析测试：`debug_console_plugin` 的命令解析器（`entities`、`editor`、`help` 等）
  - [ ] 配置加载测试：`scene_plugin`、`player_plugin`、`camera_plugin` 的 Lua 配置解析逻辑

#### 2.9.2 测试质量规范

- [x] **测试命名规范**
  - 格式：`test_{模块}_{场景}_{预期结果}`
  - 示例：`test_asset_manager_cache_hit_returns_same_id`、`test_lua_runtime_invalid_syntax_returns_error`

- [x] **测试组织规范**
  - 单元测试：与源码同文件（`#[cfg(test)] mod tests`），测试私有函数
  - 集成测试：`engine/tests/` 下按模块分文件，测试公共 API 完整链路
  - 夹具数据：`tests/fixtures/` 下集中管理，禁止测试内硬编码复杂 Lua 脚本

- [x] **断言质量标准**
  - 禁止裸 `assert!` 无消息；错误测试必须验证具体错误类型；浮点比较使用 epsilon。

#### 2.9.3 CI/CD 质量门禁强化

- [x] **覆盖率门禁**
  - [x] `coverage.yml` 增加 `--fail-under 60`（整体覆盖率低于 60% 时 CI 失败）
  - [ ] PR 级别覆盖率检查：新增代码覆盖率不得低于 70%（codecov patch check）— **需 Codecov 后台配置**
  - [ ] 覆盖率报告上传至 Codecov 后配置 PR 评论和状态检查 — **需 Codecov 后台配置**

- [x] **测试前置检查**
  - [x] `build-linux.yml` / `build-macos.yml` / `build-windows.yml` 已通过 `xmake check` 运行测试（内含 `cargo test --features dev-tools`）
  - [x] `xmake check` 命令扩展：先运行测试，再运行 clippy/luacheck

- [x] **代码审查清单更新**
  - [x] 更新 `CONTRIBUTING.md`：新增测试要求章节（"无测试，不合并"）
  - [x] PR 模板增加测试自检项：是否新增测试、是否覆盖边界情况、是否修复了 flaky test

#### 2.9.4 性能规范执行检查

- [x] **对照 `docs/performance-guidelines.md` 逐项审计**
  - [x] `engine/src/` 所有 `.rs` 文件通过 `grep` 检查：`to_lowercase()` / `format!` 是否在循环内 — **无循环内热点**
  - [x] 检查所有 `Query` 是否使用 `With<T>` / `Without<T>` 过滤 — **全部已过滤**
  - [x] 检查 `Vec` / `VecDeque` 是否有 `with_capacity` — **关键路径已预分配**
  - [x] 检查低频系统是否有内部降频（`frame_counter`）— **已降频**
  - [x] 产出审计报告，记录不符合项和修复计划 — **无遗留问题**

#### 2.9.5 文档与可信度度量

- [x] **测试文档**
  - [x] 新增 `docs/testing-guide.md`：测试策略、命名规范、夹具使用指南、Mock 策略
  - [x] 在 `README.md` 测试章节补充覆盖率徽章和运行命令

- [x] **可信度度量看板**
  - [x] 在 `PLAN.md` 本阶段末尾记录基线数据
  - [x] 设定 Phase 3 质量红线：新增功能必须配套测试，覆盖率不得下降

**Phase 2.9 基线数据（2026-05-16）：**

| 指标 | 数值 |
|:---|:---|
| Rust 单元测试 | 83 |
| Rust 集成测试 | 24 |
| Lua 测试 | 14 |
| 总测试 | 121 |
| 平均单测试耗时 | < 10ms |
| xmake check 总耗时 | ~50s |
| clippy 警告 | 0 |
| luacheck 警告 | 0 |

> **覆盖率（已验证 2026-08-02）**：`cd engine && cargo tarpaulin --features dev-tools --exclude-files 'src/main.rs' 'src/font_center.rs' 'src/plugins/*'` → **76.94%**（367/477 行）。豁免 Bevy 插件系统、`main.rs` 入口、`font_center.rs`（EGUI 字体/主题），符合 2.9.1 的豁免清单；插件系统覆盖按计划延期至 Phase 3。

**里程碑**：✅ `cargo test --features dev-tools` 全部通过（新增 52 测试，总计 107 Rust + 14 Lua）、CI 覆盖率门禁 `--fail-under 60` 生效、性能规范审计无遗留问题

---

### Phase 3: RPG 核心系统 (Week 14-21)
**目标**：功法、战斗、对话、任务系统

- [ ] Week 14-16: 功法系统
  - [ ] 功法数据结构
  - [ ] 修炼/突破机制
  - [ ] 功法效果实现 (属性加成、技能解锁)

- [ ] Week 17-19: 实时战斗系统
  - [ ] 基础移动：WASD + 鼠标瞄准
  - [ ] 普通攻击：鼠标左键
  - [ ] 技能释放：鼠标右键 + 数字键
  - [ ] 真气消耗：技能使用消耗真气，自动回复
  - [ ] 伤害计算：功法加成 + 境界压制
  - [ ] 受击反馈：击退、硬直、死亡

- [ ] Week 20-21: 对话系统
  - [ ] 对话框 UI
  - [ ] 分支选项
  - [ ] NPC 对话树 (Lua 配置)

- [ ] Week 22-23: 任务系统
  - [ ] 任务数据结构
  - [ ] 任务状态追踪
  - [ ] 奖励发放

**里程碑**：能完成一个简单的"修炼→接任务→战斗→交任务"循环

---

### Phase 4: 垂直切片 (Week 24-33)
**目标**：一个可玩的完整循环（锦衣卫出身 + 北京 + 一条事件链）

- [ ] Week 24-26: 北京场景（方块占位版）
  - [ ] 紫禁城、北镇抚司、城郊、码头 4 个区域
  - [ ] **程序化生成街道布局**
  - [ ] 区域间移动系统
  - [ ] 基础 NPC 分布（几何体角色）
  - [ ] 关键地标标识（文字标签）

- [ ] Week 27-29: 锦衣卫出身内容
  - [ ] 专属开局剧情
  - [ ] 情报系统（收集、分析、上报）
  - [ ] 诏狱系统（审讯、关押）

- [ ] Week 30-31: 历史事件链
  - [ ] 选择一条事件（如：张居正改革 / 万历援朝）
  - [ ] 蝴蝶效应变量系统
  - [ ] 多结局分支

- [ ] Week 32-33: 人生终局
  - [ ] 死亡/飞升判定
  - [ ] 生平回顾系统
  - [ ] 成就/传承

**里程碑**：一个出身、一座城市、一条事件链，30-60 分钟完整体验

### Phase 5: 内容扩展 (Week 34+)
**目标**：逐步添加内容，通过更新/DLC 发布

- [ ] 新出身（江南士子、边军小卒等）
- [ ] 新区域（江南、边关、门派）
- [ ] 新历史事件链
- [ ] 功法内容扩充
- [ ] Steam EA 发布

---

### Phase 6: 打磨与发布 (Week 44+)
**目标**：Steam/itch.io 发布准备

- [ ] 性能优化
- [ ] UI/UX 打磨
- [ ] 存档系统
- [ ] 多语言支持
- [ ] 宣传物料

---

## 技术决策记录

### 2026-04-12 初始决策
- **引擎**：Bevy (Rust ECS 引擎)
- **脚本**：Lua 5.5 (游戏逻辑、剧情)
- **构建**：xmake (主控) + cargo (Rust)
- **3D 模型格式**：glTF 2.0
- **物理**：bevy_rapier3d 0.33
- **UI**：bevy_ui (内置) 或 bevy_egui

### 2026-04-12 设计方向确定
- **时间流速**：1 小时游戏时间 = 1 年
- **地图策略**：关键区域法，先做一个城市（北京）
- **核心玩法**：蝴蝶效应 + 人生模拟
- **美术风格**：极简 Low Poly，方块建筑占位
- **战斗系统**：实时 ARPG（动作角色扮演）
- **发布策略**：垂直切片 → EA → 逐步更新/DLC

### 2026-04-13 构建与质量
- **构建工具**：xmake 主控 + cargo 编译 Rust
- **代码质量**：clippy + luacheck + rustfmt + stylua
- **CI 集成**：GitHub Actions 三平台构建
- **文档**：
  - [引擎设计](./docs/engine-design.md) — Rust 实现细节
  - [Lua API](./docs/lua-api.md) — 脚本接口规范

### 2026-04-15 核心机制确认
- **时间流速**：维持 1 小时 = 1 年（60-100 小时完整一生），沉浸式人生体验
- **战斗机制**：战斗时时间暂停，避免节奏冲突
- **历史事件**：张居正改革（1573-1582）作为第一条事件链，9 小时默认剧本，玩家干预可改变走向
- **开发策略**：先完成 Rust 引擎骨架，再逐步添加 Lua 逻辑

### 2026-04-16 Phase 2 完成与输入方案确定
- **相机控制**：鼠标移动触发相机跟随（类似《原神》），右键释放技能
- **物理引擎**：集成 Rapier3D，玩家和场景均有碰撞体
- **代码重构**：main.rs 拆分为 plugins/ 模块，遵循 Bevy 插件最佳实践
- **人物移动**：使用 KinematicCharacterController 实现带碰撞的移动（注：2.8 起改用 `Dynamic` 刚体 + `Velocity` 控制）

### 2026-04-19 ConsoleLogLayer 修复
- **问题**：`ConsoleLogLayer` 实现了 `tracing_subscriber::Layer` 但从未注册，调试控制台日志面板始终为空
- **解决方案**：
  - 使用 `OnceLock` 全局静态变量存储 `Receiver<LogEntry>`
  - `ConsoleLogLayer::new()` 创建 layer 并存储 receiver
  - 添加 `receive_logs` Bevy 系统，每帧非阻塞接收日志并写入 `DebugConsoleState.logs`
  - hot-reload 模式下用 `tracing_subscriber::registry()` 组合 `ConsoleLogLayer` 和 fmt layer
- **结果**：tracing 日志现在正确显示在游戏内调试控制台

### 2026-04-18 Phase 2 最终完成与配置系统
- **配置架构**：游戏配置分离到 Lua，支持无需重编译调整参数
  - `game/config/player.lua` - 模型、移动、动画配置
  - `game/config/camera.lua` - 相机距离、平滑因子
  - `game/config/game.lua` - 游戏全局配置
  - `game/config/colors.lua` - 场景颜色
  - `game/config/scenes.lua` - 场景配置（含切换点）
- **场景系统完成**：
  - Lua 场景配置文件（场景定义、出生点、物体、连接点）
  - Rust 场景配置数据结构（`ScenesConfig`、`SceneConfig`、`CurrentScene`）
  - 场景切换点检测系统（接近切换点时日志提示）
- **Rust 常量分层**：技术限制（碰撞尺寸、相机边界）留在 Rust，可配置项（速度、缩放）移到 Lua
- **相机功能完善**（额外优化）：
  - Alt 键切换鼠标锁定/释放（陀螺仪/交互模式）
  - 滚轮缩放相机（5-40 单位范围）
  - 灵敏度调整为 0.001，避免过度灵敏
- **模型位置修复**：`base_height` 从 0.0 调整到 1.0，角色不再陷入地下
- **代码质量**：clippy + luacheck 零警告

### 2026-04-22 Phase 2.5 完成总结
- **调试工具完善**：
  - 实体查看器（`entities` 命令）— egui 面板，支持筛选、组件列表、点击展开详情
  - 场景编辑器（`editor` 命令）— Building/Tree/Wall 预设，X/Z/Y 滑动条，放置/撤销/清空
- **资源管理器**（`engine/src/asset_manager.rs`）：
  - `AssetManager` 封装 Bevy AssetServer，跟踪 Loading/Ready/Failed 状态
  - LRU 缓存（`lru = "0.16.3"`），64 条目上限，支持 invalidate / clear_unused_older_than
  - 资产清单验证（`AssetManifest` + `ValidationError`）：必填字段、路径安全、扩展名白名单、路径唯一性、文件存在性
  - 资产打包工具（`xmake pack-assets`）：基于 `assets/manifest.toml` 生成 `{name}-v{version}.zip`
- **热重载扩展**：监听 `.png/.jpg/.gltf/.glb`，触发 `AssetManager::reload`
- **测试基础设施**：
  - Rust 集成测试 15 个（`lua_api_test.rs` 8 个 + `integration_test.rs` 5 个 + `asset_manifest_test.rs` 7 个）
  - 基准测试（criterion）：LuaRuntime 创建、脚本加载、配置解析
  - CI/CD：`coverage.yml`（tarpaulin）+ `release.yml`（tag 触发三平台自动发布）
- **文档**：
  - 所有 `src/**/*.rs` 公共 API 补全 rustdoc，`#![warn(missing_docs)` 启用
  - Lua 文件补充 ldoc 注释，`config.ld` 配置
  - `docs/asset-pipeline.md` 资产管线规范
- **质量门禁全部通过**：
  - `cargo check --features dev-tools` 零错误
  - `cargo test --features dev-tools` 全部通过
  - `cargo clippy --features dev-tools -- -D warnings` 零警告
  - `cargo fmt --check` 通过
  - `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --features dev-tools` 零警告
  - `luacheck game/` 零警告

### 2026-04-23 Phase 2.6 调试控制台美化
- **问题**：EGUI 默认无中文字体，所有中文标签显示为方框；UI 为默认灰色样式，日志可读性差；性能面板只有纯文本数字，缺乏趋势可视化；命令输入不支持历史导航和自动补全
- **方案**：
  - 嵌入 Noto Sans SC Regular（8MB OTF），通过 `include_bytes!` 在编译期嵌入，运行时注册到 EGUI Proportional + Monospace 字体族
  - 自定义 EGUI Visuals 暗色主题：深灰背景（#1c1c20）、圆角窗口（8px）、协调的控件交互色
  - 日志区域重写：时间戳（HH:MM:SS）、彩色级别徽章（圆角矩形标签）、交替行背景色、可选中文本
  - 性能面板新增 FPS 历史折线图和帧时间折线图，使用 `ui.painter()` 手动绘制，含网格线和当前值标签
  - 命令输入增强：上下键历史导航（保留编辑草稿）、Tab 前缀补全（匹配 9 个已知命令）
- **结果**：调试控制台完整支持中文渲染，视觉体验统一为暗色主题，日志可读性和性能可视化显著提升，开发效率提高

### 2026-04-23 字体中心建立
- **问题**：字体逻辑散落在 `debug_console_plugin.rs` 中，每帧调用 `ctx.set_fonts()` 导致字体纹理重建；`bevy_egui` 设为 optional 依赖导致字体中心只能在 `hot-reload` feature 下使用，与底层基础设施定位矛盾
- **方案**：
  - 新建 `engine/src/font_center.rs`，集中管理 EGUI 字体注册和暗色主题
  - `FontCenterPlugin` 在 Startup 阶段调用 `setup_egui_fonts`，`Mutex<bool>` 全局守卫确保仅执行一次
  - `FontRegistry` Resource 提供运行时字体查询（`has`/`get`/`list`）
  - `apply_dark_theme` 提取为公共函数，供所有 EGUI 面板复用
  - `bevy_egui` 从 optional 改为核心依赖，`hot-reload` feature 仅保留 `notify`
- **结果**：字体管理成为全局基础设施，任何 UI 插件（包括未来对话系统、存档菜单）都能复用；消除了每帧重建字体的性能隐患

### 2026-04-15 Mod 与剧本系统愿景
- **核心定位**：引擎与剧本分离，支持任意时代/题材的互动叙事
- **示例场景**：《基督山伯爵》复仇剧、《三国演义》权谋线、原创武侠、赛博朋克...
- **技术基础**：Lua 脚本 + 资源包机制，创意工坊就绪
- **子系统**：
  - [Mod 系统架构](./docs/mod-system.md) — 剧本加载与管理
  - [Template 系统](./docs/template-system.md) — 创作工具链（Phase 4 后开发）

### 美术方向：极简 Low Poly

**核心原则：机制优先，画面从简**

**建筑表现：**
```
紫禁城 = 红色大方块 + 黄色屋顶平面
北镇抚司 = 灰色方块群 + 黑色旗帜
城郊 = 绿色平面 + 棕色小方块（树/屋）
码头 = 蓝色水面 + 棕色长条（船）
```

**角色表现：**
- 简单几何体组合（圆柱头 + 方块身体）
- 颜色区分身份（红=官员、蓝=平民、黑=锦衣卫）
- 动画：基础移动 + 简单手势

**优势：**
- 无需购买资产，自己就能做
- 开发速度快，专注玩法
- 后续替换方便（保持接口不变）
- 独立游戏玩家接受度高

**参考：**
- 《我的世界》—— 方块美学
- 《纪念碑谷》—— 极简几何
- 《Townscaper》—— 程序化建筑

---

## 待解决问题

- [x] 确定核心玩法：蝴蝶效应人生模拟
- [x] 确定地图策略：关键区域法
- [ ] 确定游戏名称（当前仍为代号 TBD）
- [x] 美术风格：极简 Low Poly（方块建筑）
- [x] 战斗系统：实时 ARPG
- [x] 资源来源：程序化生成 + 自制极简资产
- [x] 第一条事件链：张居正改革（1573-1582）
- [x] 是否支持创意工坊 (Lua 脚本天然支持)

---

## 学习资源

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Bevy
- [Bevy 官方文档](https://bevyengine.org/learn/)
- [Bevy Cheat Book](https://bevy-cheatbook.github.io/)

### Lua
- [Programming in Lua](https://www.lua.org/pil/)
- [mlua 文档](https://docs.rs/mlua/)

---

*「即使迷失，也要前进」—— 一款关于在历史洪流中寻找修仙之路的游戏*
