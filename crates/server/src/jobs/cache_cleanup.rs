//! Background task that periodically deletes expired rows from cache tables.

use std::time::Duration;

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use tokio::time::sleep;
use tracing::{error, info, warn};

use super::retention::{CacheTableRetention, CACHE_TABLES};

const INITIAL_DELAY: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Default, Clone)]
pub struct CleanupRunStats {
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_rows_deleted: u64,
    pub per_table: Vec<TableCleanupResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableCleanupResult {
    pub table: &'static str,
    pub tier: &'static str,
    pub rows_deleted: u64,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub fn spawn(db: DatabaseConnection, interval_hours: u64) {
    let interval = Duration::from_secs(interval_hours.max(1) * 3600);

    tokio::spawn(async move {
        sleep(INITIAL_DELAY).await;

        loop {
            match run_once(&db).await {
                Ok(stats) => {
                    info!(
                        rows_deleted = stats.total_rows_deleted,
                        tables = stats.per_table.len(),
                        "cache_cleanup tick finished"
                    );
                }
                Err(e) => {
                    error!(error = %e, "cache_cleanup tick failed");
                }
            }

            sleep(interval).await;
        }
    });
}

pub async fn run_once(db: &DatabaseConnection) -> Result<CleanupRunStats, sea_orm::DbErr> {
    let mut stats = CleanupRunStats {
        started_at: Some(chrono::Utc::now()),
        ..Default::default()
    };
    for entry in CACHE_TABLES {
        let result = clean_table(db, entry).await;
        if let Some(err) = &result.error {
            warn!(
                table = result.table,
                tier = result.tier,
                error = %err,
                "cache_cleanup table failed"
            );
        } else {
            info!(
                table = result.table,
                tier = result.tier,
                rows_deleted = result.rows_deleted,
                duration_ms = result.duration_ms,
                "cache_cleanup table ok"
            );
        }
        stats.total_rows_deleted += result.rows_deleted;
        stats.per_table.push(result);
    }

    let result = clean_cache_entries(db).await;
    if let Some(err) = &result.error {
        warn!(
            table = result.table,
            tier = result.tier,
            error = %err,
            "cache_cleanup cache_entries failed"
        );
    } else {
        info!(
            table = result.table,
            tier = result.tier,
            rows_deleted = result.rows_deleted,
            duration_ms = result.duration_ms,
            "cache_cleanup cache_entries ok"
        );
    }
    stats.total_rows_deleted += result.rows_deleted;
    stats.per_table.push(result);

    stats.finished_at = Some(chrono::Utc::now());

    Ok(stats)
}

async fn clean_table(db: &DatabaseConnection, entry: &CacheTableRetention) -> TableCleanupResult {
    let started_at = std::time::Instant::now();
    let tier = tier_label(entry);
    let Some(retention_secs) = entry.tier.duration_secs() else {
        return TableCleanupResult {
            table: entry.table,
            tier,
            rows_deleted: 0,
            duration_ms: started_at.elapsed().as_millis() as u64,
            error: None,
        };
    };

    let sql = format!(
        "DELETE FROM {} WHERE {} < now() - make_interval(secs => $1)",
        entry.table, entry.timestamp_col
    );

    match db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [(retention_secs as f64).into()],
        ))
        .await
    {
        Ok(result) => TableCleanupResult {
            table: entry.table,
            tier,
            rows_deleted: result.rows_affected(),
            duration_ms: started_at.elapsed().as_millis() as u64,
            error: None,
        },
        Err(err) => TableCleanupResult {
            table: entry.table,
            tier,
            rows_deleted: 0,
            duration_ms: started_at.elapsed().as_millis() as u64,
            error: Some(err.to_string()),
        },
    }
}

async fn clean_cache_entries(db: &DatabaseConnection) -> TableCleanupResult {
    let started_at = std::time::Instant::now();

    match db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "DELETE FROM cache_entries WHERE expires_at < now()".to_string(),
        ))
        .await
    {
        Ok(result) => TableCleanupResult {
            table: "cache_entries",
            tier: "ttl",
            rows_deleted: result.rows_affected(),
            duration_ms: started_at.elapsed().as_millis() as u64,
            error: None,
        },
        Err(err) => TableCleanupResult {
            table: "cache_entries",
            tier: "ttl",
            rows_deleted: 0,
            duration_ms: started_at.elapsed().as_millis() as u64,
            error: Some(err.to_string()),
        },
    }
}

fn tier_label(entry: &CacheTableRetention) -> &'static str {
    match entry.tier {
        super::retention::RetentionTier::Volatile => "volatile",
        super::retention::RetentionTier::Short => "short",
        super::retention::RetentionTier::Medium => "medium",
        super::retention::RetentionTier::Permanent => "permanent",
    }
}
