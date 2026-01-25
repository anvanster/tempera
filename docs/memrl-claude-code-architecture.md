# MemRL-Inspired Memory System for Claude Code

## Overview

A lightweight, incremental memory system that learns which past coding experiences are actually useful for solving current problems. Starts as simple session logging, evolves into a value-based retrieval system.

**Core Principle**: Capture everything now, add intelligence later.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                           Developer Workflow                                │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                 │
│   │ Claude Code │────▶│  Hook/Skill │────▶│  Collector  │                 │
│   │   Session   │     │  (capture)  │     │   Service   │                 │
│   └─────────────┘     └─────────────┘     └──────┬──────┘                 │
│                                                   │                        │
│                                                   ▼                        │
│   ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                 │
│   │   Recall    │◀────│  Retriever  │◀────│   Episode   │                 │
│   │   Prompt    │     │   Service   │     │    Store    │                 │
│   └─────────────┘     └─────────────┘     └─────────────┘                 │
│                              │                    │                        │
│                              │              ┌─────┴─────┐                  │
│                              │              │           │                  │
│                              ▼              ▼           ▼                  │
│                       ┌───────────┐   ┌─────────┐ ┌──────────┐            │
│                       │  Vector   │   │ SQLite  │ │ Markdown │            │
│                       │    DB     │   │  (meta) │ │  (logs)  │            │
│                       └───────────┘   └─────────┘ └──────────┘            │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Session Capture (Week 1)

### Goal
Capture structured coding episodes without changing your workflow. Zero friction logging.

### Data Model

```
episodes/
├── 2025-01-23/
│   ├── session-001.md          # Human-readable log
│   ├── session-001.json        # Structured data
│   └── session-001.diff        # Code changes
```

**Episode Schema (JSON)**:
```json
{
  "id": "uuid",
  "timestamp_start": "2025-01-23T14:30:00Z",
  "timestamp_end": "2025-01-23T15:45:00Z",
  "project": "ralph",
  "intent": {
    "raw_prompt": "fix the feedback loop in ralph's validation pipeline",
    "extracted_intent": "debug async validation",
    "task_type": "bugfix",
    "domain": ["async", "validation", "rust"]
  },
  "context": {
    "files_read": ["src/validator.rs", "src/pipeline.rs"],
    "files_modified": ["src/validator.rs"],
    "tools_invoked": ["cargo test", "cargo clippy"],
    "errors_encountered": [
      {
        "type": "compile_error",
        "message": "lifetime mismatch",
        "resolved": true
      }
    ]
  },
  "outcome": {
    "status": "success",
    "tests_before": { "passed": 12, "failed": 3 },
    "tests_after": { "passed": 15, "failed": 0 },
    "commit_sha": "abc123",
    "pr_number": null
  },
  "utility": {
    "score": null,
    "retrievals": 0,
    "helpful_count": 0
  }
}
```

### Implementation: Claude Code Hook

Create a post-session hook that runs after each Claude Code session:

**File: `~/.claude/hooks/post-session.sh`**
```bash
#!/bin/bash
# Triggered after Claude Code session ends

SESSION_LOG="$1"  # Path to session transcript
PROJECT_DIR="$2"  # Current working directory

# Call the collector service
memrl-collect \
  --session "$SESSION_LOG" \
  --project "$PROJECT_DIR" \
  --extract-intent \
  --capture-diff
```

### Implementation: Collector CLI (Rust)

```rust
// src/bin/memrl-collect.rs

use clap::Parser;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    session: PathBuf,
    
    #[arg(long)]
    project: PathBuf,
    
    #[arg(long)]
    extract_intent: bool,
    
    #[arg(long)]
    capture_diff: bool,
}

#[derive(Serialize, Deserialize)]
struct Episode {
    id: String,
    timestamp_start: String,
    timestamp_end: String,
    project: String,
    intent: Intent,
    context: Context,
    outcome: Outcome,
    utility: Utility,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // 1. Parse session transcript
    let transcript = std::fs::read_to_string(&args.session)?;
    
    // 2. Extract structured data
    let episode = Episode {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp_start: extract_start_time(&transcript),
        timestamp_end: Utc::now().to_rfc3339(),
        project: extract_project_name(&args.project),
        intent: extract_intent(&transcript, args.extract_intent),
        context: extract_context(&transcript, &args.project),
        outcome: extract_outcome(&args.project),
        utility: Utility::default(),
    };
    
    // 3. Save to episodes directory
    let episodes_dir = dirs::data_dir()
        .unwrap()
        .join("memrl")
        .join("episodes")
        .join(Utc::now().format("%Y-%m-%d").to_string());
    
    std::fs::create_dir_all(&episodes_dir)?;
    
    let episode_path = episodes_dir.join(format!("session-{}.json", &episode.id[..8]));
    std::fs::write(&episode_path, serde_json::to_string_pretty(&episode)?)?;
    
    // 4. Also save human-readable markdown
    let md_path = episode_path.with_extension("md");
    std::fs::write(&md_path, episode_to_markdown(&episode))?;
    
    // 5. Capture diff if requested
    if args.capture_diff {
        let diff = capture_git_diff(&args.project)?;
        let diff_path = episode_path.with_extension("diff");
        std::fs::write(&diff_path, diff)?;
    }
    
    println!("📝 Episode captured: {}", episode_path.display());
    
    Ok(())
}

fn extract_intent(transcript: &str, use_llm: bool) -> Intent {
    if use_llm {
        // Quick Claude API call to extract intent
        // Keep it simple: just the first user message + classification
        extract_intent_with_llm(transcript)
    } else {
        // Regex-based extraction of first prompt
        Intent {
            raw_prompt: extract_first_prompt(transcript),
            extracted_intent: String::new(),
            task_type: "unknown".into(),
            domain: vec![],
        }
    }
}

fn extract_context(transcript: &str, project: &PathBuf) -> Context {
    Context {
        files_read: extract_file_reads(transcript),
        files_modified: get_modified_files(project),
        tools_invoked: extract_tool_calls(transcript),
        errors_encountered: extract_errors(transcript),
    }
}

fn extract_outcome(project: &PathBuf) -> Outcome {
    // Check git status, test results, etc.
    let commit_sha = get_latest_commit(project);
    let test_results = run_test_check(project);
    
    Outcome {
        status: if test_results.failed == 0 { "success" } else { "partial" }.into(),
        tests_before: None, // Would need pre-session snapshot
        tests_after: Some(test_results),
        commit_sha,
        pr_number: None,
    }
}
```

### Alternative: Claude Code Skill (Simpler Start)

If you want zero external tooling initially:

**File: `~/.claude/skills/memrl-capture/SKILL.md`**
```markdown
# MemRL Episode Capture Skill

## Purpose
Capture structured coding episodes for memory-based learning.

## Trigger
Run at the END of significant coding sessions via: `/memrl-capture`

## Behavior
When triggered, create an episode log:

1. Extract the main intent from this session's conversation
2. List all files that were read or modified
3. Summarize errors encountered and how they were resolved
4. Assess outcome: success/partial/failure
5. Save to `~/.memrl/episodes/YYYY-MM-DD/session-HHmm.md`

## Episode Format
```
# Episode: [extracted intent summary]

**Date**: [timestamp]
**Project**: [current directory name]
**Duration**: [estimated from conversation]

## Intent
[The core task/problem being solved]

## Files Touched
- Read: [list]
- Modified: [list]

## Key Decisions
- [decision 1]
- [decision 2]

## Errors & Resolutions
- [error] → [resolution]

## Outcome
Status: [success/partial/failure]
Commit: [sha if any]

## Tags
[auto-generated: rust, async, debugging, etc.]
```
```

### Phase 1 Deliverables
- [ ] Episode capture (hook or skill)
- [ ] JSON + Markdown storage
- [ ] Git diff capture
- [ ] Basic intent extraction

---

## Phase 2: Semantic Indexing (Week 2-3)

### Goal
Enable "find similar past episodes" via embedding search.

### Vector Database Choice

**Recommended: LanceDB** (embedded, Rust-native, no server)

```rust
// Cargo.toml
[dependencies]
lancedb = "0.4"
arrow = "50"
```

### Embedding Strategy

**Option A: Local (fast, private)**
- Use `fastembed-rs` with `BAAI/bge-small-en-v1.5`
- ~33M params, runs on CPU
- Good enough for personal use

**Option B: API (higher quality)**
- Claude's embeddings or OpenAI's `text-embedding-3-small`
- Better semantic understanding
- Adds latency + cost

### Index Schema

```rust
use lancedb::prelude::*;

#[derive(Debug, Clone, LanceRecord)]
struct EpisodeIndex {
    id: String,
    
    // Embeddings (384-dim for bge-small)
    intent_embedding: Vec<f32>,
    context_embedding: Vec<f32>,  // files + tools combined
    
    // Searchable metadata
    project: String,
    task_type: String,
    domains: Vec<String>,
    timestamp: i64,
    
    // Utility scores (Phase 3)
    utility_score: f32,
    retrieval_count: i32,
    helpful_count: i32,
}
```

### Indexing Pipeline

```rust
// src/indexer.rs

use fastembed::{TextEmbedding, EmbeddingModel};
use lancedb::{Connection, Table};

pub struct EpisodeIndexer {
    db: Connection,
    table: Table,
    embedder: TextEmbedding,
}

impl EpisodeIndexer {
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let db = lancedb::connect(db_path).execute().await?;
        
        let embedder = TextEmbedding::try_new(
            EmbeddingModel::BGESmallENV15
        )?;
        
        let table = db.open_table("episodes").execute().await
            .unwrap_or_else(|_| {
                // Create table if not exists
                db.create_table("episodes", EpisodeIndex::schema())
                    .execute()
                    .await
                    .unwrap()
            });
        
        Ok(Self { db, table, embedder })
    }
    
    pub async fn index_episode(&self, episode: &Episode) -> anyhow::Result<()> {
        // Generate embeddings
        let intent_text = format!(
            "{} {}",
            episode.intent.raw_prompt,
            episode.intent.extracted_intent
        );
        
        let context_text = format!(
            "files: {} tools: {} errors: {}",
            episode.context.files_modified.join(", "),
            episode.context.tools_invoked.join(", "),
            episode.context.errors_encountered
                .iter()
                .map(|e| &e.message)
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        let intent_emb = self.embedder.embed(vec![intent_text], None)?[0].clone();
        let context_emb = self.embedder.embed(vec![context_text], None)?[0].clone();
        
        let record = EpisodeIndex {
            id: episode.id.clone(),
            intent_embedding: intent_emb,
            context_embedding: context_emb,
            project: episode.project.clone(),
            task_type: episode.intent.task_type.clone(),
            domains: episode.intent.domain.clone(),
            timestamp: episode.timestamp_start.parse::<DateTime<Utc>>()?.timestamp(),
            utility_score: 0.0,
            retrieval_count: 0,
            helpful_count: 0,
        };
        
        self.table.add(vec![record]).execute().await?;
        
        Ok(())
    }
}
```

### Retrieval (Phase A of MemRL)

```rust
// src/retriever.rs

pub struct EpisodeRetriever {
    indexer: EpisodeIndexer,
    episode_store: PathBuf,
}

impl EpisodeRetriever {
    /// Phase A: Semantic similarity search
    pub async fn find_similar(
        &self, 
        query: &str, 
        limit: usize,
        project_filter: Option<&str>,
    ) -> anyhow::Result<Vec<RetrievedEpisode>> {
        let query_emb = self.indexer.embedder.embed(vec![query.to_string()], None)?[0].clone();
        
        let mut search = self.indexer.table
            .search(&query_emb)
            .column("intent_embedding")
            .limit(limit * 2);  // Fetch extra for Phase B filtering
        
        if let Some(project) = project_filter {
            search = search.filter(format!("project = '{}'", project));
        }
        
        let results = search.execute().await?;
        
        // Load full episodes
        let episodes: Vec<RetrievedEpisode> = results
            .iter()
            .filter_map(|r| self.load_episode(&r.id).ok())
            .map(|ep| RetrievedEpisode {
                episode: ep,
                similarity_score: r.score,
            })
            .collect();
        
        Ok(episodes)
    }
}
```

### Phase 2 Deliverables
- [ ] LanceDB integration
- [ ] Embedding generation (local)
- [ ] Index all existing episodes
- [ ] Basic similarity search CLI

---

## Phase 3: Binary Utility Tracking (Week 3-4)

### Goal
Track whether retrieved episodes were actually helpful. Simple thumbs up/down.

### Feedback Loop

```
┌─────────────────────────────────────────────────────────────┐
│                    Coding Session                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. New task arrives                                        │
│     └─► "implement rate limiting for API"                   │
│                                                              │
│  2. Retriever finds similar episodes                        │
│     └─► Returns 3 past episodes about rate limiting         │
│                                                              │
│  3. Episodes injected into context                          │
│     └─► Added to CLAUDE.md or skill prompt                  │
│                                                              │
│  4. Session completes                                       │
│                                                              │
│  5. Developer feedback (quick prompt)                       │
│     └─► "Were the suggested memories helpful? [y/n/skip]"   │
│                                                              │
│  6. Update utility scores                                   │
│     └─► helpful_count++ or no change                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Utility Update Logic

```rust
// src/utility.rs

#[derive(Debug, Clone)]
pub struct UtilityTracker {
    db: Connection,
}

impl UtilityTracker {
    /// Called when episodes are retrieved and shown to user
    pub async fn record_retrieval(&self, episode_ids: &[String]) -> anyhow::Result<()> {
        for id in episode_ids {
            sqlx::query!(
                "UPDATE episodes SET retrieval_count = retrieval_count + 1 WHERE id = ?",
                id
            )
            .execute(&self.db)
            .await?;
        }
        Ok(())
    }
    
    /// Called when user marks episodes as helpful
    pub async fn record_helpful(&self, episode_ids: &[String]) -> anyhow::Result<()> {
        for id in episode_ids {
            sqlx::query!(
                "UPDATE episodes SET helpful_count = helpful_count + 1 WHERE id = ?",
                id
            )
            .execute(&self.db)
            .await?;
            
            // Recalculate utility score
            self.update_utility_score(id).await?;
        }
        Ok(())
    }
    
    /// Simple utility: helpful_rate with confidence adjustment
    async fn update_utility_score(&self, episode_id: &str) -> anyhow::Result<()> {
        let episode = sqlx::query!(
            "SELECT retrieval_count, helpful_count FROM episodes WHERE id = ?",
            episode_id
        )
        .fetch_one(&self.db)
        .await?;
        
        // Wilson score interval (lower bound) for ranking
        // Handles low-sample uncertainty
        let n = episode.retrieval_count as f64;
        let p = if n > 0.0 { 
            episode.helpful_count as f64 / n 
        } else { 
            0.0 
        };
        
        let z = 1.96; // 95% confidence
        let score = if n > 0.0 {
            (p + z*z/(2.0*n) - z * ((p*(1.0-p) + z*z/(4.0*n))/n).sqrt()) 
            / (1.0 + z*z/n)
        } else {
            0.0
        };
        
        sqlx::query!(
            "UPDATE episodes SET utility_score = ? WHERE id = ?",
            score,
            episode_id
        )
        .execute(&self.db)
        .await?;
        
        Ok(())
    }
}
```

### Two-Phase Retrieval (Full MemRL)

```rust
// src/retriever.rs

impl EpisodeRetriever {
    /// Full MemRL retrieval: semantic + utility filtering
    pub async fn retrieve_for_task(
        &self,
        task_description: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<RetrievedEpisode>> {
        // Phase A: Semantic similarity (get candidates)
        let candidates = self.find_similar(task_description, limit * 3, None).await?;
        
        // Phase B: Utility-weighted selection
        let mut scored: Vec<_> = candidates
            .into_iter()
            .map(|ep| {
                // Combine similarity and utility
                // α controls exploration vs exploitation
                let alpha = 0.7;  // Weight toward utility
                let combined = (1.0 - alpha) * ep.similarity_score 
                             + alpha * ep.episode.utility.score.unwrap_or(0.5);
                (ep, combined)
            })
            .collect();
        
        // Sort by combined score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top N
        let selected: Vec<_> = scored
            .into_iter()
            .take(limit)
            .map(|(ep, _)| ep)
            .collect();
        
        // Record retrieval
        let ids: Vec<_> = selected.iter().map(|e| e.episode.id.clone()).collect();
        self.utility_tracker.record_retrieval(&ids).await?;
        
        Ok(selected)
    }
}
```

### Integration: Pre-Session Recall Skill

**File: `~/.claude/skills/memrl-recall/SKILL.md`**
```markdown
# MemRL Recall Skill

## Purpose
Retrieve relevant past coding experiences before starting a new task.

## Trigger
Automatically at session start, or via: `/memrl-recall [task description]`

## Behavior
1. Take the task description (from first user message or explicit input)
2. Call `memrl-retrieve --query "[task]" --limit 3`
3. Format retrieved episodes as context
4. Inject into current session

## Output Format
When relevant episodes are found, add to context:

```
## Relevant Past Experiences

### Experience 1: [intent summary]
**When**: [date]
**Outcome**: [success/partial/failure]
**Key insight**: [main takeaway]
**Files involved**: [list]

### Experience 2: ...
```

## Post-Session
After session ends, prompt:
"Were the suggested memories helpful for this task? [y/n/skip]"

If yes, run: `memrl-feedback --helpful [episode-ids]`
```

### Phase 3 Deliverables
- [ ] SQLite for utility metadata
- [ ] Retrieval count tracking
- [ ] Helpful/not-helpful feedback loop
- [ ] Wilson score utility calculation
- [ ] Two-phase retrieval (semantic + utility)
- [ ] Pre-session recall skill

---

## Phase 4: Bellman Updates (Week 5+)

### Goal
Propagate success/failure signals through chains of related episodes.

### The Intuition

If Episode A helped solve Task X, and Task X's solution helped solve Task Y, then Episode A should get partial credit for Task Y's success.

### Memory Graph

```
Episode A ──retrieved_for──▶ Session 1 ──produced──▶ Episode B
                                                          │
                                                          │ retrieved_for
                                                          ▼
                                                     Session 2 ──produced──▶ Episode C
                                                                                  │
                                                                                  │ marked_helpful
                                                                                  ▼
                                                                              Success!
```

Credit propagates: C → B → A

### Bellman Update

```rust
// src/bellman.rs

pub struct BellmanUpdater {
    gamma: f64,  // Discount factor (0.9 typical)
}

impl BellmanUpdater {
    /// Update utility based on outcome and retrieved episodes
    pub async fn update(
        &self,
        db: &Connection,
        session_id: &str,
        outcome_reward: f64,  // 1.0 for success, 0.0 for failure, 0.5 for partial
        retrieved_episode_ids: &[String],
    ) -> anyhow::Result<()> {
        // Get current episode (result of this session)
        let current_utility = self.get_utility(db, session_id).await?;
        
        // Bellman update for each retrieved episode
        for episode_id in retrieved_episode_ids {
            let old_utility = self.get_utility(db, episode_id).await?;
            
            // Q(s,a) ← Q(s,a) + α * (r + γ * max Q(s',a') - Q(s,a))
            // Simplified: we use the outcome as reward, current episode utility as future value
            let alpha = 0.1;  // Learning rate
            let target = outcome_reward + self.gamma * current_utility;
            let new_utility = old_utility + alpha * (target - old_utility);
            
            self.set_utility(db, episode_id, new_utility).await?;
        }
        
        // Also update current episode's utility
        let current_old = self.get_utility(db, session_id).await?;
        let new_current = current_old + 0.1 * (outcome_reward - current_old);
        self.set_utility(db, session_id, new_current).await?;
        
        Ok(())
    }
    
    /// Batch update: propagate through entire graph periodically
    pub async fn propagate_all(&self, db: &Connection) -> anyhow::Result<()> {
        // Get all episodes ordered by timestamp (newest first)
        let episodes = sqlx::query!(
            "SELECT id, utility_score FROM episodes ORDER BY timestamp DESC"
        )
        .fetch_all(db)
        .await?;
        
        // Backward pass: propagate from recent to old
        for episode in &episodes {
            // Find episodes that were retrieved for this one
            let parents = sqlx::query!(
                "SELECT episode_id FROM retrievals WHERE session_id = ?",
                episode.id
            )
            .fetch_all(db)
            .await?;
            
            if parents.is_empty() {
                continue;
            }
            
            // Update parent utilities
            for parent in parents {
                let parent_utility = self.get_utility(db, &parent.episode_id).await?;
                let target = self.gamma * episode.utility_score;
                let new_utility = parent_utility + 0.05 * (target - parent_utility);
                self.set_utility(db, &parent.episode_id, new_utility).await?;
            }
        }
        
        Ok(())
    }
}
```

### Tracking Retrieval Chains

New table to track which episodes were retrieved for which sessions:

```sql
CREATE TABLE retrievals (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,      -- The session that did the retrieval
    episode_id TEXT NOT NULL,      -- The episode that was retrieved
    timestamp INTEGER NOT NULL,
    was_helpful BOOLEAN,
    FOREIGN KEY (session_id) REFERENCES episodes(id),
    FOREIGN KEY (episode_id) REFERENCES episodes(id)
);

CREATE INDEX idx_retrievals_session ON retrievals(session_id);
CREATE INDEX idx_retrievals_episode ON retrievals(episode_id);
```

### Phase 4 Deliverables
- [ ] Retrieval chain tracking
- [ ] Bellman update on session completion
- [ ] Periodic batch propagation
- [ ] Utility decay for old episodes

---

## CLI Interface

```bash
# Capture current session
memrl capture --session /path/to/transcript --project .

# Index all pending episodes
memrl index

# Retrieve for a task (used by skill)
memrl retrieve --query "implement OAuth2 flow" --limit 3 --format markdown

# Record feedback
memrl feedback --helpful session-abc123,session-def456
memrl feedback --not-helpful session-xyz789

# View episode history
memrl list --project ralph --limit 10
memrl show session-abc123

# Stats
memrl stats
memrl stats --project ralph

# Maintenance
memrl reindex          # Rebuild vector index
memrl propagate        # Run Bellman updates
memrl prune --older-than 6months --min-utility 0.1
```

---

## File Structure

```
~/.memrl/
├── config.toml              # Settings
├── memrl.db                 # SQLite (metadata + utilities)
├── vectors/                 # LanceDB directory
│   └── episodes.lance
├── episodes/                # Raw episode data
│   ├── 2025-01-23/
│   │   ├── session-abc123.json
│   │   ├── session-abc123.md
│   │   └── session-abc123.diff
│   └── 2025-01-24/
│       └── ...
└── logs/
    └── memrl.log
```

### Config

```toml
# ~/.memrl/config.toml

[capture]
auto_capture = true          # Hook into Claude Code automatically
extract_intent_llm = true    # Use LLM for intent extraction
capture_diffs = true

[embedding]
model = "bge-small-en-v1.5"  # or "openai:text-embedding-3-small"
batch_size = 32

[retrieval]
default_limit = 3
similarity_weight = 0.3
utility_weight = 0.7
min_similarity = 0.5

[bellman]
gamma = 0.9                  # Discount factor
alpha = 0.1                  # Learning rate
propagate_interval = "daily"

[prune]
max_age_days = 180
min_utility_threshold = 0.05
min_retrievals = 2
```

---

## Quick Start (Today)

### Minimal Viable Version (1 hour)

1. Create the skill for manual capture:

```bash
mkdir -p ~/.claude/commands
```

Create `~/.claude/commands/memrl-capture.md`:
```markdown
---
description: Capture this session as a memory episode
---

Create a structured summary of this coding session and save it.

1. Identify the main task/intent from our conversation
2. List files that were read or modified
3. Note any errors encountered and how they were resolved
4. Assess the outcome (success/partial/failure)
5. Generate relevant tags

Save to: `~/.memrl/episodes/[today's date]/session-[time].md`

Use this format:
```
# Episode: [one-line intent]

**Date**: [timestamp]
**Project**: [directory name]
**Outcome**: [success/partial/failure]

## Intent
[2-3 sentence description of the task]

## Context
- Files read: [list]
- Files modified: [list]
- Tools used: [list]

## Key Decisions
- [decision and reasoning]

## Errors → Solutions
- [error] → [how it was fixed]

## Tags
[comma-separated: rust, async, debugging, api, etc.]
```

After saving, show me the file path.
```

2. Create recall command:

Create `~/.claude/commands/memrl-recall.md`:
```markdown
---
description: Find relevant past experiences for current task
---

Search through `~/.memrl/episodes/` for sessions relevant to my current task.

Read the episode files and find ones that match:
- Similar intent/task type
- Same project (if applicable)
- Similar files or error patterns

Return the top 3 most relevant episodes in this format:

## Relevant Past Experiences

### [Episode intent]
**When**: [date]
**Outcome**: [success/partial/failure]
**Relevance**: [why this might help]
**Key insight**: [main takeaway that could help now]
```

3. Use it:
```bash
# At end of session
/memrl-capture

# At start of new session
/memrl-recall implement rate limiting for the API
```

This gets you capturing and recalling immediately. The vector DB and utility learning come later.

---

## Integration with Ralph

Since Ralph is your autonomous coding system, the memory layer could:

1. **Auto-capture**: Every Ralph session automatically logged
2. **Pre-task retrieval**: Before Ralph starts, inject relevant memories
3. **Feedback loop**: Ralph's test pass/fail becomes automatic utility signal

```rust
// ralph/src/memory.rs

impl Ralph {
    async fn execute_task(&mut self, task: &Task) -> Result<Outcome> {
        // Pre-task: retrieve relevant memories
        let memories = self.memrl.retrieve(&task.description, 3).await?;
        
        // Inject into context
        self.context.add_memories(&memories);
        
        // Execute task
        let outcome = self.run_claude_code(task).await?;
        
        // Post-task: capture episode
        let episode = self.memrl.capture_session(&self.transcript, &outcome).await?;
        
        // Update utilities based on outcome
        let reward = match outcome.status {
            Status::Success => 1.0,
            Status::Partial => 0.5,
            Status::Failure => 0.0,
        };
        
        self.memrl.update_utilities(&memories, reward).await?;
        
        Ok(outcome)
    }
}
```

---

## Success Metrics

Track these to know if the memory system is working:

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Episodes captured | >50 in first month | Count files |
| Retrieval accuracy | >70% helpful | Feedback tracking |
| Time to fix recurring bugs | -50% | Compare similar episodes |
| Context tokens saved | -30% | Track what's loaded |
| Utility score correlation | >0.6 with success | Statistical analysis |

---

## Next Steps

1. **Today**: Create manual capture/recall skills
2. **This week**: Build Rust CLI for structured capture
3. **Next week**: Add LanceDB + embeddings
4. **Week 3**: Implement feedback loop
5. **Week 4**: Add Bellman updates
6. **Ongoing**: Tune weights, prune old episodes, analyze patterns
