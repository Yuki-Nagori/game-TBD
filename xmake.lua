-- xmake.lua
-- 明朝修仙 RPG - 构建配置
-- 原则：简单、明确、可维护、Mod 友好

set_project("ming-rpg")
set_version("0.1.0")

-- 模式设置
add_rules("mode.debug", "mode.release")

-- 编译选项：严格
set_warnings("all", "error")
set_languages("c11", "cxx17")

-- 生成 compile_commands.json（给 LSP 用）
add_rules("plugin.compile_commands.autoupdate", {outputdir = "build"})

-- 检查 Rust 工具链
function check_rust()
    local rust_version = try {function () return os.iorunv("rustc", {"--version"}) end}
    if not rust_version then
        print("错误：未检测到 Rust 工具链")
        print("请安装 Rust: https://rustup.rs/")
        print("或运行: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
        os.exit(1)
    end
    print("检测到: " .. rust_version:trim())
    
    -- 检查 cargo
    local cargo_version = try {function () return os.iorunv("cargo", {"--version"}) end}
    if not cargo_version then
        print("错误：未检测到 Cargo")
        os.exit(1)
    end
    
    return true
end

-- 配置阶段：检查依赖
on_load(function (target)
    if not os.getenv("SKIP_RUST_CHECK") then
        check_rust()
    end
end)

-- 引擎库目标（供后续 C API 导出用）
target("engine")
    set_kind("static")
    set_toolchains("rust")
    
    on_build(function (target)
        os.cd("engine")
        local mode = is_mode("release") and "--release" or ""
        os.exec("cargo build " .. mode)
        os.cd("..")
    end)
    
    on_install(function (target)
        local build_dir = is_mode("release") and "release" or "debug"
        os.cp("engine/target/" .. build_dir .. "/libming_rpg.a", target:targetdir())
    end)
    
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
    
    add_deps("engine")
    
    on_build(function (target)
        local mode = is_mode("release") and "--release" or ""
        os.exec("cargo build " .. mode .. " --manifest-path engine/Cargo.toml")
    end)
    
    on_install(function (target)
        local build_dir = is_mode("release") and "release" or "debug"
        local ext = is_plat("windows") and ".exe" or ""
        os.cp("engine/target/" .. build_dir .. "/ming-rpg" .. ext, target:targetdir())
    end)
    
    on_run(function (target)
        local build_dir = is_mode("release") and "release" or "debug"
        local ext = is_plat("windows") and ".exe" or ""
        os.exec("engine/target/" .. build_dir .. "/ming-rpg" .. ext)
    end)

target_end()

-- 格式化任务
task("format")
    set_category("plugin")
    on_run(function ()
        print("Formatting Rust code...")
        os.exec("cargo fmt --manifest-path engine/Cargo.toml")
        
        print("Formatting Lua code...")
        if try {function () return os.iorunv("which", {"stylua"}) end} then
            os.exec("stylua game/")
        else
            print("  stylua not found, install with: cargo install stylua")
        end
        
        print("Formatting C/C++ code...")
        if try {function () return os.iorunv("which", {"clang-format"}) end} then
            os.exec("find engine -name '*.c' -o -name '*.cpp' -o -name '*.h' | xargs clang-format -i 2>/dev/null || true")
        end
        
        print("Done!")
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
        print("=== Checking Rust code ===")
        print("Running clippy...")
        os.exec("cargo clippy --manifest-path engine/Cargo.toml -- -D warnings")
        
        print("Running tests...")
        os.exec("cargo test --manifest-path engine/Cargo.toml")
        
        print("\n=== Checking Lua code ===")
        if try {function () return os.iorunv("which", {"luacheck"}) end} then
            os.exec("luacheck game/")
        else
            print("  luacheck not found, install with: luarocks install luacheck")
        end
        
        print("\nAll checks passed!")
    end)
    set_menu {
        usage = "xmake check",
        description = "Run all linting and tests"
    }
task_end()

-- 快速开发模式
task("dev")
    set_category("plugin")
    on_run(function ()
        print("Starting development mode...")
        print("Features: Hot reload, debug symbols, fast compile")
        os.exec("cargo run --manifest-path engine/Cargo.toml")
    end)
    set_menu {
        usage = "xmake dev",
        description = "Run in development mode"
    }
task_end()

-- Mod 打包任务（为创意工坊准备）
task("pack-mod")
    set_category("plugin")
    on_run(function ()
        local mod_name = os.getenv("MOD_NAME") or "my-mod"
        print("Packing mod: " .. mod_name)
        
        -- 创建 mod 目录结构
        os.mkdir("build/mods/" .. mod_name)
        os.cp("game/*", "build/mods/" .. mod_name .. "/")
        
        -- 打包为 zip
        os.cd("build/mods")
        os.exec("zip -r " .. mod_name .. ".zip " .. mod_name)
        os.cd("../..")
        
        print("Mod packed: build/mods/" .. mod_name .. ".zip")
    end)
    set_menu {
        usage = "xmake pack-mod",
        description = "Pack game scripts as mod for distribution"
    }
task_end()

-- 安装依赖任务
task("setup")
    set_category("plugin")
    on_run(function ()
        print("=== Setting up development environment ===")
        
        -- 检查 Rust
        print("\n1. Checking Rust toolchain...")
        check_rust()
        
        -- 安装 Rust 工具
        print("\n2. Installing Rust tools...")
        print("  Installing stylua (Lua formatter)...")
        os.exec("cargo install stylua 2>/dev/null || echo '  stylua already installed or failed'")
        
        -- 检查 Lua 工具
        print("\n3. Checking Lua tools...")
        if not try {function () return os.iorunv("which", {"lua"}) end} then
            print("  Warning: Lua not found. Install with: apt install lua5.4")
        end
        
        if not try {function () return os.iorunv("which", {"luarocks"}) end} then
            print("  Warning: LuaRocks not found. Install with: apt install luarocks")
        else
            print("  Installing luacheck...")
            os.exec("luarocks install luacheck 2>/dev/null || echo '  luacheck install skipped'")
        end
        
        print("\n=== Setup complete ===")
        print("Run 'xmake dev' to start development")
    end)
    set_menu {
        usage = "xmake setup",
        description = "Install development dependencies"
    }
task_end()
