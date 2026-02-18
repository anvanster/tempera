use anyhow::Result;
use serde_json::Value;

use crate::{episode, store};

/// Extract project name from args or auto-detect from working directory
pub(crate) fn extract_project(args: &Value) -> String {
    args.get("project")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "unknown".to_string())
        })
}

/// Extract a string array from JSON args
pub(crate) fn extract_string_array(args: &Value, field: &str) -> Vec<String> {
    args.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Load all episodes filtered by project (exact match)
pub(crate) fn load_project_episodes(
    store: &store::EpisodeStore,
    project: &str,
) -> Result<Vec<episode::Episode>, String> {
    let all = store.list_all().map_err(|e| e.to_string())?;
    Ok(all
        .into_iter()
        .filter(|e| e.project.to_lowercase() == project.to_lowercase())
        .collect())
}

/// Record retrieval for tracking
pub(crate) fn record_mcp_retrieval(
    episodes: &[crate::retrieve::ScoredEpisode],
    query: &str,
    store: &store::EpisodeStore,
) -> Result<()> {
    let project = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    for scored in episodes {
        let mut episode = scored.episode.clone();
        episode.retrieval_history.push(episode::RetrievalRecord {
            timestamp: chrono::Utc::now(),
            project: project.clone(),
            task_description: query.to_string(),
            was_helpful: None,
        });
        episode.utility.retrieval_count += 1;
        store.update(&episode)?;
    }

    Ok(())
}
