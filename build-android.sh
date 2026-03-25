#!/usr/bin/env bash
# =============================================================================
# Sultan Backend — Android JNI Library Build Script
# =============================================================================
#
# Builds libsultan_android.so for Android targets using the Android NDK.
# The .so files are placed under jniLibs/ ready to be copied into an
# Android Studio project.
#
# Usage:
#   ./build-android.sh                    # Build for aarch64 (most common)
#   ./build-android.sh aarch64            # Build for aarch64 (ARM 64-bit)
#   ./build-android.sh armv7              # Build for armv7 (ARM 32-bit)
#   ./build-android.sh x86_64            # Build for x86_64 (emulator/ChromeOS)
#   ./build-android.sh all               # Build for all targets
#   RELEASE=0 ./build-android.sh         # Debug build
#   OUTPUT_DIR=../MyApp/app/src/main/jniLibs ./build-android.sh all
#
# Prerequisites:
#   - Android NDK installed (via Android Studio or standalone)
#   - Set ANDROID_NDK_HOME environment variable
#   - Rust targets installed (script installs them automatically)
#
# Output:
#   jniLibs/
#     arm64-v8a/libsultan_android.so
#     armeabi-v7a/libsultan_android.so
#     x86_64/libsultan_android.so
#
# =============================================================================

set -euo pipefail

# ---- Configuration ----------------------------------------------------------

# Android API level (minimum supported Android version)
# 24 = Android 7.0 (good baseline for modern devices)
API_LEVEL="${API_LEVEL:-24}"

# Release or debug build
RELEASE="${RELEASE:-1}"

# Output directory for the JNI libraries
OUTPUT_DIR="${OUTPUT_DIR:-$(pwd)/jniLibs}"

# Android NDK path
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
    # Try common locations
    if [[ -d "${HOME}/Android/Sdk/ndk" ]]; then
        # Pick the latest installed NDK version
        ANDROID_NDK_HOME=$(find "${HOME}/Android/Sdk/ndk" -maxdepth 1 -mindepth 1 -type d | sort -V | tail -1)
    elif [[ -d "${ANDROID_HOME:-}/ndk" ]]; then
        ANDROID_NDK_HOME=$(find "${ANDROID_HOME}/ndk" -maxdepth 1 -mindepth 1 -type d | sort -V | tail -1)
    fi

    if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
        echo "ERROR: ANDROID_NDK_HOME is not set and NDK was not found in common locations."
        echo ""
        echo "Set it to your NDK installation path, e.g.:"
        echo "  export ANDROID_NDK_HOME=\$HOME/Android/Sdk/ndk/27.0.12077973"
        exit 1
    fi
fi

echo "Using Android NDK: ${ANDROID_NDK_HOME}"

# Detect host OS for the NDK toolchain
case "$(uname -s)" in
    Linux*)  HOST_TAG="linux-x86_64" ;;
    Darwin*) HOST_TAG="darwin-x86_64" ;;
    *)       echo "ERROR: Unsupported host OS: $(uname -s)"; exit 1 ;;
esac

TOOLCHAIN="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/${HOST_TAG}"

if [[ ! -d "${TOOLCHAIN}" ]]; then
    echo "ERROR: NDK toolchain not found at ${TOOLCHAIN}"
    echo "Make sure ANDROID_NDK_HOME points to a valid NDK installation."
    exit 1
fi

# ---- Target Mapping ---------------------------------------------------------

# Rust target triple
declare -A TARGET_TRIPLE
TARGET_TRIPLE[aarch64]="aarch64-linux-android"
TARGET_TRIPLE[armv7]="armv7-linux-androideabi"
TARGET_TRIPLE[x86_64]="x86_64-linux-android"

# NDK clang prefix for each target
declare -A CC_PREFIX
CC_PREFIX[aarch64]="aarch64-linux-android"
CC_PREFIX[armv7]="armv7a-linux-androideabi"
CC_PREFIX[x86_64]="x86_64-linux-android"

# Android JNI ABI directory name for each target
declare -A ABI_DIR
ABI_DIR[aarch64]="arm64-v8a"
ABI_DIR[armv7]="armeabi-v7a"
ABI_DIR[x86_64]="x86_64"

# ---- Functions --------------------------------------------------------------

build_target() {
    local arch="$1"
    local triple="${TARGET_TRIPLE[$arch]}"
    local cc_prefix="${CC_PREFIX[$arch]}"
    local abi_dir="${ABI_DIR[$arch]}"

    echo ""
    echo "============================================="
    echo "  Building for ${arch} (${triple})"
    echo "  ABI dir:   ${abi_dir}"
    echo "  API Level: ${API_LEVEL}"
    echo "============================================="

    # Check if the Rust target is installed
    if ! rustup target list --installed | grep -q "${triple}"; then
        echo "Rust target '${triple}' is not installed. Installing..."
        rustup target add "${triple}"
    fi

    # Set up cross-compilation environment variables
    local cc="${TOOLCHAIN}/bin/${cc_prefix}${API_LEVEL}-clang"
    local ar="${TOOLCHAIN}/bin/llvm-ar"

    if [[ ! -f "${cc}" ]]; then
        echo "ERROR: C compiler not found at ${cc}"
        exit 1
    fi

    # Cargo uses env vars with target triple in uppercase with underscores
    local env_triple
    env_triple=$(echo "${triple}" | tr '[:lower:]-' '[:upper:]_')

    local build_args=(
        --package sultan_android
        --target "${triple}"
    )

    if [[ "${RELEASE}" == "1" ]]; then
        build_args+=(--release)
    fi

    # Build
    env \
        CC="${cc}" \
        AR="${ar}" \
        "CARGO_TARGET_${env_triple}_LINKER=${cc}" \
        cargo build "${build_args[@]}"

    # Locate the produced .so
    local profile="debug"
    if [[ "${RELEASE}" == "1" ]]; then
        profile="release"
    fi
    local so_src="target/${triple}/${profile}/libsultan_android.so"

    if [[ ! -f "${so_src}" ]]; then
        echo "ERROR: Expected .so not found at ${so_src}"
        exit 1
    fi

    # Copy to jniLibs/<abi>/
    local dest_dir="${OUTPUT_DIR}/${abi_dir}"
    mkdir -p "${dest_dir}"
    cp "${so_src}" "${dest_dir}/libsultan_android.so"

    # Strip remaining symbols using the NDK's llvm-strip
    local strip_tool="${TOOLCHAIN}/bin/llvm-strip"
    if [[ -f "${strip_tool}" ]]; then
        "${strip_tool}" --strip-unneeded "${dest_dir}/libsultan_android.so"
    fi

    local size
    size=$(du -h "${dest_dir}/libsultan_android.so" | cut -f1)
    echo ""
    echo "  Build successful!"
    echo "  Output: ${dest_dir}/libsultan_android.so  (${size})"
}

# ---- Main -------------------------------------------------------------------

ARCH="${1:-aarch64}"

case "${ARCH}" in
    aarch64|armv7|x86_64)
        build_target "${ARCH}"
        ;;
    all)
        for arch in aarch64 armv7 x86_64; do
            build_target "${arch}"
        done
        ;;
    *)
        echo "ERROR: Unknown target '${ARCH}'"
        echo ""
        echo "Available targets:"
        echo "  aarch64  - ARM 64-bit (most Android phones)  → jniLibs/arm64-v8a/"
        echo "  armv7    - ARM 32-bit (older Android phones) → jniLibs/armeabi-v7a/"
        echo "  x86_64   - x86 64-bit (emulators, ChromeOS) → jniLibs/x86_64/"
        echo "  all      - Build for all targets"
        exit 1
        ;;
esac

echo ""
echo "Done. JNI libraries are in: ${OUTPUT_DIR}"
echo ""
echo "Next step: copy the jniLibs/ directory into your Android project:"
echo "  app/src/main/jniLibs/"
