# IPC Protocol

TypeForge uses a newline-delimited JSON (NDJSON) protocol over a Unix Domain Socket.

## Client Requests
- `PredictRequest`: Requests autocomplete candidates for a given prefix.
- `LearnRequest`: Sends a user action (e.g., accepted a word) to the daemon.
- `ReloadDictionary`: Forces the daemon to reload the base dictionary.

## Daemon Responses
- `PredictResponse`: Returns a list of predictions.
- `Ack`: Acknowledges a state change.
