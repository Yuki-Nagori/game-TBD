# Lua API 设计

> Rust 引擎暴露给 Lua 脚本的接口规范

## 设计原则

1. **简单直观** — Lua 侧代码像写配置一样简单
2. **类型安全** — Rust 侧做好参数校验，Lua 错误不 panic
3. **热重载友好** — 开发时可以动态更新
4. **跨剧本通用** — 不绑定特定游戏类型

---

## API 模块概览

| 模块 | 状态 | 说明 |
|:---|:---|:---|
| `Core` | ✅ | 版本、日志 |
| `Time` | 🔨 P0 | 时间控制（暂停/恢复/获取） |
| `HotReload` | 🔨 P0 | 开发时热重载 |
| `Butterfly` | 🔨 P0 | 蝴蝶效应变量系统 |
| `Entity` | 📝 P1 | ECS 实体操作 |
| `Scene` | 📝 P1 | 场景管理 |
| `Event` | 📝 P1 | 事件注册与触发 |
| `UI` | 📝 P2 | 对话框、通知 |
| `Save` | 📝 P3 | 存档读写 |

---

## P0 - 核心机制（优先实现）

### Time 模块

```lua
-- 控制游戏时间流逝
Time.pause()                    -- 暂停（进入战斗/对话时调用）
Time.resume()                   -- 恢复（退出战斗/对话时调用）
Time.is_paused()                -- 返回 boolean

-- 获取时间
Time.get_date()                 -- 返回 "1573年01月01日"
Time.get_year()                 -- 返回 1573
Time.get_month()                -- 返回 1
Time.get_day()                  -- 返回 1
Time.get_hour()                 -- 返回 6.0

-- 推进时间
Time.advance(hours)             -- 推进指定小时数
                                -- 用于：闭关修炼、剧情跳过

-- 设置流速（可选）
Time.set_speed(multiplier)      -- 1.0 = 正常, 2.0 = 2倍速
```

**Rust 实现要点：**
- 与 `core::GameTime` 资源交互
- pause/resume 设置布尔标志，停止 `advance` 调用
- 战斗系统开始时自动调用 `Time.pause()`

### HotReload 模块

```lua
-- 开发调试专用
HotReload.enable()              -- 开启文件监听
HotReload.disable()             -- 关闭

-- 注册重载回调
HotReload.on_reload(file_path, callback)
-- 示例：
HotReload.on_reload("scripts/主线/张居正改革.lua", function()
  print("张居正改革脚本已更新！")
  Events.reload("张居正改革")  -- 重新加载事件
end)

-- 批量监听目录
HotReload.watch_directory("scripts/主线/")
```

**Rust 实现要点：**
- 使用 `notify` crate 监听文件变化
- 开发模式才启用，Release 模式禁用
- 回调在下一帧执行，避免文件写入中读取

### Butterfly 模块

```lua
-- 蝴蝶效应变量系统（核心玩法）
Butterfly.set(key, value)       -- value 可以是 number/string/boolean/table
Butterfly.get(key, default)     -- 获取变量，不存在返回 default
Butterfly.add(key, delta)       -- 数值增减，自动处理 nil 情况
Butterfly.has(key)              -- 检查变量是否存在

-- 监听变化
Butterfly.on_change(key, function(old_val, new_val)
  print(key .. " 从 " .. tostring(old_val) .. " 变为 " .. tostring(new_val))
end)

-- 批量操作
Butterfly.get_all()             -- 返回所有变量表（用于存档）
Butterfly.load_all(data)        -- 从表恢复（用于读档）

-- 历史偏离度（可选）
Butterfly.calculate_deviation(event_id)
-- 返回 0.0~1.0，表示该事件与历史原轨的偏离程度
```

**Rust 实现要点：**
- 使用 `Resource` 存储 `HashMap<String, LuaValue>`
- 支持序列化/反序列化（存档需要）
- `on_change` 用事件系统实现

---

## P1 - 实体与场景

### Entity 模块

```lua
-- 创建与销毁
local entity = Entity.create(type_name)   -- 返回实体引用
entity:destroy()

-- 变换操作
entity:set_position(x, y, z)
entity:get_position()           -- 返回 x, y, z
entity:set_rotation(x, y, z)
entity:set_scale(x, y, z)

-- 外观
entity:set_model(path)          -- "models/角色.gltf"
entity:set_material(material_id)

-- 组件（ECS 风格）
entity:add_component(name, data)
entity:get_component(name)
entity:remove_component(name)

-- 查询
local npcs = Entity.query({
  type = "npc",
  region = "北京",
  has_component = "dialogue"
})
```

### Scene 模块

```lua
-- 场景切换
Scene.load(name)                -- "北京/紫禁城"
Scene.get_current()             -- 返回当前场景名
Scene.unload()                  -- 卸载当前场景

-- 场景内操作
Scene.spawn(type, position)     -- 在场景创建实体
Scene.get_entities()            -- 获取场景内所有实体
Scene.get_entities_in_radius(x, y, z, radius)

-- 场景查询
Scene.exists(name)              -- 场景是否存在
Scene.get_regions()             -- 获取所有区域列表
```

### Event 模块

```lua
-- 注册事件
Event.on(event_name, callback)
-- 示例：
Event.on("时间推进", function(data)
  if data.year == 1573 and data.month == 6 then
    -- 触发某事件
  end
end)

-- 触发事件
Event.trigger(event_name, data)
-- 示例：
Event.trigger("玩家进入区域", {region = "北镇抚司"})

-- 一次性监听
Event.once(event_name, callback)

-- 取消监听
local handle = Event.on(...)
Event.off(handle)
```

---

## P2 - UI 系统

```lua
-- 对话框
UI.show_dialogue(text, options)
-- options = {
--   speaker = "张居正",
--   choices = {"选项A", "选项B"},
--   callback = function(choice_index) ... end
-- }

-- 通知
UI.show_notification(text, duration)
UI.show_toast(title, text)      -- 右上角弹窗

-- 选择
UI.show_choice(title, choices, callback)
-- choices = {"去京城", "回乡下", "再想想"}

-- 战斗 UI
UI.show_combat_bar()            -- 显示战斗界面
UI.hide_combat_bar()
UI.update_hp_bar(current, max)
UI.update_qi_bar(current, max)

-- 菜单
UI.open_menu(menu_id)           -- "背包", "功法", "地图"
UI.close_menu()
```

---

## P3 - 存档系统

```lua
-- 存档
Save.save(slot, name)           -- slot = 1~10, name = "存档名称"
Save.load(slot)
Save.delete(slot)

-- 查询
Save.get_list()                 -- 返回存档列表
-- {
--   {slot = 1, name = "存档1", date = "2026-04-15", play_time = 3600},
--   ...
-- }

Save.exists(slot)
```

---

## 错误处理

Lua 侧错误不 panic，返回 `nil, error_message`：

```lua
local ok, err = pcall(function()
  Entity.create("不存在的类型")
end)

if not ok then
  log_error("创建实体失败: " .. err)
end
```

---

## 实现进度追踪

| API | 设计 | Rust 实现 | Lua 测试 | 状态 |
|:---|:---:|:---:|:---:|:---|
| Core | ✅ | ✅ | ✅ | 完成 |
| Time | ✅ | 🔨 | - | 进行中 |
| HotReload | ✅ | - | - | 待开始 |
| Butterfly | ✅ | - | - | 待开始 |
| Entity | ✅ | - | - | 待开始 |
| Scene | 📝 | - | - | 设计中 |
| Event | 📝 | - | - | 设计中 |
| UI | 📝 | - | - | 设计中 |
| Save | 📝 | - | - | 设计中 |

---

*「接口是契约，一旦定下就要长期维护」*
