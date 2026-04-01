# Tempera

Persistent episodic memory system for AI coding assistants. Single Rust crate, two binaries.

## Build & Test
```bash
./scripts/ci-checks.sh          # clippy + fmt + tests
./scripts/ci-checks.sh --full   # + benchmarks, docs, coverage
cargo test --workspace           # tests only
cargo test -- <test_name>        # specific test
cargo build --release            # release build (LTO, slow)
```

## Architecture
Single crate (`src/`) with two binaries:
- `tempera` (CLI): `src/main.rs`
- `tempera-mcp` (MCP server): `src/mcp_server.rs`

Key modules: `capture.rs`, `episode.rs`, `store.rs`, `indexer.rs`, `retrieve.rs`, `utility.rs`, `llm.rs`, `config.rs`, `stats.rs`, `feedback.rs`.

Error handling: `anyhow::Result` throughout, `thiserror` for typed errors.

## Self-Referential Warning
This project IS the tempera MCP server. When rebuilding the binary, do NOT call tempera MCP tools simultaneously — the binary may be locked or replaced mid-execution. Use smelt and stellarion MCP tools instead while working on this project.

## Version
Version is in `Cargo.toml` (currently `0.2.0`). Tagging `v*` triggers GitHub Actions release for 5 platforms + npm publish.
