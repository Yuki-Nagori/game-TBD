//! 日志系统插件
//!
//! 增强的日志系统：分级日志、文件输出、游戏内查看

use bevy::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::Writer, time::FormatTime},
    layer::SubscriberExt,
    registry::LookupSpan,
    EnvFilter, Layer,
};

/// 日志配置
#[derive(Resource, Clone)]
pub struct LogConfig {
    /// 日志文件路径
    pub log_file: PathBuf,
    /// 日志级别
    pub level: LogLevel,
    /// 是否输出到文件
    pub file_output: bool,
    /// 是否输出到控制台
    pub console_output: bool,
    /// 日志文件最大大小（MB）
    pub max_file_size_mb: u64,
    /// 保留的日志文件数量
    pub max_files: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_file: PathBuf::from("logs/game.log"),
            level: LogLevel::Info,
            file_output: true,
            console_output: true,
            max_file_size_mb: 10,
            max_files: 5,
        }
    }
}

/// 日志级别
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// 日志文件写入器
pub struct FileLogWriter {
    file: Arc<Mutex<File>>,
    current_size: Arc<Mutex<u64>>,
    max_size: u64,
    log_dir: PathBuf,
    base_name: String,
}

impl std::io::Write for FileLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let message = String::from_utf8_lossy(buf);
        self.write(&message).map_err(|e| std::io::Error::other(e))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut file = self.file.lock().unwrap();
        file.flush()
    }
}

impl fmt::MakeWriter<'_> for FileLogWriter {
    type Writer = FileLogWriter;

    fn make_writer(&self) -> Self::Writer {
        FileLogWriter {
            file: Arc::clone(&self.file),
            current_size: Arc::clone(&self.current_size),
            max_size: self.max_size,
            log_dir: self.log_dir.clone(),
            base_name: self.base_name.clone(),
        }
    }
}

impl FileLogWriter {
    pub fn new(config: &LogConfig) -> anyhow::Result<Self> {
        // 确保日志目录存在
        let log_dir = config.log_file.parent().unwrap_or(PathBuf::from(".").as_ref());
        std::fs::create_dir_all(log_dir)?;

        // 检查是否需要轮转
        let base_name = config
            .log_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("game")
            .to_string();

        let file = Self::open_log_file(&config.log_file)?;
        let metadata = file.metadata()?;
        let current_size = metadata.len();

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
            current_size: Arc::new(Mutex::new(current_size)),
            max_size: config.max_file_size_mb * 1024 * 1024,
            log_dir: log_dir.to_path_buf(),
            base_name,
        })
    }

    fn open_log_file(path: &PathBuf) -> anyhow::Result<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.into())
    }

    pub fn write(&self, message: &str) -> anyhow::Result<()> {
        let mut file = self.file.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();

        // 检查是否需要轮转
        if *current_size + message.len() as u64 > self.max_size {
            drop(file);
            self.rotate_logs()?;
            let new_file = Self::open_log_file(&self.log_dir.join(format!("{}.log", self.base_name)))?;
            *file = new_file;
            *current_size = 0;
        }

        // 写入日志
        file.write_all(message.as_bytes())?;
        file.flush()?;
        *current_size += message.len() as u64;

        Ok(())
    }

    fn rotate_logs(&self) -> anyhow::Result<()> {
        // 删除最旧的日志文件
        let oldest = self.log_dir.join(format!("{}.{}.log", self.base_name, 5));
        if oldest.exists() {
            std::fs::remove_file(oldest)?;
        }

        // 重命名其他日志文件
        for i in (1..5).rev() {
            let old = self.log_dir.join(format!("{}.{}.log", self.base_name, i));
            let new = self.log_dir.join(format!("{}.{}.log", self.base_name, i + 1));
            if old.exists() {
                std::fs::rename(old, new)?;
            }
        }

        // 重命名当前日志文件
        let current = self.log_dir.join(format!("{}.log", self.base_name));
        let new = self.log_dir.join(format!("{}.1.log", self.base_name));
        std::fs::rename(current, new)?;

        Ok(())
    }
}

/// 初始化日志系统
pub fn init_logging(config: LogConfig) -> anyhow::Result<()> {
    // 设置过滤器
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ming_rpg=debug"));

    // 控制台日志层
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    // 设置全局订阅者
    if config.file_output {
        // 创建文件日志写入器
        let file_writer = FileLogWriter::new(&config)?;
        let file_layer = fmt::layer()
            .with_writer(Arc::new(file_writer))
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false);

        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(console_layer)
            .with(file_layer);

        tracing::subscriber::set_global_default(subscriber)?;
        info!("日志系统初始化完成（控制台+文件）");
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(filter)
            .with(console_layer);

        tracing::subscriber::set_global_default(subscriber)?;
        info!("日志系统初始化完成（仅控制台）");
    }

    Ok(())
}

/// 日志系统插件
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        let config = LogConfig::default();

        if let Err(e) = init_logging(config.clone()) {
            eprintln!("日志系统初始化失败: {}", e);
        }

        app.insert_resource(config);
    }
}

/// 游戏内日志查看器组件
#[derive(Component)]
pub struct LogViewer;

/// 日志缓冲区（用于游戏内显示）
#[derive(Resource)]
pub struct LogBuffer {
    pub entries: Vec<LogEntry>,
    pub max_entries: usize,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 1000,
        }
    }
}

/// 日志条目
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

impl LogBuffer {
    pub fn add_entry(&mut self, entry: LogEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn get_filtered(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| {
                let e_level = e.level as i32;
                let filter_level = level as i32;
                e_level >= filter_level
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// 自定义时间格式化
pub struct GameTimeFormatter;

impl FormatTime for GameTimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        use std::time::SystemTime;
        let now = SystemTime::now();
        let datetime = chrono::DateTime::<chrono::Local>::from(now);
        write!(w, "{}", datetime.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}
