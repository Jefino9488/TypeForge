#!/bin/bash
set -e

echo "Installing TypeForge..."

# Verify dependencies
if ! command -v fcitx5 &> /dev/null; then
    echo "Error: fcitx5 is not installed or not in PATH."
    echo "Please install fcitx5 first."
    exit 1
fi

if [ -z "$XDG_SESSION_TYPE" ]; then
    echo "Warning: XDG_SESSION_TYPE is not set. Assuming Wayland/X11 environment."
elif [[ "$XDG_SESSION_TYPE" != "wayland" && "$XDG_SESSION_TYPE" != "x11" && "$XDG_SESSION_TYPE" != "tty" ]]; then
    echo "Warning: Unknown session type ($XDG_SESSION_TYPE). TypeForge relies on Wayland or X11."
fi

# Determine directories
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
LIB_DIR="${PREFIX}/lib/fcitx5"
ADDON_DIR="${PREFIX}/share/fcitx5/addon"
CONFIG_DIR="${HOME}/.config/typeforge"
DATA_DIR="${PREFIX}/share/typeforge"

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$LIB_DIR"
mkdir -p "$ADDON_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"

echo "Copying binaries..."
if [ -f "bin/typeforge-daemon" ]; then
    cp bin/typeforge-daemon "$BIN_DIR/"
    cp bin/typeforge-cli "$BIN_DIR/typeforge"
    chmod +x "$BIN_DIR/typeforge-daemon"
    "$BIN_DIR/typeforge"
else
    # Development fallback
    cp target/release/typeforge-daemon "$BIN_DIR/"
    cp target/release/typeforge-cli "$BIN_DIR/typeforge"
    chmod +x "$BIN_DIR/typeforge-daemon"
    chmod +x "$BIN_DIR/typeforge"
fi

echo "Copying Fcitx5 adapter..."
if [ -f "lib/fcitx5-typeforge.so" ]; then
    cp lib/fcitx5-typeforge.so "$LIB_DIR/"
    cp lib/typeforge.conf "$ADDON_DIR/"
    cp lib/typeforge-im.conf "$ADDON_DIR/"
elif [ -f "adapters/fcitx5/build/fcitx5-typeforge.so" ]; then
    cp adapters/fcitx5/build/fcitx5-typeforge.so "$LIB_DIR/"
    cp adapters/fcitx5/build/typeforge.conf "$ADDON_DIR/"
    cp adapters/fcitx5/build/typeforge-im.conf "$ADDON_DIR/"
fi

echo "Copying assets..."
if [ -f "assets/dictionary-v1.csv.gz" ]; then
    cp assets/dictionary-v1.csv.gz "$DATA_DIR/dictionary.csv.gz"
fi

echo "Setting up configuration..."
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    cat << 'EOF' > "$CONFIG_DIR/config.toml"
[general]
learning = true
candidate_limit = 5

[dictionary]
language = "en"

[logging]
level = "info"

[session]
memory = true
EOF
fi

echo "Restarting Fcitx5..."
if pgrep fcitx5 > /dev/null; then
    # Kill gracefully, it usually auto-restarts or we start it
    fcitx5 -r -d > /dev/null 2>&1 || true
    echo "Fcitx5 restarted."
else
    echo "Fcitx5 is not currently running. Start it to use TypeForge."
fi

echo "Installation complete!"
echo "Run 'typeforge doctor' to verify."
