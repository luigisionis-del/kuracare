#!/usr/bin/env bash
# Builds the signed release APK of Kuracare.
#
# - Rust compiled in release mode by dx;
# - Gradle "release" build (not debuggable, minified) with our own manifest,
#   network security config and icons (crates/kuracare-app/android/): no backups,
#   network limited to the 127.0.0.1 websocket that Dioxus needs;
# - signed with the key in ~/.config/kuracare (never in the repository).
#
# Output: target/kuracare-<version>.apk
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CONF=${KURACARE_CONF:-$HOME/.config/kuracare}
# shellcheck disable=SC1091
set -a; source "$CONF/env"; set +a   # KURACARE_KS, KURACARE_KEY_ALIAS, KURACARE_KS_PASS

export GRADLE_USER_HOME=${GRADLE_USER_HOME:-$HOME/.cache/kuracare-gradle}
export ANDROID_HOME=${ANDROID_HOME:-$HOME/.cache/fmis-android/sdk}
export ANDROID_SDK_ROOT=$ANDROID_HOME
export ANDROID_NDK_HOME=${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}
export JAVA_HOME=${JAVA_HOME:-$HOME/.cache/agrometeo-android/jdk17}
BT=${BUILD_TOOLS:-$ANDROID_HOME/build-tools/36.0.0}

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
APP="$ROOT/crates/kuracare-app"
GEN="$ROOT/target/dx/kuracare-app/release/android/app"
OUT="$ROOT/target/kuracare-$VERSION.apk"

echo "==> Rust (release) e progetto Android"
(cd "$APP" && dx build --platform android --release --target aarch64-linux-android)

echo "==> Manifest, configurazione di rete, icone, regole di backup"
cp "$APP/android/AndroidManifest.xml" "$GEN/app/src/main/AndroidManifest.xml"
rm -rf "$GEN"/app/src/main/res/mipmap-* "$GEN"/app/src/main/res/drawable*/ic_launcher_*
cp -r "$APP/android/res/." "$GEN/app/src/main/res/"

echo "==> Gradle release"
(cd "$GEN" && ./gradlew --quiet assembleRelease)
UNSIGNED="$GEN/app/build/outputs/apk/release/app-release-unsigned.apk"

echo "==> Firma"
"$BT/zipalign" -f -p 4 "$UNSIGNED" "$OUT.aligned"
"$BT/apksigner" sign --ks "$KURACARE_KS" --ks-key-alias "$KURACARE_KEY_ALIAS" \
  --ks-pass env:KURACARE_KS_PASS --key-pass env:KURACARE_KS_PASS --out "$OUT" "$OUT.aligned"
rm -f "$OUT.aligned" "$OUT.idsig"
"$BT/apksigner" verify "$OUT"

echo "==> Controlli"
PERMS=$("$BT/aapt2" dump permissions "$OUT" | sed -n "s/^uses-permission: name='\(.*\)'/\1/p" | grep -v '^org.kuracare.app\.' || true)
if [ "$PERMS" != "android.permission.INTERNET" ]; then
  echo "ERRORE: permessi inattesi: $PERMS" >&2; exit 1
fi
MANIFEST=$("$BT/aapt2" dump xmltree --file AndroidManifest.xml "$OUT")
if ! grep -q 'networkSecurityConfig' <<<"$MANIFEST" || ! grep -q 'allowBackup(0x01010280)=false' <<<"$MANIFEST"; then
  echo "ERRORE: manca la configurazione di rete o il divieto di backup" >&2; exit 1
fi
if grep -q 'application-debuggable' <<<"$("$BT/aapt2" dump badging "$OUT")"; then
  echo "ERRORE: l'APK è debuggabile" >&2; exit 1
fi
"$BT/aapt2" dump permissions "$OUT"
ls -lh "$OUT"
echo "Pronto: $OUT"
