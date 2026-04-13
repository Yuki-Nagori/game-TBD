# 明朝修仙 RPG

一款明朝历史背景的修仙 RPG，核心玩法是**蝴蝶效应人生模拟**。

## 核心概念

- **时间流逝**：游戏内 1 小时 = 现实 1 年
- **历史干预**：玩家作为变量介入历史事件，产生蝴蝶效应
- **人生模拟**：从出生到死亡（或飞升）的完整一生
- **多周目**：不同出身、不同选择、不同历史线

## 技术栈

- **引擎**：Rust + Bevy (ECS)
- **脚本**：Lua 5.4 (游戏逻辑)
- **构建**：xmake + cargo
- **美术**：极简 Low Poly（方块占位）

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
xmake clean        # 清理构建产物

# 4. 打包发布
xmake config -m release     # 切换到 release 模式
xmake build                # 构建
xmake pack             # 打包（TODO：实现自动打包）
```

**模式说明：**
| 模式 | 优化 | 调试信息 | 用途 |
|------|------|---------|------|
| `debug` | 无 | 有 | 快速迭代开发 |
| `releasedbg` | 有 | 有 | **默认**，开发测试 |
| `release` | 最大 | 无 | 最终发布 |

## 项目结构

```
~/game/
├── engine/              # Rust 核心引擎
│   ├── src/
│   │   ├── main.rs      # 入口
│   │   ├── lua_api/     # Lua 绑定
│   │   └── core/        # 核心系统
│   └── Cargo.toml
├── game/                # Lua 游戏逻辑
│   ├── main.lua         # 入口
│   ├── entities/        # 实体定义
│   ├── systems/         # 游戏系统
│   ├── scenes/          # 场景配置
│   └── story/           # 剧情脚本
├── assets/              # 游戏资源
├── docs/                # 文档
├── xmake.lua            # 构建配置
└── PLAN.md              # 开发计划
```

## 开发计划

见 [PLAN.md](./PLAN.md)

## 许可证

双许可：MIT OR Apache-2.0

你可以选择任一许可证条款使用本项目。

> 注：未来可能转为闭源商业授权，但当前版本保持开源。
