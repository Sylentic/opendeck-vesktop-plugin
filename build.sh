#!/usr/bin/env bash
# Build and install the Vesktop OpenDeck plugin.
#
# Usage:
#   ./build.sh <output_directory> <binary_name> <target_triple>
#
# Example (install into OpenDeck plugins dir):
#   ./build.sh ~/.config/opendeck/plugins/com.sylentic.opendeck-vesktop.sdPlugin opendeck-vesktop x86_64-unknown-linux-gnu
#
# The target triple is used as a binary suffix and is passed to `cargo build --target`.
# For native builds you need the target installed: `rustup target add <target_triple>`

set -euo pipefail

if [ $# -ne 3 ]; then
	echo "Usage: $0 <output_directory> <binary_name> <target_triple>"
	echo "Example: $0 ~/.config/opendeck/plugins/com.sylentic.opendeck-vesktop.sdPlugin opendeck-vesktop x86_64-unknown-linux-gnu"
	exit 1
fi

OUTPUT_DIR="$1"
BINARY_NAME="$2"
TARGET_TRIPLE="$3"

# Clean and copy assets
rm -rf "$OUTPUT_DIR"
cp -r assets/ "$OUTPUT_DIR"

# Build the plugin for the given target
cargo build --release --target "$TARGET_TRIPLE"

# Copy the binary with the target-triple suffix
cp "target/$TARGET_TRIPLE/release/$BINARY_NAME" "$OUTPUT_DIR/$BINARY_NAME-$TARGET_TRIPLE"

echo "Plugin installed to $OUTPUT_DIR"
