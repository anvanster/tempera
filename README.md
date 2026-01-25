# MemRL - Memory-Augmented Reinforcement Learning for Claude Code

MemRL is a memory system that helps Claude Code learn from past coding sessions. It captures experiences, indexes them semantically, and uses reinforcement learning to surface the most valuable memories when relevant.

## Features

- **Session Capture**: Automatically extract and store coding experiences
- **Semantic Search**: Find relevant past experiences using vector embeddings (BGE-Small-EN)
- **Utility Learning**: Episodes gain/lose value based on feedback and usage patterns
- **MCP Integration**: Direct integration with Claude Code via Model Context Protocol

## Installation

### Build from source

```bash
git clone https://github.com/yourusername/MemRL.git
cd MemRL
cargo build --release
```

This produces two binaries:
- `target/release/memrl` - CLI tool
- `target/release/memrl-mcp` - MCP server for Claude Code

## Quick Start

### 1. Initialize MemRL

```bash
./target/release/memrl init
```

### 2. Capture a session

```bash
./target/release/memrl capture --prompt "Fixed authentication bug in login flow"
```

### 3. Index for semantic search

```bash
./target/release/memrl index
```

### 4. Retrieve relevant memories

```bash
./target/release/memrl retrieve "authentication issues"
```

## Claude Code Integration

### Configure MCP Server

**Option 1: Using Claude Code CLI (Recommended)**

```bash
claude mcp add memrl --scope user -- /path/to/MemRL/target/release/memrl-mcp
```

The `--scope user` flag makes it available across all projects.

**Option 2: Edit `~/.claude.json` directly**

Add to your `~/.claude.json`:

```json
{
  "mcpServers": {
    "memrl": {
      "command": "/path/to/MemRL/target/release/memrl-mcp",
      "args": []
    }
  }
}
```

Replace `/path/to/MemRL` with your actual installation path.

### Verify Configuration

After restarting Claude Code, check that the server is loaded:

```
/mcp
```

You should see `memrl` listed with its tools.

### Available MCP Tools

Once configured, Claude Code has access to these tools:

| Tool | Description |
|------|-------------|
| `memrl_retrieve` | Search past experiences by semantic similarity |
| `memrl_capture` | Save the current session as an episode |
| `memrl_feedback` | Mark episodes as helpful or not helpful |
| `memrl_stats` | View memory statistics and health |

### Usage Examples

Ask Claude to:
- "Search my past sessions for authentication bugs"
- "What did I learn about React hooks?"
- "Save this session - we fixed the database connection issue"
- "Mark the last retrieved memory as helpful"

## CLI Commands

```bash
# Initialize MemRL
memrl init

# Capture an episode
memrl capture --prompt "Description of what happened"
memrl capture --session ./path/to/session.json --extract-intent

# Index episodes for semantic search
memrl index

# Retrieve similar episodes
memrl retrieve "search query" --limit 5

# Provide feedback
memrl feedback helpful --episodes ep_123,ep_456
memrl feedback not-helpful --last

# Run utility propagation (updates episode values)
memrl propagate
memrl propagate --temporal --project myproject

# Prune low-value episodes
memrl prune --older-than 90 --min-utility 0.3
memrl prune --execute  # Actually delete (default is dry-run)

# View statistics
memrl stats
```

## Configuration

MemRL stores its data in `~/.memrl/`:

```
~/.memrl/
├── config.toml              # Configuration file
├── feedback.log             # Feedback history
├── episodes/                # Episode storage
│   └── YYYY-MM-DD/         # Date-organized
│       └── session-*.json  # Episode files
└── vectors/                 # Vector database
    └── episodes.lance/     # LanceDB storage
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Required for LLM-based intent extraction (`--extract-intent`) |
| `MEMRL_DATA_DIR` | Override default data directory |

## How It Works

### 1. Episode Capture
Sessions are parsed and stored as episodes with:
- Intent/goal description
- Context (project, files, errors)
- Outcome (success/failure)
- Tags for categorization

### 2. Semantic Indexing
Episodes are embedded using BGE-Small-EN-v1.5 (384 dimensions) and stored in LanceDB for fast similarity search.

### 3. Utility Learning
Episodes have a utility score (0.0-1.0) that evolves:
- **Feedback**: Direct user feedback adjusts utility
- **Decay**: Unused episodes slowly lose value
- **Bellman Propagation**: Value spreads to semantically similar episodes
- **Temporal Credit**: Episodes before successful outcomes gain credit

### 4. Retrieval
When you search, MemRL:
1. Embeds your query
2. Finds similar episodes via vector search
3. Ranks by similarity × utility score
4. Returns the most relevant experiences

## Development

### Run tests

```bash
cargo test
```

### Build debug version

```bash
cargo build
```

### Project Structure

```
src/
├── main.rs         # CLI entry point
├── mcp_server.rs   # MCP server binary
├── episode.rs      # Episode data model
├── store.rs        # Storage layer
├── indexer.rs      # Vector indexing with LanceDB
├── retrieve.rs     # Retrieval with semantic search
├── capture.rs      # Session parsing
├── feedback.rs     # Feedback handling
├── utility.rs      # Utility learning algorithms
├── llm.rs          # Anthropic API integration
├── stats.rs        # Statistics and metrics
└── config.rs       # Configuration management
```

## Utility Learning Details

MemRL uses reinforcement learning concepts to manage episode value:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `decay_rate` | 0.01 | 1% utility decay per day for unused episodes |
| `discount_factor` | 0.9 | Standard RL gamma for Bellman propagation |
| `learning_rate` | 0.1 | Conservative alpha for utility updates |
| `propagation_threshold` | 0.5 | Minimum 50% similarity for value spread |

### Propagation Commands

```bash
# Basic Bellman propagation
memrl propagate

# Include temporal credit assignment
memrl propagate --temporal

# Filter by project
memrl propagate --project myproject --temporal
```

## Troubleshooting

### MCP server not loading
1. Check the binary path is correct: `ls /path/to/memrl-mcp`
2. Verify configuration: `cat ~/.claude.json`
3. Restart Claude Code completely
4. Run `/mcp` to see loaded servers

### Embeddings slow on first run
The BGE-Small model (~90MB) downloads on first use. Subsequent runs use the cached model.

### Vector search not working
Run `memrl index` to create/update the vector database after adding new episodes.

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR.
