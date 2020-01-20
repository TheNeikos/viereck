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
      case ${i:0:1} in
        '#')
          text_color="101010FF"
          bg_color="9FBC00FF"
          ;;
        '+')
          text_color="FFFFFFFF"
          bg_color="000000FF" 
          ;;
        ':')
          text_color="FFFFFFFF"
          bg_color="777777FF"
          ;;
        '!')
          text_color="FFFFFFFF"
          bg_color="FF0000FF" 
          ;;
        *)
          bg_color="222222FF"
          text_color="DDDDDDFF"
          ;;
      esac
      text=$(cat <<-EOF
        {
          "type": "Text",
          "font": "DejaVu Sans Mono",
          "text": "${i:1}",
          "font_size": 12,
          "color": {
            "Rgba32": $((16#$text_color))
          },
          "style": {}
        }
EOF
)
      container=$(cat <<-EOF
        {
          "type": "Container",
          "children": [$text],
          "style": {
            "padding": {
              "start": {
                "points": 5
              },
              "end": {
                "points": 5
              },
              "top": {
                "points": 2
              },
              "bottom": {
                "points": 5
              }
            },
            "align_items": "center"
          },
          "background": {
            "Rgba32": $((16#$bg_color))
          } 
        }
EOF
)
      tag_objects+=("-c $(jq -c . <<< "$container")")
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
        windowtitle="${cmd[*]:2}"
        ;;
    esac
  done
} 
