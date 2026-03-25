use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use sultan::config::AppConfig;

/// Global tokio runtime for the server.
static RUNTIME: Lazy<Mutex<Option<tokio::runtime::Runtime>>> = Lazy::new(|| Mutex::new(None));

/// Global shutdown sender to signal the server to stop.
static SHUTDOWN_TX: Lazy<Mutex<Option<tokio::sync::oneshot::Sender<()>>>> =
    Lazy::new(|| Mutex::new(None));

/// Guard flag set to `true` while start() is in progress.
/// Prevents two concurrent start() calls from both passing the "already running"
/// check before RUNTIME is populated.
static STARTING: AtomicBool = AtomicBool::new(false);

/// JNI: com.lekapin.sultan.SultanServer.start(dbPath, jwtSecret, port)
///
/// Starts the Sultan server in a background Tokio runtime.
///
/// Parameters:
///   db_path   - Absolute path to the SQLite database file (e.g. /data/data/com.myapp/files/sultan.db)
///   jwt_secret - Secret key for JWT signing
///   port      - TCP port to listen on (e.g. 8721)
///
/// Returns true on success, false on failure.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_start(
    mut env: JNIEnv,
    _class: JClass,
    db_path: JString,
    jwt_secret: JString,
    port: jint,
) -> jboolean {
    // Init Android logger (safe to call multiple times)
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("sultan"),
    );

    // Atomically claim the "starting" slot. If another thread is already inside
    // start() (STARTING == true) or has just set RUNTIME, we bail out immediately.
    // This ensures no two callers can both pass the RUNTIME check before it is set.
    if STARTING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        log::warn!("Sultan server is already starting or running");
        return JNI_FALSE;
    }

    // Secondary check: RUNTIME may already be Some if a previous start completed
    // just before we claimed STARTING (stop() clears RUNTIME but also lets
    // STARTING fall back to false, so this path covers rapid stop+start races).
    {
        let rt_lock = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
        if rt_lock.is_some() {
            STARTING.store(false, Ordering::SeqCst);
            log::warn!("Sultan server is already running");
            return JNI_FALSE;
        }
    }

    // Extract Java strings
    let db_path: String = match env.get_string(&db_path) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get db_path: {:?}", e);
            return JNI_FALSE;
        }
    };

    let jwt_secret: String = match env.get_string(&jwt_secret) {
        Ok(s) => s.into(),
        Err(e) => {
            log::error!("Failed to get jwt_secret: {:?}", e);
            return JNI_FALSE;
        }
    };

    let port = port as u16;

    // Build config directly — no env var mutation needed.
    let config = AppConfig {
        database_url: format!("sqlite://{}", db_path),
        jwt_secret,
        write_log_to_file: false,
        access_token_ttl: time::Duration::seconds(900),
        refresh_token_ttl: time::Duration::days(365),
        database_max_connections: 5,
    };

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Store shutdown sender
    {
        let mut tx_lock = SHUTDOWN_TX.lock().unwrap_or_else(|e| e.into_inner());
        *tx_lock = Some(shutdown_tx);
    }

    // Build and start the tokio runtime
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to create tokio runtime: {:?}", e);
            return JNI_FALSE;
        }
    };

    let started = rt.block_on(async move {
        match sultan::server::create_app_with_config(config).await {
            Ok(app) => {
                let addr = format!("0.0.0.0:{}", port);
                match tokio::net::TcpListener::bind(&addr).await {
                    Ok(listener) => {
                        match listener.local_addr() {
                            Ok(actual_addr) => {
                                log::info!("Sultan server listening on {}", actual_addr)
                            }
                            Err(e) => log::warn!(
                                "Sultan server started but could not get local addr: {:?}",
                                e
                            ),
                        };
                        // Spawn server in background task
                        tokio::spawn(async move {
                            if let Err(e) = axum::serve(listener, app)
                                .with_graceful_shutdown(async {
                                    shutdown_rx.await.ok();
                                    log::info!("Sultan server shutting down");
                                })
                                .await
                            {
                                log::error!("Sultan server stopped with error: {:?}", e);
                            } else {
                                log::info!("Sultan server stopped cleanly");
                            }
                        });
                        true
                    }
                    Err(e) => {
                        log::error!("Failed to bind to {}: {:?}", addr, e);
                        false
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create app: {:?}", e);
                false
            }
        }
    });

    if started {
        let mut rt_lock = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
        *rt_lock = Some(rt);
        // Release the starting guard only after RUNTIME is populated so that any
        // concurrent start() caller that lost the compare_exchange sees a consistent
        // state when it retries (it will find RUNTIME is Some).
        STARTING.store(false, Ordering::SeqCst);
        log::info!("Sultan server started on port {}", port);
        JNI_TRUE
    } else {
        // Also release the guard on failure so a future start() attempt is possible.
        STARTING.store(false, Ordering::SeqCst);
        JNI_FALSE
    }
}

/// JNI: com.lekapin.sultan.SultanServer.stop()
///
/// Gracefully stops the Sultan server.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_stop(_env: JNIEnv, _class: JClass) {
    // Send shutdown signal
    {
        let mut tx_lock = SHUTDOWN_TX.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = tx_lock.take() {
            let _ = tx.send(());
        }
    }

    // Shut down the runtime
    let mut rt_lock = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(rt) = rt_lock.take() {
        rt.shutdown_background();
        log::info!("Sultan server stopped");
    }
}

/// JNI: com.lekapin.sultan.SultanServer.isRunning()
///
/// Returns true if the server is currently running.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_isRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let rt_lock = RUNTIME.lock().unwrap_or_else(|e| e.into_inner());
    if rt_lock.is_some() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
