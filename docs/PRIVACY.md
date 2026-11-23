# Privacy

Heimdall is **local-first**.

- No required cloud account or outbound telemetry by default.
- Data stays on disk under the application data directory (Phase 3+).
- Process lists and logs are treated as sensitive; exports redact paths and usernames by default.
- No keystroke logging, screen recording, or clipboard access.
- Optional Ollama runs locally; Heimdall does not auto-download models.