# 测试指南

> 本项目的测试策略与规范

## 测试策略

| 类型 | 位置 | 范围 | 要求 |
|:---|:---|:---|:---|
| 单元测试 | 源码文件底部 `#[cfg(test)] mod tests` | 单个函数/模块 | 新增功能必须配套 |
| 集成测试 | `engine/tests/*.rs` | 跨模块公共 API 链路 | 修复 Bug 必须配套回归测试 |
| 基准测试 | `engine/benches/*.rs` | 性能敏感路径 | 优化前必须建立基线 |

### 单元测试 vs 集成测试

**单元测试**放在源码文件底部：
- 可访问私有成员
- 与源码同文件，维护方便
- 聚焦单个函数/类型的边界条件和错误分支

**集成测试**放在 `engine/tests/`：
- 只能通过 crate 公共接口访问
- 测试跨模块完整链路
- 使用 `tests/fixtures/` 中的夹具数据

## 命名规范

格式：`test_{模块}_{场景}_{预期结果}`

示例：
- `test_asset_manager_cache_hit_returns_same_id`
- `test_lua_runtime_invalid_syntax_returns_error`
- `test_game_time_advance_crosses_year_boundary`

## 断言规范

- 禁止裸 `assert!` 无消息，必须提供失败原因
- 错误测试必须验证具体错误类型，禁止仅 `assert!(result.is_err())`
- 浮点比较使用 epsilon，禁止直接 `==`

```rust
// ✅ 好
assert!(
    matches!(result, Err(ValidationError::MissingField(ref s)) if s == "name"),
    "空 name 应返回 MissingField(name)"
);

// ✅ 好
assert!((value - 1.5).abs() < f32::EPSILON, "值应在误差范围内");

// ❌ 坏
assert!(result.is_err());
assert_eq!(float1, float2);
```

## 夹具使用

复杂 Lua 脚本和配置数据放入 `tests/fixtures/`，禁止在测试内硬编码长脚本。

```rust
let script_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("tests/fixtures/test_config.lua");
runtime.load_main_script(&script_path).unwrap();
```

## Mock 策略

本项目遵循"业务逻辑脱离 Bevy App 测试"原则：
- `core/`、`lua_api/`、`asset_manager/` 中的纯函数和结构体可直接单元测试
- Bevy 插件系统中的 `run_if` 条件和状态计算逻辑，优先提取为独立函数后测试
- 涉及 ECS World 的交互通过集成测试在 `engine/tests/` 中验证

## 运行测试

```bash
# 全部检查（clippy + test + luacheck）
xmake check

# 仅 Rust 测试
cd engine && cargo test --features dev-tools

# 仅 Lua 测试
cd game && busted --pattern=test_ tests/

# 覆盖率报告
cd engine && cargo tarpaulin --features dev-tools --out Html
```

## 覆盖率门禁

- 整体行覆盖率 ≥ 60%
- PR Patch 覆盖率 ≥ 70%
- 豁免：Bevy `App` 构建、EGUI 绘制回调、`main.rs` 入口、纯 `Component` 派生

---
*「无测试，不合并」*
