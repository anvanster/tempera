---
description: Retrieve relevant past coding experiences for the current task
allowed-tools: Read, Glob, Grep
---

Search through past coding episodes to find experiences relevant to the current task.

## Instructions

1. **Understand Current Task**: Look at the conversation context or use the provided argument as the task description

2. **Search Episodes**: Look through `~/.memrl/episodes/` directories

3. **Match Criteria** (in order of importance):
   - Similar intent/problem type
   - Same or related project
   - Similar file patterns
   - Similar error types
   - Matching tags

4. **Rank Results**: Prefer:
   - Recent episodes (recency)
   - Successful outcomes (proven solutions)
   - High relevance to current task

5. **Extract Insights**: Pull out the most useful information for the current task

## Search Strategy

```bash
# First, list available episodes
ls ~/.memrl/episodes/

# Read episode files and grep for relevant terms
grep -r "{{relevant keywords}}" ~/.memrl/episodes/
```

## Output Format

Return the top 3 most relevant episodes:

---

## 🧠 Relevant Past Experiences

### 1. {{Episode intent}}
**When**: {{date}}  
**Project**: {{project name}}  
**Outcome**: {{success/partial/failure}} {{✅ or ⚠️ or ❌}}

**Why relevant**: {{1-2 sentences on why this episode might help}}

**Key insight**:
> {{The most important takeaway that could help with the current task}}

**What worked**: {{brief summary}}

**What to avoid**: {{if applicable}}

---

### 2. {{Episode intent}}
...

---

### 3. {{Episode intent}}
...

---

## 💡 Synthesis

Based on these past experiences, here's what might help with your current task:

{{2-3 bullet points synthesizing the most relevant lessons}}

---

If no relevant episodes found, say:
"No directly relevant past experiences found. This appears to be a new problem domain. I'll capture this session for future reference when we're done."

## Arguments

If called with an argument, use that as the task description:
`/memrl-recall implement rate limiting` → search for rate limiting related episodes

If called without argument, infer the task from the current conversation context.
