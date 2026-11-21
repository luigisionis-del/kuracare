#!/usr/bin/env bash
# Checks on the connected phone (adb) that Kuracare has never exchanged data
# with any network. Android's network statistics do not count loopback, so the
# 127.0.0.1 websocket used by Dioxus does not appear: any entry means traffic
# left the phone. Only Kuracare's lines are read; other apps are not printed.
set -euo pipefail
PKG=org.kuracare.app
UID_APP=$(adb shell dumpsys package "$PKG" | grep -m1 -oE '(userId|appId)=[0-9]+' | cut -d= -f2)
if [ -z "$UID_APP" ]; then echo "Kuracare non è installata sul telefono collegato." >&2; exit 2; fi
STATS=$(adb shell dumpsys netstats detail)
ENTRIES=$(grep -cE "uid=$UID_APP set=" <<<"$STATS" || true)
if [ "$ENTRIES" -eq 0 ]; then
  echo "OK: nessun traffico di rete registrato per Kuracare (uid $UID_APP)."
else
  TOTALS=$(grep -A4 -E "uid=$UID_APP set=" <<<"$STATS" | grep -oE "rb=[0-9]+ rp=[0-9]+ tb=[0-9]+" \
    | awk -F'[= ]' '{rb+=$2; tb+=$6} END {printf "%d %d", rb, tb}')
  read -r RB TB <<<"$TOTALS"
  echo "ATTENZIONE: traffico di rete registrato per Kuracare (uid $UID_APP): ricevuti $RB byte, inviati $TB byte." >&2
  echo "(I contatori sono cumulativi: confronta con il valore precedente per vedere se è aumentato.)" >&2
  exit 1
fi
