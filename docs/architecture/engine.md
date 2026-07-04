# Engine Architecture

The core of TypeForge is the `typeforge-engine` crate. It is completely decoupled from any IPC or daemon logic, meaning it can be compiled into WASM or embedded directly in future adapters.

## The Prediction Pipeline

1. **Prior Candidates**: The engine fetches baseline dictionary words that match the current prefix (using binary search over a sorted vector).
2. **Learned Candidates**: The engine queries the SQLite databases for words the user has typed previously that match the prefix.
3. **Scoring**: The `ScorePipeline` runs each candidate through a series of heuristics (dictionary frequency, learning weights, session recency, context).
4. **Output**: The top N candidates are returned.
