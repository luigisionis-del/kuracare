#!/usr/bin/env bash
# Builds the Android launcher icons from assets/icon.svg (needs rsvg-convert and ImageMagick).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
RES="$ROOT/crates/kuracare-app/android/res"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Foreground of the adaptive icon: the heart and the three saffron threads, on transparent, inside the 72/108 safe zone.
cat > "$TMP/fg.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <g transform="translate(256 256) scale(0.62) translate(-256 -262)">
    <path d="M256 392c-74-46-124-96-124-156 0-40 30-70 66-70 26 0 46 13 58 33 12-20 32-33 58-33 36 0 66 30 66 70 0 60-50 110-124 156z" fill="#fbf6ee"/>
    <g fill="none" stroke="#e8a317" stroke-width="13" stroke-linecap="round"><path d="M256 318C248 262 222 206 192 166"/><path d="M256 318C256 254 256 196 256 140"/><path d="M256 318C264 262 290 206 320 166"/></g><g fill="#e8a317"><ellipse cx="188" cy="158" rx="15" ry="20" transform="rotate(-34 188 158)"/><ellipse cx="256" cy="130" rx="15" ry="20"/><ellipse cx="324" cy="158" rx="15" ry="20" transform="rotate(34 324 158)"/></g>
  </g>
</svg>
SVG

for d in mdpi:48:108 hdpi:72:162 xhdpi:96:216 xxhdpi:144:324 xxxhdpi:192:432; do
  IFS=: read -r name legacy fg <<<"$d"
  mkdir -p "$RES/mipmap-$name"
  rsvg-convert -w "$legacy" -h "$legacy" "$ROOT/assets/icon.svg" -o "$TMP/l.png"
  magick "$TMP/l.png" -define webp:lossless=true "$RES/mipmap-$name/ic_launcher.webp"
  rsvg-convert -w "$fg" -h "$fg" "$TMP/fg.svg" -o "$TMP/f.png"
  magick "$TMP/f.png" -define webp:lossless=true "$RES/mipmap-$name/ic_launcher_foreground.webp"
done
echo "Icone scritte in $RES"
