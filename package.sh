package_linux() {
    ./resources/linux/package-linux.sh
}

package_macos() {
    ./resources/macos/package-macos.sh
}

CURRENT_PLATFORM=$(uname)

case "$CURRENT_PLATFORM" in
  "Linux") package_linux;;
  "Darwin") package_macos;;
  *)
    echo "unknown current platform: $CURRENT_PLATFORM"
    ;;
esac
