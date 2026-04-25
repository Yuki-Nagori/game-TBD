//! 调试控制台插件
//!
//! 游戏中按 `~` 键呼出调试控制台
//! 支持命令输入、日志查看、性能监控
//!
//! 特性：
//! - 中文字体支持（Noto Sans SC 嵌入）
//! - 暗色主题
//! - 时间戳与级别徽章
//! - FPS / 帧时间折线图
//! - 命令历史与 Tab 补全

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use std::collections::VecDeque;

/// 已知命令列表（用于 Tab 补全）
const KNOWN_COMMANDS: &[&str] = &[
    "help", "clear", "lua", "reload", "fps", "entities", "editor", "quit", "exit",
];

/// 调试控制台状态
#[derive(Resource, Default)]
pub struct DebugConsoleState {
    /// 是否可见
    pub visible: bool,
    /// 输入缓冲区
    pub input_buffer: String,
    /// 命令历史
    pub history: Vec<String>,
    /// 历史导航索引（None 表示不在历史中）
    pub history_index: Option<usize>,
    /// 编辑中草稿（历史导航时保留原始输入）
    pub draft_input: String,
    /// 日志缓冲区
    pub logs: VecDeque<LogEntry>,
    /// 最大日志条数
    pub max_logs: usize,
    /// 自动滚动
    pub auto_scroll: bool,
    /// 筛选级别
    pub filter_level: LogLevel,
    /// 是否显示实体查看器
    pub show_entity_viewer: bool,
    /// 实体查看器筛选文本
    pub entity_filter: String,
    /// 选中的实体（用于展开详情）
    pub selected_entity: Option<Entity>,
}

/// 日志条目
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// 日志级别
    pub level: LogLevel,
    /// 日志消息
    pub message: String,
    /// 时间戳（Unix 时间，秒）
    pub timestamp: f64,
}

/// 日志级别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// 调试
    #[default]
    Debug,
    /// 信息
    Info,
    /// 警告
    Warn,
    /// 错误
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    /// 返回级别的显示颜色
    fn color(self) -> egui::Color32 {
        match self {
            LogLevel::Debug => egui::Color32::from_gray(160),
            LogLevel::Info => egui::Color32::from_rgb(200, 200, 200),
            LogLevel::Warn => egui::Color32::from_rgb(255, 200, 80),
            LogLevel::Error => egui::Color32::from_rgb(255, 90, 90),
        }
    }

    /// 返回级别的徽章背景色
    fn badge_bg(self) -> egui::Color32 {
        match self {
            LogLevel::Debug => egui::Color32::from_rgb(80, 80, 100),
            LogLevel::Info => egui::Color32::from_rgb(60, 100, 140),
            LogLevel::Warn => egui::Color32::from_rgb(140, 110, 40),
            LogLevel::Error => egui::Color32::from_rgb(140, 50, 50),
        }
    }

    /// 返回级别的优先级数值（用于筛选比较）
    fn priority(self) -> u8 {
        match self {
            LogLevel::Debug => 0,
            LogLevel::Info => 1,
            LogLevel::Warn => 2,
            LogLevel::Error => 3,
        }
    }
}

/// 性能监控状态
#[derive(Resource, Default)]
pub struct PerformanceMonitor {
    /// 是否显示性能面板
    pub visible: bool,
    /// FPS 历史
    pub fps_history: VecDeque<f32>,
    /// 帧时间历史（毫秒）
    pub frame_time_history: VecDeque<f32>,
    /// 最大历史长度
    pub max_history: usize,
    /// 当前 FPS
    pub current_fps: f32,
    /// 平均 FPS
    pub avg_fps: f32,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// 实体数量
    pub entity_count: usize,
}

/// 场景编辑器状态
#[derive(Resource, Default)]
pub struct SceneEditorState {
    /// 是否启用编辑器
    pub enabled: bool,
    /// 选中的预设类型
    pub selected_prefab: String,
    /// 放置位置
    pub placement_position: Vec3,
    /// 放置历史（用于撤销）
    pub history: Vec<Entity>,
}

/// 调试控制台插件
pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        let dev_mode = std::env::var("MING_RPG_DEV_MODE")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if !dev_mode {
            info!("调试控制台已禁用（设置 MING_RPG_DEV_MODE=1 启用）");
            return;
        }

        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        app.init_resource::<DebugConsoleState>()
            .init_resource::<PerformanceMonitor>()
            .init_resource::<SceneEditorState>()
            .add_systems(Startup, setup_console)
            .add_systems(
                Update,
                (
                    toggle_console,
                    draw_console,
                    draw_entity_viewer,
                    draw_scene_editor,
                    draw_performance_monitor,
                    update_performance_data,
                    receive_logs,
                ),
            );

        info!("调试控制台已启动（按 ~ 键呼出）");
    }
}

fn setup_console(mut console: ResMut<DebugConsoleState>) {
    console.max_logs = 200;
    console.auto_scroll = true;
    console.filter_level = LogLevel::Info;
}

/// 切换控制台显示
fn toggle_console(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsoleState>,
    mut perf_monitor: ResMut<PerformanceMonitor>,
) {
    if keyboard.just_pressed(KeyCode::Backquote) || keyboard.just_pressed(KeyCode::F1) {
        console.visible = !console.visible;

        perf_monitor.visible = console.visible;
    }
}

/// 将 Unix 时间戳格式化为 HH:MM:SS
fn timestamp_to_hhmmss(timestamp: f64) -> String {
    let secs = timestamp as u64;
    let hh = (secs / 3600) % 24;
    let mm = (secs / 60) % 60;
    let ss = secs % 60;
    format!("{:02}:{:02}:{:02}", hh, mm, ss)
}

/// 绘制调试控制台主窗口
fn draw_console(
    mut contexts: EguiContexts,
    mut console: ResMut<DebugConsoleState>,
    mut editor: ResMut<SceneEditorState>,
    lua: Res<crate::lua_api::LuaRuntime>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !console.visible {
        return;
    }

    let ctx = contexts.ctx_mut().expect("Primary Egui context not found");

    egui::Window::new("调试控制台")
        .default_pos([500.0, 10.0])
        .default_size([770.0, 700.0])
        .min_size([200.0, 400.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("清除").size(13.0)).clicked() {
                    console.logs.clear();
                }
                ui.checkbox(
                    &mut console.auto_scroll,
                    egui::RichText::new("自动滚动").size(13.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("筛选:").size(13.0));
                    egui::ComboBox::from_id_salt("filter_level")
                        .width(80.0)
                        .selected_text(format!("{:?}", console.filter_level))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut console.filter_level,
                                LogLevel::Debug,
                                "Debug",
                            );
                            ui.selectable_value(&mut console.filter_level, LogLevel::Info, "Info");
                            ui.selectable_value(&mut console.filter_level, LogLevel::Warn, "Warn");
                            ui.selectable_value(
                                &mut console.filter_level,
                                LogLevel::Error,
                                "Error",
                            );
                        });
                });
            });

            ui.separator();

            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style) + 4.0;

            let visible_logs: Vec<&LogEntry> = console
                .logs
                .iter()
                .filter(|log| log.level.priority() >= console.filter_level.priority())
                .collect();

            let should_scroll_to_bottom = console.auto_scroll && !visible_logs.is_empty();
            let last_index = visible_logs.len().saturating_sub(1);

            let log_area_height = (ui.available_height() - 50.0).max(80.0);

            egui::ScrollArea::vertical()
                .max_height(log_area_height)
                .auto_shrink([false; 2])
                .show_rows(ui, row_height, visible_logs.len(), |ui, row_range| {
                    for i in row_range {
                        if let Some(log) = visible_logs.get(i) {
                            let is_even = i % 2 == 0;
                            let bg_color = if is_even {
                                egui::Color32::from_rgb(26, 26, 30)
                            } else {
                                egui::Color32::from_rgb(22, 22, 26)
                            };

                            ui.horizontal(|ui| {
                                let full_width = ui.available_width();
                                ui.set_min_size(egui::vec2(full_width, row_height));

                                let row_rect = egui::Rect::from_min_size(
                                    ui.cursor().min,
                                    egui::vec2(full_width, row_height),
                                );
                                ui.painter().rect_filled(row_rect, 0.0, bg_color);

                                let badge_galley = ui.painter().layout_no_wrap(
                                    format!(" {} ", log.level),
                                    egui::FontId::monospace(10.0),
                                    egui::Color32::WHITE,
                                );
                                let badge_size = badge_galley.rect.size();
                                let badge_rect = egui::Rect::from_min_size(
                                    ui.cursor().min,
                                    egui::vec2(badge_size.x + 4.0, row_height - 4.0),
                                );
                                ui.painter()
                                    .rect_filled(badge_rect, 4.0, log.level.badge_bg());
                                ui.painter().galley(
                                    badge_rect.center() - badge_size * 0.5 + egui::vec2(2.0, 0.0),
                                    badge_galley,
                                    egui::Color32::WHITE,
                                );
                                ui.add_space(badge_rect.width() + 6.0);

                                ui.label(
                                    egui::RichText::new(timestamp_to_hhmmss(log.timestamp))
                                        .monospace()
                                        .size(11.0)
                                        .color(egui::Color32::from_gray(140)),
                                );
                                ui.add_space(8.0);

                                let msg = egui::RichText::new(&log.message)
                                    .size(13.0)
                                    .color(log.level.color());
                                let response = ui.selectable_label(false, msg);

                                if should_scroll_to_bottom && i == last_index {
                                    response.scroll_to_me(Some(egui::Align::BOTTOM));
                                }
                            });
                        }
                    }
                });

            ui.separator();

            let mut execute = false;
            let mut navigate_up = false;
            let mut navigate_down = false;
            let mut tab_complete = false;

            let response = ui.add(
                egui::TextEdit::singleline(&mut console.input_buffer)
                    .hint_text("输入命令...")
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );

            if ui.input(|i| i.key_down(egui::Key::Enter)) {
                execute = true;
            }
            if ui.input(|i| i.key_down(egui::Key::ArrowUp)) {
                navigate_up = true;
            }
            if ui.input(|i| i.key_down(egui::Key::ArrowDown)) {
                navigate_down = true;
            }
            if ui.input(|i| i.key_down(egui::Key::Tab)) {
                tab_complete = true;
            }

            if tab_complete && !console.input_buffer.is_empty() {
                let input = console.input_buffer.clone();
                if let Some(matched) = KNOWN_COMMANDS.iter().find(|cmd| cmd.starts_with(&input)) {
                    console.input_buffer = matched.to_string();

                    response.request_focus();
                }
            }

            if navigate_up {
                if console.history_index.is_none() && !console.input_buffer.is_empty() {
                    console.draft_input = console.input_buffer.clone();
                }
                let max_idx = console.history.len().saturating_sub(1);
                let new_idx = console
                    .history_index
                    .map_or(max_idx, |i| i.saturating_sub(1));
                if !console.history.is_empty() && new_idx < console.history.len() {
                    console.history_index = Some(new_idx);
                    console.input_buffer = console.history[new_idx].clone();
                }
                response.request_focus();
            }

            if navigate_down {
                if let Some(idx) = console.history_index {
                    if idx + 1 < console.history.len() {
                        let new_idx = idx + 1;
                        console.history_index = Some(new_idx);
                        console.input_buffer = console.history[new_idx].clone();
                    } else {
                        console.history_index = None;
                        console.input_buffer = console.draft_input.clone();
                    }
                }
                response.request_focus();
            }

            if execute && !console.input_buffer.is_empty() {
                let command = console.input_buffer.clone();
                execute_command(&command, &mut console, &mut editor, &lua, &mut app_exit);
                console.history.push(command);
                console.input_buffer.clear();
                console.draft_input.clear();
                console.history_index = None;
                response.request_focus();
            }
        });
}

/// Lua 执行结果
enum LuaResult {
    /// 无返回值（nil）
    Nil,
    /// 有返回值
    Value(String),
}

/// 执行控制台命令
fn execute_command(
    command: &str,
    console: &mut DebugConsoleState,
    editor: &mut SceneEditorState,
    lua: &crate::lua_api::LuaRuntime,
    app_exit: &mut MessageWriter<AppExit>,
) {
    console.add_log(LogLevel::Info, format!("> {}", command));

    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => {
            console.add_log(LogLevel::Info, "可用命令:".to_string());
            console.add_log(LogLevel::Info, "  help - 显示帮助".to_string());
            console.add_log(LogLevel::Info, "  clear - 清除日志".to_string());
            console.add_log(LogLevel::Info, "  lua <code> - 执行Lua代码".to_string());
            console.add_log(LogLevel::Info, "  reload - 重载Lua脚本".to_string());
            console.add_log(LogLevel::Info, "  fps - 显示FPS".to_string());
            console.add_log(LogLevel::Info, "  entities - 打开实体查看器".to_string());
            console.add_log(LogLevel::Info, "  editor - 打开场景编辑器".to_string());
            console.add_log(LogLevel::Info, "  quit - 退出游戏".to_string());
        }
        "clear" => {
            console.logs.clear();
        }
        "lua" => {
            let code = parts[1..].join(" ");
            if code.is_empty() {
                console.add_log(LogLevel::Warn, "用法: lua <代码>".to_string());
                return;
            }
            match lua.execute_with_return(&code) {
                Ok(result_str) => {
                    let result = if result_str == "nil" {
                        LuaResult::Nil
                    } else {
                        LuaResult::Value(result_str)
                    };
                    match result {
                        LuaResult::Nil => {
                            console.add_log(LogLevel::Info, "执行成功 (无返回值)".to_string());
                        }
                        LuaResult::Value(v) => {
                            console.add_log(LogLevel::Info, format!("<= {}", v));
                        }
                    }
                }
                Err(e) => console.add_log(LogLevel::Error, format!("Error: {}", e)),
            }
        }
        "reload" => match lua.load_main_script("game/main.lua") {
            Ok(_) => console.add_log(LogLevel::Info, "Lua脚本重载成功".to_string()),
            Err(e) => console.add_log(LogLevel::Error, format!("重载失败: {}", e)),
        },
        "fps" => {
            console.add_log(LogLevel::Info, "查看右上角性能面板".to_string());
        }
        "entities" => {
            console.show_entity_viewer = true;
            console.add_log(LogLevel::Info, "实体查看器已打开".to_string());
        }
        "editor" => {
            editor.enabled = true;
            console.add_log(
                LogLevel::Info,
                "场景编辑器已打开（在性能面板下方）".to_string(),
            );
        }
        "quit" | "exit" => {
            console.add_log(LogLevel::Info, "正在退出...".to_string());
            app_exit.write(AppExit::Success);
        }
        _ => {
            console.add_log(LogLevel::Warn, format!("未知命令: {}", parts[0]));
        }
    }
}

/// 绘制实体查看器面板
#[allow(clippy::too_many_arguments)]
fn draw_entity_viewer(
    mut contexts: EguiContexts,
    mut console: ResMut<DebugConsoleState>,
    all_entities: Query<(Entity, Option<&Name>)>,
    player_query: Query<Entity, With<crate::components::Player>>,
    camera_query: Query<Entity, With<crate::components::ThirdPersonCamera>>,
    motion_query: Query<Entity, With<crate::components::CharacterMotion>>,
    animation_query: Query<Entity, With<crate::components::PlaceholderWalkAnimation>>,
    transform_query: Query<Entity, With<Transform>>,
) {
    if !console.show_entity_viewer {
        return;
    }

    let ctx = contexts.ctx_mut().expect("Primary Egui context not found");

    egui::Window::new("实体查看器")
        .default_pos([10.0, 510.0])
        .default_size([370.0, 200.0])
        .min_size([260.0, 200.0])
        .collapsible(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("筛选:").size(13.0));
                ui.add(egui::TextEdit::singleline(&mut console.entity_filter).desired_width(180.0));
                if ui.button("关闭").clicked() {
                    console.show_entity_viewer = false;
                }
            });

            ui.separator();
            ui.label(
                egui::RichText::new(format!("实体总数: {}", all_entities.iter().count()))
                    .size(13.0)
                    .color(egui::Color32::from_rgb(160, 200, 255)),
            );
            ui.separator();

            let filter = console.entity_filter.to_lowercase();
            let mut entities: Vec<_> = all_entities.iter().collect();
            entities.sort_by_key(|(e, _)| e.index());

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (entity, name) in entities {
                    let name_str = name.map(|n| n.as_str()).unwrap_or("<未命名>");
                    let id_str = format!("{}: {}", entity.index(), name_str);

                    if !filter.is_empty() && !id_str.to_lowercase().contains(&filter) {
                        continue;
                    }

                    let mut components = Vec::new();
                    if player_query.get(entity).is_ok() {
                        components.push("Player");
                    }
                    if camera_query.get(entity).is_ok() {
                        components.push("ThirdPersonCamera");
                    }
                    if motion_query.get(entity).is_ok() {
                        components.push("CharacterMotion");
                    }
                    if animation_query.get(entity).is_ok() {
                        components.push("PlaceholderWalkAnimation");
                    }
                    if transform_query.get(entity).is_ok() {
                        components.push("Transform");
                    }

                    let is_selected = console.selected_entity == Some(entity);
                    let response = ui.selectable_label(
                        is_selected,
                        egui::RichText::new(format!("{} ({} 个组件)", id_str, components.len()))
                            .size(13.0),
                    );

                    if response.clicked() {
                        console.selected_entity = Some(entity);
                    }

                    if is_selected {
                        ui.indent("details", |ui| {
                            for comp in &components {
                                ui.label(
                                    egui::RichText::new(format!("  - {}", comp))
                                        .size(12.0)
                                        .color(egui::Color32::from_gray(180)),
                                );
                            }
                        });
                    }
                }
            });
        });
}

/// 绘制场景编辑器面板
fn draw_scene_editor(
    mut contexts: EguiContexts,
    mut editor: ResMut<SceneEditorState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_colors: Res<crate::plugins::scene_plugin::SceneColorRes>,
    editor_placed: Query<Entity, With<crate::components::EditorPlaced>>,
) {
    if !editor.enabled {
        return;
    }

    let ctx = contexts.ctx_mut().expect("Primary Egui context not found");

    egui::Window::new("场景编辑器")
        .default_pos([10.0, 240.0])
        .default_size([370.0, 260.0])
        .min_size([180.0, 200.0])
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("预设:").size(13.0).strong());
            ui.horizontal(|ui| {
                let presets = [("building", "Building"), ("tree", "Tree"), ("wall", "Wall")];
                for (key, label) in presets {
                    let selected = editor.selected_prefab == key;
                    if ui
                        .selectable_label(
                            selected,
                            egui::RichText::new(label).size(12.0).color(if selected {
                                egui::Color32::from_rgb(120, 180, 255)
                            } else {
                                egui::Color32::from_gray(180)
                            }),
                        )
                        .clicked()
                    {
                        editor.selected_prefab = key.to_string();
                    }
                }
            });

            ui.separator();
            ui.label(egui::RichText::new("位置:").size(13.0).strong());
            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(egui::Slider::new(
                    &mut editor.placement_position.x,
                    -50.0..=50.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Z:");
                ui.add(egui::Slider::new(
                    &mut editor.placement_position.z,
                    -50.0..=50.0,
                ));
            });
            ui.horizontal(|ui| {
                ui.label("Y:");
                ui.add(egui::DragValue::new(&mut editor.placement_position.y));
            });

            ui.separator();
            if ui.button("放置").clicked()
                && let Some(entity) = crate::plugins::scene_plugin::spawn_scene_object(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &scene_colors.colors,
                    &editor.selected_prefab,
                    editor.placement_position,
                )
            {
                editor.history.push(entity);
            }
            ui.horizontal(|ui| {
                if ui.button("撤销").clicked()
                    && let Some(entity) = editor.history.pop()
                {
                    commands.entity(entity).despawn();
                }
                if ui.button("清空").clicked() {
                    editor.history.clear();
                    for entity in &editor_placed {
                        commands.entity(entity).despawn();
                    }
                }
            });
        });
}

/// 绘制性能监控面板（含 FPS / 帧时间折线图）
fn draw_performance_monitor(mut contexts: EguiContexts, perf_monitor: Res<PerformanceMonitor>) {
    if !perf_monitor.visible {
        return;
    }

    let ctx = contexts.ctx_mut().expect("Primary Egui context not found");

    egui::Window::new("性能监控")
        .default_pos([10.0, 10.0])
        .default_size([300.0, 220.0])
        .min_size([180.0, 200.0])
        .collapsible(true)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            let fps_text_color = fps_color(perf_monitor.current_fps);
            ui.label(
                egui::RichText::new(format!("FPS: {:.1}", perf_monitor.current_fps))
                    .size(14.0)
                    .strong()
                    .color(fps_text_color),
            );
            ui.label(
                egui::RichText::new(format!("平均FPS: {:.1}", perf_monitor.avg_fps))
                    .size(12.0)
                    .color(egui::Color32::from_gray(180)),
            );
            ui.label(
                egui::RichText::new(format!("帧时间: {:.2}ms", perf_monitor.frame_time_ms))
                    .size(12.0)
                    .color(egui::Color32::from_gray(180)),
            );
            ui.label(
                egui::RichText::new(format!("实体数: {}", perf_monitor.entity_count))
                    .size(12.0)
                    .color(egui::Color32::from_gray(180)),
            );

            ui.separator();

            ui.label(egui::RichText::new("FPS 历史").size(12.0).strong());
            draw_line_chart(ui, &perf_monitor.fps_history, 0.0, 120.0, fps_color);

            ui.separator();

            ui.label(egui::RichText::new("帧时间历史 (ms)").size(12.0).strong());
            draw_line_chart(ui, &perf_monitor.frame_time_history, 0.0, 33.0, |v| {
                if v < 16.0 {
                    egui::Color32::from_rgb(100, 220, 120)
                } else if v < 33.0 {
                    egui::Color32::from_rgb(255, 200, 80)
                } else {
                    egui::Color32::from_rgb(255, 90, 90)
                }
            });
        });
}

/// 根据 FPS 值返回显示颜色
fn fps_color(fps: f32) -> egui::Color32 {
    if fps >= 60.0 {
        egui::Color32::from_rgb(100, 220, 120)
    } else if fps >= 30.0 {
        egui::Color32::from_rgb(255, 200, 80)
    } else {
        egui::Color32::from_rgb(255, 90, 90)
    }
}

/// 在 UI 中绘制折线图
///
/// `values` 是原始值队列，`min`/`max` 是 Y 轴范围，`color_fn` 根据当前值决定线条颜色
fn draw_line_chart(
    ui: &mut egui::Ui,
    values: &VecDeque<f32>,
    min: f32,
    max: f32,
    color_fn: impl Fn(f32) -> egui::Color32,
) {
    let desired_size = egui::vec2(ui.available_width(), 48.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    if values.len() < 2 {
        ui.painter()
            .rect_filled(rect, 4.0, egui::Color32::from_gray(24));
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "收集数据中...",
            egui::FontId::proportional(11.0),
            egui::Color32::from_gray(100),
        );
        return;
    }

    let painter = ui.painter_at(rect);
    let bg_color = egui::Color32::from_gray(22);
    painter.rect_filled(rect, 4.0, bg_color);

    let grid_color = egui::Color32::from_gray(40);
    for i in 1..4 {
        let t = i as f32 / 4.0;
        let y = rect.bottom() - t * rect.height();
        painter.hline(rect.x_range(), y, egui::Stroke::new(1.0, grid_color));
    }

    let range = max - min;
    if range <= 0.0 {
        return;
    }

    let len = values.len();
    let points: Vec<egui::Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.left() + (i as f32 / (len.saturating_sub(1) as f32)) * rect.width();
            let t = ((v - min) / range).clamp(0.0, 1.0);
            let y = rect.bottom() - t * rect.height();
            egui::pos2(x, y.clamp(rect.top(), rect.bottom()))
        })
        .collect();

    if points.len() >= 2 {
        let last_val = values.back().copied().unwrap_or(0.0);
        let main_color = color_fn(last_val);
        painter.add(egui::Shape::line(
            points.clone(),
            egui::Stroke::new(1.5, main_color),
        ));

        if let Some(&last_point) = points.last() {
            painter.circle_filled(last_point, 3.0, main_color);
        }
    }

    if let Some(&latest) = values.back() {
        let label = format!("{:.1}", latest);
        let galley = painter.layout_no_wrap(
            label,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        let label_pos = egui::pos2(rect.right() - galley.rect.width() - 4.0, rect.top() + 2.0);
        painter.rect_filled(
            egui::Rect::from_min_size(
                label_pos - egui::vec2(2.0, 0.0),
                galley.rect.size() + egui::vec2(4.0, 2.0),
            ),
            2.0,
            egui::Color32::from_black_alpha(180),
        );
        painter.galley(label_pos, galley, egui::Color32::WHITE);
    }
}

/// 更新性能数据
fn update_performance_data(
    mut perf_monitor: ResMut<PerformanceMonitor>,
    time: Res<Time>,
    query: Query<Entity>,
) {
    let delta = time.delta_secs();
    if delta > 0.0 {
        let fps = 1.0 / delta;
        perf_monitor.current_fps = fps;
        perf_monitor.frame_time_ms = delta * 1000.0;

        perf_monitor.fps_history.push_back(fps);
        if perf_monitor.fps_history.len() > perf_monitor.max_history {
            perf_monitor.fps_history.pop_front();
        }

        let frame_time_ms = perf_monitor.frame_time_ms;
        perf_monitor.frame_time_history.push_back(frame_time_ms);
        if perf_monitor.frame_time_history.len() > perf_monitor.max_history {
            perf_monitor.frame_time_history.pop_front();
        }

        let sum: f32 = perf_monitor.fps_history.iter().sum();
        perf_monitor.avg_fps = sum / perf_monitor.fps_history.len().max(1) as f32;
    }

    perf_monitor.entity_count = query.iter().count();
}

impl DebugConsoleState {
    /// 添加日志条目
    pub fn add_log(&mut self, level: LogLevel, message: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        self.logs.push_back(LogEntry { level, message, timestamp });

        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }
    }
}

/// 全局日志接收器（将 tracing 日志转发到控制台）
pub struct ConsoleLogLayer {
    /// 日志发送通道
    sender: std::sync::mpsc::Sender<LogEntry>,
}

/// 存储 Receiver 的全局静态变量
use std::sync::OnceLock;
static LOG_RECEIVER: OnceLock<std::sync::Mutex<std::sync::mpsc::Receiver<LogEntry>>> =
    OnceLock::new();

impl ConsoleLogLayer {
    /// 创建新的日志层（单例，只能调用一次）
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        if LOG_RECEIVER.set(std::sync::Mutex::new(rx)).is_err() {
            panic!("ConsoleLogLayer::new() called more than once; it is a singleton");
        }
        Self { sender: tx }
    }
}

/// 接收日志系统：将 tracing 日志转发到 DebugConsoleState
fn receive_logs(mut console: ResMut<DebugConsoleState>) {
    if let Some(receiver) = LOG_RECEIVER.get() {
        let rx = receiver.lock().unwrap();
        while let Ok(entry) = rx.try_recv() {
            console.add_log(entry.level, entry.message);
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for ConsoleLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut message = String::new();
        event.record(&mut MessageVisitor(&mut message));

        if message.is_empty() {
            message = event.metadata().name().to_string();
        }

        let level = match *event.metadata().level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            _ => LogLevel::Debug,
        };

        let entry = LogEntry {
            level,
            message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
        };

        let _ = self.sender.send(entry);
    }
}

/// 用于从 tracing 事件中提取消息
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {
        if field.name() == "message" {}
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }
}
