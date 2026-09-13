#!/usr/bin/env bash
# Build the Shifra Android app (com.shifra.language) as a signed APK.
#
# The APK ships the interpreter as lib/<abi>/libshifra.so (extractNativeLibs)
# because Android forbids execve from app-private data dirs.
#
# Usage:
#   ./build.sh                      build (reusing existing release binaries)
#   RUST_BUILD=1 ./build.sh         also cross-compile the interpreter first
#
# Environment (all optional):
#   ANDROID_HOME          Android SDK root         (default: /home/amr/dev/Android/SDK)
#   BUILD_TOOLS           build-tools version dir  (default: 36.1.0)
#   PLATFORM              android platform jar     (default: android-35)
#   RUST_TARGET_DIR       cargo target dir         (default: <repo>/target)
#   VER                   app version label        (default: 0.3.7)
#   NDK_ROOT, LIBC_DIRS   only used with RUST_BUILD=1 (see below)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_dir="$repo_root/android"

ANDROID_HOME="${ANDROID_HOME:-}"
if [[ -z "$ANDROID_HOME" ]]; then
    ANDROID_HOME=/home/amr/dev/Android/SDK
fi
if [[ ! -d "$ANDROID_HOME/build-tools" ]]; then
    if [[ -d /home/amr/dev/Android/SDK/build-tools ]]; then
        ANDROID_HOME=/home/amr/dev/Android/SDK
    elif [[ -d /home/amr/Android/SDK/build-tools ]]; then
        ANDROID_HOME=/home/amr/Android/SDK
    fi
fi
BUILD_TOOLS="${BUILD_TOOLS:-36.1.0}"
PLATFORM="${PLATFORM:-android-35}"
RUST_TARGET_DIR="${RUST_TARGET_DIR:-$repo_root/target}"
VER="${VER:-0.3.7}"

aapt2="$ANDROID_HOME/build-tools/$BUILD_TOOLS/aapt2"
zipalign="$ANDROID_HOME/build-tools/$BUILD_TOOLS/zipalign"
apksigner="$ANDROID_HOME/build-tools/$BUILD_TOOLS/apksigner"
d8="$ANDROID_HOME/cmdline-tools/latest/bin/d8"
android_jar="$ANDROID_HOME/platforms/$PLATFORM/android.jar"

abi_arm=arm64-v8a
abi_x86=x86_64
rust_arm=aarch64-linux-android
rust_x86=x86_64-linux-android

bin_arm="$RUST_TARGET_DIR/$rust_arm/release/rustpython"
bin_x86="$RUST_TARGET_DIR/$rust_x86/release/rustpython"

mkdir -p "$app_dir/dist"

if [[ "${RUST_BUILD:-0}" == "1" ]]; then
    echo "== cross-compiling interpreter =="
    cargo build --release --target "$rust_arm" \
        --bin rustpython --no-default-features \
        --features stdlib,stdio,threading,importlib,freeze-stdlib
    cargo build --release --target "$rust_x86" \
        --bin rustpython --no-default-features \
        --features stdlib,stdio,threading,importlib,freeze-stdlib
fi

for f in "$bin_arm" "$bin_x86" "$android_jar"; do
    if [[ ! -f "$f" ]]; then
        echo "missing: $f" >&2
        echo "build the Android interpreters first (see AGENTS.md) or set RUST_BUILD=1" >&2
        exit 1
    fi
done

out="$app_dir/dist/shifra-$VER.apk"
rm -rf "$app_dir/build"
mkdir -p "$app_dir/build/res" "$app_dir/build/classes" "$app_dir/build/dex" "$app_dir/build/work" "$app_dir/build/gen"

echo "== aapt2 compile/link =="
"$aapt2" compile --dir "$app_dir/res" -o "$app_dir/build/res/res.zip"
"$aapt2" link -o "$app_dir/build/work/app.apk" -I "$android_jar" \
    --manifest "$app_dir/AndroidManifest.xml" \
    -R "$app_dir/build/res/res.zip" --auto-add-overlay --java "$app_dir/build/gen"

echo "== javac =="
if [[ -z "${JAVA_HOME:-}" ]]; then
    if command -v javac >/dev/null 2>&1; then
        JAVA_HOME="$(dirname "$(dirname "$(command -v javac)")")"
    elif [[ -d /usr/lib/jvm/jdk-24.0.2-oracle-x64 ]]; then
        JAVA_HOME=/usr/lib/jvm/jdk-24.0.2-oracle-x64
    else
        echo "javac not found; set JAVA_HOME" >&2
        exit 1
    fi
fi
"$JAVA_HOME/bin/javac" -source 8 -target 8 -cp "$android_jar" \
    -d "$app_dir/build/classes" \
    "$app_dir/build/gen/com/shifra/language/R.java" \
    $(find "$app_dir/src" -name '*.java')

echo "== d8 =="
"$d8" --lib "$android_jar" --output "$app_dir/build/dex" \
    $(find "$app_dir/build/classes" -name '*.class')

echo "== packaging =="
w="$app_dir/build/work"
cp "$app_dir/build/dex/classes.dex" "$w/"
mkdir -p "$w/lib/$abi_arm" "$w/lib/$abi_x86" "$w/assets"
cp "$bin_arm" "$w/lib/$abi_arm/libshifra.so"
cp "$bin_x86" "$w/lib/$abi_x86/libshifra.so"
cp "$app_dir/assets/demo.sf" "$w/assets/"
(cd "$w" && zip -q -r app.apk classes.dex lib assets)

"$zipalign" -f 4 "$w/app.apk" "$w/app.aligned.apk"
"$apksigner" sign --ks "$app_dir/keystore/debug.keystore" \
    --ks-pass pass:android --ks-key-alias androiddebugkey \
    --key-pass pass:android --out "$out" "$w/app.aligned.apk"

echo "== done: $out =="
ls -la "$out"