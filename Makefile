.PHONY: all build bridge plugin install dev clean test bench run-daemon restart-fcitx lint fmt

PREFIX ?= ~/.local
FCITX5_SHARE = $(PREFIX)/share/fcitx5

all: build

build: bridge plugin

bridge:
	cargo build --release -p typeforge-fcitx5-bridge

plugin: bridge
	mkdir -p build && cd build && cmake -DCMAKE_INSTALL_PREFIX=$(PREFIX) ../adapters/fcitx5/cpp && make -j$(nproc)

install: plugin
	cd build && make install

run-daemon:
	cargo run --release -p typeforge-daemon

restart-fcitx:
	fcitx5 -r -d

dev: install
	@echo "Restarting fcitx5..."
	$(MAKE) restart-fcitx
	@echo "Starting daemon..."
	killall typeforge-daemon || true
	$(MAKE) run-daemon

test:
	cargo test --all-targets --all-features

bench:
	cargo bench

lint:
	cargo clippy --all-targets --all-features

fmt:
	cargo fmt

clean:
	cargo clean
	rm -rf build
