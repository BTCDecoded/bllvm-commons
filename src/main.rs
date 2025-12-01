use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::Datelike;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::Duration;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod audit;
mod authorization;
mod backup;
mod build;
mod config;
mod crypto;
mod database;
mod economic_nodes;
mod enforcement;
mod error;
mod github;
mod governance;
mod node_registry;
mod nostr;
mod ots;
mod resilience;
mod validation;
mod webhooks;

use audit::AuditLogger;
use authorization::internal_api::internal_api_auth_middleware;
use config::AppConfig;
use database::Database;
use economic_nodes::EconomicNodeRegistry;
use governance::{ConfigRegistry, ContributionAggregator, FeeForwardingTracker};
use governance::message_dedup::MessageDeduplicator;
use nostr::{NostrClient, StatusPublisher, ZapTracker};
use ots::{OtsClient, RegistryAnchorer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bllvm_commons=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Bitcoin Commons (bllvm-commons)");

    // Load configuration
    let config = AppConfig::load()?;
    info!("Configuration loaded");

    // Initialize database
    let database = Database::new(&config.database_url).await?;
    info!("Database connected");

    // Run migrations
    database.run_migrations().await?;
    info!("Database migrations completed");

    // Start automated backup task
    let database_for_backup = database.clone();
    let backup_config = backup::BackupConfig {
        directory: std::path::PathBuf::from("/opt/bllvm-commons/backups"),
        retention_days: 30,
        compression: true,
        interval: std::time::Duration::from_secs(86400), // Daily
        enabled: true,
    };
    let backup_manager = Arc::new(backup::BackupManager::new(
        database_for_backup,
        backup_config,
    ));
    backup_manager.clone().start_backup_task();
    info!("Automated backup task started");

    // Start database health monitoring task with reconnection capability
    let database_for_health = database.clone();
    let database_url_for_reconnect = config.database_url.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Check every 60 seconds
        let mut consecutive_failures = 0u32;
        let mut current_db = database_for_health;

        loop {
            interval.tick().await;

            // Check database health
            match current_db.check_health().await {
                Ok(true) => {
                    if consecutive_failures > 0 {
                        info!(
                            "Database health check passed after {} failures",
                            consecutive_failures
                        );
                        consecutive_failures = 0;
                    }

                    // Log pool stats periodically (every 10 checks = 10 minutes)
                    if consecutive_failures == 0 {
                        if let Ok(stats) = current_db.get_pool_stats().await {
                            debug!(
                                "Database pool stats: size={}, idle={}, closed={}",
                                stats.size, stats.idle, stats.is_closed
                            );
                        }
                    }
                }
                Ok(false) | Err(_) => {
                    consecutive_failures += 1;
                    warn!(
                        "Database health check failed (consecutive failures: {})",
                        consecutive_failures
                    );

                    // After 3 consecutive failures, attempt reconnection
                    if consecutive_failures >= 3 {
                        error!("Database connection unhealthy after {} consecutive failures - attempting reconnection", consecutive_failures);

                        // Check if pool is closed before attempting reconnection
                        let should_reconnect = current_db
                            .get_pool_stats()
                            .await
                            .map(|stats| stats.is_closed)
                            .unwrap_or(true);

                        if should_reconnect {
                            // Attempt to reconnect using stored database URL
                            match Database::new(&database_url_for_reconnect).await {
                                Ok(new_db) => {
                                    info!("Database reconnection successful");
                                    current_db = new_db;
                                    consecutive_failures = 0;
                                }
                                Err(e) => {
                                    error!("Database reconnection failed: {} - will retry on next health check", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Initialize audit logger
    let audit_logger = if config.audit.enabled {
        Some(AuditLogger::new(config.audit.log_path.clone())?)
    } else {
        None
    };
    info!("Audit logger initialized");

    // Initialize Nostr client and status publisher
    let nostr_client = if config.nostr.enabled {
        let nsec = std::fs::read_to_string(&config.nostr.server_nsec_path)
            .map_err(|e| format!("Failed to read Nostr key: {}", e))?;

        let client = NostrClient::new(nsec, config.nostr.relays.clone())
            .await
            .map_err(|e| format!("Failed to create Nostr client: {}", e))?;

        Some(client)
    } else {
        None
    };

    let status_publisher = if let Some(ref client) = nostr_client {
        Some(StatusPublisher::new(
            client.clone(),
            database.clone(),
            config.server_id.clone(),
            std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "bllvm-commons".to_string()),
            "config.toml".to_string(),
            if config.audit.enabled {
                Some(config.audit.log_path.clone())
            } else {
                None
            },
        ))
    } else {
        None
    };

    // Initialize OTS client and registry anchorer
    let ots_client = if config.ots.enabled {
        Some(OtsClient::new(config.ots.aggregator_url.clone()))
    } else {
        None
    };

    let registry_anchorer = if let Some(client) = ots_client {
        Some(RegistryAnchorer::new(
            client,
            database.clone(),
            config.ots.registry_path.clone(),
            config.ots.proofs_path.clone(),
        ))
    } else {
        None
    };

    // Start background tasks
    let config_clone = config.clone();
    let database_clone = database.clone();
    // TODO: Implement audit logger cloning or use Arc

    // Nostr status publisher task
    if let Some(publisher) = status_publisher {
        let publish_interval = Duration::from_secs(config.nostr.publish_interval_secs);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(publish_interval);
            loop {
                interval.tick().await;
                if let Err(e) = publisher.publish_status().await {
                    error!("Failed to publish Nostr status: {}", e);
                }
            }
        });
        info!("Nostr status publisher started");
    }

    // OTS monthly anchoring task
    if let Some(anchorer) = registry_anchorer {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(86400)); // Check daily
            loop {
                interval.tick().await;
                let now = chrono::Utc::now();
                if now.day() == config_clone.ots.monthly_anchor_day as u32 {
                    if let Err(e) = anchorer.anchor_registry().await {
                        error!("Failed to anchor registry: {}", e);
                    }
                }
            }
        });
        info!("OTS registry anchorer started");
    }

    // Audit log rotation task
    if audit_logger.is_some() {
        let rotation_interval =
            Duration::from_secs(config.audit.rotation_interval_days as u64 * 86400);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(rotation_interval);
            loop {
                interval.tick().await;
                // Rotate audit log (implement rotation logic)
                info!("Audit log rotation triggered");
            }
        });
        info!("Audit log rotation started");
    }

    // Initialize governance services
    let pool = database
        .get_sqlite_pool()
        .ok_or_else(|| "Database pool not available".to_string())?;

    // Start zap tracker if Nostr is enabled and governance tracking enabled
    if config.nostr.enabled && config.governance.contribution_tracking_enabled {
        if let Some(ref nostr_client) = nostr_client {
            // Collect all bot pubkeys for zap tracking
            let mut bot_pubkeys = Vec::new();
            if let Some(zap_addr) = &config.nostr.zap_address {
                // Legacy single bot - extract pubkey if available
                // For now, we'll need the pubkey from the nsec
                if let Ok(nsec) = std::fs::read_to_string(&config.nostr.server_nsec_path) {
                    // Parse nsec to get pubkey (simplified - in production use proper parsing)
                    // For now, we'll track the configured zap address
                    info!("Zap tracking configured for legacy zap address");
                }
            }

            // Add bot pubkeys from multi-bot config
            for (bot_id, bot_config) in &config.nostr.bots {
                bot_pubkeys.push(bot_config.npub.clone());
                info!(
                    "Zap tracking configured for bot: {} (npub: {})",
                    bot_id, bot_config.npub
                );
            }

            if !bot_pubkeys.is_empty() {
                let zap_tracker =
                    ZapTracker::new(pool.clone(), Arc::new(nostr_client.clone()), bot_pubkeys);
                if let Err(e) = zap_tracker.start_tracking().await {
                    error!("Failed to start zap tracking: {}", e);
                } else {
                    info!("Zap tracker started");
                }
            }
        }
    }

    // Initialize fee forwarding tracker (if Commons addresses configured)
    let fee_forwarding_tracker = if !config.governance.commons_addresses.is_empty() {
        Some(FeeForwardingTracker::from_network_string(
            pool.clone(),
            config.governance.commons_addresses.clone(),
            &config.governance.network,
        ))
    } else {
        None
    };

    if fee_forwarding_tracker.is_some() {
        info!(
            "Fee forwarding tracker initialized for {} Commons addresses on {}",
            config.governance.commons_addresses.len(),
            config.governance.network
        );
    }

    // Start periodic weight update task (if enabled)
    if config.governance.weight_updates_enabled {
        let pool_for_weights = pool.clone();
        let update_interval = Duration::from_secs(config.governance.weight_update_interval_secs);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            loop {
                interval.tick().await;
                info!("Starting periodic weight update");

                let aggregator = ContributionAggregator::new(pool_for_weights.clone());
                if let Err(e) = aggregator.update_all_weights().await {
                    error!("Failed to update participation weights: {}", e);
                } else {
                    info!("Periodic weight update completed");
                }
            }
        });
        info!(
            "Periodic weight update task started (interval: {}s)",
            config.governance.weight_update_interval_secs
        );
    }

    // Initialize P2P message deduplicator
    let dedup = Arc::new(MessageDeduplicator::new(pool.clone()));
    
    // Initialize governance-controlled configuration registry
    let mut config_registry = ConfigRegistry::new(pool.clone());
    
    // Try to find governance config path from environment or use default
    let governance_config_path = std::env::var("GOVERNANCE_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            // Try relative paths
            let candidates = vec![
                PathBuf::from("../governance/config"),
                PathBuf::from("governance/config"),
                PathBuf::from("./governance/config"),
            ];
            candidates.into_iter().find(|p| p.exists())
        });
    
    // Set config path for YAML sync
    if let Some(ref config_path) = governance_config_path {
        config_registry.set_config_path(config_path.clone());
    }
    
    // Sync from YAML files first (source of truth)
    if let Some(ref config_path) = governance_config_path {
        if let Err(e) = config_registry.sync_from_yaml(config_path.clone()).await {
            warn!("Failed to sync from YAML files: {}. Continuing with defaults.", e);
        } else {
            info!("Synced configuration from YAML files");
        }
    }
    
    // Initialize all governance configuration defaults (forkable variables)
    // This will register any missing configs (fallback to hardcoded if YAML not available)
    if let Err(e) = governance::initialize_governance_defaults(
        &Arc::new(config_registry.clone()),
        governance_config_path.clone(),
    ).await {
        warn!("Failed to initialize governance defaults: {}", e);
        // Continue anyway - defaults may already be registered
    } else {
        info!("Governance-controlled configuration defaults initialized");
    }

    // Create ConfigReader for unified config access
    // Add YAML loader for fallback access to YAML files
    let yaml_loader = governance_config_path.as_ref()
        .map(|path| governance::yaml_loader::YamlConfigLoader::new(path.clone()));
    let config_reader = Arc::new(governance::ConfigReader::with_yaml_loader(
        Arc::new(config_registry.clone()),
        yaml_loader,
    ));
    
    // Link ConfigReader to ConfigRegistry for automatic cache invalidation
    config_registry.set_config_reader(config_reader.clone());
    let config_registry = Arc::new(config_registry);
    
    info!("Configuration reader initialized with automatic cache invalidation");

    // Initialize EconomicNodeRegistry with ConfigReader-enabled phase calculator
    let phase_calculator = Arc::new(GovernancePhaseCalculator::with_config(
        pool.clone(),
        config_reader.clone(),
    ));
    let registry = Arc::new(EconomicNodeRegistry::with_config_reader(
        pool.clone(),
        Some(config_reader.clone()),
        Some((*phase_calculator).clone()),
    ));

    // Start message deduplication cleanup task
    let dedup_for_cleanup = dedup.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Cleanup every hour
        loop {
            interval.tick().await;
            if let Err(e) = dedup_for_cleanup.cleanup().await {
                error!("Failed to cleanup message deduplication: {}", e);
            }
        }
    });
    info!("Message deduplication cleanup task started");

    // Build application
    let port = config.server_port;
    
    // Create VetoManager with ConfigReader for governance-controlled thresholds
    let veto_manager = Arc::new(economic_nodes::VetoManager::with_config(
        pool.clone(),
        config_reader.clone(),
    ));
    
    // Add node registry API routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/webhooks/github", post(webhooks::github::handle_webhook))
        .route(
            "/webhooks/block",
            post(webhooks::block::handle_block_notification),
        )
        .route("/status", get(status_endpoint))
        .merge(node_registry::api::create_router())
        // Merkle proof endpoints (public, for economic nodes to verify their signals were counted)
        .route(
            "/governance/merkle/proof/:pr_id/:voting_key",
            get(handle_merkle_proof),
        )
        .route(
            "/governance/merkle/root/:pr_id",
            get(handle_merkle_root),
        )
        // Add internal API routes for P2P governance messages (protected by API key)
        .nest(
            "/internal",
            governance::internal_api::create_internal_router(pool.clone(), registry.clone(), dedup.clone())
                .layer(axum::middleware::from_fn(internal_api_auth_middleware)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state((config, database, veto_manager, config_registry.clone()));

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "bllvm-commons",
        "timestamp": chrono::Utc::now()
    }))
}

/// Handle Merkle proof request for a voting key
async fn handle_merkle_proof(
    axum::extract::Path((pr_id, voting_key)): axum::extract::Path<(i32, String)>,
    State((_config, _database, veto_manager, _config_registry)): State<(AppConfig, Database, Arc<economic_nodes::VetoManager>, Arc<governance::ConfigRegistry>)>,
) -> Json<serde_json::Value> {
    match veto_manager.get_merkle_proof_for_voting_key(pr_id, &voting_key).await {
        Ok(Some(proof)) => Json(serde_json::json!({
            "status": "ok",
            "pr_id": pr_id,
            "voting_key": voting_key,
            "proof": proof
        })),
        Ok(None) => Json(serde_json::json!({
            "status": "not_found",
            "pr_id": pr_id,
            "voting_key": voting_key,
            "message": "No Merkle proof found for this voting key and PR"
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "pr_id": pr_id,
            "voting_key": voting_key,
            "error": e.to_string()
        }))
    }
}

/// Handle Merkle root hash request for a PR
async fn handle_merkle_root(
    axum::extract::Path(pr_id): axum::extract::Path<i32>,
    State((_config, _database, veto_manager, _config_registry)): State<(AppConfig, Database, Arc<economic_nodes::VetoManager>, Arc<governance::ConfigRegistry>)>,
) -> Json<serde_json::Value> {
    match veto_manager.get_merkle_root_hash(pr_id).await {
        Ok(Some(root_hash)) => Json(serde_json::json!({
            "status": "ok",
            "pr_id": pr_id,
            "root_hash": root_hash
        })),
        Ok(None) => Json(serde_json::json!({
            "status": "not_found",
            "pr_id": pr_id,
            "message": "No veto signals found for this PR"
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "pr_id": pr_id,
            "error": e.to_string()
        }))
    }
}

async fn status_endpoint(
    State((config, database, _veto_manager, _config_registry)): State<(AppConfig, Database, Arc<economic_nodes::VetoManager>, Arc<governance::ConfigRegistry>)>,
) -> Json<serde_json::Value> {
    let pool = database.get_sqlite_pool();
    let governance_status = if let Some(pool) = pool {
        // Check governance tables exist
        let tables_exist = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('unified_contributions', 'participation_weights', 'zap_contributions')"
        )
        .fetch_one(pool)
        .await
        .ok()
        .map(|count| count >= 3)
        .unwrap_or(false);

        // Get contributor count
        let contributor_count: i64 =
            sqlx::query_scalar("SELECT COUNT(DISTINCT contributor_id) FROM unified_contributions")
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        serde_json::json!({
            "enabled": config.governance.contribution_tracking_enabled,
            "tables_exist": tables_exist,
            "contributor_count": contributor_count,
            "weight_updates_enabled": config.governance.weight_updates_enabled,
            "commons_addresses_count": config.governance.commons_addresses.len(),
        })
    } else {
        serde_json::json!({
            "enabled": false,
            "error": "Database pool not available"
        })
    };

    let mut status = serde_json::json!({
        "status": "healthy",
        "service": "bllvm-commons",
        "timestamp": chrono::Utc::now(),
        "server_id": config.server_id,
        "features": {
            "nostr": config.nostr.enabled,
            "ots": config.ots.enabled,
            "audit": config.audit.enabled,
            "dry_run": config.dry_run_mode,
            "governance": governance_status,
        }
    });

    // Add database status
    if let Ok(stats) = database.get_performance_stats().await {
        status["database"] = serde_json::json!({
            "status": "healthy",
            "cache_size": stats.cache_size,
            "slow_queries": stats.slow_queries_count
        });
    } else {
        status["database"] = serde_json::json!({
            "status": "error"
        });
    }

    Json(status)
}
