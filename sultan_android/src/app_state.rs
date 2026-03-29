use axum::Router;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use tokio::runtime::Runtime;

/// Server mode holds the channel to signal graceful shutdown.
pub enum Mode {
    Server {
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
    },
    Direct,
}

/// Combined state for either run mode.
/// The `router` already has `AppState` baked in via `with_state()`, so it is
/// the single source of truth for both serving over TCP and in-process calls.
pub struct App {
    pub rt: Runtime,
    pub router: Router,
    pub mode: Mode,
}

/// Global application instance — `None` means not initialized.
pub static APP: Lazy<Mutex<Option<App>>> = Lazy::new(|| Mutex::new(None));

/// Guard flag to prevent concurrent start/init calls.
pub static STARTING: AtomicBool = AtomicBool::new(false);
