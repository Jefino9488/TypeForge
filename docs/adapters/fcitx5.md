# Fcitx5 Adapter

The Fcitx5 adapter acts as the bridge between Linux desktop text fields and the TypeForge daemon.

It is split into:
1. **C++ Addon**: Interfaces with Fcitx5 core APIs.
2. **Rust Bridge**: A static library (`libtypeforge_fcitx5_bridge.a`) called by the C++ code via FFI to handle socket networking safely.
