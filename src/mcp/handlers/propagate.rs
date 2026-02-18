use serde_json::Value;

use crate::{indexer, store, utility};

/// Run utility propagation
pub(crate) async fn handle(args: &Value) -> Result<String, String> {
    let temporal = args
        .get("temporal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let project_filter = args.get("project").and_then(|v| v.as_str());

    let store = store::EpisodeStore::new().map_err(|e| e.to_string())?;
    let params = utility::UtilityParams::default();

    let mut output = String::from("📈 Running utility propagation...\n\n");

    // Get episodes (optionally filtered by project)
    let all_episodes = store.list_all().map_err(|e| e.to_string())?;
    let episodes: Vec<_> = if let Some(proj) = project_filter {
        all_episodes
            .into_iter()
            .filter(|e| e.project.to_lowercase().contains(&proj.to_lowercase()))
            .collect()
    } else {
        all_episodes
    };

    output.push_str(&format!("Processing {} episodes...\n", episodes.len()));

    // Apply decay
    let mut decayed_count = 0;
    for ep in &episodes {
        if let Some(last_retrieval) = ep.retrieval_history.last() {
            let days_since = (chrono::Utc::now() - last_retrieval.timestamp).num_days() as f64;
            if days_since > 0.0 {
                let decay = (1.0 - params.decay_rate).powf(days_since);
                if decay < 0.99 {
                    decayed_count += 1;
                }
            }
        }
    }

    output.push_str(&format!(
        "  📉 Decay applied to {} episodes\n",
        decayed_count
    ));

    // Run Bellman propagation using vector similarity
    let (propagated_count, propagation_delta) =
        utility::run_bellman_propagation(&store, &params, project_filter)
            .await
            .unwrap_or((0, 0.0));

    output.push_str(&format!(
        "  🔄 Propagated value to {} episodes\n",
        propagated_count
    ));
    output.push_str(&format!(
        "  📊 Total utility change: {:+.3}\n",
        propagation_delta
    ));

    // Temporal credit assignment
    if temporal {
        output.push_str("\n⏱️  Running temporal credit assignment...\n");

        let credited = utility::temporal_credit_assignment(&store, project_filter, &params)
            .map_err(|e| e.to_string())?;

        output.push_str(&format!("  ✅ Credited {} episodes\n", credited));
    }

    // Sync to vector index
    if let Ok(mut indexer) = indexer::EpisodeIndexer::new().await {
        let updated_episodes = store.list_all().map_err(|e| e.to_string())?;
        for ep in &updated_episodes {
            let _ = indexer.index_episode(ep).await;
        }
        output.push_str("  💾 Synced to vector index\n");
    }

    output.push_str("\n✅ Propagation complete!");

    Ok(output)
}
