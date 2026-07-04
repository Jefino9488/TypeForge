# Build Instructions

## Requirements
- Rust (Cargo)
- CMake, Extra CMake Modules (ECM)
- Fcitx5 Development Headers (`fcitx5-devel`)

## Building
```bash
cargo build --release
mkdir -p adapters/fcitx5/build && cd adapters/fcitx5/build
cmake -DCMAKE_INSTALL_PREFIX=/usr ..
make
```
