//! 调试控制台插件
//!
//! 游戏中按 `~` 键呼出调试控制台
//! 支持命令输入、日志查看、性能监控

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use std::collections::VecDeque;

/// 调试控制台状态
#[derive(Resource, Default)]
pub struct DebugConsoleState {
    /// 是否可见
    pub visible: bool,
    /// 输入缓冲区
    pub input_buffer: String,
    /// 命令历史
    pub history: Vec<String>,
    /// 日志缓冲区
    pub logs: VecDeque<LogEntry>,
    /// 最大日志条数
    pub max_logs: usize,
    /// 自动滚动
    pub auto_scroll: bool,
    /// 筛选级别
    pub filter_level: LogLevel,
}

/// 日志条目
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: f64,
}

/// 日志级别
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LogLevel {
    #[default]
    Debug,
    Info,
    Warn,
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

/// 性能监控状态
#[derive(Resource, Default)]
pub struct PerformanceMonitor {
    /// 是否显示性能面板
    pub visible: bool,
    /// FPS历史
    pub fps_history: VecDeque<f32>,
    /// 最大历史长度
    pub max_history: usize,
    /// 当前FPS
    pub current_fps: f32,
    /// 平均FPS
    pub avg_fps: f32,
    /// 帧时间（毫秒）
    pub frame_time_ms: f32,
    /// 实体数量
    pub entity_count: usize,
}

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        // 检查是否启用开发模式
        let dev_mode = std::env::var("MING_RPG_DEV_MODE")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if !dev_mode {
            info!("调试控制台已禁用（设置 MING_RPG_DEV_MODE=1 启用）");
            return;
        }

        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.init_resource::<DebugConsoleState>()
            .init_resource::<PerformanceMonitor>()
            .add_systems(Startup, setup_console)
            .add_systems(
                Update,
                (
                    toggle_console,
                    draw_console,
                    draw_performance_monitor,
                    update_performance_data,
                    receive_logs,
                ),
            );

        info!("调试控制台已启动（按 ~ 键呼出）");
    }
}

fn setup_console(mut console: ResMut<DebugConsoleState>) {
    console.max_logs = 1000;
    console.auto_scroll = true;
    console.filter_level = LogLevel::Debug;
}

/// 切换控制台显示
fn toggle_console(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsoleState>,
    mut perf_monitor: ResMut<PerformanceMonitor>,
) {
    // ` 键或 F1 键
    if keyboard.just_pressed(KeyCode::Backquote) || keyboard.just_pressed(KeyCode::F1) {
        console.visible = !console.visible;
        // 同时切换性能监控
        perf_monitor.visible = console.visible;
    }
}

/// 绘制调试控制台
fn draw_console(
    mut contexts: EguiContexts,
    mut console: ResMut<DebugConsoleState>,
    lua: Res<crate::lua_api::LuaRuntime>,
    mut app_exit: EventWriter<AppExit>,
) {
    if !console.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("调试控制台")
        .default_size([600.0, 400.0])
        .resizable(true)
        .show(ctx, |ui| {
            // 工具栏
            ui.horizontal(|ui| {
                if ui.button("清除").clicked() {
                    console.logs.clear();
                }
                ui.checkbox(&mut console.auto_scroll, "自动滚动");

                ui.label("筛选:");
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", console.filter_level))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut console.filter_level, LogLevel::Debug, "Debug");
                        ui.selectable_value(&mut console.filter_level, LogLevel::Info, "Info");
                        ui.selectable_value(&mut console.filter_level, LogLevel::Warn, "Warn");
                        ui.selectable_value(&mut console.filter_level, LogLevel::Error, "Error");
                    });
            });

            ui.separator();

            // 日志显示区域
            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);

            // 预过滤日志，避免虚拟滚动出现空白间隙
            let visible_logs: Vec<&LogEntry> = console
                .logs
                .iter()
                .filter(|log| match (console.filter_level, log.level) {
                    (LogLevel::Debug, _) => true,
                    (LogLevel::Info, LogLevel::Debug) => false,
                    (LogLevel::Info, _) => true,
                    (LogLevel::Warn, LogLevel::Debug | LogLevel::Info) => false,
                    (LogLevel::Warn, _) => true,
                    (LogLevel::Error, LogLevel::Error) => true,
                    (LogLevel::Error, _) => false,
                })
                .collect();

            // 计算是否应该在渲染后滚动到底部
            let should_scroll_to_bottom = console.auto_scroll && !visible_logs.is_empty();
            let last_index = visible_logs.len().saturating_sub(1);

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show_rows(ui, row_height, visible_logs.len(), |ui, row_range| {
                    for i in row_range {
                        if let Some(log) = visible_logs.get(i) {
                            let color = match log.level {
                                LogLevel::Debug => egui::Color32::GRAY,
                                LogLevel::Info => egui::Color32::WHITE,
                                LogLevel::Warn => egui::Color32::YELLOW,
                                LogLevel::Error => egui::Color32::RED,
                            };
                            let response =
                                ui.colored_label(color, format!("[{}] {}", log.level, log.message));
                            // 如果启用自动滚动且这是最后一行，滚动到底部
                            if should_scroll_to_bottom && i == last_index {
                                response.scroll_to_me(Some(egui::Align::BOTTOM));
                            }
                        }
                    }
                });

            ui.separator();

            // 命令输入
            let response = ui.text_edit_singleline(&mut console.input_buffer);

            // 按回车执行命令
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let command = console.input_buffer.clone();
                if !command.is_empty() {
                    execute_command(&command, &mut console, &lua, &mut app_exit);
                    console.history.push(command);
                    console.input_buffer.clear();
                }
                // 重新获取焦点
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
    lua: &crate::lua_api::LuaRuntime,
    app_exit: &mut EventWriter<AppExit>,
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
                    // 解析返回结果类型
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
            // FPS显示在性能监控面板
            console.add_log(LogLevel::Info, "查看右上角性能面板".to_string());
        }
        "quit" | "exit" => {
            console.add_log(LogLevel::Info, "正在退出...".to_string());
            app_exit.send(AppExit);
        }
        _ => {
            console.add_log(LogLevel::Warn, format!("未知命令: {}", parts[0]));
        }
    }
}

/// 绘制性能监控面板
fn draw_performance_monitor(mut contexts: EguiContexts, perf_monitor: Res<PerformanceMonitor>) {
    if !perf_monitor.visible {
        return;
    }

    let ctx = contexts.ctx_mut();

    egui::Window::new("性能监控")
        .default_pos([10.0, 10.0])
        .default_size([200.0, 150.0])
        .collapsible(true)
        .show(ctx, |ui| {
            ui.label(format!("FPS: {:.1}", perf_monitor.current_fps));
            ui.label(format!("平均FPS: {:.1}", perf_monitor.avg_fps));
            ui.label(format!("帧时间: {:.2}ms", perf_monitor.frame_time_ms));
            ui.label(format!("实体数: {}", perf_monitor.entity_count));

            // FPS历史 (文本显示)
            if !perf_monitor.fps_history.is_empty() {
                ui.separator();
                // 检查历史记录非空后再计算平均FPS，避免除零
                let history_len = perf_monitor.fps_history.len();
                if history_len > 0 {
                    let avg_fps: f32 =
                        perf_monitor.fps_history.iter().sum::<f32>() / history_len as f32;
                    ui.label(format!("平均 FPS: {:.1}", avg_fps));
                }
            }
        });
}

/// 更新性能数据
fn update_performance_data(
    mut perf_monitor: ResMut<PerformanceMonitor>,
    time: Res<Time>,
    query: Query<Entity>,
) {
    // 计算FPS
    let delta = time.delta_seconds();
    if delta > 0.0 {
        let fps = 1.0 / delta;
        perf_monitor.current_fps = fps;
        perf_monitor.frame_time_ms = delta * 1000.0;

        // 添加到历史（使用 VecDeque 避免 O(n) remove(0)）
        perf_monitor.fps_history.push_back(fps);
        if perf_monitor.fps_history.len() > perf_monitor.max_history {
            perf_monitor.fps_history.pop_front();
        }

        // 计算平均FPS（历史记录在push_back后至少有一个元素）
        let sum: f32 = perf_monitor.fps_history.iter().sum();
        perf_monitor.avg_fps = sum / perf_monitor.fps_history.len().max(1) as f32;
    }

    // 更新实体数量
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

        // 限制日志数量（使用 pop_front 避免 O(n) remove(0)）
        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }
    }
}

/// 全局日志接收器（将tracing日志转发到控制台）
pub struct ConsoleLogLayer {
    sender: std::sync::mpsc::Sender<LogEntry>,
}

/// 存储 Receiver 的全局静态变量
use std::sync::OnceLock;
static LOG_RECEIVER: OnceLock<std::sync::Mutex<std::sync::mpsc::Receiver<LogEntry>>> =
    OnceLock::new();

impl ConsoleLogLayer {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        // 存储 receiver 到全局静态变量
        let _ = LOG_RECEIVER.set(std::sync::Mutex::new(rx));
        Self { sender: tx }
    }
}

/// 接收日志系统：将 tracing 日志转发到 DebugConsoleState
fn receive_logs(mut console: ResMut<DebugConsoleState>) {
    if let Some(receiver) = LOG_RECEIVER.get() {
        let rx = receiver.lock().unwrap();
        // 非阻塞地接收所有可用日志
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

        // 如果没有提取到消息（空字符串），使用事件名称
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

        // 忽略发送失败（接收端可能已被丢弃）
        let _ = self.sender.send(entry);
    }
}

/// 用于从tracing事件中提取消息
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            // 优先尝试record_str，避免Debug格式化的引号
            // 如果值实现了Display，这里会被调用两次：
            // 1. 第一次尝试record_str（如果值是字符串）
            // 2. 如果没有实现record_str，则调用record_debug
            // 因此这里先不处理，让record_str优先
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }
}
