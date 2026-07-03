# TypeForge

TypeForge is a privacy-first, offline, cross-platform writing intelligence engine designed to bring modern predictive typing, autocorrection, and AI-assisted text input to every desktop platform.

Rather than being tied to a specific input method or editor, TypeForge is built as a standalone Rust daemon exposing a stable Unix Socket API that any frontend can use.

## Project Principles

- **Fully Offline**: Bundled assets and zero runtime network dependencies.
- **Privacy First**: User data and learned words never leave the device.
- **Zero Telemetry**: No tracking or analytics.
- **Low Latency**: Designed to process sequential pipelines rapidly, with a target of `<5 ms`.
- **Cross Platform**: Core engine runs anywhere Rust compiles.
- **Extensible**: Modular trait-based architecture for swapping implementations.
- **AI Optional**: Core functions without AI, but AI can be plugged in seamlessly.
