#!/bin/bash

set -e

# Command line parameters will be passed to compile.sh, check its help for usage

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SOURCE_DIR=$DIR/../../

# Check if the SDK for the minimum build target is available.
# If not, use the one for the installed macOS Version
OSX_MIN_VERSION="12.3"
SDK_DIRECTORY="/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX$OSX_MIN_VERSION.sdk"

OSX_VERSION=$(sw_vers -productVersion | cut -d . -f 1,2)

if [ ! -d "$SDK_DIRECTORY" ]; then
   OSX_MIN_VERSION=$OSX_VERSION
   SDK_DIRECTORY="/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX$OSX_VERSION.sdk"
   if [ ! -d "$SDK_DIRECTORY" ]; then
      # If the SDK for the current macOS Version can't be found, use whatever is linked to MacOSX.sdk
      SDK_DIRECTORY="/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
   fi
fi

WLVERSION="2.0.0-rc4"
DESTINATION="packages/macos-release"
SOURCE_DIR="target/release"
RESOURCES="resources"

echo ""
echo "   Source:      $SOURCE_DIR"
echo "   Version:     $WLVERSION"
echo "   Destination: $DESTINATION"
#echo "   Type:        $TYPE"
echo "   macOS:       $OSX_VERSION"
echo "   Target:      $OSX_MIN_VERSION"
#echo "   Compiler:    $COMPILER"
echo ""

function MakeDMG {
   # Sanity check: Make sure tirra is there.
   test -f $DESTINATION/Tirra.app/Contents/MacOS/tirra

   find $DESTINATION -name ".?*" -exec rm -v {} \;
   UP=$(dirname $DESTINATION)

   #echo "Copying COPYING"
   #cp "$SOURCE_DIR"/COPYING  "$DESTINATION"/COPYING.txt

   echo "Creating DMG ..."

   rm -Rf $UP/*.dmg

    HDI_MAX_TRIES=1

   HDI_TRY=0
   HDI_RESULT=0
   while true; do
      HDI_TRY=$(( ++HDI ))
      hdiutil create -fs HFS+ -volname "Tirra $WLVERSION" -srcfolder "$DESTINATION" \
              "$UP/tirra_${OSX_MIN_VERSION}_${WLVERSION}.dmg" || HDI_RESULT=$?
      if [ $HDI_RESULT -eq 0 ]; then
         return
      fi
      # EBUSY is error code 16. We only allow that, all others should fail immediately.
      if [ $HDI_RESULT -ne 16 -o $HDI_TRY -eq $HDI_MAX_TRIES ]; then
         exit $HDI_RESULT
      fi

      echo "  will retry after 10 seconds..."
      sleep 10
   done
}

function MakeAppPackage {
   echo "Making $DESTINATION/Tirra.app now."
   rm -Rf "$DESTINATION"

   mkdir -p "$DESTINATION"
   mkdir "$DESTINATION"/Tirra.app
   mkdir "$DESTINATION"/Tirra.app/Contents
   mkdir "$DESTINATION"/Tirra.app/Contents/Resources
   mkdir "$DESTINATION"/Tirra.app/Contents/MacOS
   cp "$RESOURCES"/macos/icon.icns "$DESTINATION"/Tirra.app/Contents/Resources/Tirra.icns
   ln -s /Applications "$DESTINATION"/Applications

   cat > $DESTINATION/Tirra.app/Contents/Info.plist << EOF
{
   CFBundleName = tirra;
   CFBundleDisplayName = Tirra;
   CFBundleIdentifier = "org.tirra.wl";
   CFBundleVersion = "$WLVERSION";
   CFBundleShortVersionString = "$WLVERSION";
   CFBundleInfoDictionaryVersion = "6.0";
   CFBundlePackageType = APPL;
   CFBundleSignature = trra;
   CFBundleExecutable = tirra;
   CFBundleIconFile = "Tirra.icns";
}
EOF

   #echo "Copying data files ..."
   #rsync -Ca "$SOURCE_DIR"/data "$DESTINATION"/Widelands.app/Contents/MacOS/

   echo "Copying binary ..."
   cp -a "$SOURCE_DIR"/gui $DESTINATION/Tirra.app/Contents/MacOS/tirra

   # Locate ASAN Library by asking llvm (nice trick by SirVer I suppose)
   #ASANLIB=$(echo "int main(void){return 0;}" | xcrun clang -fsanitize=address \
    #   -xc -o/dev/null -v - 2>&1 |   tr ' ' '\n' | grep libclang_rt.asan_osx_dynamic.dylib)


    #ASANPATH=$(dirname "$ASANLIB")

   #echo "Copying and fixing dynamic libraries... "
   # $SOURCE_DIR/utils/macos/bundle-dylibs.sh \
   #  -l ../libs \
   #  $DESTINATION/Widelands.app

   # Alternative Tool to use
   #dylibbundler --overwrite-dir --bundle-deps --no-codesign \
	#--search-path "$ASANPATH" \
	#--fix-file "$DESTINATION/Widelands.app/Contents/MacOS/widelands" \
	#--dest-dir "$DESTINATION/Widelands.app/Contents/libs"

   #echo "Re-sign libraries with an 'ad-hoc signing' see man codesign"
   #codesign --sign - --force $DESTINATION/Tirra.app/Contents/libs/*

   echo "Stripping binary ..."
   strip -u -r $DESTINATION/Tirra.app/Contents/MacOS/tirra
}

function buidTirra() {
    cargo build --bin gui --profile release
    echo "Done building."
}

buidTirra "$@"
MakeAppPackage
MakeDMG
