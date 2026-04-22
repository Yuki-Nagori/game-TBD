use criterion::{Criterion, criterion_group, criterion_main};
use ming_rpg::lua_api::LuaRuntime;

fn bench_lua_runtime_creation(c: &mut Criterion) {
    c.bench_function("lua_runtime_new", |b| b.iter(|| LuaRuntime::new().unwrap()));
}

fn bench_script_loading(c: &mut Criterion) {
    let runtime = LuaRuntime::new().unwrap();
    c.bench_function("load_main_script", |b| {
        b.iter(|| runtime.load_main_script("game/main.lua").unwrap())
    });
}

fn bench_config_parsing(c: &mut Criterion) {
    let runtime = LuaRuntime::new().unwrap();
    runtime.load_main_script("game/main.lua").unwrap();
    c.bench_function("get_config", |b| {
        b.iter(|| {
            let _config: Option<std::collections::HashMap<String, serde_json::Value>> =
                runtime.get_config("PLAYER_CONFIG");
        })
    });
}

criterion_group!(
    benches,
    bench_lua_runtime_creation,
    bench_script_loading,
    bench_config_parsing
);
criterion_main!(benches);
