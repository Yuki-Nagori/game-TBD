-- Lua配置测试
-- 使用busted测试框架

describe("游戏配置系统", function()
    describe("游戏主配置", function()
        it("应该能加载game配置", function()
            local config = require("config/game")
            assert.is_table(config)
            assert.is_string(config.version)
            assert.is_number(config.start_year)
            assert.is_number(config.time_scale)
        end)

        it("版本号应该符合语义化版本规范", function()
            local config = require("config/game")
            local major, minor, patch = config.version:match("(%d+)%.(%d+)%.(%d+)")
            assert.is_not_nil(major, "版本号格式错误: " .. config.version)
        end)
    end)

    describe("玩家配置", function()
        it("应该能加载player配置", function()
            local config = require("config/player")
            assert.is_table(config)
            assert.is_table(config.model)
            assert.is_table(config.movement)
            assert.is_table(config.animation)
        end)

        it("移动速度应该在合理范围内", function()
            local config = require("config/player")
            assert.is_number(config.movement.speed)
            assert.is_true(config.movement.speed > 0)
            assert.is_number(config.movement.rotation_speed)
        end)
    end)

    describe("相机配置", function()
        it("应该能加载camera配置", function()
            local config = require("config/camera")
            assert.is_table(config)
            assert.is_number(config.distance)
            assert.is_number(config.smooth_factor)
        end)

        it("相机参数应该在合理范围内", function()
            local config = require("config/camera")
            assert.is_true(
                config.distance >= 5 and config.distance <= 50,
                "相机距离应在5-50之间，实际: " .. config.distance
            )
            assert.is_true(
                config.smooth_factor >= 0 and config.smooth_factor <= 1,
                "平滑因子应在0-1之间"
            )
        end)
    end)

    describe("场景配置", function()
        it("应该能加载scenes配置", function()
            local config = require("config/scenes")
            assert.is_table(config)
            assert.is_string(config.current)
            assert.is_table(config.scenes)
        end)

        it("当前场景应该在场景列表中", function()
            local config = require("config/scenes")
            assert.is_not_nil(
                config.scenes[config.current],
                "当前场景 '" .. config.current .. "' 不在场景列表中"
            )
        end)
    end)
end)
