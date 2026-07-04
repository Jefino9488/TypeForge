# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-07-04
### Added
- Integrated TypeForge Fcitx5 Themes, bundling customized high-quality themes (e.g. Catppuccin Mocha Mauve) directly into the installer.
- Added new safe CLI configuration tools for themes and layouts (`typeforge theme apply`, `typeforge layout set`).
- TypeForge CLI now correctly parses `classicui.conf` without overwriting other user settings.
- Restored rock-solid Inline completion (buffer-based) dropping the async wayland emulation.

### Fixed
- Fixed layout hint logic in Fcitx5 adapter to decouple layout engine rules from TypeForge engine rules.

## [0.2.2] - 2026-07-04
### Fixed
- Fixed a bug where fast-typing punctuation marks or uppercase letters would cause the entire typed word to silently vanish.
- Added smart casing to the engine backend so uppercase typed letters intelligently match lowercase dictionary words and restore capitalization.

## [0.2.1] - 2026-07-04
### Fixed
- Added `install.sh` script to the release tarball to simplify installation.
- Updated installation instructions to properly use `sudo`.

## [0.2.0] - 2026-07-04
### Added
- Context-aware suggestions based on the active Fcitx5 window.
- Positive reinforcement learning module backed by SQLite.
- `first_seen`, `last_used`, and `confidence` tracking for custom typed words.
- Full Unicode and Emoji support via `unicode-segmentation`.
- GitHub Action workflows for CI, formatting, and performance benchmarking.
- Extensive documentation in the `docs/` folder.
- Spellcheck fallback using SymSpell for typos and misspellings with a distance of up to 4.

### Fixed
- UTF-8 byte slicing bug in Fcitx5 C++ adapter that corrupted characters on backspace.
- Fixed an overflow bug in the dictionary compiler causing high-frequency words to be skipped.
- Re-balanced prediction scoring to heavily prioritize exact matches and shorter word lengths, improving the predictive typing experience.

## [0.1.0] - 2026-06-25
### Added
- Initial Rust daemon and `TypeForgeEngine`.
- Fcitx5 C++ adapter and Rust FFI bridge.
- SymSpell-based spelling correction module.
- Immutable GZ-compressed CSV dictionaries.
