// Copyright 2024-2026 Andrey Vasilevsky <anvanster@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
use anyhow::Result;
use chrono::{Datelike, Utc};
use colored::Colorize;
use std::collections::HashMap;
use tabled::{Table, Tabled};

use crate::config::Config;
use crate::episode::{Episode, OutcomeStatus};
use crate::store::EpisodeStore;

/// List episodes
pub async fn list(
    limit: usize,
    project: Option<String>,
    tag: Option<String>,
    outcome: Option<String>,
    _config: &Config,
) -> Result<()> {
    let store = EpisodeStore::new()?;

    let episodes = store.list_filtered(
        limit,
        project.as_deref(),
        tag.as_deref(),
        outcome.as_deref(),
    )?;

    if episodes.is_empty() {
        println!("No episodes found.");
        return Ok(());
    }

    println!("{}", "📚 Episodes".bold());
    println!();

    // Convert to table rows
    let rows: Vec<EpisodeRow> = episodes
        .iter()
        .map(|ep| EpisodeRow {
            id: ep.id[..8].to_string(),
            date: ep.timestamp_start.format("%Y-%m-%d").to_string(),
            project: truncate(&ep.project, 15),
            intent: truncate(
                if ep.intent.extracted_intent.is_empty() {
                    &ep.intent.raw_prompt
                } else {
                    &ep.intent.extracted_intent
                },
                40,
            ),
            outcome: format_outcome(&ep.outcome.status),
            utility: format!("{:.0}%", ep.utility.calculate_score() * 100.0),
            retrievals: ep.utility.retrieval_count.to_string(),
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);

    Ok(())
}

/// Show a single episode in detail
pub async fn show(id: &str, _config: &Config) -> Result<()> {
    let store = EpisodeStore::new()?;

    let episode = if id.to_lowercase() == "latest" || id.to_lowercase() == "last" {
        store.load_latest()?
    } else {
        store.load(id)?
    };

    // Print episode details
    println!("{}", "📄 Episode Details".bold());
    println!();
    println!("{}", episode.to_markdown());

    // Additional details not in markdown
    println!("{}", "## Utility Metrics".bold());
    println!("Retrieval count: {}", episode.utility.retrieval_count);
    println!("Helpful count: {}", episode.utility.helpful_count);
    println!(
        "Utility score: {:.2}%",
        episode.utility.calculate_score() * 100.0
    );

    Ok(())
}

/// Show statistics
pub async fn run(project: Option<String>, _config: &Config) -> Result<()> {
    let store = EpisodeStore::new()?;
    let stats = store.get_stats(project.as_deref())?;

    println!("{}", "📊 Tempera Statistics".bold());
    println!();

    if let Some(ref proj) = project {
        println!("Filter: project = {}", proj);
        println!();
    }

    // Overview
    println!("{}", "Overview".underline());
    println!("Total episodes: {}", stats.total);
    println!(
        "Success rate: {:.1}%",
        if stats.total > 0 {
            (stats.success_count as f32 / stats.total as f32) * 100.0
        } else {
            0.0
        }
    );
    println!();

    // Outcome breakdown
    println!("{}", "Outcomes".underline());
    println!(
        "  ✅ Success: {} ({:.1}%)",
        stats.success_count,
        percentage(stats.success_count, stats.total)
    );
    println!(
        "  ⚠️  Partial: {} ({:.1}%)",
        stats.partial_count,
        percentage(stats.partial_count, stats.total)
    );
    println!(
        "  ❌ Failure: {} ({:.1}%)",
        stats.failure_count,
        percentage(stats.failure_count, stats.total)
    );
    println!();

    // Utility metrics
    println!("{}", "Utility Metrics".underline());
    println!("Total retrievals: {}", stats.total_retrievals);
    println!("Total helpful: {}", stats.total_helpful);
    println!(
        "Helpful rate: {:.1}%",
        if stats.total_retrievals > 0 {
            (stats.total_helpful as f32 / stats.total_retrievals as f32) * 100.0
        } else {
            0.0
        }
    );
    println!("Average utility score: {:.1}%", stats.avg_utility * 100.0);
    println!();

    // Projects
    if !stats.projects.is_empty() {
        println!("{}", "Projects".underline());
        for proj in &stats.projects {
            println!("  - {}", proj);
        }
        println!();
    }

    // Top tags
    if !stats.top_tags.is_empty() {
        println!("{}", "Top Tags".underline());
        for (tag, count) in &stats.top_tags {
            println!("  {} ({})", tag, count);
        }
    }

    Ok(())
}

/// Table row for episode list
#[derive(Tabled)]
struct EpisodeRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Project")]
    project: String,
    #[tabled(rename = "Intent")]
    intent: String,
    #[tabled(rename = "Outcome")]
    outcome: String,
    #[tabled(rename = "Utility")]
    utility: String,
    #[tabled(rename = "Retriev")]
    retrievals: String,
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format outcome status
fn format_outcome(status: &OutcomeStatus) -> String {
    match status {
        OutcomeStatus::Success => "✅".to_string(),
        OutcomeStatus::Partial => "⚠️".to_string(),
        OutcomeStatus::Failure => "❌".to_string(),
    }
}

/// Calculate percentage
fn percentage(part: usize, total: usize) -> f32 {
    if total > 0 {
        (part as f32 / total as f32) * 100.0
    } else {
        0.0
    }
}

// === Trend Analytics ===

/// Time-bucketed metrics for trend analysis
#[derive(Debug, Clone, serde::Serialize)]
pub struct TimeBucket {
    pub period: String,
    pub episode_count: usize,
    pub success_rate: f32,
    pub avg_utility: f32,
    pub helpful_rate: f32,
}

/// Domain growth trend
#[derive(Debug, Clone, serde::Serialize)]
pub struct DomainTrend {
    pub domain: String,
    pub episodes_total: usize,
    pub episodes_recent_30d: usize,
    pub avg_utility: f32,
}

/// A single point on the learning curve
#[derive(Debug, Clone, serde::Serialize)]
pub struct LearningPoint {
    pub episode_number: usize,
    pub cumulative_success_rate: f32,
    pub cumulative_helpful_rate: f32,
}

/// Full trend analytics result
#[derive(Debug, Clone, serde::Serialize)]
pub struct TrendAnalytics {
    pub buckets: Vec<TimeBucket>,
    pub domain_trends: Vec<DomainTrend>,
    pub learning_curve: Vec<LearningPoint>,
    pub total_episodes: usize,
}

/// Compute trend analytics from episodes
pub fn compute_trends(episodes: &[Episode], bucket_size: &str) -> TrendAnalytics {
    if episodes.is_empty() {
        return TrendAnalytics {
            buckets: vec![],
            domain_trends: vec![],
            learning_curve: vec![],
            total_episodes: 0,
        };
    }

    // Sort chronologically
    let mut sorted: Vec<&Episode> = episodes.iter().collect();
    sorted.sort_by_key(|e| e.timestamp_start);

    // Time buckets
    let mut bucket_map: HashMap<String, Vec<&Episode>> = HashMap::new();
    for ep in &sorted {
        let key = match bucket_size {
            "monthly" => format!(
                "{}-{:02}",
                ep.timestamp_start.year(),
                ep.timestamp_start.month()
            ),
            _ => {
                // weekly (default)
                let iso = ep.timestamp_start.iso_week();
                format!("{}-W{:02}", iso.year(), iso.week())
            }
        };
        bucket_map.entry(key).or_default().push(ep);
    }

    let mut buckets: Vec<TimeBucket> = bucket_map
        .into_iter()
        .map(|(period, eps)| {
            let count = eps.len();
            let successes = eps
                .iter()
                .filter(|e| e.outcome.status == OutcomeStatus::Success)
                .count();
            let total_util: f32 = eps.iter().map(|e| e.utility.calculate_score()).sum();
            let total_retrievals: u32 = eps.iter().map(|e| e.utility.retrieval_count).sum();
            let total_helpful: u32 = eps.iter().map(|e| e.utility.helpful_count).sum();

            TimeBucket {
                period,
                episode_count: count,
                success_rate: if count > 0 {
                    successes as f32 / count as f32
                } else {
                    0.0
                },
                avg_utility: if count > 0 {
                    total_util / count as f32
                } else {
                    0.0
                },
                helpful_rate: if total_retrievals > 0 {
                    total_helpful as f32 / total_retrievals as f32
                } else {
                    0.0
                },
            }
        })
        .collect();
    buckets.sort_by(|a, b| a.period.cmp(&b.period));

    // Domain trends
    let now = Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);
    let mut domain_map: HashMap<String, (usize, usize, f32)> = HashMap::new(); // (total, recent, util_sum)

    for ep in &sorted {
        for tag in &ep.intent.domain {
            let entry = domain_map.entry(tag.clone()).or_default();
            entry.0 += 1;
            if ep.timestamp_start >= thirty_days_ago {
                entry.1 += 1;
            }
            entry.2 += ep.utility.calculate_score();
        }
    }

    let mut domain_trends: Vec<DomainTrend> = domain_map
        .into_iter()
        .map(|(domain, (total, recent, util_sum))| DomainTrend {
            domain,
            episodes_total: total,
            episodes_recent_30d: recent,
            avg_utility: if total > 0 {
                util_sum / total as f32
            } else {
                0.0
            },
        })
        .collect();
    domain_trends.sort_by(|a, b| b.episodes_total.cmp(&a.episodes_total));
    domain_trends.truncate(15);

    // Learning curve (cumulative success and helpful rates)
    let mut cumulative_successes = 0usize;
    let mut cumulative_retrievals = 0u32;
    let mut cumulative_helpful = 0u32;
    let step = (sorted.len() / 20).max(1); // ~20 points on the curve

    let learning_curve: Vec<LearningPoint> = sorted
        .iter()
        .enumerate()
        .filter_map(|(i, ep)| {
            if ep.outcome.status == OutcomeStatus::Success {
                cumulative_successes += 1;
            }
            cumulative_retrievals += ep.utility.retrieval_count;
            cumulative_helpful += ep.utility.helpful_count;

            let n = i + 1;
            if n % step == 0 || n == sorted.len() {
                Some(LearningPoint {
                    episode_number: n,
                    cumulative_success_rate: cumulative_successes as f32 / n as f32,
                    cumulative_helpful_rate: if cumulative_retrievals > 0 {
                        cumulative_helpful as f32 / cumulative_retrievals as f32
                    } else {
                        0.0
                    },
                })
            } else {
                None
            }
        })
        .collect();

    TrendAnalytics {
        buckets,
        domain_trends,
        learning_curve,
        total_episodes: sorted.len(),
    }
}

/// Render trend analytics as text for CLI/MCP output
pub fn render_trends(analytics: &TrendAnalytics) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "📈 Trend Analytics ({} episodes)\n",
        analytics.total_episodes
    ));
    out.push_str(&"=".repeat(40));
    out.push_str("\n\n");

    // Time buckets
    if !analytics.buckets.is_empty() {
        out.push_str("Helpfulness Over Time:\n");
        let max_count = analytics
            .buckets
            .iter()
            .map(|b| b.episode_count)
            .max()
            .unwrap_or(1);

        for bucket in &analytics.buckets {
            let bar_len = (bucket.episode_count as f32 / max_count as f32 * 20.0) as usize;
            let bar: String = "█".repeat(bar_len);
            out.push_str(&format!(
                "  {}: {} {:>2} eps, {:.0}% success, {:.0}% util\n",
                bucket.period,
                bar,
                bucket.episode_count,
                bucket.success_rate * 100.0,
                bucket.avg_utility * 100.0
            ));
        }
        out.push('\n');
    }

    // Domain trends
    if !analytics.domain_trends.is_empty() {
        out.push_str("Domain Growth:\n");
        for dt in analytics.domain_trends.iter().take(10) {
            let activity = if dt.episodes_recent_30d > 0 {
                format!("+{} recent", dt.episodes_recent_30d)
            } else {
                "inactive".to_string()
            };
            out.push_str(&format!(
                "  {:<20} {:>3} total ({}, {:.0}% util)\n",
                dt.domain,
                dt.episodes_total,
                activity,
                dt.avg_utility * 100.0
            ));
        }
        out.push('\n');
    }

    // Learning curve
    if analytics.learning_curve.len() >= 2 {
        out.push_str("Learning Curve:\n");
        let first = &analytics.learning_curve[0];
        let last = analytics.learning_curve.last().unwrap();

        out.push_str(&format!(
            "  Start: {:.0}% success, {:.0}% helpful\n",
            first.cumulative_success_rate * 100.0,
            first.cumulative_helpful_rate * 100.0
        ));
        out.push_str(&format!(
            "  Now:   {:.0}% success, {:.0}% helpful\n",
            last.cumulative_success_rate * 100.0,
            last.cumulative_helpful_rate * 100.0
        ));

        let success_delta = last.cumulative_success_rate - first.cumulative_success_rate;
        let direction = if success_delta > 0.02 {
            "improving"
        } else if success_delta < -0.02 {
            "declining"
        } else {
            "stable"
        };
        out.push_str(&format!("  Trend: {}\n", direction));
    }

    out
}

/// Run the trends command from CLI
pub async fn trends(project: Option<String>, bucket: &str, _config: &Config) -> Result<()> {
    let store = EpisodeStore::new()?;
    let mut episodes = store.list_all()?;

    if let Some(proj) = &project {
        episodes.retain(|e| e.project.to_lowercase().contains(&proj.to_lowercase()));
    }

    if episodes.is_empty() {
        println!("No episodes found.");
        return Ok(());
    }

    let analytics = compute_trends(&episodes, bucket);
    println!("{}", render_trends(&analytics));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(50, 100), 50.0);
        assert_eq!(percentage(0, 0), 0.0);
    }

    #[test]
    fn test_compute_trends_empty() {
        let analytics = compute_trends(&[], "weekly");
        assert_eq!(analytics.total_episodes, 0);
        assert!(analytics.buckets.is_empty());
        assert!(analytics.domain_trends.is_empty());
        assert!(analytics.learning_curve.is_empty());
    }

    #[test]
    fn test_compute_trends_basic() {
        use crate::episode::Episode;

        let mut ep1 = Episode::new("proj".to_string(), "fix bug".to_string());
        ep1.outcome.status = OutcomeStatus::Success;
        ep1.intent.domain = vec!["rust".to_string(), "auth".to_string()];
        ep1.utility.retrieval_count = 3;
        ep1.utility.helpful_count = 2;

        let mut ep2 = Episode::new("proj".to_string(), "add feature".to_string());
        ep2.outcome.status = OutcomeStatus::Partial;
        ep2.intent.domain = vec!["rust".to_string(), "api".to_string()];

        let episodes = vec![ep1, ep2];
        let analytics = compute_trends(&episodes, "weekly");

        assert_eq!(analytics.total_episodes, 2);
        assert!(!analytics.buckets.is_empty());

        // Both episodes are today, so they should be in the same bucket
        assert_eq!(analytics.buckets.len(), 1);
        assert_eq!(analytics.buckets[0].episode_count, 2);
        assert!((analytics.buckets[0].success_rate - 0.5).abs() < 0.01);

        // Domain trends: rust should have 2 episodes
        let rust_trend = analytics
            .domain_trends
            .iter()
            .find(|d| d.domain == "rust")
            .unwrap();
        assert_eq!(rust_trend.episodes_total, 2);
        assert_eq!(rust_trend.episodes_recent_30d, 2);

        // Learning curve should have at least one point
        assert!(!analytics.learning_curve.is_empty());
        let last = analytics.learning_curve.last().unwrap();
        assert_eq!(last.episode_number, 2);
        assert!((last.cumulative_success_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_trends_monthly() {
        use crate::episode::Episode;

        let ep = Episode::new("proj".to_string(), "task".to_string());
        let analytics = compute_trends(&[ep], "monthly");

        assert_eq!(analytics.total_episodes, 1);
        // Monthly bucket should be YYYY-MM format
        assert!(analytics.buckets[0].period.len() == 7); // e.g. "2026-04"
    }

    #[test]
    fn test_render_trends_not_empty() {
        use crate::episode::Episode;

        let mut ep = Episode::new("proj".to_string(), "task".to_string());
        ep.outcome.status = OutcomeStatus::Success;
        ep.intent.domain = vec!["rust".to_string()];

        let analytics = compute_trends(&[ep], "weekly");
        let output = render_trends(&analytics);

        assert!(output.contains("Trend Analytics"));
        assert!(output.contains("Helpfulness Over Time"));
        assert!(output.contains("Domain Growth"));
        assert!(output.contains("rust"));
    }
}
