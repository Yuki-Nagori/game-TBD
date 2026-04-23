-- Luacheck 配置文件
-- 明朝修仙 RPG - Lua 代码检查配置

-- 全局变量（预期会定义）
globals = {
    -- Busted 测试框架
    "describe", "it", "before_each", "after_each",
    "setup", "teardown", "pending",
    -- 游戏回调函数（由 Rust 调用）
    "init", "update",
    -- 游戏全局配置（由 main.lua 定义）
    "GAME_CONFIG", "PLAYER_CONFIG", "PLAYER_MOVEMENT",
    "WALK_ANIMATION", "CAMERA_CONFIG", "SCENE_COLORS", "SCENE_CONFIG",
    "COLOR_CONFIG",
    -- 由 Rust 注入的 Lua API
    "Entity", "Event", "World", "Player", "History",
    "Core",
}

-- 只读全局变量
read_globals = {
    -- Lua 标准库
    "pairs", "ipairs", "next", "tonumber", "tostring",
    "type", "print", "error", "pcall", "xpcall",
    "require", "dofile", "loadfile", "load",
    "table", "string", "math", "os", "io",
    "coroutine", "package", "debug", "utf8",
    -- Busted 断言
    "assert",
    -- 游戏全局变量（由 Rust 注入）
    "log_info", "log_debug", "log_warn", "log_error",
}

-- 排除文件
exclude_files = {
    "game/libs/**",
}

-- 允许设置未定义的全局变量（游戏脚本常见）
allow_defined = true
allow_defined_top = true

-- 启用未使用变量检查
unused = true
unused_args = false
unused_globals = false  -- 允许导出全局变量供 Rust 读取

-- 最大行长度
max_line_length = 120
