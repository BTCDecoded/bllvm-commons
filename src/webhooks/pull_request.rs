use serde_json::Value;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::database::Database;
use crate::nostr::{publish_merge_action, publish_review_period_notification};
use crate::validation::tier_classification;
use crate::validation::threshold::ThresholdValidator;

pub async fn handle_pull_request_event(
    database: &Database,
    payload: &Value,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    let repo_name = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let pr_number = payload
        .get("pull_request")
        .and_then(|pr| pr.get("number"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);

    let head_sha = payload
        .get("pull_request")
        .and_then(|pr| pr.get("head").and_then(|h| h.get("sha")))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    info!("Processing PR #{} in {}", pr_number, repo_name);

    // Determine layer based on repository
    let layer = match repo_name {
        repo if repo.contains("bllvm-spec") || repo.contains("orange-paper") => 1,
        repo if repo.contains("bllvm-consensus") || repo.contains("consensus-proof") => 2,
        repo if repo.contains("bllvm-protocol") || repo.contains("protocol-engine") => 3,
        repo if repo.contains("bllvm-node") || repo.contains("reference-node") || repo.contains("/bllvm") => 4,
        repo if repo.contains("bllvm-sdk") || repo.contains("developer-sdk") => 5,
        repo if repo.contains("bllvm-commons") || repo.contains("governance-app") => 6,
        _ => {
            warn!("Unknown repository: {}", repo_name);
            return Ok(axum::response::Json(
                serde_json::json!({"status": "unknown_repo"}),
            ));
        }
    };

    // Classify PR tier based on file changes (check for override first)
    let tier = tier_classification::classify_pr_tier_with_db(
        database,
        payload,
        repo_name,
        pr_number as i32,
    ).await;
    info!("PR #{} classified as Tier {}", pr_number, tier);

    // Store PR in database
    match database
        .create_pull_request(repo_name, pr_number as i32, head_sha, layer)
        .await
    {
        Ok(_) => {
            info!("PR #{} stored in database", pr_number);

            // Log governance event
            let _ = database
                .log_governance_event(
                    "pr_opened",
                    Some(repo_name),
                    Some(pr_number as i32),
                    None,
                    &serde_json::json!({
                        "tier": tier,
                        "layer": layer,
                        "head_sha": head_sha
                    }),
                )
                .await;

            // Publish review period notification to Nostr (if enabled)
            // Note: This requires config to be passed, which we don't have here
            // For now, we'll publish from the status check handler instead
            // TODO: Pass config to this handler or publish from status check handler

            Ok(axum::response::Json(serde_json::json!({
                "status": "stored",
                "tier": tier,
                "layer": layer
            })))
        }
        Err(e) => {
            warn!("Failed to store PR: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle PR merge event - publish to Nostr
pub async fn handle_pr_merged(
    config: &AppConfig,
    database: &Database,
    payload: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_name = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let pr_number = payload
        .get("pull_request")
        .and_then(|pr| pr.get("number"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0) as i32;

    let commit_hash = payload
        .get("pull_request")
        .and_then(|pr| pr.get("merge_commit_sha"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    info!("PR #{} merged in {}, publishing to Nostr", pr_number, repo_name);

    // Get PR info to determine layer and tier
    let pr_info = database.get_pull_request(repo_name, pr_number).await?;
    
    if let Some(pr) = pr_info {
        let layer = pr.layer;
        
        // Get tier from database or re-classify
        // For now, we'll need to get it from the PR details or re-classify
        // This is a simplified version - in practice, tier should be stored with PR
        let tier = tier_classification::classify_pr_tier_with_db(
            database,
            payload,
            repo_name,
            pr_number,
        ).await;

        // Publish merge action to Nostr
        publish_merge_action(
            config,
            database,
            repo_name,
            pr_number,
            commit_hash,
            layer,
            tier,
        ).await?;

        info!("Successfully published merge action to Nostr");
    } else {
        warn!("PR #{} not found in database, cannot publish to Nostr", pr_number);
    }

    Ok(())
}
