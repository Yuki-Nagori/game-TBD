-- config/player.lua
-- 玩家相关配置

return {
    -- 模型配置
    model = {
        scene = "models/fox-eared_game_endfield/scene.gltf#Scene0",
        scale = 1.0,
        base_height = 1.0,
        yaw_offset = 0.0,
    },

    -- 移动配置
    movement = {
        speed = 5.0,
        rotation_speed = 10.0,
    },

    -- 动画配置
    animation = {
        bob_amplitude = 0.08,
        bob_speed = 10.0,
        recover_speed = 8.0,
    },
}
