# Learning Model

TypeForge uses a **Positive Reinforcement Model**. We do not penalize words when they are ignored.

- **Prediction Accepted**: `+10` boost.
- **Word Typed Manually**: `+2` boost (if uncommon).
- **Common Dictionary Word Typed**: Ignored.

All data is stored in `~/.local/share/typeforge/learning.db`.
