#!/bin/bash
set -x

src=icon.png

conv_opts="-colors 256 -background none -density 300"

# the linux icon
for size in "16" "24" "32" "48" "64" "96" "128" "256" "512"; do
  target="./icons/hicolor/${size}x${size}/apps"
  mkdir -p "$target"
  convert $conv_opts -resize "!${size}x${size}" "$src" "$target/tirra.png"
done
