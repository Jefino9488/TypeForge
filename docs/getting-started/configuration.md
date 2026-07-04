# Configuration

TypeForge is highly configurable via a TOML file. By default, the installer places this file at:
`~/.config/typeforge/config.toml`

## Default Configuration

```toml
[general]
# Set to false to entirely disable TypeForge from learning new words you type.
learning = true

# The maximum number of autocomplete candidates to display in the Fcitx5 popup.
candidate_limit = 5

[dictionary]
# The primary language (currently only English 'en' is supported).
language = "en"

# Path to the immutable dictionary. If left blank, it defaults to the bundled one.
path = ""

[logging]
# The verbosity of the daemon log (trace, debug, info, warn, error).
# Logs are stored in ~/.local/state/typeforge/logs/daemon.log
level = "info"

[session]
# Set to false to disable short-term session recency tracking.
memory = true
```

## Applying Changes
After making changes to `config.toml`, simply restart the TypeForge daemon (or restart Fcitx5) for the changes to take effect:

```bash
pkill typeforge-daemon
# Fcitx5 will automatically restart the daemon when you start typing again.
```
