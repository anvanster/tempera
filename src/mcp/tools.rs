use serde_json::json;

use super::protocol::Tool;

/// Return all MCP tool definitions
pub(crate) fn tool_definitions() -> Vec<Tool> {
    vec![
        Tool {
            name: "tempera_retrieve".to_string(),
            description: "MANDATORY at session start for non-trivial tasks. Search episodic memory for similar problems you've solved before. If this is your first action in a session, always check for relevant memories first.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language description of what you're trying to do, OR an episode ID to get full details"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of episodes to retrieve (default: 5)",
                        "default": 5
                    },
                    "project": {
                        "type": "string",
                        "description": "Filter by project name (optional)"
                    },
                    "all": {
                        "type": "boolean",
                        "description": "If true, list all episodes instead of searching (ignores query)",
                        "default": false
                    }
                },
                "required": []
            }),
        },
        Tool {
            name: "tempera_capture".to_string(),
            description: "MANDATORY after completing any feature, bugfix, or refactor. Don't wait for user to ask - proactively capture successful sessions. Automatically runs utility propagation after capture.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Brief summary of what was accomplished"
                    },
                    "task_type": {
                        "type": "string",
                        "enum": ["bugfix", "feature", "refactor", "test", "docs", "research", "debug", "setup"],
                        "description": "Type of task completed"
                    },
                    "outcome": {
                        "type": "string",
                        "enum": ["success", "partial", "failure"],
                        "description": "Outcome of the task"
                    },
                    "project": {
                        "type": "string",
                        "description": "Override project name (default: auto-detect from working directory). Use for cross-project insights."
                    },
                    "files_modified": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of files that were modified"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Domain tags for categorization"
                    },
                    "errors_resolved": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "error": { "type": "string" },
                                "resolution": { "type": "string" }
                            }
                        },
                        "description": "Errors encountered and how they were resolved"
                    }
                },
                "required": ["summary", "task_type", "outcome"]
            }),
        },
        Tool {
            name: "tempera_feedback".to_string(),
            description: "Record whether retrieved episodes were helpful. Call this after using memories - your feedback improves future retrieval quality.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "episode_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "IDs of episodes to provide feedback on"
                    },
                    "helpful": {
                        "type": "boolean",
                        "description": "Whether the episodes were helpful"
                    }
                },
                "required": ["episode_ids", "helpful"]
            }),
        },
        Tool {
            name: "tempera_stats".to_string(),
            description: "Get statistics about the episodic memory system.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Filter stats by project (optional)"
                    }
                }
            }),
        },
        Tool {
            name: "tempera_status".to_string(),
            description: "Check memory health for current project. Shows last capture date, episode count, and unused memories. Use this to understand your memory state.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project to check (default: auto-detect from working directory)"
                    }
                }
            }),
        },
        Tool {
            name: "tempera_propagate".to_string(),
            description: "Run utility propagation to spread value from helpful episodes to similar ones. Use periodically to improve memory quality.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "temporal": {
                        "type": "boolean",
                        "description": "Also run temporal credit assignment (credits episodes that preceded successful outcomes)",
                        "default": false
                    },
                    "project": {
                        "type": "string",
                        "description": "Filter propagation to a specific project (optional)"
                    }
                }
            }),
        },
        Tool {
            name: "tempera_review".to_string(),
            description: "Review and consolidate memories after completing a series of related tasks. Identifies duplicate/similar episodes, stale memories, and optimization opportunities.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project to review (default: auto-detect from working directory)"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["analyze", "cleanup"],
                        "description": "analyze: show recommendations only. cleanup: apply safe optimizations (removes zero-utility duplicates)",
                        "default": "analyze"
                    }
                }
            }),
        },
    ]
}
