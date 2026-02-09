use std::{fs, path::Path};

use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

use crate::types::{AppSettings, ExportResult, HistoryFilter, HistoryPage, SpeedTestResult, TimeRange};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).context("failed to create app data dir")?;
        }

        let url_path = db_path.to_string_lossy().replace("\\", "/");
        let url = if url_path.contains(":/") {
            format!("sqlite:///{}", url_path)
        } else {
            format!("sqlite://{}", url_path)
        };
        let pool = SqlitePoolOptions::new().max_connections(4).connect(&url).await?;

        let db = Self { pool };
        db.initialize().await?;
        Ok(db)
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                json TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS speed_tests (
                task_id TEXT PRIMARY KEY,
                endpoint TEXT NOT NULL,
                download_mbps REAL NOT NULL,
                upload_mbps REAL,
                latency_ms REAL,
                jitter_ms REAL,
                loss_pct REAL,
                started_at TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_settings(&self) -> anyhow::Result<Option<AppSettings>> {
        let row = sqlx::query("SELECT json FROM settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let raw: String = row.try_get("json")?;
            let parsed: AppSettings = serde_json::from_str(&raw)?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let raw = serde_json::to_string(settings)?;
        sqlx::query("INSERT INTO settings (id, json) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET json = excluded.json")
            .bind(raw)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn insert_speed_test(&self, result: &SpeedTestResult) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO speed_tests (
                task_id,
                endpoint,
                download_mbps,
                upload_mbps,
                latency_ms,
                jitter_ms,
                loss_pct,
                started_at,
                duration_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&result.task_id)
        .bind(&result.endpoint)
        .bind(result.download_mbps)
        .bind(result.upload_mbps)
        .bind(result.latency_ms)
        .bind(result.jitter_ms)
        .bind(result.loss_pct)
        .bind(result.started_at.to_rfc3339())
        .bind(result.duration_ms)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn query_history(&self, filter: &HistoryFilter) -> anyhow::Result<HistoryPage> {
        let page = filter.page.max(1);
        let page_size = filter.page_size.clamp(1, 200);
        let offset = (page - 1) * page_size;

        let mut where_parts: Vec<String> = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(from) = filter.from {
            where_parts.push("started_at >= ?".to_string());
            binds.push(from.to_rfc3339());
        }
        if let Some(to) = filter.to {
            where_parts.push("started_at <= ?".to_string());
            binds.push(to.to_rfc3339());
        }

        let where_sql = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(1) as total FROM speed_tests {}", where_sql);
        let mut count_q = sqlx::query(&count_sql);
        for b in &binds {
            count_q = count_q.bind(b);
        }
        let total_row = count_q.fetch_one(&self.pool).await?;
        let total: i64 = total_row.try_get("total")?;

        let items_sql = format!(
            "SELECT task_id, endpoint, download_mbps, upload_mbps, latency_ms, jitter_ms, loss_pct, started_at, duration_ms
             FROM speed_tests {}
             ORDER BY started_at DESC
             LIMIT ? OFFSET ?",
            where_sql
        );

        let mut items_q = sqlx::query(&items_sql);
        for b in &binds {
            items_q = items_q.bind(b);
        }
        items_q = items_q.bind(page_size as i64).bind(offset as i64);

        let rows = items_q.fetch_all(&self.pool).await?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let started_raw: String = row.try_get("started_at")?;
            let started_at = DateTime::parse_from_rfc3339(&started_raw)
                .map(|v| v.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            items.push(SpeedTestResult {
                task_id: row.try_get("task_id")?,
                endpoint: row.try_get("endpoint")?,
                download_mbps: row.try_get("download_mbps")?,
                upload_mbps: row.try_get("upload_mbps")?,
                latency_ms: row.try_get("latency_ms")?,
                jitter_ms: row.try_get("jitter_ms")?,
                loss_pct: row.try_get("loss_pct")?,
                started_at,
                duration_ms: row.try_get("duration_ms")?,
            });
        }

        Ok(HistoryPage { total, items })
    }

    pub async fn export_history_csv(&self, export_path: &Path, range: &TimeRange) -> anyhow::Result<ExportResult> {
        if let Some(parent) = export_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let filter = HistoryFilter {
            page: 1,
            page_size: 10_000,
            from: range.from,
            to: range.to,
        };
        let page = self.query_history(&filter).await?;

        let mut csv = String::from("task_id,endpoint,download_mbps,upload_mbps,latency_ms,jitter_ms,loss_pct,started_at,duration_ms\n");
        for item in &page.items {
            csv.push_str(&format!(
                "{},{},{:.4},{},{},{},{},{},{}\n",
                item.task_id,
                item.endpoint,
                item.download_mbps,
                item.upload_mbps.map(|v| format!("{v:.4}")).unwrap_or_default(),
                item.latency_ms.map(|v| format!("{v:.4}")).unwrap_or_default(),
                item.jitter_ms.map(|v| format!("{v:.4}")).unwrap_or_default(),
                item.loss_pct.map(|v| format!("{v:.4}")).unwrap_or_default(),
                item.started_at.to_rfc3339(),
                item.duration_ms,
            ));
        }

        fs::write(export_path, csv)?;
        Ok(ExportResult {
            path: export_path.to_string_lossy().to_string(),
            rows: page.items.len() as u64,
        })
    }

    pub async fn prune_old_history(&self, keep_days: i64) -> anyhow::Result<()> {
        let cutoff = (Utc::now() - Duration::days(keep_days.max(1))).to_rfc3339();
        sqlx::query("DELETE FROM speed_tests WHERE started_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}


