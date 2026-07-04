# Contributing to TypeForge

Thank you for your interest in contributing to TypeForge! We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, and code changes.

## Development Workflow

1. Fork the repository and clone it locally.
2. Install the necessary dependencies (see `docs/development/build.md`).
3. Create a branch for your feature or bug fix (`git checkout -b feature/my-feature`).
4. Write your code and ensure all tests pass (`just test`).
5. Ensure your code is formatted (`just lint` / `cargo fmt`).
6. Commit using Conventional Commits format (e.g., `feat(engine): add cool feature`).
7. Push your branch and open a Pull Request.

## Architecture Guidelines

Before working on the core engine, please review the Architecture Decision Records (`docs/adr/`) and the overarching architecture document (`docs/architecture/overview.md`).

Key rules:
- **No blocking the main thread**: The C++ adapter must never block Fcitx5's event loop.
- **Fail gracefully**: If the daemon crashes, the IM must continue working without autocomplete.
- **Avoid string copies**: Rust and C++ boundaries should avoid unnecessary allocations where possible.

## Pull Request Process

1. Provide a detailed description of your change.
2. Link any related issues.
3. If your change affects performance, provide `just bench` output.
4. A maintainer will review your PR and provide feedback. Once approved and CI passes, it will be merged.
