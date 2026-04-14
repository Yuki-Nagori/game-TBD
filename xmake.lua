-- xmake.lua
-- 明朝修仙 RPG - 构建配置
-- 原则：简单、明确、可维护、Mod 友好

set_project("ming-rpg")
set_version("0.1.0")

-- 模式设置（默认 releasedbg：release + debug info）
add_rules("mode.debug", "mode.release", "mode.releasedbg")
set_defaultmode("releasedbg")

-- 自定义 releasedbg 模式配置（release 优化 + 调试信息）
if is_mode("releasedbg") then
    set_optimize("fastest")    -- 最高优化级别
    set_symbols("debug")       -- 包含调试信息
end

-- 添加自定义模式：releasedbg（release with debug info）
-- 使用方式：xmake f -m releasedbg

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

-- 引擎库目标（供后续 C API 导出用）
target("engine")
    set_kind("static")
    set_toolchains("rust")
    
    on_build(function (target)
        os.cd("engine")
        local mode_flag = ""
        if is_mode("release") then
            mode_flag = "--release"
        elseif is_mode("releasedbg") then
            mode_flag = "--profile releasedbg"
        end
        os.exec("cargo build " .. mode_flag)
        os.cd("..")
    end)
    
    on_install(function (target)
        local build_dir = "debug"
        if is_mode("release") then
            build_dir = "release"
        elseif is_mode("releasedbg") then
            build_dir = "releasedbg"
        end
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
        local mode_flag = ""
        if is_mode("release") then
            mode_flag = "--release"
        elseif is_mode("releasedbg") then
            mode_flag = "--profile releasedbg"
        end
        os.exec("cargo build " .. mode_flag .. " --manifest-path engine/Cargo.toml")
    end)
    
    on_install(function (target)
        local build_dir = "debug"
        if is_mode("release") then
            build_dir = "release"
        elseif is_mode("releasedbg") then
            build_dir = "releasedbg"
        end
        local ext = is_plat("windows") and ".exe" or ""
        os.cp("engine/target/" .. build_dir .. "/ming-rpg" .. ext, target:targetdir())
    end)
    
    on_run(function (target)
        local build_dir = "debug"
        if is_mode("release") then
            build_dir = "release"
        elseif is_mode("releasedbg") then
            build_dir = "releasedbg"
        end
        local ext = is_plat("windows") and ".exe" or ""
        
        local target_dir = path.absolute("engine/target/" .. build_dir)
        local deps_dir = path.join(target_dir, "deps")
        
        -- 跨平台动态库路径设置
        if is_plat("windows") then
            -- Windows: 使用 PATH
            local env_path = os.getenv("PATH") or ""
            local new_path = target_dir .. ";" .. deps_dir
            if env_path ~= "" then
                new_path = new_path .. ";" .. env_path
            end
            os.setenv("PATH", new_path)
        elseif is_plat("macosx") then
            -- macOS: 使用 DYLD_LIBRARY_PATH
            local env_dyld_path = os.getenv("DYLD_LIBRARY_PATH") or ""
            local new_dyld_path = target_dir .. ":" .. deps_dir
            if env_dyld_path ~= "" then
                new_dyld_path = new_dyld_path .. ":" .. env_dyld_path
            end
            os.setenv("DYLD_LIBRARY_PATH", new_dyld_path)
        else
            -- Linux: 使用 LD_LIBRARY_PATH
            -- 获取 Rust 工具链的 lib 目录（包含 libstd-*.so）
            local rust_lib_dir = ""
            local rustc_path = try {function () return os.iorunv("rustc", {"--print", "sysroot"}) end}
            if rustc_path then
                -- 获取当前 Rust 工具链的目标三重组
                local rust_target = try {function () return os.iorunv("rustc", {"--print", "host"}) end}
                if rust_target then
                    rust_target = rust_target:trim()
                    rust_lib_dir = path.join(rustc_path:trim(), "lib", "rustlib", rust_target, "lib")
                else
                    -- 回退到硬编码值（兼容旧版本）
                    rust_lib_dir = path.join(rustc_path:trim(), "lib", "rustlib", "x86_64-unknown-linux-gnu", "lib")
                end
            end
            
            local env_ld_path = os.getenv("LD_LIBRARY_PATH") or ""
            local new_ld_path = target_dir .. ":" .. deps_dir
            if rust_lib_dir ~= "" then
                new_ld_path = new_ld_path .. ":" .. rust_lib_dir
            end
            if env_ld_path ~= "" then
                new_ld_path = new_ld_path .. ":" .. env_ld_path
            end
            os.setenv("LD_LIBRARY_PATH", new_ld_path)
        end
        
        -- 执行
        os.exec("\"" .. target_dir .. "/ming-rpg" .. ext .. "\"")
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
        local mode = get_config("mode") or "releasedbg"
        local mode_flag = ""
        if mode == "release" then
            mode_flag = "--release"
        elseif mode == "releasedbg" then
            mode_flag = "--profile releasedbg"
        end

        print("=== Checking Rust code ===")
        print("Running clippy...")
        os.exec("cargo clippy --manifest-path engine/Cargo.toml " .. mode_flag .. " --no-deps -- -D warnings")
        
        print("Running tests...")
        os.exec("cargo test --manifest-path engine/Cargo.toml " .. mode_flag)
        
        print("\n=== Checking Lua code ===")
        if try {function () return os.iorunv("which", {"luacheck"}) end} then
            -- 在所有平台上逐个文件检查，避免 Windows 上的目录权限问题
            local lua_files = {}
            -- 收集所有 Lua 文件
            local function collect_lua_files(dir)
                local files = os.files(path.join(dir, "**.lua"))
                if files then
                    for _, f in ipairs(files) do
                        table.insert(lua_files, f)
                    end
                end
                -- 递归检查子目录
                local dirs = os.dirs(path.join(dir, "*"))
                if dirs then
                    for _, d in ipairs(dirs) do
                        collect_lua_files(d)
                    end
                end
            end

            collect_lua_files("game")

            if #lua_files > 0 then
                for _, file in ipairs(lua_files) do
                    os.exec("luacheck \"" .. file .. "\"")
                end
            else
                print("  No Lua files found in game/")
            end
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

-- 游戏打包任务（发布用）
task("pack")
    set_category("plugin")
    on_run(function ()
        local version = "0.1.0"
        -- 获取当前配置的模式
        local mode = get_config("mode") or "releasedbg"
        local build_dir = "debug"
        if mode == "release" then
            build_dir = "release"
        elseif mode == "releasedbg" then
            build_dir = "releasedbg"
        end
        
        local ext = is_plat("windows") and ".exe" or ""
        local binary_name = "ming-rpg" .. ext
        local binary_path = path.absolute("engine/target/" .. build_dir .. "/" .. binary_name)
        
        -- 检查二进制是否存在
        if not os.isfile(binary_path) then
            print("错误：找不到二进制文件 " .. binary_path)
            print("当前模式: " .. mode)
            print("请先运行：xmake b")
            os.exit(1)
        end
        
        -- 创建发布目录
        local dist_dir = "dist/ming-rpg-v" .. version
        os.mkdir(dist_dir)
        
        -- 复制二进制
        os.cp(binary_path, dist_dir .. "/")
        
        -- 复制动态库（Rust std + bevy_dylib）
        local target_dir = "engine/target/" .. build_dir
        
        -- 1. Rust 标准库动态链接库
        local std_pattern = path.absolute(target_dir .. "/libstd*.so")
        if is_plat("windows") then
            std_pattern = path.absolute(target_dir .. "/std*.dll")
        elseif is_plat("macosx") then
            std_pattern = path.absolute(target_dir .. "/libstd*.dylib")
        end
        
        local std_libs = os.files(std_pattern)
        for _, lib in ipairs(std_libs) do
            os.cp(lib, dist_dir .. "/")
        end
        
        -- 2. Bevy 动态链接库
        local dylib_ext = is_plat("windows") and ".dll" or (is_plat("macosx") and ".dylib" or ".so")
        local dylib_prefix = is_plat("windows") and "" or "lib"
        local dylib_pattern = path.absolute(target_dir .. "/deps/" .. dylib_prefix .. "bevy_dylib*" .. dylib_ext)
        
        local dylibs = os.files(dylib_pattern)
        for _, dylib in ipairs(dylibs) do
            os.cp(dylib, dist_dir .. "/")
        end
        
        -- 复制游戏脚本
        os.mkdir(dist_dir .. "/game")
        os.cp("game/*", dist_dir .. "/game/")
        
        -- 复制文档
        os.cp("README.md", dist_dir .. "/")
        os.cp("COPYING", dist_dir .. "/")
        
        print("=== 打包完成 ===")
        print("输出目录: " .. dist_dir)
        print("")
        print("手动打包命令:")
        print("  cd dist && zip -r ming-rpg-v" .. version .. ".zip ming-rpg-v" .. version)
    end)
    set_menu {
        usage = "xmake pack",
        description = "Pack game for distribution"
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
        
        -- 打包为 zip（跨平台）
        local zip_path = "build/mods/" .. mod_name .. ".zip"
        local dir_path = "build/mods/" .. mod_name
        if os.exists(zip_path) then
            os.rm(zip_path)
        end
        os.zip(zip_path, dir_path)
        
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
        -- 内部函数：检查 Rust 工具链
        local function check_rust()
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
        end
        
        print("=== Setting up development environment ===")
        
        -- 检查 Rust
        print("\n1. Checking Rust toolchain...")
        check_rust()
        
        -- 安装 Rust 工具
        print("\n2. Installing Rust tools...")
        print("  Installing stylua (Lua formatter)...")
        local stylua_installed = try {function () return os.iorunv("which", {"stylua"}) end}
        if stylua_installed then
            print("    stylua already installed")
        else
            local ok = try {function () os.run("cargo install stylua") return true end}
            if ok then
                print("    stylua installed successfully")
            else
                print("    stylua install failed (may already be installed)")
            end
        end
        
        -- 检查 Lua 工具
        print("\n3. Checking Lua tools...")
        if not try {function () return os.iorunv("which", {"lua"}) end} then
            print("  Warning: Lua not found. Install with: apt(brew/scoop) install lua5.4")
        end
        
        if not try {function () return os.iorunv("which", {"luarocks"}) end} then
            print("  Warning: LuaRocks not found. Install with: apt(brew/scoop) install luarocks")
        else
            print("  Installing luacheck...")
            local ok = try {function () os.run("luarocks install luacheck") return true end}
            if ok then
                print("    luacheck installed successfully")
            else
                print("    luacheck install failed (may already be installed)")
            end
        end
        
        print("\n=== Setup complete ===")
        print("Run 'xmake build' to start development")
    end)
    set_menu {
        usage = "xmake setup",
        description = "Install development dependencies"
    }
task_end()
