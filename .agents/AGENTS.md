# Global Rules

- Zero warnings on main. Code must compile cleanly with `cargo clippy --all-targets --all-features` with no warnings before committing.

# Git Rules

- **No direct pushes to main:** Never push directly to the `main` branch. Always create a new feature branch for your work (e.g., `git checkout -b feature/name`).
- **Atomic Commits:** Always create multiple, logical commits based on the changes rather than lumping everything into a single massive commit.
- **Conventional Commits:** Always use the conventional commit format for messages: `type(scope): message` (e.g., `feat(ui): add dark mode toggle`, `fix(cli): resolve panic on startup`).
