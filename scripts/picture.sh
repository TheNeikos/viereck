#! /usr/bin/bash

set -e
set -o nounset
set -o pipefail

container() { ../target/release/viereck-container "$@" ;}
text() { ../target/release/viereck-text "$@" ;}
image() { ../target/release/viereck-image --path "$1" ;}

simple_text() { text --font "DejaVu Sans Mono" --font-size 12 --align-self center "$@" ;}

uniq_linebuffered() {
  awk '$0 != l { print ; l=$0 ; fflush(); }' "$@"
}

cont=$(container --padding 5 --background 0xFF --grow 1 --align-items center --justify-content center -c "$(image $1)")

echo [$cont]
