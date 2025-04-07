VERSION="0.9.99"

package_linux() {
    ./resources/linux/package-linux.sh $VERSION
}

package_windows() {
    ./resources/windows/package-win.sh $VERSION
}

package_macos() {
    ./resources/macos/package-macos.sh
}




package_linux
package_windows
#package_macos  # hdiutil is absent and other issues.