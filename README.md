# TypeForge ⚡

![TypeForge Status](https://img.shields.io/badge/Status-Alpha-orange) ![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)

> **TypeForge** is a blazing-fast, system-wide autocomplete engine and input method that learns how you type.

Instead of writing the same variable names, commands, or email addresses repeatedly, TypeForge predicts what you want to type across **any application**—your IDE, terminal, or browser. 

---

## 🚀 Why TypeForge?

- **Blazing Fast**: Written in Rust, predictions happen in under `1ms`.
- **System-Wide**: Integrated natively into your desktop via Fcitx5 (Linux). No Electron, no clunky extensions.
- **Context-Aware**: Learns that `Vec<String>` belongs in your code editor, but `Best regards,` belongs in your email client.
- **Privacy First**: Everything runs 100% locally on your machine.

## 🛠️ Supported Platforms

| Platform | Status |
|---|---|
| **Linux (Fcitx5)** | ✓ Supported |
| **Windows** | Planned |
| **macOS** | Planned |

## 📦 Quick Install (Linux)

Ensure you have Rust, CMake, and `fcitx5-devel` installed.

```bash
git clone https://github.com/Jefino9488/TypeForge.git
cd TypeForge
just install
```
Restart Fcitx5 and add the TypeForge engine.

## 📖 Documentation

Dive deeper into how TypeForge works:
- [Architecture Overview](docs/architecture/overview.md)
- [How the Learning Engine Works](docs/architecture/learning.md)
- [Build Instructions](docs/development/build.md)

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for architecture guidelines, rules, and how to get started.

## 📜 License

TypeForge is licensed under the [Apache 2.0 License](LICENSE).
