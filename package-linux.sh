#!/bin/bash

ARCH="x86_64"
TARGET="tirra"
VERSION="0.0.1"
PROFILE="release"
ASSETS_DIR="resources/linux"
RELEASE_DIR="target/$PROFILE"
BINARY="$RELEASE_DIR/gui"
PACKAGE_NAME="${TARGET}_0.0-1"
ARCHIVE_DIR="$RELEASE_DIR/$PACKAGE_NAME"

#ARCHIVE_NAME="$TARGET-$VERSION-$ARCH-linux.tar.gz"
#ARCHIVE_PATH="$RELEASE_DIR/$ARCHIVE_NAME"

build() {
  cargo build --bin gui --profile $PROFILE
}

package() {
  build

  rm -rf $ARCHIVE_DIR

  install -Dm755 $BINARY -t $ARCHIVE_DIR/usr/bin
  mv $ARCHIVE_DIR/usr/bin/gui $ARCHIVE_DIR/usr/bin/tirra
  #install -Dm644 $ASSETS_DIR/org.squidowl.halloy.appdata.xml -t $ARCHIVE_DIR/share/metainfo
  install -Dm644 $ASSETS_DIR/tirra.desktop -t $ARCHIVE_DIR/usr/share/applications
  install -Dm644 $ASSETS_DIR/control -t $ARCHIVE_DIR/DEBIAN/
  cp -r $ASSETS_DIR/icons $ARCHIVE_DIR/usr/share/

  cd $RELEASE_DIR
  dpkg-deb --build $PACKAGE_NAME
  echo "Packaged Deb Created ..."
}

case "$1" in
  "package") package;;
  *)
    echo "avaiable commands: package"
    ;;
esac