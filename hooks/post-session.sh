#!/bin/bash
# MemRL Post-Session Hook for Claude Code
#
# This hook captures the session as an episode when Claude Code completes.
# Install by adding to your Claude Code hooks configuration.
#
# Required environment variables (set by Claude Code):
# - CLAUDE_SESSION_FILE: Path to session transcript
# - CLAUDE_PROJECT_DIR: Current project directory

set -e

# Check if memrl is available
if ! command -v memrl &> /dev/null; then
    echo "memrl not found in PATH, skipping capture"
    exit 0
fi

# Check if session file exists
if [ -z "$CLAUDE_SESSION_FILE" ] || [ ! -f "$CLAUDE_SESSION_FILE" ]; then
    # Try to find the most recent session file
    SESSION_DIR="${HOME}/.claude/sessions"
    if [ -d "$SESSION_DIR" ]; then
        CLAUDE_SESSION_FILE=$(ls -t "$SESSION_DIR"/*.jsonl 2>/dev/null | head -1)
    fi
fi

if [ -z "$CLAUDE_SESSION_FILE" ]; then
    echo "No session file found, skipping capture"
    exit 0
fi

# Get project name from directory
PROJECT="${CLAUDE_PROJECT_DIR:-$(pwd)}"
PROJECT_NAME=$(basename "$PROJECT")

# Capture the session
echo "Capturing session for project: $PROJECT_NAME"
memrl capture --session "$CLAUDE_SESSION_FILE" --project "$PROJECT_NAME" 2>/dev/null || true

# Update the index
memrl index 2>/dev/null || true

echo "Session captured successfully"
