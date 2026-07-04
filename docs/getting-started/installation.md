# Installation

TypeForge currently runs natively on Linux (Wayland & X11) via the Fcitx5 input method framework.

## Prerequisites
- **Fcitx5**: Ensure `fcitx5` is installed on your system.

## Installing from a Release (Recommended)

1. Navigate to the [Releases](https://github.com/Jefino9488/TypeForge/releases) page and download the latest `.tar.gz` for Linux.
2. Extract the archive:
   ```bash
   tar -xzf TypeForge-*.tar.gz
   cd typeforge
   ```
3. Run the automated installer:
   ```bash
   ./install.sh
   ```

The installer will:
- Copy the daemon and CLI to `~/.local/bin/`.
- Copy the C++ adapter plugin to `~/.local/lib/fcitx5/`.
- Set up a default configuration in `~/.config/typeforge/`.
- Restart Fcitx5.

## Verifying the Installation

Run the TypeForge Doctor to ensure everything is hooked up correctly:
```bash
typeforge doctor
```

If all checks pass, TypeForge is ready to use!

## Uninstalling

If you need to remove TypeForge, you can run the provided uninstallation script:
```bash
./uninstall.sh
```
This will cleanly remove all binaries and the Fcitx5 plugin, and optionally delete your configuration and learning databases.
