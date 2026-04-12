# 明朝修仙 RPG

> 「迷子でもいい、前へ進め」—— 即使迷失，也要前进。

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

```bash
# 克隆仓库
git clone <repo-url>
cd ming-rpg

# 构建
xmake

# 运行
xmake run

# 开发模式（热重载）
xmake dev

# 格式化代码
xmake format

# 检查代码
xmake check
```

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
