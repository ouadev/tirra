#!/bin/bash
set -x

src=icon-2.png

# Generate Linux Icons
conv_opts="-colors 256 -background none -density 300"
for size in "16" "24" "32" "48" "64" "96" "128" "256" "512"; do
  target="linux/icons/hicolor/${size}x${size}/apps"
  mkdir -p "$target"
  convert $conv_opts -resize "!${size}x${size}" "$src" "$target/tirra.png"
done

# Generate Macos Icon
macos_out_dir=macos/icon.iconset

# the convert command comes from imagemagick
for size in 16 32 64 128 256 512; do
  convert "$src" -resize x$size $macos_out_dir/icon_${size}x${size}.png
done

#mv $output_path ~/Desktop/icon.iconset
iconutil -c icns $macos_out_dir
#rm -r ~/Desktop/icon.iconset
