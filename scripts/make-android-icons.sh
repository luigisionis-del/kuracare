#!/usr/bin/env bash
# Builds the Android launcher icons from assets/icon.svg (needs rsvg-convert and ImageMagick).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
RES="$ROOT/crates/kuracare-app/android/res"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# Foreground of the adaptive icon: the heart only, on transparent, inside the 72/108 safe zone.
cat > "$TMP/fg.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <g transform="translate(256 256) scale(0.62) translate(-256 -268)">
    <path d="M256 392c-74-46-124-96-124-156 0-40 30-70 66-70 26 0 46 13 58 33 12-20 32-33 58-33 36 0 66 30 66 70 0 60-50 110-124 156z" fill="#f6f1ea"/>
    <path d="M256 330c0-62 26-104 74-128-20 44-34 86-74 128z" fill="#8cc4ab"/>
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
