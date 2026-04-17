-- main.lua
-- 游戏主入口脚本
-- 由 Rust 引擎加载执行

log_info("Lua 脚本系统启动")

-- 游戏配置
GAME_CONFIG = {
    version = "0.1.0",
    start_year = 1573, -- 万历元年
    time_scale = 1.0, -- 时间流速倍率
}

local elapsed = 0.0
local spawned_entity_id = nil
local removed_spawned_entity = false

-- 初始化函数
function init()
    log_info("游戏初始化中...")

    -- 基础 ECS API 演示：创建实体 + 添加组件
    local spawned = Entity.create("npc")
    spawned_entity_id = spawned.id
    Entity.add_component(spawned_entity_id, "faction", { name = "锦衣卫", rank = 1 })
    Entity.set_position(spawned_entity_id, -220.0, -120.0, 0.0)

    log_info("游戏初始化完成")
end

-- 主循环（每帧调用）
function update(dt)
    elapsed = elapsed + dt

    -- Lua 控制主角方块左右摆动（Week 4 MVP） - 暂时禁用，由玩家输入控制
    -- local x = math.sin(elapsed * 1.8) * 320.0
    -- Entity.set_position("player", x, 0.0, 0.0)

    -- ECS 创建/销毁演示：3秒后销毁 init 阶段创建的实体
    if (not removed_spawned_entity) and elapsed > 3.0 and spawned_entity_id ~= nil then
        Entity.destroy(spawned_entity_id)
        removed_spawned_entity = true
        log_info("已销毁测试实体: " .. spawned_entity_id)
    end
end

-- 启动
init()
