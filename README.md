# TypeForge ⚡

![TypeForge Status](https://img.shields.io/badge/Status-Alpha-orange) ![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)

> **Type smarter.**
>
> Offline AI-ready predictive keyboard for Linux.

Instead of writing the same variable names, commands, or email addresses repeatedly, TypeForge predicts what you want to type across **any application**—your IDE, terminal, or browser. 

---

## 📦 Installation

TypeForge currently supports **Linux (Wayland & X11)** via Fcitx5.

### Pre-compiled Release (Recommended)

1. Download the latest release from the [Releases](https://github.com/Jefino9488/TypeForge/releases) page.
2. Extract the archive.
3. Run the installer:
```bash
tar -xzf TypeForge-Linux-x86_64.tar.gz
cd typeforge
chmod +x install.sh
sudo ./install.sh
```

### 🚀 Quick Start

Once installed, simply type `typeforge doctor` in your terminal to verify everything is running correctly:

```bash
$ typeforge doctor
Theme: catppuccin-mocha-mauve
Layout: Horizontal
TypeForge Doctor
Checking system health...

✓ Daemon running
✓ Socket found (/tmp/typeforge.sock)
✓ Fcitx plugin installed
✓ Config valid
✓ Dictionary loaded (en)
✓ Learning enabled
```

You can customize the visual appearance using the bundled themes:
```bash
typeforge theme list
typeforge theme apply catppuccin-mocha-mauve
typeforge layout set horizontal
```

Open any text editor and start typing!

---

## ✨ Features

- **Blazing Fast**: Written in Rust, predictions happen in under `1ms`.
- **System-Wide**: Integrated natively into your desktop via Fcitx5 (Linux). No Electron, no clunky extensions.
- **Beautiful Themes**: Bundled with customized Catppuccin themes to make your predictions look incredibly sleek out-of-the-box.
- **Context-Aware**: Learns that `Vec<String>` belongs in your code editor, but `Best regards,` belongs in your email client.
- **Spellcheck Fallback**: SymSpell integration instantly corrects typos up to 4 edit distances away.
- **Privacy First**: Everything runs 100% locally on your machine.
- **Diagnostics Included**: Run `typeforge doctor` or configure via `typeforge theme` instantly from the terminal.

---

## 🏗️ Architecture

TypeForge is a decoupled engine designed for high performance:
- **Daemon (`typeforge-daemon`)**: The background Rust service powering predictions and managing the local SQLite learning models.
- **Adapter (`fcitx5-typeforge.so`)**: A lightweight C++ plugin that connects to Fcitx5 and relays keystrokes to the Daemon.
- **CLI (`typeforge`)**: The developer and user command-line interface.

Learn more in the [Architecture Overview](docs/architecture/overview.md).

---

## 📖 Documentation

Dive deeper into how TypeForge works and how to configure it:

- [Getting Started & Installation](docs/getting-started/installation.md)
- [Configuration](docs/getting-started/configuration.md)
- [How the Learning Engine Works](docs/architecture/learning.md)
- [Build Instructions](docs/development/build.md)

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for architecture guidelines, rules, and how to get started.

## 📜 License

TypeForge is licensed under the [Apache 2.0 License](LICENSE).
