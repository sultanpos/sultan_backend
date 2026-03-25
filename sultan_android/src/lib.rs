use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global tokio runtime for the server.
static RUNTIME: Lazy<Mutex<Option<tokio::runtime::Runtime>>> = Lazy::new(|| Mutex::new(None));

/// Global shutdown sender to signal the server to stop.
static SHUTDOWN_TX: Lazy<Mutex<Option<tokio::sync::oneshot::Sender<()>>>> =
    Lazy::new(|| Mutex::new(None));

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

    // If already running, return false
    {
        let rt_lock = RUNTIME.lock().unwrap();
        if rt_lock.is_some() {
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

    // Set environment variables for the server configuration.
    // Safety: no other threads are reading env vars at this point; called once before
    // the tokio runtime (which may spawn threads) is started.
    unsafe {
        std::env::set_var("DATABASE_URL", format!("sqlite://{}", db_path));
        std::env::set_var("JWT_SECRET", &jwt_secret);
        std::env::set_var("WRITE_LOG_TO_FILE", "0");
    }

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Store shutdown sender
    {
        let mut tx_lock = SHUTDOWN_TX.lock().unwrap();
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
        match sultan::server::create_app().await {
            Ok(app) => {
                let addr = format!("0.0.0.0:{}", port);
                match tokio::net::TcpListener::bind(&addr).await {
                    Ok(listener) => {
                        let actual_addr = listener.local_addr().unwrap();
                        log::info!("Sultan server listening on {}", actual_addr);
                        // Spawn server in background task
                        tokio::spawn(async move {
                            axum::serve(listener, app)
                                .with_graceful_shutdown(async {
                                    shutdown_rx.await.ok();
                                    log::info!("Sultan server shutting down");
                                })
                                .await
                                .ok();
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
        let mut rt_lock = RUNTIME.lock().unwrap();
        *rt_lock = Some(rt);
        log::info!("Sultan server started on port {}", port);
        JNI_TRUE
    } else {
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
        let mut tx_lock = SHUTDOWN_TX.lock().unwrap();
        if let Some(tx) = tx_lock.take() {
            let _ = tx.send(());
        }
    }

    // Shut down the runtime
    let mut rt_lock = RUNTIME.lock().unwrap();
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
    let rt_lock = RUNTIME.lock().unwrap();
    if rt_lock.is_some() {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
