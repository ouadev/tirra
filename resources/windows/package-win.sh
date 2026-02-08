# Generate an EXE file for Windows from MacOS.
#
# Instructions to cross-compile Tirra for Windows on MacOS.
#
# . use rustup to add windows target files. [missing command]
# . Install build tools : brew install mingw-w64
# . Build : cargo build --target x86_64-pc-windows-gnu --profile release --bin gui
VERSION="$1"

mkdir -p packages/windows-release
# Build Windows Resource file Object.
x86_64-w64-mingw32-windres resources/windows/resources.rc -O coff -o packages/windows-release/tirra-win.lib

# Build the application.
cargo build --target x86_64-pc-windows-gnu --profile release --bin gui
cargo build --target x86_64-pc-windows-gnu --profile release --bin cli

# Copy the exe
cp target/x86_64-pc-windows-gnu/release/gui.exe packages/tirra-$VERSION.exe
cp target/x86_64-pc-windows-gnu/release/cli.exe packages/tirra-cli-$VERSION.exe
