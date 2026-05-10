use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Config ────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: LevelFilter,
    pub max_file_size_mb: u64,
    pub max_files: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: LevelFilter::Info,
            max_file_size_mb: 50,
            max_files: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LevelFilter {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            _ => Self::Info, // default fallback
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => " WARN",
            Self::Info => " INFO",
            Self::Debug => "DEBUG",
        }
    }
}

// ── Logger ─────────────────────────────────────────────────────────────
pub(crate) struct Logger {
    writer: Option<BufWriter<File>>,
    current_path: Option<PathBuf>,
    level: LevelFilter,
    max_size: u64,
    max_files: usize,
    log_dir: PathBuf,
    disabled: bool,
}

pub(crate) static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();

pub fn init(config: &LoggingConfig) {
    if !config.enabled {
        // init with disabled logger — macros will no-op
        let logger = Logger {
            writer: None,
            current_path: None,
            level: config.level,
            max_size: config.max_file_size_mb * 1024 * 1024,
            max_files: config.max_files,
            log_dir: log_dir(),
            disabled: true,
        };
        let _ = LOGGER.set(Mutex::new(logger));
        return;
    }

    let dir = log_dir();
    let _ = fs::create_dir_all(&dir);

    let ts = now_ts();
    let filename = format!("codeloom_{}.log", ts);
    let path = dir.join(&filename);

    let file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[logger] Cannot create log file {}: {}", path.display(), e);
            let logger = Logger {
                writer: None,
                current_path: None,
                level: config.level,
                max_size: config.max_file_size_mb * 1024 * 1024,
                max_files: config.max_files,
                log_dir: dir,
                disabled: true,
            };
            let _ = LOGGER.set(Mutex::new(logger));
            return;
        }
    };

    let mut logger = Logger {
        writer: Some(BufWriter::new(file)),
        current_path: Some(path),
        level: config.level,
        max_size: config.max_file_size_mb * 1024 * 1024,
        max_files: config.max_files,
        log_dir: dir,
        disabled: false,
    };

    // Cleanup old files on startup
    logger.cleanup_old_files();
    let _ = LOGGER.set(Mutex::new(logger));
}

fn log_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codeloom")
        .join("logs")
}

fn now_ts() -> String {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since.as_secs();
    let ms = since.subsec_millis();
    // YYYYMMDD-HHMMSS_ms
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;

    // Calculate year/month/day from days since epoch
    // Using a simple algorithm (valid for 1970-2100)
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let month_days: [i64; 12] = [
        31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];
    let mut m = 1usize;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    let day = d + 1;

    format!("{:04}{:02}{:02}-{:02}{:02}{:02}_{:03}", y, m, day, hours, minutes, seconds, ms)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

impl Logger {
    pub(crate) fn log(&mut self, level: LevelFilter, module: &str, msg: &str) {
        if self.disabled || self.writer.is_none() {
            return;
        }
        if level > self.level {
            return;
        }

        // Rotate if needed
        self.check_rotate();

        if let Some(ref mut w) = self.writer {
            let timestamp = format_now();
            let pid = std::process::id();
            let tid = thread_id();
            let _ = writeln!(
                w,
                "{} [{pid}:{tid}] {} {}: {msg}",
                timestamp,
                level.label(),
                module
            );
            // Flush each line so logs survive crashes
            let _ = w.flush();
        }
    }

    fn check_rotate(&mut self) {
        let path = match &self.current_path {
            Some(p) => p.clone(),
            None => return,
        };
        let size = match fs::metadata(&path) {
            Ok(m) => m.len(),
            Err(_) => return,
        };

        if size <= self.max_size {
            return;
        }

        // Close current file
        self.writer = None;

        // Create new file with new timestamp
        let ts = now_ts();
        let filename = format!("codeloom_{}.log", ts);
        let new_path = self.log_dir.join(&filename);

        match OpenOptions::new().create(true).append(true).open(&new_path) {
            Ok(f) => {
                self.writer = Some(BufWriter::new(f));
                self.current_path = Some(new_path);
                // Cleanup after rotation
                self.cleanup_old_files();
            }
            Err(e) => {
                eprintln!("[logger] Cannot rotate log file: {}", e);
                self.disabled = true;
            }
        }
    }

    fn cleanup_old_files(&mut self) {
        let mut entries: Vec<_> = match fs::read_dir(&self.log_dir) {
            Ok(iter) => iter
                .filter_map(|e| {
                    let e = e.ok()?;
                    let path = e.path();
                    if path.extension().map_or(false, |ext| ext == "log") {
                        let mod_time = fs::metadata(&path).ok()?.modified().ok()?;
                        Some((mod_time, path))
                    } else {
                        None
                    }
                })
                .collect(),
            Err(_) => return,
        };

        if entries.len() <= self.max_files {
            return;
        }

        // Sort by modification time ascending (oldest first)
        entries.sort_by_key(|(t, _)| *t);

        let to_remove = entries.len() - self.max_files;
        for (_, path) in entries.iter().take(to_remove) {
            let _ = fs::remove_file(path);
        }
    }
}

fn format_now() -> String {
    let since = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = since.as_secs();
    let ms = since.subsec_millis();
    let days = secs / 86400;
    let time = secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;

    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let diy = if is_leap(y) { 366 } else { 365 };
        if d < diy {
            break;
        }
        d -= diy;
        y += 1;
    }
    let md: [i64; 12] = [
        31, if is_leap(y) { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];
    let mut m = 1usize;
    for &mdv in &md {
        if d < mdv {
            break;
        }
        d -= mdv;
        m += 1;
    }
    let day = d + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        y, m, day, hours, minutes, seconds, ms
    )
}

fn thread_id() -> u64 {
    // Use a simple counter for thread ID since std doesn't expose native TID
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    thread_local! {
        static THREAD_ID: u64 = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    }
    THREAD_ID.with(|&id| id)
}

// ── Public macros ──────────────────────────────────────────────────────

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {{
        if let Some(lock) = $crate::logger::LOGGER.get() {
            if let Ok(mut logger) = lock.lock() {
                logger.log($crate::logger::LevelFilter::Error, $module, &format!($($arg)*));
            }
        }
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {{
        if let Some(lock) = $crate::logger::LOGGER.get() {
            if let Ok(mut logger) = lock.lock() {
                logger.log($crate::logger::LevelFilter::Warn, $module, &format!($($arg)*));
            }
        }
    }};
}

#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {{
        if let Some(lock) = $crate::logger::LOGGER.get() {
            if let Ok(mut logger) = lock.lock() {
                logger.log($crate::logger::LevelFilter::Info, $module, &format!($($arg)*));
            }
        }
    }};
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {{
        if let Some(lock) = $crate::logger::LOGGER.get() {
            if let Ok(mut logger) = lock.lock() {
                logger.log($crate::logger::LevelFilter::Debug, $module, &format!($($arg)*));
            }
        }
    }};
}
