#!/bin/bash
# MemRL Pre-Task Hook for Claude Code
#
# This hook retrieves relevant past episodes before starting a new task.
# Install by adding to your Claude Code hooks configuration.
#
# Required environment variables:
# - CLAUDE_USER_PROMPT: The user's initial prompt (if available)

set -e

# Check if memrl is available
if ! command -v memrl &> /dev/null; then
    exit 0
fi

# Check if we have a prompt to search with
if [ -z "$CLAUDE_USER_PROMPT" ]; then
    exit 0
fi

# Get project name
PROJECT_NAME=$(basename "$(pwd)")

# Retrieve relevant episodes (silent on error)
echo "---"
echo "Checking episodic memory for relevant past experiences..."
echo ""

memrl retrieve "$CLAUDE_USER_PROMPT" --project "$PROJECT_NAME" --limit 3 --format text 2>/dev/null || \
memrl retrieve "$CLAUDE_USER_PROMPT" --limit 3 --format text 2>/dev/null || \
echo "No relevant past episodes found."

echo "---"
