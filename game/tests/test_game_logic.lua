-- 游戏逻辑测试

-- Mock Rust 注入的日志函数
_G.log_info = function(...)
    print("[INFO]", ...)
end
_G.log_debug = function(...)
    print("[DEBUG]", ...)
end
_G.log_warn = function(...)
    print("[WARN]", ...)
end
_G.log_error = function(...)
    print("[ERROR]", ...)
end

describe("游戏逻辑", function()
    setup(function()
        -- 测试前加载主脚本
        require("main")
    end)

    describe("全局变量", function()
        it("应该定义了GAME_CONFIG", function()
            assert.is_table(GAME_CONFIG)
        end)

        it("应该定义了PLAYER_CONFIG", function()
            assert.is_table(PLAYER_CONFIG)
        end)

        it("应该定义了CAMERA_CONFIG", function()
            assert.is_table(CAMERA_CONFIG)
        end)
    end)

    describe("init函数", function()
        it("应该存在init函数", function()
            assert.is_function(init)
        end)
    end)

    describe("update函数", function()
        it("应该存在update函数", function()
            assert.is_function(update)
        end)

        it("update应该能正常执行", function()
            -- 模拟几帧更新
            for i = 1, 10 do
                update(0.016) -- 约60fps
            end
            -- 如果没有报错就通过
            assert.is_true(true)
        end)
    end)
end)
