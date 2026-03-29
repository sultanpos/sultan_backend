# Sultan Backend — Android Integration Guide

Sultan Backend can be embedded directly inside your Android app as a native JNI library (`libsultan_android.so`). The Android app loads the library and calls JNI methods to interact with the backend. No separate process or Termux is required.

Two integration modes are available:

| Mode | Description | Use when |
|---|---|---|
| **Server Mode** | Starts a Tokio runtime + Axum TCP server. App makes HTTP requests to `localhost:<port>`. | You need LAN access or want standard HTTP semantics |
| **Direct Call Mode** | Builds the Axum router in-process. Requests are dispatched via Tower `oneshot()` — no TCP socket. | App-only usage; lower overhead, no port conflicts |

## How It Works

### Server Mode

```
Android App (Kotlin/Java)
  └── System.loadLibrary("sultan_android")
  └── SultanServer.start(dbPath, jwtSecret, port)
        └── Spawns Tokio runtime in background thread
        └── Starts Axum HTTP server on 0.0.0.0:<port>
  └── App makes HTTP requests to http://localhost:<port>/api/...
  └── SultanServer.stop()  ←─ when app exits
```

### Direct Call Mode

```
Android App (Kotlin/Java)
  └── System.loadLibrary("sultan_android")
  └── SultanServer.init(dbPath, jwtSecret)
        └── Spawns Tokio runtime
        └── Builds Axum Router in-process (no TCP socket)
  └── SultanServer.call("POST", "/api/auth", "", body)  →  "{...}"
  └── SultanServer.call("GET",  "/api/branch/123", token, "")  →  "{...}"
  └── (No stop needed — process-lifetime runtime)
```

## Prerequisites

### 1. Android NDK

Install via Android Studio: **Settings → SDK Manager → SDK Tools → NDK (Side by side)**

Then set the environment variable:

```bash
export ANDROID_NDK_HOME="$HOME/Android/Sdk/ndk/27.0.12077973"  # adjust version
# Add to ~/.bashrc or ~/.zshrc for persistence
```

The build script auto-detects the NDK in `~/Android/Sdk/ndk/` if the variable is not set.

### 2. Rust Android Targets

The build script installs these automatically, or install manually:

```bash
rustup target add aarch64-linux-android    # ARM 64-bit (most modern phones)
rustup target add armv7-linux-androideabi  # ARM 32-bit (older phones)
rustup target add x86_64-linux-android     # x86_64 (emulator)
```

## Building the Library

```bash
# Build for ARM 64-bit (default, most common)
./build-android.sh

# Build for a specific architecture
./build-android.sh aarch64
./build-android.sh armv7
./build-android.sh x86_64

# Build for all architectures at once
./build-android.sh all

# Build directly into your Android project's jniLibs directory
OUTPUT_DIR=../MyAndroidApp/app/src/main/jniLibs ./build-android.sh all

# Debug build
RELEASE=0 ./build-android.sh

# Custom API level (default: 24 = Android 7.0)
API_LEVEL=28 ./build-android.sh all
```

The output is placed in the `jniLibs/` directory:

```
jniLibs/
  arm64-v8a/libsultan_android.so      ← for most modern phones
  armeabi-v7a/libsultan_android.so    ← for older/32-bit phones
  x86_64/libsultan_android.so         ← for the emulator
```

## Android Project Setup

### 1. Copy the JNI Libraries

Copy the `jniLibs/` directory into your Android project:

```
MyAndroidApp/
  app/
    src/
      main/
        jniLibs/               ← copy here
          arm64-v8a/
            libsultan_android.so
          armeabi-v7a/
            libsultan_android.so
          x86_64/
            libsultan_android.so
```

Or use the `OUTPUT_DIR` option to build directly into your project:

```bash
OUTPUT_DIR=../MyAndroidApp/app/src/main/jniLibs ./build-android.sh all
```

### 2. Create the Java/Kotlin Wrapper Class

Create the `SultanServer` class that matches the JNI function names:

**Kotlin** (`app/src/main/java/com/lekapin/sultan/SultanServer.kt`):

```kotlin
package com.lekapin.sultan

object SultanServer {

    init {
        System.loadLibrary("sultan_android")
    }

    // ── Server Mode ───────────────────────────────────────────────────────────

    /**
     * Starts Sultan as a TCP server.
     *
     * @param dbPath    Absolute path to the SQLite database file.
     *                  Use context.getDatabasePath("sultan.db").absolutePath
     * @param jwtSecret Secret key for JWT token signing.
     * @param port      TCP port to listen on (e.g. 8721).
     * @return          true if the server started successfully.
     */
    external fun start(dbPath: String, jwtSecret: String, port: Int): Boolean

    /** Gracefully stops the TCP server and shuts down the runtime. */
    external fun stop()

    /** Returns true if the server is currently running in server mode. */
    external fun isRunning(): Boolean

    // ── Direct Call Mode ──────────────────────────────────────────────────────

    /**
     * Initialises Sultan in direct call mode — no TCP server is started.
     * The Axum router is built in-process; use [call] to invoke endpoints.
     *
     * @param dbPath    Absolute path to the SQLite database file.
     * @param jwtSecret Secret key for JWT token signing.
     * @return          true on success.
     */
    external fun init(dbPath: String, jwtSecret: String): Boolean

    /**
     * Calls an API endpoint directly in-process (no TCP, no HTTP round-trip).
     *
     * @param method  HTTP method: "GET", "POST", "PATCH", "PUT", "DELETE"
     * @param path    Full path, e.g. "/api/branch", "/api/product/123456789"
     * @param token   Raw Bearer token (without "Bearer " prefix). Pass "" for public endpoints.
     * @param body    JSON request body as a string.
     *                Pass the real JSON for POST/PATCH/PUT endpoints that expect a body.
     *                Pass "{}" as a safe default for endpoints that do not read the body.
     *                Pass "" only for GET/DELETE or endpoints that never read the body.
     * @return        JSON string: {"status": <http_status_code>, "body": <response_body>}
     *                where `body` is the parsed JSON response (or a plain string when not JSON).
     *                On internal errors before an HTTP response is produced, `status` is 0.
     */
    external fun call(method: String, path: String, token: String, body: String): String
}
```

**Java** (`app/src/main/java/com/lekapin/sultan/SultanServer.java`):

```java
package com.lekapin.sultan;

public class SultanServer {

    static {
        System.loadLibrary("sultan_android");
    }

    // Server Mode
    public static native boolean start(String dbPath, String jwtSecret, int port);
    public static native void stop();
    public static native boolean isRunning();

    // Direct Call Mode
    public static native boolean init(String dbPath, String jwtSecret);
    public static native String call(String method, String path, String token, String body);
}
```

### 3. Use Sultan in Your Application

#### Option A — Server Mode

**Kotlin** (`app/src/main/java/com/myapp/MyApplication.kt`):

```kotlin
package com.myapp

import android.app.Application
import android.util.Log
import com.lekapin.sultan.SultanServer

class MyApplication : Application() {

    override fun onCreate() {
        super.onCreate()

        val dbPath = getDatabasePath("sultan.db").absolutePath
        getDatabasePath("sultan.db").parentFile?.mkdirs()

        val jwtSecret = "your-secret-key-here"  // load from secure storage in production
        val port = 8721

        val started = SultanServer.start(dbPath, jwtSecret, port)
        if (started) {
            Log.i("Sultan", "Server started on port $port")
        } else {
            Log.e("Sultan", "Failed to start server")
        }
    }

    override fun onTerminate() {
        SultanServer.stop()
        super.onTerminate()
    }
}
```

#### Option B — Direct Call Mode

```kotlin
package com.myapp

import android.app.Application
import android.util.Log
import com.lekapin.sultan.SultanServer

class MyApplication : Application() {

    override fun onCreate() {
        super.onCreate()

        val dbPath = getDatabasePath("sultan.db").absolutePath
        getDatabasePath("sultan.db").parentFile?.mkdirs()

        val jwtSecret = "your-secret-key-here"

        val ok = SultanServer.init(dbPath, jwtSecret)
        if (ok) {
            Log.i("Sultan", "Sultan initialised in direct call mode")
        } else {
            Log.e("Sultan", "Failed to initialise Sultan")
        }
    }
}
```

**Making calls** (direct call mode):

`call()` always returns a JSON envelope:
```json
{"status": 200, "body": { ... }}
```
Check `status` to distinguish success (`2xx`) from errors (`4xx`/`5xx`). A `status` of `0` means an internal error occurred before the request reached the router.

```kotlin
// Login (no token needed for public endpoints)
val loginResp = SultanServer.call(
    "POST", "/api/auth", "",
    """{"username":"sultan","password":"sultan"}"""
)
// loginResp = {"status": 200, "body": {"access_token":"...","refresh_token":"..."}}

val parsed = JSONObject(loginResp)
if (parsed.getInt("status") == 200) {
    val token = parsed.getJSONObject("body").getString("access_token")

    // Create a branch (POST body required)
    val createResp = SultanServer.call(
        "POST", "/api/branch", token,
        """{"name":"Main Branch","code":"MAIN","is_main":true}"""
    )

    // Get a branch by ID (no body; pass "" for GET)
    val getResp = SultanServer.call("GET", "/api/branch/123456789", token, "")

    // Update a branch (PATCH body required)
    val updateResp = SultanServer.call(
        "PATCH", "/api/branch/123456789", token,
        """{"name":"Updated Name"}"""
    )

    // Delete a branch (no body; pass "" for DELETE)
    val deleteResp = SultanServer.call("DELETE", "/api/branch/123456789", token, "")
}
```

Register the Application class in `AndroidManifest.xml`:

```xml
<application
    android:name=".MyApplication"
    ...>
```

### 4. Grant Internet Permission

Add to `AndroidManifest.xml` (required for other devices on the LAN to reach the server):

```xml
<uses-permission android:name="android.permission.INTERNET" />
```

### 5. Making API Requests from the App

#### Server Mode — HTTP to localhost

Once the server is running, make requests to `http://localhost:<port>`:

```kotlin
// Using OkHttp or Retrofit pointed at localhost
val client = OkHttpClient()
val request = Request.Builder()
    .url("http://localhost:8721/api/auth/login")
    .post(body)
    .build()
```

#### Direct Call Mode — in-process via `call()`

No HTTP client needed. Call `SultanServer.call()` directly. The return value is always a JSON envelope `{"status": <code>, "body": ...}`:

```kotlin
val resp = SultanServer.call("POST", "/api/auth", "", """{"username":"sultan","password":"sultan"}""")
// Returns: {"status": 200, "body": {"access_token":"...","refresh_token":"..."}}

## Architecture Reference

| Architecture | JNI ABI dir | Common devices |
|---|---|---|
| `aarch64` | `arm64-v8a` | Most modern Android phones (2016+) |
| `armv7` | `armeabi-v7a` | Older / budget Android phones |
| `x86_64` | `x86_64` | Android emulators, ChromeOS |

## Default Credentials

On first launch, Sultan automatically creates:
- **Branch**: `Sultan`
- **User**: username `sultan`, password `sultan`

Change the password after first login.

## Troubleshooting

### `java.lang.UnsatisfiedLinkError: sultan_android`

- Make sure `libsultan_android.so` is in the correct `jniLibs/<abi>/` directory.
- Confirm the package name in the Kotlin/Java class matches `com.lekapin.sultan`.
- Check that `System.loadLibrary("sultan_android")` is called (without `lib` prefix and without `.so`).

### Check which ABI your device uses

```bash
adb shell getprop ro.product.cpu.abi
# arm64-v8a   → use aarch64
# armeabi-v7a → use armv7
# x86_64      → use x86_64
```

### Server returns false / doesn’t start

- Ensure the `dbPath` parent directory exists and is writable by the app.
- Check Logcat for `sultan` tag for detailed error messages.
- Make sure the port isn't already in use (server mode only).

### `call()` returns `{"error": "Sultan not initialised — call init() first"}`

- `SultanServer.init()` was not called before `call()`, or it returned `false`.
- Check Logcat for init errors (database path, permissions, etc.).

### `call()` returns an unexpected empty string

- Pass `""` for an empty body, not `null`. The JNI bridge requires a non-null string.
- The endpoint may return a 204 No Content — check whether it is expected to have a body.

### `ANDROID_NDK_HOME is not set`

Set the variable to your NDK path. See [Prerequisites](#1-android-ndk).
