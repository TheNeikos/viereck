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

text=$(simple_text --color 0xFFFFFFFF --text "Asdf")
cnt="$(container --grow 1 --margin 1 --background 0x123456FF --justify-content center --children "$text")"
cnt="$(container --width 100% --height 100% --justify-content space_around --children "$cnt" --children "$cnt")"
cnt="$(container --width 100% --height 100% --justify-content space_around --flex-direction column --children "$cnt" --children "$cnt")"

items=()
items+=("$cnt")
echo "[${items[@]}]"
