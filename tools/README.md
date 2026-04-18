# 开发工具集

> 「工欲善其事，必先利其器」—— 孔子（大概也玩独立游戏）

## 快速开始

```bash
# 1. 安装依赖
xmake setup

# 2. 构建（默认 debug 模式，自动启用热重载）
xmake b

# 3. 运行
xmake run
```

## 工具列表

### 1. 构建工具

| 工具 | 命令 | 说明 |
|:---|:---|:---|
| xmake | `xmake b` | 主构建系统（开发模式自动启用热重载） |
| cargo | `cargo build` | Rust编译（直接使用） |
| setup | `xmake setup` | 安装开发依赖 |

### 2. 代码质量

| 工具 | 命令 | 说明 |
|:---|:---|:---|
| format | `xmake format` | 格式化所有代码 |
| check | `xmake check` | 运行所有检查（clippy + test + luacheck） |
| format-check | `xmake format-check` | 检查代码格式（CI用） |

### 3. 编译加速工具（由 xmake setup 安装）

| 工具 | 作用 | 安装方式 |
|:---|:---|:---|
| sccache | 编译缓存 | `cargo install sccache` |
| stylua | Lua 格式化 | `cargo install stylua` |

### 4. 脚本工具

| 脚本 | 用途 | 用法 |
|:---|:---|:---|
| `scripts/quick-test.sh` | 快速测试 | `./scripts/quick-test.sh` |
| `scripts/run-with-log.sh` | 带日志运行 | `./scripts/run-with-log.sh` |
| `scripts/clean-all.sh` | 彻底清理 | `./scripts/clean-all.sh` |

## 开发特性

### 热重载（开发模式默认启用）

开发模式下（非 release），以下功能自动启用：

- **Lua热重载**: 修改 `game/` 目录下的 Lua 文件自动重载
- **调试控制台**: 游戏中按 `~` 键呼出
- **详细日志**: `RUST_LOG=debug` 级别日志

手动控制热重载：
```bash
# 禁用热重载（强制）
MING_RPG_HOT_RELOAD=0 xmake run

# 显式启用（开发模式已默认启用）
MING_RPG_HOT_RELOAD=1 xmake run
```

### 快捷键

| 按键 | 功能 |
|:---|:---|
| `~` | 打开/关闭调试控制台 |
| `F1` | 切换性能监控 |
| `F5` | 手动重载Lua脚本 |
| `F12` | 截图 |

## 构建模式

```bash
# debug 模式（默认）：热重载 + 最快编译
xmake b
xmake f -m debug && xmake b

# 发布模式：优化 + 无热重载
xmake f -m release && xmake b

# Release + Debug Info
xmake f -m releasedbg && xmake b
```

## 编译优化

详见 `docs/build-optimization.md`，已配置的优化：

- **sccache 编译缓存**: 跨项目共享编译结果
- **增量编译**: 默认开启
- **Profile 配置**: 开发模式快速编译，发布模式全优化

## 调试与日志

```bash
# 查看所有日志
xmake run 2>&1 | cat

# 仅查看错误
RUST_LOG=error xmake run

# 查看特定模块日志
RUST_LOG=lua_api=debug xmake run
```
