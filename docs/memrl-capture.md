---
description: Capture this coding session as a structured memory episode
---

Create a structured summary of this coding session for future retrieval.

## Instructions

1. **Extract Intent**: What was the main task or problem being solved?

2. **Gather Context**:
   - Which files were read during this session?
   - Which files were modified?
   - What tools/commands were run (cargo, npm, git, etc.)?

3. **Document Errors**: List any errors encountered and how they were resolved

4. **Assess Outcome**:
   - `success` - Task completed, tests pass
   - `partial` - Some progress, but incomplete
   - `failure` - Couldn't solve the problem

5. **Generate Tags**: Keywords for future search (language, domain, patterns)

## Output

Save to: `~/.memrl/episodes/{{current_date}}/session-{{current_time}}.md`

Create the directory if it doesn't exist.

Use this exact format:

```markdown
# Episode: {{one-line intent summary}}

**ID**: {{generate short uuid}}
**Date**: {{ISO timestamp}}
**Project**: {{current directory name}}
**Duration**: {{estimate from conversation}}
**Outcome**: {{success|partial|failure}}

## Intent

{{2-3 sentence description of what we were trying to accomplish}}

## Context

### Files Read
{{bullet list of files, or "None" if no files were explicitly read}}

### Files Modified  
{{bullet list of files that were changed, or "None"}}

### Commands/Tools Used
{{bullet list: cargo test, npm install, git commit, etc.}}

## Key Decisions

{{List important architectural or implementation decisions made, with brief reasoning}}

- Decision: {{what}}
  Reason: {{why}}

## Errors → Resolutions

{{List errors encountered and how they were fixed. This is critical for future debugging.}}

| Error | Resolution |
|-------|------------|
| {{error message or type}} | {{how it was fixed}} |

## What Worked

{{Brief notes on approaches that were effective}}

## What Didn't Work

{{Brief notes on approaches that failed - equally valuable for learning}}

## Related Episodes

{{If this session built on or related to previous sessions, note that}}

## Tags

{{comma-separated keywords: rust, async, lifetime-error, api-design, testing, refactor, etc.}}
```

After saving, confirm with: "📝 Episode captured: [filepath]"
