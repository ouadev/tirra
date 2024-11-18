#!/bin/bash

TARGET="tirra"
VERSION="0.0.1"
PACKAGE_NAME="${TARGET}_${VERSION}"
ASSETS_DIR="resources/linux"
BINARY="target/release/gui"
PACKAGING_DIR="packages/linux-release/"
PACKAGE_PATH="$PACKAGING_DIR/$PACKAGE_NAME"

build() {
  cargo build --bin gui --profile release
}

package() {
  build

  rm -rf $PACKAGE_PATH

  install -Dm755 $BINARY -t $PACKAGE_PATH/usr/bin
  mv $PACKAGE_PATH/usr/bin/gui $PACKAGE_PATH/usr/bin/tirra
  install -Dm644 $ASSETS_DIR/tirra.desktop -t $PACKAGE_PATH/usr/share/applications
  install -Dm644 $ASSETS_DIR/control -t $PACKAGE_PATH/DEBIAN/
  cp -r $ASSETS_DIR/icons $PACKAGE_PATH/usr/share/

  cd $PACKAGING_DIR
  dpkg-deb --build $PACKAGE_NAME
  echo "Packaged Deb Created ..."
}

# call main function

package
