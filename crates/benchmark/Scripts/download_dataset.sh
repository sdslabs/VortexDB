#!/bin/bash

set -e

if ! CARGO_TOML_PATH=$(cargo locate-project --message-format plain 2>/dev/null); then
    echo "❌ Error: Could not find Cargo.toml. Are you running this inside a Rust project?" >&2
    exit 1
fi

CRATE_ROOT=$(dirname "$CARGO_TOML_PATH")

ROOT_DIR="$CRATE_ROOT/Datasets"
BASE_URL="ftp://ftp.irisa.fr/local/texmex/corpus"

if [ -d "$ROOT_DIR" ]; then
    echo "✅ '$ROOT_DIR' already exists. Skipping download."
    exit 0
fi

echo "📂 Creating directory at: $ROOT_DIR"
mkdir -p "$ROOT_DIR"

DATASETS=("siftsmall.tar.gz" "sift.tar.gz")

for FILENAME in "${DATASETS[@]}"; do
    URL="$BASE_URL/$FILENAME"
    TEMP_FILE_PATH="$ROOT_DIR/$FILENAME"
    EXPECTED_FOLDER="${FILENAME%%.tar.gz}"

    echo "⬇️  Downloading $FILENAME using curl..."

    if ! curl -# -L -o "$TEMP_FILE_PATH" "$URL"; then
        echo "❌ Failed to download $URL" >&2
        exit 1
    fi

    echo "📦 Extracting $FILENAME..."

    tar -xzf "$TEMP_FILE_PATH" -C "$ROOT_DIR"

    rm "$TEMP_FILE_PATH"

    echo "✨ Extracted to $ROOT_DIR/$EXPECTED_FOLDER"
done

echo -e "\n🎉 Setup complete! Your file tree is ready."