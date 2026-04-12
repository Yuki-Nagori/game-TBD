-- main.lua
-- 游戏主入口脚本
-- 由 Rust 引擎加载执行

log_info("Lua 脚本系统启动")

-- 游戏配置
GAME_CONFIG = {
    version = "0.1.0",
    start_year = 1573,  -- 万历元年
    time_scale = 1.0,   -- 时间流速倍率
}

-- 初始化函数
function init()
    log_info("游戏初始化中...")
    
    -- TODO: 加载配置、初始化系统、创建起始场景
    
    log_info("游戏初始化完成")
end

-- 主循环（每帧调用）
function update(dt)
    -- TODO: 更新游戏逻辑
end

-- 启动
init()
