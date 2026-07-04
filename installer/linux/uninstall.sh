#!/bin/bash
set -e

echo "Uninstalling TypeForge..."

PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
LIB_DIR="${PREFIX}/lib/fcitx5"
ADDON_DIR="${PREFIX}/share/fcitx5/addon"
CONFIG_DIR="${HOME}/.config/typeforge"
DATA_DIR="${PREFIX}/share/typeforge"
STATE_DIR="${PREFIX}/state/typeforge"

echo "Stopping daemon..."
pkill typeforge-daemon || true

echo "Removing binaries..."
rm -f "$BIN_DIR/typeforge-daemon"
rm -f "$BIN_DIR/typeforge"

echo "Removing Fcitx5 adapter..."
rm -f "$LIB_DIR/fcitx5-typeforge.so"
rm -f "$ADDON_DIR/typeforge.conf"
rm -f "$ADDON_DIR/typeforge-im.conf"

echo "Removing assets and state..."
rm -rf "$DATA_DIR"
rm -rf "$STATE_DIR"

echo "Restarting Fcitx5..."
if pgrep fcitx5 > /dev/null; then
    fcitx5 -r -d > /dev/null 2>&1 || true
    echo "Fcitx5 restarted."
fi

read -p "Do you want to remove configuration files in $CONFIG_DIR? [y/N] " remove_config
if [[ "$remove_config" =~ ^[Yy]$ ]]; then
    rm -rf "$CONFIG_DIR"
    echo "Configuration removed."
fi

echo "Uninstallation complete."
