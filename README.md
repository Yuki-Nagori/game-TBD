# 明朝修仙 RPG

一款明朝历史背景的修仙 RPG，核心玩法是**蝴蝶效应人生模拟**。

## 核心概念

- **时间流逝**：游戏内 1 小时 = 现实 1 年
- **历史干预**：玩家作为变量介入历史事件，产生蝴蝶效应
- **人生模拟**：从出生到死亡（或飞升）的完整一生
- **多周目**：不同出身、不同选择、不同历史线

## 技术栈

- **引擎**：Rust + Bevy 0.14 (ECS)
- **脚本**：Lua 5.4 (游戏逻辑)
- **构建**：xmake + cargo
- **美术**：极简 Low Poly（方块占位）
- **物理**：Rapier3D

## 快速开始

### 前置依赖

在运行 `xmake setup` 之前，请根据你的平台安装以下系统依赖：

#### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libx11-dev \
    libgl1-mesa-dev \
    libgles2-mesa-dev \
    libvulkan-dev \
    libegl1-mesa-dev \
    pkg-config \
    lua5.4 \
    luarocks
```

#### Fedora / RHEL

```bash
sudo dnf install -y \
    alsa-lib-devel \
    systemd-devel \
    libxkbcommon-devel \
    wayland-devel \
    libX11-devel \
    mesa-libGL-devel \
    mesa-libGLES-devel \
    vulkan-loader-devel \
    mesa-libEGL-devel \
    pkgconfig \
    lua \
    luarocks
```

#### Arch Linux / Manjaro

```bash
sudo pacman -S \
    alsa-lib \
    systemd-libs \
    libxkbcommon \
    wayland \
    libx11 \
    mesa \
    vulkan-icd-loader \
    pkgconf \
    lua \
    luarocks
```

#### Windows (MSVC)

**Visual Studio 2022 + MSVC (推荐)**

1. 安装 [Visual Studio 2022](https://visualstudio.microsoft.com/downloads/)
2. 在 Visual Studio Installer 中勾选 **"使用 C++ 的桌面开发"**
3. 确保包含 **Windows 11 SDK** 和 **MSVC v143** 或更高版本

```powershell
# 在 PowerShell 或 VS Developer Command Prompt 中运行：
# Bevy 在 Windows 上通常不需要额外安装系统库
# 如果需要 Lua，可以从 https://luabinaries.sourceforge.net/ 下载
```

**注意：** MSVC 是 Windows 上 Rust/Bevy 开发的首选工具链，性能最好，与 Visual Studio 调试器集成完善。

#### Windows (MSYS2/MinGW)

如果你更喜欢类 Unix 环境：

```bash
# 安装 MSYS2 后，在 MSYS2 MinGW 64-bit 终端中运行：
pacman -Syu
pacman -S \
    mingw-w64-x86_64-pkg-config \
    mingw-w64-x86_64-alsa-lib \
    mingw-w64-x86_64-lua \
    mingw-w64-x86_64-luarocks
```

#### macOS

```bash
# 安装 Homebrew (如果还没有)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装依赖
brew install pkg-config lua luarocks

# 注意：macOS 不需要额外安装 ALSA/udev，Bevy 使用 CoreAudio 和 IOKit
```

### 项目设置

```bash
cd ~/game-TBD

# 1. 安装所有开发依赖（一次性）
xmake setup

# 2. 配置构建模式（可选，默认 releasedbg）
xmake config -m debug        # 开发模式：编译最快，无优化
xmake config -m releasedbg   # 发布调试模式（默认）：优化 + 调试信息
xmake config -m release      # 发布模式：最大优化，最小体积

# 3. 常用开发命令
xmake build        # 构建（使用当前配置的模式）
xmake run          # 构建并运行
xmake check        # 代码检查（clippy + test + luacheck）
xmake format       # 代码格式化（rustfmt + stylua）
xmake format-check # 检查代码格式（不修改文件）
xmake clean        # 清理构建产物

# 4. 基准测试
cd engine
cargo bench

# 5. 打包发布
xmake config -m release     # 切换到 release 模式
xmake build                # 构建
xmake pack                 # 打包发布产物
xmake pack-assets          # 基于 manifest.toml 打包资产包
```

**模式说明：**
| 模式 | 优化 | 调试信息 | 用途 |
|------|------|---------|------|
| `debug` | 无 | 有 | 快速迭代开发 |
| `releasedbg` | 有 | 有 | **默认**，开发测试 |
| `release` | 最大 | 无 | 最终发布 |

## 项目结构

```
~/game-TBD/
├── engine/                  # Rust 核心引擎
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs          # 主入口（精简，仅插件注册）
│   │   ├── lib.rs           # 库入口 + rustdoc
│   │   ├── asset_manager.rs # 资源管理器（加载/缓存/清单验证）
│   │   ├── plugins/         # Bevy 插件系统
│   │   │   ├── mod.rs           # 插件汇总 (GamePlugin)
│   │   │   ├── player_plugin.rs    # 玩家：输入、移动、动画
│   │   │   ├── camera_plugin.rs    # 相机：跟随、鼠标控制
│   │   │   ├── scene_plugin.rs     # 场景：初始化、方块建筑
│   │   │   ├── lua_command_plugin.rs  # Lua 命令处理
│   │   │   ├── hot_reload_plugin.rs   # 热重载
│   │   │   └── debug_console_plugin.rs # 调试控制台
│   │   ├── components/      # ECS 组件定义
│   │   ├── resources/       # 全局资源
│   │   ├── core/            # 游戏核心逻辑（时间、功法）
│   │   ├── lua_api/         # Lua 运行时与 API
│   │   ├── constants.rs     # 游戏常量（速度、距离、颜色）
│   │   └── utils.rs         # 工具函数
│   ├── tests/               # 集成测试
│   │   ├── lua_api_test.rs
│   │   ├── integration_test.rs
│   │   ├── asset_manifest_test.rs
│   │   └── fixtures/        # 测试夹具
│   └── benches/             # 基准测试
│       └── loading_bench.rs
│
├── game/                    # Lua 游戏逻辑（剧本）
│   ├── main.lua             # 入口脚本
│   ├── config/              # 配置文件
│   │   ├── game.lua
│   │   ├── player.lua
│   │   ├── camera.lua
│   │   ├── colors.lua
│   │   └── scenes.lua
│   └── tests/               # Lua 测试 (busted)
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
│   ├── MOD_API.md           # Mod API 架构文档
│   └── template-system.md   # Mod 模板工具使用
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

## 当前功能

### 游戏系统
- [x] 3D 场景初始化（光照、地面、方块建筑）
- [x] 第三人称角色控制（WASD 移动 + 鼠标控制相机）
- [x] 人物朝向独立于相机
- [x] 滚轮缩放相机（5-40 单位）
- [x] Alt 键切换鼠标锁定/释放
- [x] Rapier3D 物理碰撞
- [x] Lua 配置驱动（玩家/相机/场景/颜色）
- [x] Lua 脚本热重载（F5 或文件变化自动重载）

### 调试工具（`MING_RPG_DEV_MODE=1`）
- [x] 游戏内调试控制台（`~` 键呼出）
- [x] 实时 FPS / 帧时间 / 实体数监控
- [x] 日志筛选（Debug/Info/Warn/Error）
- [x] Lua 代码即时执行
- [x] **实体查看器**（`entities` 命令）— 查看所有 ECS 实体与组件
- [x] **场景编辑器**（`editor` 命令）— 可视化放置 Building/Tree/Wall
- [x] 性能基准测试面板

### 资源管理
- [x] 异步资源加载封装（`AssetManager`）
- [x] LRU 资源缓存（64 条目上限）
- [x] 资源热更新（纹理/模型文件变化自动重载）
- [x] 资产清单验证（`AssetManifest`）— 路径/格式/版本检查
- [x] 资产打包工具（`xmake pack-assets`）

### 测试与质量
- [x] Rust 单元测试 / 集成测试（`cargo test --features dev-tools`）
- [x] Lua 单元测试（busted）
- [x] 加载时间基准测试（criterion）
- [x] clippy 零警告强制检查
- [x] luacheck 零警告强制检查
- [x] rustdoc 完整文档（`RUSTDOCFLAGS="-D warnings"`）
- [x] Lua API 文档（ldoc）
- [x] 三平台 CI 构建 + 自动发布 + 代码覆盖率

## 开发计划

见 [PLAN.md](./PLAN.md)

## 许可证

双许可：MIT OR Apache-2.0

你可以选择任一许可证条款使用本项目。

> 注：未来可能转为闭源商业授权，但当前版本保持开源。
