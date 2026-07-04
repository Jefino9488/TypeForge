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
killall typeforge-daemon 2>/dev/null || true
rm -f /usr/local/bin/typeforge-daemon
cp typeforge-daemon /usr/local/bin/
chmod 755 /usr/local/bin/typeforge-daemon

echo "[4/5] Setting up autostart..."
mkdir -p /etc/xdg/autostart
cat > /etc/xdg/autostart/typeforge-daemon.desktop << EOF
[Desktop Entry]
Type=Application
Name=TypeForge Daemon
Exec=/usr/local/bin/typeforge-daemon
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
EOF
chmod 644 /etc/xdg/autostart/typeforge-daemon.desktop

echo "[5/5] Starting daemon and restarting Fcitx5 (if running as user)..."
if [ -n "$SUDO_USER" ]; then
    # Start the daemon in the background for the current user
    sudo -u $SUDO_USER nohup /usr/local/bin/typeforge-daemon >/dev/null 2>&1 &
    
    # Try to restart fcitx5 (might fail if DBUS is not available in sudo)
    sudo -u $SUDO_USER fcitx5-remote -r 2>/dev/null || true
fi

echo ""
echo "✅ Installation complete!"
echo "The TypeForge daemon has been started in the background and configured to auto-start on login."
echo "You can verify the installation by running: typeforge doctor"
