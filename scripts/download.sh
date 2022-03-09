#!/usr/bin/env bash
set -e

MODE="prompt"

# Parse command-line flag
case "$1" in
    --zip) MODE="zip" ;;
    --git) MODE="git" ;;
    "" )   MODE="prompt" ;;   # no args → prompt
    *) echo "Usage: $0 [--zip|--git]"; exit 1 ;;
esac

prompt_download() {
    if [ -d "$REPO_NAME" ]; then
        echo "Folder '$REPO_NAME' already exists, skipping download."
        return 0
    fi

    echo "Folder '$REPO_NAME' not found."
    echo "Do you want to download via:"
    echo "  1) Zip file"
    echo "  2) Git repo"
    read -rp "Choose [1/2]: " choice

    case "$choice" in
    1)
        echo "Downloading zip..."
        ZIP_URL="https://github.com/mlafroce/$REPO_NAME/archive/refs/heads/master.zip"
        wget -O "$REPO_NAME.zip" "$ZIP_URL"
        unzip -q "$REPO_NAME.zip"
        mv "$REPO_NAME-master" "$REPO_NAME"
        rm "$REPO_NAME.zip"
        ;;
    2)
        echo "Cloning repo..."
        REPO_URL="https://github.com/mlafroce/$REPO_NAME"
        git clone "$REPO_URL" "$REPO_NAME"
        ;;
    *)
        echo "Invalid choice, exiting."
        return 1
        ;;
    esac
}

REPO_NAME="barbaos-binutils-gdb"
prompt_download

REPO_NAME="barbaos-gcc"
prompt_download
