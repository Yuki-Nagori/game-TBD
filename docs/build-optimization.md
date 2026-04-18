# 编译优化指南

> 「编译慢不是Rust的错，是配置的错。」—— Oblivionis

## 快速开始（xmake）

本项目使用 **xmake** 作为主构建系统，已集成所有优化配置：

```bash
# 开发构建（自动应用优化）
xmake b

# 运行（开发模式自动启用热重载）
xmake run

# 检查（clippy + test + luacheck）
xmake check
```

## xmake 与 Cargo 的配合

### 构建流程

```
xmake b
    ↓
检测模式 (debug/release/releasedbg)
    ↓
cargo build [--release] [--features hot-reload]
    ↓
生成可执行文件
```

### 模式对应关系

| xmake 模式 | cargo 参数 | 特性 |
|:---|:---|:---|
| `debug` (默认) | 默认 dev profile | 热重载、最快编译 |
| `releasedbg` | `--profile releasedbg` | 调试信息、适度优化 |
| `release` | `--release` | 无热重载、全优化 |

### 直接使用 cargo

如需绕过 xmake 直接使用 cargo：

```bash
cd engine

# 开发编译（与 xmake b 等效）
cargo build --profile releasedbg --features hot-reload

# 发布编译（与 xmake f -m release && xmake b 等效）
cargo build --release

# 检查（与 xmake check 等效）
cargo clippy --profile releasedbg --no-deps -- -D warnings
cargo test --profile releasedbg
```

## 已应用的优化

### 1. Cargo 配置 (`engine/.cargo/config.toml`)

```toml
[build]
incremental = true              # 增量编译
rustc-wrapper = "sccache"       # 编译缓存
```

### 2. Profile 配置 (`engine/Cargo.toml`)

**开发模式 (`releasedbg`)**:
- `opt-level = 2`: 适度优化
- `codegen-units = 256`: 最大并行度
- `lto = false`: 禁用链接时优化（提速）
- `debug = true`: 完整调试信息

**依赖项 (`dev.package.*`)**:
- `opt-level = 1`: 平衡编译速度和运行时性能
- `codegen-units = 256`: 更多并行

## 推荐工具

### sccache (编译缓存)

跨项目共享编译结果：

```bash
cargo install sccache

# 启用：编辑 engine/.cargo/config.toml
[build]
rustc-wrapper = "sccache"
```

### cargo-watch (自动重编译)

```bash
cargo install cargo-watch

# 使用（仅监控 Rust 代码）
cd engine
cargo watch -x "build --profile releasedbg --features hot-reload"
```

## 编译时间对比

| 配置 | 预估时间 | 适用场景 |
|:---|:---|:---|
| 默认配置 | 2-5分钟 | 初次编译 |
| 优化后 (本配置) | 30-60秒 | 日常开发 |
| `--release` | 3-10分钟 | 发布构建 |

## 常用命令速查

```bash
# ===== xmake 方式（推荐）=====
xmake b                    # 构建（开发模式）
xmake run                  # 运行
xmake check                # 检查 + 测试
xmake format               # 格式化
xmake f -m release         # 切换到发布模式
xmake f -m releasedbg      # 切换到开发模式（默认）

# ===== cargo 方式（备用）=====
cd engine
cargo build --profile releasedbg --features hot-reload
cargo run --profile releasedbg --features hot-reload
cargo test --profile releasedbg
cargo clippy --profile releasedbg --no-deps -- -D warnings
cargo fmt
cargo check                # 只检查语法（最快）
```

## Bevy 特定优化

Bevy 0.13 编译时间确实较长，以下技巧有帮助：

1. **动态链接** (仅限开发):
   ```bash
   cd engine
   cargo run --profile releasedbg --features "hot-reload bevy/dynamic_linking"
   ```
   注意：需要在 `Cargo.toml` 中确认 feature 可用

2. **减少 Bevy 功能**:
   当前已精简: `features = ["png", "jpeg", "webp"]`
   如果不需要某些功能，可以进一步精简

3. **使用 `cargo check` 代替 `cargo build`**:
   写代码时频繁检查，只在需要运行时才 build

   ```bash
   cd engine
   cargo check --profile releasedbg --features hot-reload
   ```

## 参考

- [Bevy 编译时间优化](https://bevyengine.org/learn/book/getting-started/setup/#enable-fast-compiles-optional)
- [Cargo Profile 文档](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [xmake 文档](https://xmake.io/#/zh-cn/)
