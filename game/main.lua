--- 游戏主入口脚本
-- @script main
-- @description 初始化游戏配置，加载场景和实体定义
-- luacheck: ignore 131

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

--- 初始化函数
-- @function init
function init()
    log_info("游戏初始化中...")
    log_info("游戏版本: " .. GAME_CONFIG.version)
    log_info("游戏初始化完成")
end

--- 主循环（每帧调用）
-- @function update
-- @param dt 帧间隔时间（秒）
function update(dt)
    elapsed = elapsed + dt
    -- 游戏逻辑更新（留空供后续开发）
end

-- 启动
init()
