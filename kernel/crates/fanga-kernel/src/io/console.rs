use core::fmt;
use spin::Mutex;

/// Console abstraction that can write to multiple outputs
pub struct Console {
    write_serial: bool,
    write_framebuffer: bool,
}

impl Console {
    pub const fn new() -> Self {
        Self {
            write_serial: true,
            write_framebuffer: true,
        }
    }

    pub fn set_serial_enabled(&mut self, enabled: bool) {
        self.write_serial = enabled;
    }

    pub fn set_framebuffer_enabled(&mut self, enabled: bool) {
        self.write_framebuffer = enabled;
    }

    pub fn write_str(&mut self, s: &str) {
        if self.write_serial {
            use core::fmt::Write;
            let _ = fanga_arch_x86_64::serial::_print(core::format_args!("{}", s));
        }
        
        if self.write_framebuffer {
            super::framebuffer::_print(core::format_args!("{}", s));
        }
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        if self.write_serial {
            fanga_arch_x86_64::serial::_print(args);
        }
        
        if self.write_framebuffer {
            super::framebuffer::_print(args);
        }
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Global console
static CONSOLE: Mutex<Console> = Mutex::new(Console::new());

/// Get access to the console
pub fn console() -> spin::MutexGuard<'static, Console> {
    CONSOLE.lock()
}

/// Print to the console (both serial and framebuffer)
pub fn _print(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args);
}

#[macro_export]
macro_rules! console_print {
    ($($arg:tt)*) => {{
        $crate::io::console::_print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! console_println {
    () => {$crate::console_print!("\n")};
    ($fmt:expr) => {$crate::console_print!(concat!($fmt, "\n"))};
    ($fmt:expr, $($arg:tt)*) => {$crate::console_print!(concat!($fmt, "\n"), $($arg)*)};
}
