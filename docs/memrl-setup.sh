#!/bin/bash
# memrl-setup.sh - Initialize the MemRL memory system directories

set -e

MEMRL_DIR="$HOME/.memrl"

echo "🧠 Setting up MemRL Memory System..."
echo ""

# Create main directory
mkdir -p "$MEMRL_DIR"
echo "✓ Created $MEMRL_DIR"

# Create episodes directory
mkdir -p "$MEMRL_DIR/episodes"
echo "✓ Created episodes directory"

# Create today's episode directory
TODAY=$(date +%Y-%m-%d)
mkdir -p "$MEMRL_DIR/episodes/$TODAY"
echo "✓ Created today's directory: $TODAY"

# Initialize feedback log
touch "$MEMRL_DIR/feedback.log"
echo "✓ Initialized feedback log"

# Create config file
if [ ! -f "$MEMRL_DIR/config.toml" ]; then
cat > "$MEMRL_DIR/config.toml" << 'EOF'
# MemRL Configuration
# See: https://github.com/your-repo/memrl

[capture]
# Automatically capture sessions (requires hook setup)
auto_capture = false

# Use LLM to extract intent from sessions
extract_intent_llm = true

# Capture git diffs with each episode
capture_diffs = true

# Default project if not detected
default_project = "misc"

[retrieval]
# Number of episodes to retrieve by default
default_limit = 3

# Minimum similarity score to consider relevant (0.0 - 1.0)
min_similarity = 0.5

# Weight for semantic similarity vs utility score
# Higher = prefer utility, Lower = prefer similarity
utility_weight = 0.7

[utility]
# Initial utility score for new episodes
initial_score = 0.5

# Learning rate for utility updates
learning_rate = 0.1

# Discount factor for Bellman updates (future reward propagation)
gamma = 0.9

[storage]
# Maximum age of episodes to keep (days)
max_age_days = 180

# Minimum utility score to keep during pruning
min_utility_threshold = 0.05

# Path to vector database (Phase 2+)
# vector_db_path = "~/.memrl/vectors"
EOF
echo "✓ Created default config"
else
echo "⏭ Config already exists, skipping"
fi

# Create a README
cat > "$MEMRL_DIR/README.md" << 'EOF'
# MemRL Memory System

Personal coding memory with learned utility scores.

## Quick Start

```bash
# At end of coding session
/memrl-capture

# At start of new task  
/memrl-recall implement feature X

# After completing task
/memrl-feedback helpful
```

## Directory Structure

```
~/.memrl/
├── config.toml          # Settings
├── feedback.log         # Utility tracking
├── episodes/            # Captured sessions
│   ├── 2025-01-23/
│   │   ├── session-abc123.md
│   │   └── session-def456.md
│   └── 2025-01-24/
│       └── ...
└── README.md
```

## Commands

| Command | Description |
|---------|-------------|
| `/memrl-capture` | Save current session as episode |
| `/memrl-recall [task]` | Find relevant past episodes |
| `/memrl-feedback` | Mark episodes as helpful/not |
| `/memrl-list` | Browse episodes |
| `/memrl-show [id]` | View episode details |
| `/memrl-stats` | System statistics |

## How It Works

1. **Capture**: At the end of coding sessions, structured episodes are saved
2. **Recall**: Before new tasks, similar past episodes are retrieved
3. **Feedback**: You mark whether retrieved episodes were helpful
4. **Learn**: Utility scores update based on feedback, improving future retrieval

Based on MemRL: https://arxiv.org/abs/2601.03192
EOF
echo "✓ Created README"

echo ""
echo "🎉 MemRL setup complete!"
echo ""
echo "Directory: $MEMRL_DIR"
echo ""
echo "Next steps:"
echo "1. Copy the slash commands to ~/.claude/commands/"
echo "2. Start using /memrl-capture at the end of sessions"
echo "3. Use /memrl-recall before starting new tasks"
echo ""
