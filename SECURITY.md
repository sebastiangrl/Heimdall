# Security Policy

## Supported versions

Early development (`0.1.x`). Security fixes land on `main`.

## Reporting

Report vulnerabilities privately to the repository owner (GitHub security advisory or maintainer contact). Do not open public issues for exploitable flaws.

## Design commitments

- No remote access in current phases
- No arbitrary shell execution from the UI
- LLM output cannot trigger system actions
- Daemon bound to local Unix socket with filesystem permissions (Phase 6)