# TypeForge Justfile

# Build everything for development
dev:
	cargo build

# Run all tests
test:
	cargo test

# Run linter
lint:
	cargo fmt --check
	cargo clippy -- -D warnings

# Run benchmarks
bench:
	cargo bench

# Build and install the Fcitx5 adapter locally
install: build-release
	mkdir -p adapters/fcitx5/build
	cd adapters/fcitx5/build && cmake -DCMAKE_INSTALL_PREFIX=/usr .. && make && sudo make install
	echo "Install complete. Please restart fcitx5."

# Build for release
build-release:
	cargo build --release

# Run the daemon locally (Dogfooding)
dogfood: build-release
	RUST_LOG=info ./target/release/typeforge-daemon
