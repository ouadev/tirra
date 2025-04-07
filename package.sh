package_linux() {
    ./resources/linux/package-linux.sh
}

package_macos() {
    ./resources/macos/package-macos.sh
}

package_windows() {
    ./resources/windows/package-win.sh
}



package_linux
package_windows
#package_macos  # hdiutil is absent and other issues.