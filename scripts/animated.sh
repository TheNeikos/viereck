#! /usr/bin/bash

set -e
set -o nounset
set -o pipefail

container() { ../target/release/viereck-container "$@" ;}
text() { ../target/release/viereck-text "$@" ;}

simple_text() { text --font "DejaVu Sans Mono" --font-size 12 --align-self center "$@" ;}

uniq_linebuffered() {
  awk '$0 != l { print ; l=$0 ; fflush(); }' "$@"
}

while true; do
  for i in $(seq 10 100; seq 100 -1 10); do
    items=()
    text=$(simple_text --color 0xFF --text $i)
    items+=($(container --width $i% --background 0xFF22FFFF --justify-content center --children "$text"))
    echo "[${items[@]}]"
    sleep 0.0001
  done
done
