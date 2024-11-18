package_linux() {
    ./resources/linux/package-linux.sh
}

package_macos() {
    ./resources/macos/package-macos.sh
}

CURRENT_PLATFORM=$(uname)
echo $CURRENT_PLATFORM
case "$CURRENT_PLATFORM" in
  "linux") package_linux;;
  "Darwin") package_macos;;
  *)
    echo "unknown current platform: $CURRENT_PLATFORM"
    ;;
esac
