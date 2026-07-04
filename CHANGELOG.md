# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Context-aware suggestions based on the active Fcitx5 window.
- Positive reinforcement learning module backed by SQLite.
- `first_seen`, `last_used`, and `confidence` tracking for custom typed words.
- Full Unicode and Emoji support via `unicode-segmentation`.
- GitHub Action workflows for CI, formatting, and performance benchmarking.
- Extensive documentation in the `docs/` folder.

### Fixed
- UTF-8 byte slicing bug in Fcitx5 C++ adapter that corrupted characters on backspace.

## [0.1.0] - 2026-06-25
### Added
- Initial Rust daemon and `TypeForgeEngine`.
- Fcitx5 C++ adapter and Rust FFI bridge.
- SymSpell-based spelling correction module.
- Immutable GZ-compressed CSV dictionaries.
