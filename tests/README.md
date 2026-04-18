# 测试框架

> 「未经测试的代码就是bug的温床」—— Oblivionis

## 测试结构

```
engine/tests/          # Rust集成测试（cargo 要求）
├── core_test.rs       # 核心模块测试
└── lua_api_test.rs    # Lua API 测试

game/tests/            # Lua单元测试
├── test_config.lua
└── test_game_logic.lua
```

## 运行测试

```bash
# 所有测试（Rust + Lua）
xmake check

# 仅 Rust 测试
cd engine && cargo test

# 仅 Lua 测试（需要 busted）
cd game/tests && busted .
```

## 测试规范

### Rust 测试

位于 `engine/tests/` 目录，集成测试风格：

```rust
use ming_rpg::core::{GameTime, Cultivation};

#[test]
fn test_game_time_advance() {
    let mut time = GameTime::default();
    time.advance(48.0);
    assert_eq!(time.day, 3);
}
```

### Lua 测试

位于 `game/tests/` 目录，使用 busted 框架：

```lua
describe("游戏配置", function()
    it("应该正确加载配置", function()
        local config = require("config/game")
        assert.is_table(config)
        assert.equals("0.1.0", config.version)
    end)
end)
```
