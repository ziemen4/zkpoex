//! A lightweight logging facade. Downstream crates can use `shared::log_*!` macros without
//! depending directly on the `tracing` crate.

// We rely on the external `tracing` crate being a dependency of the shared crate.
// Macros use absolute paths so downstream crates need no direct tracing dependency.

/// Log at INFO level
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::tracing::info!($($arg)*);
    };
}

/// Log at DEBUG level
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::tracing::debug!($($arg)*);
    };
}

/// Log at WARN level
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::tracing::warn!($($arg)*);
    };
}

/// Log at ERROR level
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::tracing::error!($($arg)*);
    };
}
