# 2. IPC Protocol Selection

Date: 2026-06-20

## Status
Accepted

## Context
The TypeForge input method adapters (e.g., Fcitx5) run within the context of the user's desktop environment and are extremely latency-sensitive. The heavy lifting (prediction, learning, dictionary loading) is handled by a background daemon. We needed a way for the adapter to communicate with the daemon efficiently.

## Alternatives Considered
- **gRPC**: Too heavy. Requires HTTP/2 stack in the client, which adds unnecessary latency and binary bloat for a local-only daemon.
- **Shared Memory**: Extremely fast, but complex to implement safely across Rust and C++ boundaries, and overkill for small text payloads.
- **Unix Domain Sockets + JSON**: Simple, native to Linux, and fast enough for text-based payloads on loopback.

## Decision
We chose **Unix Domain Sockets** using a simple **JSON over TCP/Stream** protocol. A lightweight Tokio async server handles connections on the daemon side.

## Consequences
- Extremely simple to write clients in any language (Rust, C++, Python for testing).
- JSON serialization adds a small overhead, but benchmarks show it takes <1ms, which is well within our 16ms latency budget.
- Only works on Unix-like systems (Linux, macOS). For Windows, we will eventually need to implement Named Pipes.
