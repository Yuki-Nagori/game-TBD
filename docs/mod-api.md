# Mod API 设计文档

> 为 Steam 创意工坊预留的脚本接口规范

## 设计原则

1. **沙箱化**：Mod 只能访问暴露的 API，无法操作文件系统或网络
2. **版本兼容**：API 版本化，游戏更新不破坏旧 Mod
3. **事件驱动**：Mod 通过事件钩子介入游戏逻辑
4. **数据优先**：Mod 主要提供数据（配置、定义），而非逻辑

## API 架构

```
┌─────────────────────────────────────────┐
│           Mod (Lua 脚本)                 │
│  - 事件处理                              │
│  - 数据定义                              │
│  - 简单逻辑                              │
└─────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│         Mod API (Rust 暴露)              │
│  - Entity 操作                           │
│  - Component 读写                        │
│  - Event 订阅/发布                       │
│  - 资源查询                              │
└─────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────┐
│           Game Core (Rust)               │
│  - ECS 系统                              │
│  - 核心逻辑                              │
│  - 存档管理                              │
└─────────────────────────────────────────┘
```

## API 分类

### 1. 实体操作 API

```lua
-- 创建实体
local npc = Entity.create("npc")

-- 添加组件
npc:add_component("Transform", {
    position = {x = 100, y = 0, z = 200},
    rotation = {x = 0, y = 90, z = 0}
})

npc:add_component("Character", {
    name = "张三",
    age = 30,
    faction = "donglin"
})

npc:add_component("Cultivation", {
    realm = "foundation",  -- 筑基
    qi = 500
})

-- 销毁实体
Entity.destroy(npc.id)
```

### 2. 事件系统 API

```lua
-- 订阅事件
Event.on("day_start", function(data)
    print("新的一天开始了，日期：" .. data.date)
end)

Event.on("character_die", function(data)
    if data.character.faction == "player" then
        -- 玩家角色死亡，触发特殊剧情
        Event.trigger("custom_story", {type = "player_death"})
    end
end)

-- 触发自定义事件
Event.trigger("my_mod_event", {foo = "bar"})
```

### 3. 历史事件干预 API

```lua
-- 注册历史事件钩子
History.on("zhang_juzheng_reform", {
    -- 事件触发前
    before = function(ctx)
        -- 检查玩家是否介入
        if Player.has_item("secret_letter") then
            -- 改变事件走向
            ctx:set_branch("exposed_corruption")
            return true  -- 阻止默认流程
        end
        return false  -- 继续默认流程
    end,
    
    -- 事件触发后
    after = function(ctx)
        -- 根据结果调整世界状态
        if ctx.result == "success" then
            World.set_variable("court_stability", 80)
        else
            World.set_variable("court_stability", 30)
        end
    end
})
```

### 4. 世界状态 API

```lua
-- 读取世界变量
local stability = World.get_variable("court_stability") or 50

-- 设置世界变量（影响蝴蝶效应）
World.set_variable("player_reputation", 100)

-- 查询势力分布
local factions = World.get_factions()
for _, faction in ipairs(factions) do
    print(faction.name .. ": " .. faction.power)
end
```

### 5. 玩家操作 API

```lua
-- 给予物品
Player.add_item({
    id = "ancient_scroll",
    name = "上古残卷",
    type = "cultivation_method"
})

-- 学习功法
Player.learn_cultivation("tai_xu_jing")

-- 添加关系
Player.add_relation("npc_001", {
    type = "friend",
    value = 50  -- 好感度
})
```

## Mod 结构规范

```
my-mod/
├── mod.json          # Mod 元数据
├── init.lua          # 入口脚本
├── entities/         # 实体定义
│   ├── npcs.lua
│   └── items.lua
├── events/           # 事件脚本
│   ├── hooks.lua
│   └── stories.lua
└── assets/           # 资源文件（可选）
    ├── textures/
    └── models/
```

### mod.json 示例

```json
{
    "id": "my-unique-mod",
    "name": "我的Mod",
    "version": "1.0.0",
    "author": "Your Name",
    "description": "这是一个示例Mod",
    "api_version": "0.1",
    "dependencies": [],
    "conflicts": [],
    "load_order": 100
}
```

## 安全限制

Mod **不能**：
- 访问文件系统（除了自己的目录）
- 进行网络请求
- 执行系统命令
- 访问其他 Mod 的私有数据
- 修改核心游戏逻辑（只能通过 API）

Mod **可以**：
- 创建/修改实体和组件
- 订阅和触发自定义事件
- 读取世界状态
- 添加新的内容（NPC、物品、事件）

## 版本兼容性

```lua
-- 在 init.lua 中检查 API 版本
if API.version < "0.1" then
    error("This mod requires API version 0.1 or higher")
end
```

游戏更新时：
- 小版本更新（0.1.0 -> 0.1.1）：保持兼容
- 大版本更新（0.1 -> 0.2）：可能破坏旧 Mod，提供迁移指南

## 调试支持

```lua
-- 日志输出
Log.debug("调试信息")
Log.info("普通信息")
Log.warn("警告")
Log.error("错误")

-- 在 Mod 目录创建 debug 文件
Debug.dump_state("debug/state.json")
```

## 示例 Mod

见 `examples/mod-template/` 目录
