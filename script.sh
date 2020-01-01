#! /usr/bin/bash

LEFT=$(./target/debug/viereck-container -w 50 -b 0xDDFFCC)
CENTER=$(./target/debug/viereck-container -w 100 -b 0x34ae5d)
RIGHT=$(./target/debug/viereck-container -w 50 -b 0x12345678)

echo "[$LEFT, $CENTER, $RIGHT]"
