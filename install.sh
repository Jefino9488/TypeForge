#!/bin/bash
set -e

echo "TypeForge Installer"
echo "==================="

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo ./install.sh)"
  exit 1
fi

echo "[1/4] Installing fcitx5 addon..."
mkdir -p /usr/lib/fcitx5
cp fcitx5-typeforge.so /usr/lib/fcitx5/
chmod 755 /usr/lib/fcitx5/fcitx5-typeforge.so

echo "[2/4] Installing fcitx5 configuration..."
mkdir -p /usr/share/fcitx5/addon
mkdir -p /usr/share/fcitx5/inputmethod
cp typeforge.conf /usr/share/fcitx5/addon/
cp typeforge-im.conf /usr/share/fcitx5/inputmethod/
chmod 644 /usr/share/fcitx5/addon/typeforge.conf
chmod 644 /usr/share/fcitx5/inputmethod/typeforge-im.conf

echo "[3/4] Installing daemon..."
mkdir -p /usr/local/bin
cp typeforge-daemon /usr/local/bin/
chmod 755 /usr/local/bin/typeforge-daemon

echo "[4/4] Restarting Fcitx5 (if running as user)..."
if command -v fcitx5-remote &> /dev/null; then
    sudo -u $SUDO_USER fcitx5-remote -r || true
fi

echo ""
echo "✅ Installation complete!"
echo "Please run 'typeforge-daemon' in the background or set it up as a systemd user service."
echo "You can verify the installation by running: typeforge doctor"
