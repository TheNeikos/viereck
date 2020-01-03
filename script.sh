#! /usr/bin/bash

set -e
set -o nounset
set -o pipefail

container() { ./target/release/viereck-container "$@" ;}
text() { ./target/release/viereck-text "$@" ;}

simple_text() { text --font "DejaVu Sans Mono" --font-size 12 "$@" ;}

uniq_linebuffered() {
  awk '$0 != l { print ; l=$0 ; fflush(); }' "$@"
}

function join_by { local IFS="$1"; shift; echo "$*"; }

{
    while true ; do
        # "date" output is checked once a second, but an event is only
        # generated if the output changed compared to the previous run.
        date +$'date\t%H:%M:%S, %Y-%m-%d'
        sleep 1 || break
    done &
    herbstclient --idle 
} | {
  IFS=$'\t' read -ra tags <<< "$(herbstclient tag_status)"
  date=""
  windowtitle=""
  while true; do
    tag_objects=()

    for i in "${tags[@]}" ; do
      tag_objects+=("-c")
      case ${i:0:1} in
        '#')
          inner_text=$(simple_text --color 0x101010FF --text "${i:1}")
          tag_objects+=("$(container \
            --padding 2 --padding-end 5 --padding-start 5 \
            --align-items center --background 0x9FBC00FF -c "$inner_text")")
          ;;
        '+')
          tag_objects+=("$(container \
            --padding 2 --padding-end 5 --padding-start 5 \
            --align-items center --background 0xFF -c "$(simple_text --color 0xFF --text "${i:1}")")")
          ;;
        ':')
          inner_text=$(simple_text --color 0xFFFFFFFF --text "${i:1}")
          tag_objects+=("$(container \
            --padding 2 --padding-end 5 --padding-start 5 \
            --align-items center --background 0x777777FF -c "$inner_text")")
          ;;
        '!')
          tag_objects+=("$(container \
            --padding 2 --padding-end 5 --padding-start 5 \
            --align-items center --background 0xFF -c "$(simple_text --color 0xFF --text "${i:1}")")")
          ;;
        *)
          tag_objects+=("$(container \
            --padding 2 --padding-end 5 --padding-start 5 \
            --align-items center --background 0x222222FF -c "$(simple_text --color 0xDDDDDDFF --text "${i:1}")")")
          ;;
      esac
    done

    TAG_TEXT=$(container "${tag_objects[@]}")

    LEFT=$(container \
      --children "$TAG_TEXT")

    TITLE_TEXT=$(simple_text --color 0xDDDDDDFF --text "$windowtitle")

    CENTER=$(container \
      --padding 2 --padding-end 5 --padding-start 5 \
      --background 0xFF \
      --children "$TITLE_TEXT" \
      --grow 1)

    DATE_TEXT=$(text \
      --text "$date" \
      --font "DejaVu Sans Mono" \
      --font-size 12 \
      --color 0xFFFFFFFF )

    RIGHT=$(container \
      --padding 2 --padding-end 5 --padding-start 5 \
      --background 0x123456FF \
      --children "$DATE_TEXT")

    echo "[$LEFT, $CENTER, $RIGHT]"

    IFS=$'\t' read -ra cmd || break
    case "${cmd[0]}" in
      tag*)
        # echo "resetting tags" >&2
        IFS=$'\t' read -ra tags <<< "$(herbstclient tag_status)"
        ;;
      date)
        date="${cmd[*]:1}"
        ;;
      focus_changed|window_title_changed)
        windowtitle="${cmd[@]:2}"
        ;;
    esac
  done
} 
