mod app_state;

use app_state::{APP, App, Mode, STARTING};
use axum::body::Body;
use axum::http::Request;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jstring};
use std::sync::atomic::Ordering;
use sultan::config::AppConfig;
use tower::ServiceExt;

// ============================================================================
// Shared helpers
// ============================================================================

fn make_config(db_path: String, jwt_secret: String) -> AppConfig {
    AppConfig {
        database_url: format!("sqlite://{}", db_path),
        jwt_secret,
        write_log_to_file: false,
        access_token_ttl: time::Duration::seconds(900),
        refresh_token_ttl: time::Duration::days(365),
        database_max_connections: 5,
    }
}

fn get_jstring(env: &mut JNIEnv, s: &JString) -> Option<String> {
    env.get_string(s).ok().map(|s| s.into())
}

fn make_error_string(env: &mut JNIEnv, msg: &str) -> jstring {
    let json = serde_json::json!({ "error": msg }).to_string();
    env.new_string(json)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

// ============================================================================
// Server Mode — start / stop / isRunning
// ============================================================================

/// JNI: com.lekapin.sultan.SultanServer.start(dbPath, jwtSecret, port)
///
/// Starts the Sultan REST API server in a background Tokio runtime.
/// Returns `true` on success, `false` if already running or an error occurs.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_start(
    mut env: JNIEnv,
    _class: JClass,
    db_path: JString,
    jwt_secret: JString,
    port: jint,
) -> jboolean {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("sultan"),
    );

    if STARTING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        log::warn!("Sultan server is already starting or running");
        return JNI_FALSE;
    }

    {
        let lock = APP.lock().unwrap_or_else(|e| e.into_inner());
        if lock.is_some() {
            STARTING.store(false, Ordering::SeqCst);
            log::warn!("Sultan server is already running");
            return JNI_FALSE;
        }
    }

    let db_path = match get_jstring(&mut env, &db_path) {
        Some(s) => s,
        None => {
            log::error!("Failed to get db_path");
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };
    let jwt_secret = match get_jstring(&mut env, &jwt_secret) {
        Some(s) => s,
        None => {
            log::error!("Failed to get jwt_secret");
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };
    let port = port as u16;
    let config = make_config(db_path, jwt_secret);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to create tokio runtime: {:?}", e);
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };

    let result = rt.block_on(async move {
        let app_state = sultan::server::create_app_state(&config).await?;
        let router = sultan::server::build_router(app_state)?;
        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        if let Ok(actual_addr) = listener.local_addr() {
            log::info!("Sultan server listening on {}", actual_addr);
        }
        let serve_router = router.clone();
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, serve_router)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                    log::info!("Sultan server shutting down");
                })
                .await
            {
                log::error!("Sultan server stopped with error: {:?}", e);
            }
        });
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(router)
    });

    match result {
        Ok(router) => {
            let mut lock = APP.lock().unwrap_or_else(|e| e.into_inner());
            *lock = Some(App {
                rt,
                router,
                mode: Mode::Server { shutdown_tx },
            });
            STARTING.store(false, Ordering::SeqCst);
            log::info!("Sultan server started on port {}", port);
            JNI_TRUE
        }
        Err(e) => {
            log::error!("Failed to start Sultan server: {:?}", e);
            STARTING.store(false, Ordering::SeqCst);
            JNI_FALSE
        }
    }
}

/// JNI: com.lekapin.sultan.SultanServer.stop()
///
/// Gracefully stops the server and shuts down the runtime.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_stop(_env: JNIEnv, _class: JClass) {
    let mut lock = APP.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(app) = lock.take() {
        if let Mode::Server { shutdown_tx } = app.mode {
            let _ = shutdown_tx.send(());
        }
        app.rt.shutdown_background();
        log::info!("Sultan server stopped");
    }
}

/// JNI: com.lekapin.sultan.SultanServer.isRunning()
///
/// Returns `true` if the server is currently running in server mode.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_isRunning(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let lock = APP.lock().unwrap_or_else(|e| e.into_inner());
    if matches!(lock.as_ref().map(|a| &a.mode), Some(Mode::Server { .. })) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

// ============================================================================
// Direct Call Mode — init / call
// ============================================================================

/// JNI: com.lekapin.sultan.SultanServer.init(dbPath, jwtSecret)
///
/// Initialises Sultan in **direct call mode** — no TCP server is started.
/// After this succeeds, use `call()` to invoke endpoints directly in-process.
///
/// Returns `true` on success, `false` if already initialised or an error occurs.
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_init(
    mut env: JNIEnv,
    _class: JClass,
    db_path: JString,
    jwt_secret: JString,
) -> jboolean {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("sultan"),
    );

    if STARTING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        log::warn!("Sultan already initialising");
        return JNI_FALSE;
    }

    {
        let lock = APP.lock().unwrap_or_else(|e| e.into_inner());
        if lock.is_some() {
            STARTING.store(false, Ordering::SeqCst);
            log::warn!("Sultan already initialised");
            return JNI_FALSE;
        }
    }

    let db_path = match get_jstring(&mut env, &db_path) {
        Some(s) => s,
        None => {
            log::error!("Failed to get db_path");
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };
    let jwt_secret = match get_jstring(&mut env, &jwt_secret) {
        Some(s) => s,
        None => {
            log::error!("Failed to get jwt_secret");
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };
    let config = make_config(db_path, jwt_secret);

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("Failed to create tokio runtime: {:?}", e);
            STARTING.store(false, Ordering::SeqCst);
            return JNI_FALSE;
        }
    };

    let result = rt.block_on(async move {
        let app_state = sultan::server::create_app_state(&config).await?;
        sultan::server::build_router(app_state)
    });

    match result {
        Ok(router) => {
            let mut lock = APP.lock().unwrap_or_else(|e| e.into_inner());
            *lock = Some(App {
                rt,
                router,
                mode: Mode::Direct,
            });
            STARTING.store(false, Ordering::SeqCst);
            log::info!("Sultan initialised in direct call mode");
            JNI_TRUE
        }
        Err(e) => {
            log::error!("Failed to init Sultan: {:?}", e);
            STARTING.store(false, Ordering::SeqCst);
            JNI_FALSE
        }
    }
}

/// JNI: com.lekapin.sultan.SultanServer.call(method, path, token, body)
///
/// Calls a Sultan endpoint directly in-process — identical code path to HTTP.
///
/// Parameters:
/// - `method` — HTTP method: `"GET"`, `"POST"`, `"PATCH"`, `"PUT"`, `"DELETE"`
/// - `path`   — Full API path, e.g. `"/api/branch"`, `"/api/product/123456789"`
/// - `token`  — Raw Bearer token (without `"Bearer "` prefix). Pass `""` for public endpoints.
/// - `body`   — JSON request body as a string. Pass `"{}"` when no body is needed.
///
/// Returns the raw response body string exactly as the HTTP server would produce it.
/// The HTTP status code is encoded in the body for errors (which already contain `{"error": "..."}`).
#[unsafe(no_mangle)]
pub extern "C" fn Java_com_lekapin_sultan_SultanServer_call(
    mut env: JNIEnv,
    _class: JClass,
    method: JString,
    path: JString,
    token: JString,
    body: JString,
) -> jstring {
    let method = match get_jstring(&mut env, &method) {
        Some(s) => s,
        None => return make_error_string(&mut env, "invalid method parameter"),
    };
    let path = match get_jstring(&mut env, &path) {
        Some(s) => s,
        None => return make_error_string(&mut env, "invalid path parameter"),
    };
    let token = match get_jstring(&mut env, &token) {
        Some(s) => s,
        None => return make_error_string(&mut env, "invalid token parameter"),
    };
    let body_str = match get_jstring(&mut env, &body) {
        Some(s) => s,
        None => return make_error_string(&mut env, "invalid body parameter"),
    };

    // Clone the router + rt handle before releasing the lock.
    let (rt_handle, router) = {
        let lock = APP.lock().unwrap_or_else(|e| e.into_inner());
        match lock.as_ref() {
            Some(app) => (app.rt.handle().clone(), app.router.clone()),
            None => {
                return make_error_string(&mut env, "Sultan not initialised — call init() first");
            }
        }
    };

    let response_body = rt_handle.block_on(async move {
        let mut builder = Request::builder()
            .method(method.as_str())
            .uri(path.as_str())
            .header("content-type", "application/json");

        if !token.is_empty() {
            builder = builder.header("authorization", format!("Bearer {}", token));
        }

        let request = match builder.body(Body::from(body_str)) {
            Ok(r) => r,
            Err(e) => return format!("{{\"error\": \"failed to build request: {}\"}}", e),
        };

        let response = match router.oneshot(request).await {
            Ok(r) => r,
            Err(e) => return format!("{{\"error\": \"internal error: {}\"}}", e),
        };

        let bytes = match axum::body::to_bytes(response.into_body(), 10 * 1024 * 1024).await {
            Ok(b) => b,
            Err(e) => return format!("{{\"error\": \"failed to read response: {}\"}}", e),
        };

        String::from_utf8_lossy(&bytes).into_owned()
    });

    match env.new_string(response_body) {
        Ok(s) => s.into_raw(),
        Err(_) => make_error_string(&mut env, "failed to create response string"),
    }
}
