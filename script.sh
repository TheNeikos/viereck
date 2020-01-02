#! /usr/bin/bash

while true; do
  TEXT=$(./target/debug/viereck-text \
    --text "Hello World" \
    --font "DejaVu Sans Mono" \
    --font-size 12 \
    --color 0xFF )

  LEFT=$(./target/debug/viereck-container \
    --padding 1 \
    --background 0xDDFFCC \
    --children "$TEXT")

  CENTER=$(./target/debug/viereck-container \
    --background 0x34ae5d \
    --grow 1)

  DATE=$(./target/debug/viereck-text \
    --text "$(date)" \
    --font "DejaVu Sans Mono" \
    --font-size 12 \
    --color 0xFF )

  RIGHT=$(./target/debug/viereck-container \
    --padding 2 \
    --background 0x12345678 \
    --children "$DATE")

  echo "[$LEFT, $CENTER, $RIGHT]" | jq -c .

  sleep 1
done
