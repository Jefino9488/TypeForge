# 1. Monorepo Structure

Date: 2026-06-15

## Status
Accepted

## Context
TypeForge consists of several distinct pieces: a background daemon (Rust), a client library (Rust), input method adapters (C++ / Rust), and a command-line interface. Managing these across multiple repositories introduces overhead in versioning, dependency management, and continuous integration.

## Alternatives Considered
- **Multi-repo**: One repository for the core engine, one for the daemon, one for Fcitx5 adapter.
  - *Pros*: Strict decoupling.
  - *Cons*: Difficult to synchronize IPC changes. Requires publishing crates internally before testing the adapter.

## Decision
We chose a **monorepo structure** powered by a Cargo workspace for all Rust crates, alongside C++ adapter code in the `adapters/` directory. 

## Consequences
- Single commit history makes bisecting easier.
- Atomic commits for IPC protocol changes.
- Requires slightly more complex CI/CD workflows to test different languages (Rust + C++) in one repository.
