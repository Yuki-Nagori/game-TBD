--- 玩家配置
-- @module config.player
-- @description 玩家模型、移动和动画参数

return {
    --- 模型配置
    -- @table model
    model = {
        scene = "models/fox-eared_game_endfield/scene.gltf#Scene0",
        scale = 1.0,
        base_height = 1.0,
        yaw_offset = 0.0,
    },

    --- 移动配置
    -- @table movement
    movement = {
        speed = 5.0,
        rotation_speed = 10.0,
    },

    --- 动画配置
    -- @table animation
    animation = {
        bob_amplitude = 0.08,
        bob_speed = 10.0,
        recover_speed = 8.0,
    },
}
