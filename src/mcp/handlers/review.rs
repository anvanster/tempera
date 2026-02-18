use serde_json::Value;

use crate::mcp::helpers::{extract_project, load_project_episodes};
use crate::store;

/// Review and consolidate memories
pub(crate) async fn handle(args: &Value) -> Result<String, String> {
    let project = extract_project(args);
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("analyze");

    let store = store::EpisodeStore::new().map_err(|e| e.to_string())?;
    let project_episodes = load_project_episodes(&store, &project)?;

    if project_episodes.is_empty() {
        return Ok(format!("No memories found for project '{}'.", project));
    }

    let mut output = format!("🔍 Memory Review for '{}'\n", project);
    output.push_str(&"=".repeat(22 + project.len()));
    output.push_str("\n\n");

    // Find stale memories (old with low utility)
    let stale: Vec<_> = project_episodes
        .iter()
        .filter(|e| {
            let age_days = (chrono::Utc::now() - e.timestamp_start).num_days();
            let utility = e.utility.calculate_score();
            age_days > 30 && utility < 0.2
        })
        .collect();

    // Find potential duplicates (similar intents)
    let mut duplicates: Vec<(&crate::episode::Episode, &crate::episode::Episode)> = Vec::new();
    for i in 0..project_episodes.len() {
        for j in (i + 1)..project_episodes.len() {
            let e1 = &project_episodes[i];
            let e2 = &project_episodes[j];
            if e1.intent.task_type == e2.intent.task_type {
                let s1 = e1.intent.extracted_intent.to_lowercase();
                let s2 = e2.intent.extracted_intent.to_lowercase();
                let words1: std::collections::HashSet<_> = s1.split_whitespace().collect();
                let words2: std::collections::HashSet<_> = s2.split_whitespace().collect();
                let intersection = words1.intersection(&words2).count();
                let union = words1.union(&words2).count();
                if union > 0 && (intersection as f64 / union as f64) > 0.6 {
                    duplicates.push((e1, e2));
                }
            }
        }
    }

    // Find zero-utility episodes
    let zero_utility: Vec<_> = project_episodes
        .iter()
        .filter(|e| e.utility.calculate_score() < 0.05 && e.utility.retrieval_count == 0)
        .collect();

    output.push_str("📊 Analysis Results:\n");
    output.push_str(&format!("  - Total memories: {}\n", project_episodes.len()));
    output.push_str(&format!("  - Stale (>30d, low utility): {}\n", stale.len()));
    output.push_str(&format!("  - Potential duplicates: {}\n", duplicates.len()));
    output.push_str(&format!(
        "  - Zero utility (never used): {}\n\n",
        zero_utility.len()
    ));

    if !stale.is_empty() {
        output.push_str("📅 Stale Memories:\n");
        for ep in stale.iter().take(5) {
            let summary: String = ep.intent.extracted_intent.chars().take(50).collect();
            output.push_str(&format!("  - {} ({}...)\n", &ep.id[..8], summary));
        }
        if stale.len() > 5 {
            output.push_str(&format!("  ... and {} more\n", stale.len() - 5));
        }
        output.push('\n');
    }

    if !duplicates.is_empty() {
        output.push_str("🔄 Potential Duplicates:\n");
        for (e1, e2) in duplicates.iter().take(3) {
            output.push_str(&format!("  - {} ≈ {}\n", &e1.id[..8], &e2.id[..8]));
        }
        if duplicates.len() > 3 {
            output.push_str(&format!("  ... and {} more pairs\n", duplicates.len() - 3));
        }
        output.push('\n');
    }

    if action == "cleanup" {
        output.push_str("🧹 Cleanup Actions:\n");
        let mut removed = 0;

        for ep in &zero_utility {
            let is_duplicate = duplicates
                .iter()
                .any(|(e1, e2)| e1.id == ep.id || e2.id == ep.id);
            if is_duplicate {
                if store.delete(&ep.id).is_ok() {
                    removed += 1;
                    output.push_str(&format!(
                        "  ✓ Removed {} (zero-utility duplicate)\n",
                        &ep.id[..8]
                    ));
                }
            }
        }

        if removed == 0 {
            output.push_str("  No safe cleanup actions available.\n");
            output.push_str("  (Only zero-utility duplicates are auto-removed)\n");
        } else {
            output.push_str(&format!("\n✅ Removed {} episode(s)\n", removed));
        }
    } else {
        output.push_str("💡 Recommendations:\n");
        if !zero_utility.is_empty() && !duplicates.is_empty() {
            output.push_str("  - Run with action: 'cleanup' to remove zero-utility duplicates\n");
        }
        if !stale.is_empty() {
            output.push_str("  - Consider manually reviewing stale memories\n");
        }
        if duplicates.is_empty() && stale.is_empty() && zero_utility.is_empty() {
            output.push_str("  - Your memory is healthy! No issues found.\n");
        }
    }

    Ok(output)
}
