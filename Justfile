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
	cd adapters/fcitx5/build && cmake -DCMAKE_INSTALL_PREFIX=/usr ../cpp && make && sudo make install
	echo "Install complete. Please restart fcitx5."

# Build the daemon and dict compiler
build:
	cargo build --release -p typeforge-dict-compiler
	cargo run --release -p typeforge-dict-compiler -- compile assets/dictionary-v1.csv.gz assets/dictionary.bin
	cargo build --release -p typeforge-daemon
	cargo build --release -p typeforge-cli

# Build for release
build-release:
	cargo build --release -p typeforge-dict-compiler
	cargo run --release -p typeforge-dict-compiler -- compile assets/dictionary-v1.csv.gz assets/dictionary.bin
	cargo build --release

# Run the daemon locally (Dogfooding)
dogfood: build-release
	RUST_LOG=info ./target/release/typeforge-daemon
