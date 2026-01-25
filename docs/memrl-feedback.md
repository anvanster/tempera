---
description: Mark past episodes as helpful or not helpful for utility tracking
allowed-tools: Read, Write, Glob
---

Record feedback on whether retrieved episodes were helpful for the current task.

## Purpose

This feedback trains the memory system to surface more useful episodes over time.
Episodes marked as "helpful" get higher utility scores and are more likely to be retrieved in the future.

## Usage

Called automatically after `/memrl-recall` at end of session, or manually:

- `/memrl-feedback helpful` - The retrieved episodes were useful
- `/memrl-feedback not-helpful` - The retrieved episodes weren't relevant
- `/memrl-feedback mixed` - Some were helpful, some weren't

## Instructions

1. **Identify Retrieved Episodes**: Look for episodes that were surfaced via `/memrl-recall` in this session

2. **Update Utility Tracking**: Append feedback to a tracking file

3. **Update Episode Files**: Add retrieval/feedback metadata to the episode files

## Tracking File

Append to `~/.memrl/feedback.log`:

```
{{ISO timestamp}}|{{feedback_type}}|{{episode_ids comma-separated}}|{{current_project}}|{{brief_task_description}}
```

Example:
```
2025-01-23T15:30:00Z|helpful|abc123,def456|ralph|implement validation pipeline
2025-01-23T16:45:00Z|not-helpful|xyz789|crucible|setup database migrations
```

## Episode Metadata Update

For each episode that was retrieved, update its file to add/increment:

At the bottom of the episode markdown file, add or update a `## Retrieval History` section:

```markdown
## Retrieval History

| Date | Project | Task | Helpful |
|------|---------|------|---------|
| 2025-01-23 | ralph | validation pipeline | ✅ |
| 2025-01-24 | crucible | error handling | ❌ |
```

## Utility Score Calculation (for reference)

The utility score will eventually be calculated as:
```
utility = helpful_count / retrieval_count
```

With Wilson score adjustment for confidence:
- Episodes with few retrievals get neutral scores (~0.5)
- Episodes with many helpful retrievals get high scores (~0.9)
- Episodes with many unhelpful retrievals get low scores (~0.1)

## Output

Confirm with:
```
📊 Feedback recorded:
- Episodes: {{list}}
- Feedback: {{helpful/not-helpful/mixed}}
- Task: {{brief description}}

This helps improve future episode retrieval. Thank you!
```

## If No Episodes Were Retrieved

If `/memrl-recall` wasn't used in this session:
```
No episodes were retrieved this session. Use `/memrl-recall` at the start of a task to get relevant past experiences.
```
