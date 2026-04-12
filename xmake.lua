-- xmake.lua
-- 明朝修仙 RPG - 构建配置
-- 原则：简单、明确、可维护

set_project("ming-rpg")
set_version("0.1.0")

-- 模式设置
add_rules("mode.debug", "mode.release")

-- 编译选项：严格
set_warnings("all", "error")
set_languages("c11", "cxx17")

-- 格式化工具配置
-- 使用 rustfmt 格式化 Rust，stylua 格式化 Lua
-- 运行: xmake format
add_rules("plugin.compile_commands.autoupdate", {outputdir = "build"})

-- 引擎模块 (Rust)
target("engine")
    set_kind("static")
    set_toolchains("rust")
    
    -- Cargo 构建
    on_build(function (target)
        os.cd("engine")
        os.exec("cargo build --release")
        os.cd("..")
    end)
    
    -- 安装库文件
    on_install(function (target)
        os.cp("engine/target/release/libengine.a", target:targetdir())
    end)
    
    -- 清理
    on_clean(function (target)
        os.cd("engine")
        os.exec("cargo clean")
        os.cd("..")
    end)

target_end()

-- 主游戏可执行文件
target("ming-rpg")
    set_kind("binary")
    set_toolchains("rust")
    
    -- 依赖引擎
    add_deps("engine")
    
    -- 直接调用 cargo 构建主程序
    on_build(function (target)
        os.exec("cargo build --release --manifest-path engine/Cargo.toml")
    end)
    
    -- 运行
    on_run(function (target)
        os.exec("cargo run --release --manifest-path engine/Cargo.toml")
    end)

target_end()

-- 格式化任务
task("format")
    set_category("plugin")
    on_run(function ()
        print("Formatting Rust code...")
        os.exec("cargo fmt --manifest-path engine/Cargo.toml")
        
        print("Formatting Lua code...")
        -- 如果安装了 stylua
        if os.exec("which stylua", {try = true}) ~= "" then
            os.exec("stylua game/")
        else
            print("stylua not found, install with: cargo install stylua")
        end
        
        print("Formatting C/C++ code...")
        -- 如果安装了 clang-format
        if os.exec("which clang-format", {try = true}) ~= "" then
            os.exec("find engine -name '*.c' -o -name '*.cpp' -o -name '*.h' | xargs clang-format -i")
        end
    end)
    set_menu {
        usage = "xmake format",
        description = "Format all source code"
    }
task_end()

-- 检查任务
task("check")
    set_category("plugin")
    on_run(function ()
        print("Running Rust clippy...")
        os.exec("cargo clippy --manifest-path engine/Cargo.toml -- -D warnings")
        
        print("Running Rust tests...")
        os.exec("cargo test --manifest-path engine/Cargo.toml")
        
        print("Checking Lua syntax...")
        -- 如果安装了 luacheck
        if os.exec("which luacheck", {try = true}) ~= "" then
            os.exec("luacheck game/")
        else
            print("luacheck not found, install with: luarocks install luacheck")
        end
    end)
    set_menu {
        usage = "xmake check",
        description = "Run all linting and tests"
    }
task_end()

-- 快速运行（开发模式）
task("dev")
    set_category("plugin")
    on_run(function ()
        print("Starting development server...")
        os.exec("cargo run --manifest-path engine/Cargo.toml")
    end)
    set_menu {
        usage = "xmake dev",
        description = "Run in development mode with hot reload"
    }
task_end()
