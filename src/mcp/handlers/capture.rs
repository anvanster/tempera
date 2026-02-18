use serde_json::Value;

use crate::mcp::helpers::{extract_project, extract_string_array};
use crate::{episode, indexer, store, utility};

/// Capture a new episode
pub(crate) async fn handle(args: &Value) -> Result<String, String> {
    let summary = args
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or("Missing summary parameter")?;

    let task_type_str = args
        .get("task_type")
        .and_then(|v| v.as_str())
        .ok_or("Missing task_type parameter")?;

    let outcome_str = args
        .get("outcome")
        .and_then(|v| v.as_str())
        .ok_or("Missing outcome parameter")?;

    let files_modified = extract_string_array(args, "files_modified");
    let tags = extract_string_array(args, "tags");

    let project = extract_project(args);

    // Create episode
    let mut ep = episode::Episode::new(project.clone(), summary.to_string());

    // Set task type
    ep.intent.task_type = match task_type_str {
        "bugfix" => episode::TaskType::Bugfix,
        "feature" => episode::TaskType::Feature,
        "refactor" => episode::TaskType::Refactor,
        "test" => episode::TaskType::Test,
        "docs" => episode::TaskType::Docs,
        "research" => episode::TaskType::Research,
        "debug" => episode::TaskType::Debug,
        "setup" => episode::TaskType::Setup,
        _ => episode::TaskType::Unknown,
    };

    // Set outcome
    ep.outcome.status = match outcome_str {
        "success" => episode::OutcomeStatus::Success,
        "partial" => episode::OutcomeStatus::Partial,
        "failure" => episode::OutcomeStatus::Failure,
        _ => episode::OutcomeStatus::Partial,
    };

    // Set context
    ep.context.files_modified = files_modified;
    ep.intent.domain = tags;
    ep.intent.extracted_intent = summary.to_string();

    // Parse errors if provided
    if let Some(errors) = args.get("errors_resolved").and_then(|v| v.as_array()) {
        for err in errors {
            if let (Some(error_msg), resolution) = (
                err.get("error").and_then(|v| v.as_str()),
                err.get("resolution").and_then(|v| v.as_str()),
            ) {
                ep.context.errors_encountered.push(episode::ErrorRecord {
                    error_type: "runtime".to_string(),
                    message: error_msg.to_string(),
                    resolved: resolution.is_some(),
                    resolution: resolution.map(String::from),
                });
            }
        }
    }

    // Update end timestamp
    ep.timestamp_end = chrono::Utc::now();

    // Save episode
    let store = store::EpisodeStore::new().map_err(|e| e.to_string())?;
    store.save(&ep).map_err(|e| e.to_string())?;

    // Index the new episode
    if let Ok(mut indexer) = indexer::EpisodeIndexer::new().await {
        let _ = indexer.index_episode(&ep).await;
    }

    let mut output = format!(
        "Episode captured successfully!\n\
         - ID: {}\n\
         - Project: {}\n\
         - Type: {}\n\
         - Outcome: {}\n",
        &ep.id[..8],
        ep.project,
        ep.intent.task_type,
        ep.outcome.status
    );

    // Auto-propagate utility to spread value
    output.push_str("\n📈 Running auto-propagation...\n");
    let params = utility::UtilityParams::default();
    match utility::run_bellman_propagation(&store, &params, Some(project.as_str())).await {
        Ok((count, _)) => output.push_str(&format!("  Propagated value to {} episode(s)\n", count)),
        Err(e) => output.push_str(&format!("  (propagation skipped: {})\n", e)),
    }

    output.push_str("\nThis experience is now stored for future reference.");
    Ok(output)
}
