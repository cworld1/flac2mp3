#!/bin/bash
set -euo pipefail

export ARTIFACT_NAME="flac2mp3-$1"
export flac2mp3_GEN_COMPLETIONS=1

# Build for the target
cargo build --release --locked --target "$1"

# Create the artifact
mkdir -p "$ARTIFACT_NAME/completions"
cp "target/$1/release/flac2mp3" "$ARTIFACT_NAME"
cp flac2mp3-cli/completions/* "$ARTIFACT_NAME/completions"
cp flac2mp3-boot/completions/* "$ARTIFACT_NAME/completions"
cp README.md LICENSE "$ARTIFACT_NAME"

# Zip the artifact
if ! command -v zip &> /dev/null
then
	sudo apt-get update && sudo apt-get install -yq zip
fi
zip -r "$ARTIFACT_NAME.zip" "$ARTIFACT_NAME"
