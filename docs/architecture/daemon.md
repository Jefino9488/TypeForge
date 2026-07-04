# Daemon Architecture

The `typeforge-daemon` is a Tokio-based asynchronous server that loads dictionaries into memory and listens for connections on a Unix Domain Socket (default: `/tmp/typeforge.sock`).

It is responsible for:
- Initializing the SQLite learning databases.
- Holding the `TypeForgeEngine` in memory.
- Handling concurrent client requests efficiently.
