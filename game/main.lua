-- main.lua
-- 游戏主入口脚本
-- 由 Rust 引擎加载执行

log_info("Lua 脚本系统启动")

-- 加载分文件配置
local game_config = require("config/game")
local player_config = require("config/player")
local camera_config = require("config/camera")
local colors_config = require("config/colors")
local scenes_config = require("config/scenes")

-- 导出到全局供 Rust 读取
GAME_CONFIG = game_config

-- 玩家配置（扁平化结构，匹配 Rust 期望）
PLAYER_CONFIG = {
    model_scene = player_config.model.scene,
    scale = player_config.model.scale,
    base_height = player_config.model.base_height,
    yaw_offset = player_config.model.yaw_offset,
}

PLAYER_MOVEMENT = player_config.movement

WALK_ANIMATION = player_config.animation

-- 相机配置
CAMERA_CONFIG = camera_config

-- 场景颜色
SCENE_COLORS = colors_config

-- 场景配置
SCENE_CONFIG = scenes_config

-- 游戏逻辑状态
local elapsed = 0.0
local spawned_entity_id = nil
local removed_spawned_entity = false

-- 初始化函数
function init()
    log_info("游戏初始化中...")
    log_info("游戏版本: " .. GAME_CONFIG.version)

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

    -- ECS 创建/销毁演示：3秒后销毁 init 阶段创建的实体
    if (not removed_spawned_entity) and elapsed > 3.0 and spawned_entity_id ~= nil then
        Entity.destroy(spawned_entity_id)
        removed_spawned_entity = true
        log_info("已销毁测试实体: " .. spawned_entity_id)
    end
end

-- 启动
init()
