# Tempera - Persistent Memory for Claude Code

Tempera gives Claude Code a persistent memory that learns from experience. Instead of starting fresh each session, Claude can recall past solutions, learn what works, and get smarter over time.

## Why Tempera?

**The Problem**: Claude Code forgets everything between sessions. You solve the same problems repeatedly, and Claude can't learn from past successes or failures.

**The Solution**: Tempera captures coding sessions as "episodes", indexes them for semantic search, and uses reinforcement learning to surface the most valuable memories when relevant.

```
Without Tempera:                    With Tempera:
┌─────────────┐                  ┌─────────────┐
│  Session 1  │ ──forgotten──>   │  Session 1  │ ──captured──┐
└─────────────┘                  └─────────────┘             │
┌─────────────┐                  ┌─────────────┐             ▼
│  Session 2  │ ──forgotten──>   │  Session 2  │ ◄──recalls──┤
└─────────────┘                  └─────────────┘             │
┌─────────────┐                  ┌─────────────┐             │
│  Session 3  │ ──forgotten──>   │  Session 3  │ ◄──recalls──┘
└─────────────┘                  └─────────────┘
     │                                 │
     ▼                                 ▼
  No learning                    Continuous improvement
```

## How It Works

### The Learning Loop

```
┌────────────────────────────────────────────────────────────────┐
│  1. START TASK                                                 │
│     User: "Fix the login redirect bug"                         │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│  2. RETRIEVE MEMORIES                                          │
│     Claude searches: "login redirect bug"                      │
│     Finds: "Fixed similar issue by sanitizing return URLs"     │
│     + Session context: related episodes from the same task     │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│  3. SOLVE FASTER                                               │
│     Claude uses past experience to solve the problem           │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│  4. CAPTURE SESSION                                            │
│     Claude saves: what was done, what worked, what failed      │
│     Auto-links to current session for multi-step tasks         │
└────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────┐
│  5. LEARN FROM FEEDBACK                                        │
│     User: "That memory was helpful!"                           │
│     → Episode utility increases                                │
│     → Multi-hop Bellman propagation spreads value               │
│     → Session-linked episodes get boosted                      │
│     → Unhelpful memories fade over time                        │
└────────────────────────────────────────────────────────────────┘
```

### What Makes It "Learn"

| Mechanism | What It Does |
|-----------|--------------|
| **Feedback** | Helpful episodes gain utility score |
| **Multi-hop Bellman Propagation** | Value spreads through the similarity graph across multiple hops |
| **Session Chaining** | Related episodes in multi-step tasks are linked and boost each other |
| **Temporal Credit** | Episodes before successes get credit (even across session boundaries) |
| **Recency Boost** | Fresh episodes can be weighted higher in retrieval (opt-in) |
| **Decay** | Unused memories fade (configurable, default 1% per day) |
| **Retrieval Ranking** | Combined similarity + utility + recency scoring with weight normalization |
| **BKM Consolidation** | Similar captures are merged instead of duplicated |

Over time, frequently helpful knowledge rises to the top, while stale or unhelpful memories fade away.

## Installation

### Build from Source

```bash
# Clone and build
git clone https://github.com/anvanster/tempera.git
cd tempera
cargo build --release

# Two binaries are created:
# - target/release/tempera      (CLI tool)
# - target/release/tempera-mcp  (MCP server for Claude Code)
```

### Install from crates.io

```bash
cargo install tempera
```

### First Run - Model Download

On first use, Tempera downloads the BGE-Small embedding model (~128MB) for semantic search. This happens automatically and only once:

```bash
# Initialize and trigger model download
tempera init

# Output:
# 🔄 Loading embedding model (this may download the model on first run)...
# ✅ Embedding model loaded
```

The model is cached globally at `~/.tempera/models/` and shared across all projects.

## Setup with Claude Code

### 1. Add the MCP Server

```bash
claude mcp add tempera --scope user -- /path/to/Tempera/target/release/tempera-mcp
```

The `--scope user` flag makes it available across all your projects.

### 2. Restart Claude Code

Exit and restart Claude Code to load the new MCP server.

### 3. Verify

Run `/mcp` in Claude Code. You should see `tempera` with 7 tools.

## MCP Tools

Once connected, Claude has access to these tools:

| Tool | Description | When to Use |
|------|-------------|-------------|
| `tempera_retrieve` | Search memories by query, list all, or show details. Surfaces session context for linked episodes. | **Start of session** - always check first |
| `tempera_capture` | Save session as episode. Auto-detects session links and runs propagation. | **End of task** - capture successes proactively |
| `tempera_feedback` | Mark episodes as helpful/not helpful | After using retrieved memories |
| `tempera_status` | Check memory health for current project | Understand memory state |
| `tempera_stats` | View statistics or trend analytics (helpfulness over time, domain growth, learning curve) | Analytics and monitoring |
| `tempera_propagate` | Multi-hop Bellman propagation with convergence tracking | Periodic maintenance |
| `tempera_review` | Consolidate and cleanup memories | After related task series |

### Key Lifecycle Behaviors

**Start of session**: Claude should automatically check for relevant memories using `tempera_retrieve` before starting non-trivial tasks.

**End of task**: Claude should proactively capture successful sessions using `tempera_capture` - don't wait to be asked. The capture tool automatically runs utility propagation.

**Session chaining**: Episodes captured within 2 hours of each other in the same project are automatically linked into a session. This allows multi-step tasks to share utility and context.

**Cross-project insights**: Use `tempera_capture(project: "other-project")` to tag memories for a different project.

### tempera_retrieve Modes

```
Search:       tempera_retrieve(query: "authentication bug")
List all:     tempera_retrieve(all: true)
Show details: tempera_retrieve(query: "abc12345")  # episode ID
```

## Example Conversation

```
User: Fix the login redirect vulnerability

Claude: Let me check if we've solved similar problems before...
        [Calls tempera_retrieve("login redirect vulnerability")]

        Found 2 relevant past experiences:
        1. "Fixed open redirect by validating return URLs against whitelist"
           - 78% similarity, 85% utility
        2. "Auth redirect issue - added URL sanitization"
           - 65% similarity, 72% utility
        📎 Session abc1 has 2 related episode(s):
           - def45678 (Added CSP headers for redirect...)

        Based on past experience, I'll implement URL validation...
        [Works on the fix]

        Done! Let me save this session.
        [Calls tempera_capture(summary: "Fixed login redirect...", outcome: "success")]

User: That retrieved memory was really helpful!

Claude: [Calls tempera_feedback(helpful: true, episodes: ["abc123"])]
        Marked as helpful - this will improve future retrievals!
```

## CLI Commands

```bash
# Initialize Tempera
tempera init

# Capture an episode manually
tempera capture --prompt "Fixed the authentication bug"

# Index episodes for semantic search
tempera index

# Search memories
tempera retrieve "database connection issues"

# Provide feedback
tempera feedback helpful --episodes abc123,def456

# Run utility propagation (now with multi-hop and convergence tracking)
tempera propagate --temporal

# Prune old/low-value episodes (uses config.toml fallbacks when flags omitted)
tempera prune --older-than 90 --min-utility 0.2 --execute

# View statistics
tempera stats

# View trend analytics (helpfulness over time, domain growth, learning curve)
tempera trends
tempera trends --project myproject --bucket monthly
```

## Data Storage

Tempera stores everything locally in `~/.tempera/` (shared across all projects):

```
~/.tempera/
├── config.toml              # Configuration (all RL params configurable)
├── episodes/                # Episode JSON files
│   └── 2026-01-25/
│       └── session-abc123.json
├── vectors/                 # Vector database (vectrust/RocksDB)
│   └── episodes/
└── models/                  # Embedding model cache (~128MB)
    └── models--Xenova--bge-small-en-v1.5/
```

All projects share the same memory database, enabling cross-project learning.

## Configuration

All RL parameters are configurable via `~/.tempera/config.toml`:

```toml
[retrieval]
similarity_weight = 0.3        # Weight for semantic similarity
utility_weight = 0.7           # Weight for learned utility
recency_weight = 0.0           # Weight for recency (0 = off, opt-in)
recency_halflife_days = 30.0   # Episodes score 0.5 at this age
mmr_lambda = 0.7               # MMR diversity (0=diverse, 1=relevant)
min_similarity = 0.5           # Filter threshold

[bellman]
gamma = 0.9                    # Discount factor for Bellman updates
alpha = 0.1                    # Learning rate
decay_rate = 0.01              # Utility decay per day (1%)
propagation_threshold = 0.5    # Min similarity for propagation
max_propagation_depth = 2      # Multi-hop depth (hops)
temporal_credit_window_hours = 1  # Lookback for temporal credit

[storage]
max_age_days = 180             # Max episode age for pruning
min_utility_threshold = 0.05   # Min utility to keep
min_retrievals = 2             # Min retrievals before pruning allowed
consolidation_threshold = 0.85 # BKM merge threshold
cluster_threshold = 0.85       # Duplicate clustering threshold
stale_age_days = 30            # Age threshold for stale detection
stale_utility_threshold = 0.2  # Utility threshold for stale detection
```

## The RL Behind the Scenes

### Multi-hop Bellman Propagation

Value from helpful episodes spreads through the similarity graph in multiple hops:

```
Hop 0: Source episodes (high helpfulness, ≥2 retrievals)
  │
  ▼  γ¹ discount
Hop 1: Similar episodes updated
  │
  ▼  γ² discount
Hop 2: Episodes similar to hop-1 updated
  │
  ▼  Converges when no updates occur
```

### Session Chaining

Episodes captured within 2 hours of each other in the same project are automatically linked:

```
Session abc123:
  ├── Episode 1: "Investigated auth bug" (debug)
  ├── Episode 2: "Found root cause in token validation" (research)
  └── Episode 3: "Fixed token expiry check" (bugfix, success)
       ↓
  Temporal credit flows back to episodes 1 & 2
  Session-linked propagation boosts all 3
```

### Scoring Formula

Retrieval ranking combines three signals with normalized weights:

```
score = (sim_w × similarity + util_w × utility + rec_w × recency) / (sim_w + util_w + rec_w)
```

Default: 30% similarity, 70% utility, 0% recency (recency is opt-in via config).

## Maintenance

Run periodically to keep memory healthy:

```bash
# Weekly: Propagate utility values (now multi-hop with convergence)
tempera propagate --temporal

# Monthly: Clean up old/useless episodes
tempera prune --older-than 90 --min-utility 0.2 --execute

# As needed: Check trends
tempera trends

# As needed: Review and consolidate
# (via MCP) tempera_review(action: "consolidate")
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | For LLM-based intent extraction (`--extract-intent`) |
| `TEMPERA_DATA_DIR` | Override default data directory |
| `FASTEMBED_CACHE_DIR` | Override embedding model cache location |

## Troubleshooting

### MCP server not loading
1. Check path: `ls /path/to/tempera-mcp`
2. Check config: `cat ~/.claude.json`
3. Restart Claude Code completely
4. Run `/mcp` to verify

### Embeddings slow on first run
The BGE-Small model (~128MB) downloads on first use from HuggingFace. This requires internet access. After download, the model is cached at `~/.tempera/models/` and works offline.

### Vector search not finding anything
Run `tempera index` to create/update the vector database.

### Model download fails
If behind a firewall or proxy, ensure access to `huggingface.co`. The model files are downloaded via HTTPS.

## License

Apache 2.0

## Contributing

Contributions welcome! Please open an issue or PR.
