use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

/// Log levels for the kernel logging framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn color(&self) -> u32 {
        match self {
            LogLevel::Debug => 0xFF888888, // Gray
            LogLevel::Info => 0xFFFFFFFF,  // White
            LogLevel::Warn => 0xFFFFAA00,  // Orange
            LogLevel::Error => 0xFFFF0000, // Red
        }
    }
}

/// Global log level filter
static LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Debug as u8);

/// Set the global log level
pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

/// Get the current log level
pub fn get_log_level() -> LogLevel {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        0 => LogLevel::Debug,
        1 => LogLevel::Info,
        2 => LogLevel::Warn,
        3 => LogLevel::Error,
        _ => LogLevel::Info,
    }
}

/// Check if a log level should be printed
pub fn should_log(level: LogLevel) -> bool {
    level >= get_log_level()
}

/// Log a message with the given level
pub fn log(level: LogLevel, args: fmt::Arguments) {
    if !should_log(level) {
        return;
    }

    // Set color based on log level (for framebuffer)
    let mut fb = super::framebuffer::framebuffer();
    if fb.is_initialized() {
        let old_fg = fb.fg_color;
        fb.set_fg_color(level.color());
        drop(fb);
        
        super::console::_print(core::format_args!("[{}] ", level.as_str()));
        super::console::_print(args);
        super::console::_print(core::format_args!("\n"));
        
        // Restore color
        let mut fb = super::framebuffer::framebuffer();
        fb.set_fg_color(old_fg);
    } else {
        // Framebuffer not initialized, just print without color
        super::console::_print(core::format_args!("[{}] ", level.as_str()));
        super::console::_print(args);
        super::console::_print(core::format_args!("\n"));
    }
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {{
        $crate::io::logger::log(
            $crate::io::logger::LogLevel::Debug,
            core::format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {{
        $crate::io::logger::log(
            $crate::io::logger::LogLevel::Info,
            core::format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {{
        $crate::io::logger::log(
            $crate::io::logger::LogLevel::Warn,
            core::format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {{
        $crate::io::logger::log(
            $crate::io::logger::LogLevel::Error,
            core::format_args!($($arg)*)
        );
    }};
}
