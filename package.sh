CURRENT_TAG=$(git describe --tags)

package_linux() {
    ./resources/linux/package-linux.sh $CURRENT_TAG
}

package_windows() {
    ./resources/windows/package-win.sh $CURRENT_TAG
}

package_macos() {
    ./resources/macos/package-macos.sh
}


package_linux
package_windows
#package_macos  # hdiutil is absent and other issues.