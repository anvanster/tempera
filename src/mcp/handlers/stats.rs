use serde_json::Value;

use crate::{episode, store};

/// Get memory statistics
pub(crate) async fn handle(args: &Value) -> Result<String, String> {
    let project_filter = args.get("project").and_then(|v| v.as_str());

    let store = store::EpisodeStore::new().map_err(|e| e.to_string())?;
    let episodes = store.list_all().map_err(|e| e.to_string())?;

    let filtered: Vec<_> = if let Some(proj) = project_filter {
        episodes
            .iter()
            .filter(|e| e.project.to_lowercase().contains(&proj.to_lowercase()))
            .collect()
    } else {
        episodes.iter().collect()
    };

    let total = filtered.len();
    let successful = filtered
        .iter()
        .filter(|e| e.outcome.status == episode::OutcomeStatus::Success)
        .count();
    let total_retrievals: u32 = filtered.iter().map(|e| e.utility.retrieval_count).sum();
    let total_helpful: u32 = filtered.iter().map(|e| e.utility.helpful_count).sum();

    let avg_utility: f32 = if total > 0 {
        filtered
            .iter()
            .map(|e| e.utility.calculate_score())
            .sum::<f32>()
            / total as f32
    } else {
        0.0
    };

    // Task type breakdown
    let mut task_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for ep in &filtered {
        *task_counts
            .entry(ep.intent.task_type.to_string())
            .or_insert(0) += 1;
    }

    let mut output = String::from("Tempera Memory Statistics\n");
    output.push_str("=======================\n\n");

    if let Some(proj) = project_filter {
        output.push_str(&format!("Project: {}\n\n", proj));
    }

    output.push_str(&format!("Total Episodes: {}\n", total));
    output.push_str(&format!(
        "Success Rate: {:.1}%\n",
        if total > 0 {
            (successful as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    ));
    output.push_str(&format!("Total Retrievals: {}\n", total_retrievals));
    output.push_str(&format!("Helpful Retrievals: {}\n", total_helpful));
    output.push_str(&format!("Average Utility: {:.1}%\n\n", avg_utility * 100.0));

    output.push_str("By Task Type:\n");
    for (task_type, count) in task_counts {
        output.push_str(&format!("  - {}: {}\n", task_type, count));
    }

    Ok(output)
}
