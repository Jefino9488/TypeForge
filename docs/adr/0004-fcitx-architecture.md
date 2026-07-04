# 4. Fcitx5 Adapter Architecture

Date: 2026-06-25

## Status
Accepted

## Context
TypeForge requires deep integration with Linux desktop environments to capture keystrokes and display candidates. Fcitx5 is the most modern and extensible input method framework on Linux.

## Alternatives Considered
- **IBus**: Older, heavily tied to GNOME, harder to write modern C++ modules for.
- **Wayland IM protocols directly**: Extremely complex, requires implementing a full compositor-level client.

## Decision
We chose to build an **Fcitx5 Addon**. 
Because Fcitx5 is written in C++, but our core engine is written in Rust, we built a thin C++ wrapper (`TypeForgeEngine.cpp`) that intercepts keystrokes and immediately calls into a Rust FFI static library (`libtypeforge_fcitx5_bridge.a`). The Rust bridge then handles all asynchronous IPC communication with the daemon using Tokio.

## Consequences
- **Pros**: Fcitx5 handles the UI (Input Panel) and application context switching for us. We just supply candidates.
- **Cons**: We have a language barrier (C++ to Rust). Care must be taken to ensure string encodings (UTF-8) and pointers are correctly managed across the FFI boundary to prevent memory leaks or crashes.
