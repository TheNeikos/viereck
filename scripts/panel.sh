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

{
    while true ; do
        # "date" output is checked once a second, but an event is only
        # generated if the output changed compared to the previous run.
        date +$'date\t%H:%M:%S %Y-%m-%d'
        sleep 1 || break
    done &
    herbstclient --idle &
} | {
  IFS=$'\t' read -ra tags <<< "$(herbstclient tag_status)"
  date=""
  windowtitle=""
  while true; do
    debug "Starting loop"
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
    debug "Done checking tags"

    TAG_TEXT=$(container "${tag_objects[@]}")

    LEFT=$(container \
      --children "$TAG_TEXT")

    TITLE_TEXT=$(simple_text --color 0xDDDDDDFF --text "$windowtitle")

    CENTER=$(container \
      --padding 2 --padding-end 5 --padding-start 5 \
      --background 0xFF \
      --children "$TITLE_TEXT" \
      --align-items center \
      --grow 1)

    DATE_TEXT=$(simple_text \
      --text "$date" \
      --color 0xFFFFFFFF )

    RIGHT=$(container \
      --padding 2 --padding-end 5 --padding-start 5 \
      --align-items center \
      --background 0x123456FF \
      --shrink 0 \
      --children "$DATE_TEXT")

    echo "[$LEFT, $CENTER, $RIGHT]"
    debug "Done printing JSON"

    IFS=$'\t' read -ra cmd || break

    debug "Received: ${cmd[*]}" 
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
