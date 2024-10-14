use std::{fs::File, time::Duration};

use color_eyre::{eyre::Context, Result};
use tracing::Level;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::EnvFilter;

// Re-export the `tracing` macros so that they can be used in other modules
// without needing to import them.

pub use tracing::{debug, error, info, instrument, trace, warn};

/// Initialize the tracing subscriber to log to a file
///
/// This function initializes the tracing subscriber to log to a file named `tracing.log` in the
/// current directory. The function returns a [`WorkerGuard`] that must be kept alive for the
/// duration of the program to ensure that logs are flushed to the file on shutdown. The logs are
/// written in a non-blocking fashion to ensure that the logs do not block the main thread.
pub fn init_tracing() -> Result<WorkerGuard> {
    let file = File::create("tracing.log").wrap_err("failed to create tracing.log")?;
    let (non_blocking, guard) = non_blocking(file);

    // By default, the subscriber is configured to log all events with a level of `DEBUG` or higher,
    // but this can be changed by setting the `RUST_LOG` environment variable.
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::DEBUG.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(env_filter)
        .init();

    Ok(guard)
}
