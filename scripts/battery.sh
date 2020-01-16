#! /usr/bin/bash

set -e
set -o nounset
set -o pipefail

container() { ../target/release/viereck-container "$@" ;}
text() { ../target/release/viereck-text "$@" ;}

simple_text() { text --font "DejaVu Sans Mono" --font-size 12 "$@" ;}

uniq_linebuffered() {
  awk '$0 != l { print ; l=$0 ; fflush(); }' "$@"
}

ELAPSED=$(date +%s.%N)

debug() {
  now=$(date +%s.%N)
  echo "[$(bc <<< "$now - $ELAPSED")]" "$*" >&2
  ELAPSED=$(date +%s.%N)
}

BAT_PCT=$(acpi | cut -f2 -d, | tr -d \[:space:\])

BG="0x00FF00FF"

if [ "${BAT_PCT::-1}" -lt 40 ]; then
  BG="0xFFFF00FF"
elif [ "${BAT_PCT::-1}" -lt 20 ]; then
  BG="0xFF0000FF"
fi

#BAT_PCT="60%"

BAT_TEXT=$(simple_text --text "$BAT_PCT" --color 0xFF)

INNER=$(container --width "$BAT_PCT" --background "$BG" --align-items center --justify-content center -c "$BAT_TEXT")

OUTER=$(container --padding 5 -c "$INNER" --grow 1)

echo "[$OUTER]"
