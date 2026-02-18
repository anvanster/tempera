use serde_json::Value;

use crate::store;

/// Record feedback on episodes
pub(crate) async fn handle(args: &Value) -> Result<String, String> {
    let episode_ids: Vec<String> = args
        .get("episode_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or("Missing episode_ids parameter")?;

    let helpful = args
        .get("helpful")
        .and_then(|v| v.as_bool())
        .ok_or("Missing helpful parameter")?;

    let store = store::EpisodeStore::new().map_err(|e| e.to_string())?;
    let mut updated = 0;

    for id in &episode_ids {
        if let Ok(episodes) = store.list_all() {
            for ep in episodes {
                if ep.id.starts_with(id) || ep.id[..8] == *id {
                    let mut episode = ep.clone();

                    if helpful {
                        episode.utility.helpful_count += 1;
                    }
                    episode.utility.score = Some(episode.utility.calculate_score());

                    if let Some(last) = episode.retrieval_history.last_mut() {
                        last.was_helpful = Some(helpful);
                    }

                    if store.update(&episode).is_ok() {
                        updated += 1;
                    }
                    break;
                }
            }
        }
    }

    let feedback_type = if helpful { "helpful" } else { "not helpful" };
    Ok(format!(
        "Feedback recorded: {} episode(s) marked as {}.\n\
         This helps improve future retrieval quality.",
        updated, feedback_type
    ))
}
