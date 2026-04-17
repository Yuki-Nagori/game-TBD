-- config/scenes.lua
-- 场景配置

return {
    -- 当前场景ID
    current = "suburb",

    -- 场景定义
    scenes = {
        -- 城郊
        suburb = {
            name = "城郊",
            description = "北京城外，一片宁静的土地",
            spawn_point = { x = 0.0, y = 1.0, z = 0.0 },
            ground_size = 50.0,
            objects = {
                -- 示例建筑
                { type = "building", x = 10.0, z = 10.0, color = "wall" },
                { type = "tree", x = -15.0, z = 8.0 },
                { type = "tree", x = -12.0, z = 12.0 },
            },
            -- 连接的其他场景
            connections = {
                { to = "city_gate", x = 0.0, z = 25.0, name = "进城" },
            },
        },

        -- 城门（预留）
        city_gate = {
            name = "城门",
            description = "北京城门，进出要道",
            spawn_point = { x = 0.0, y = 1.0, z = 0.0 },
            ground_size = 30.0,
            objects = {},
            connections = {
                { to = "suburb", x = 0.0, z = -15.0, name = "出城" },
            },
        },
    },
}
