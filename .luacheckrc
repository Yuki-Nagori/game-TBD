-- Lua 静态检查配置
-- 运行: luacheck .
-- 安装: luarocks install luacheck

std = "lua54"

-- 忽略的文件
exclude_files = {
    "engine/",
    "build/",
    ".git/",
}

-- 全局变量（由 Rust 注入或由 main.lua 导出供 Rust 读取）
globals = {
    "Engine",
    "Entity",
    "Component",
    "Event",
    "Log",
    "log_info",
    "GAME_CONFIG",
    "PLAYER_CONFIG",
    "PLAYER_MOVEMENT",
    "WALK_ANIMATION",
    "CAMERA_CONFIG",
    "SCENE_COLORS",
    "SCENE_CONFIG",
    "init",
    "update",
}

-- 允许未使用的参数（回调函数常见）
unused_args = false

-- 最大行长度
max_line_length = 100

-- 忽略的规则
ignore = {
    "212", -- 未使用的参数
    "213", -- 未使用的循环变量
}
