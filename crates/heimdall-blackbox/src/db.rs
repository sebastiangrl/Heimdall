use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::types::{IncidentRecord, IncidentSeverity};

fn severity_str(s: IncidentSeverity) -> &'static str {
    match s {
        IncidentSeverity::Info => "info",
        IncidentSeverity::Warning => "warning",
        IncidentSeverity::Critical => "critical",
    }
}

pub struct BlackBoxDb {
    conn: Connection,
}

impl BlackBoxDb {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS incidents (
                id TEXT PRIMARY KEY,
                detected_at TEXT NOT NULL,
                severity TEXT NOT NULL,
                kind TEXT NOT NULL,
                title_key TEXT NOT NULL,
                summary TEXT NOT NULL,
                rule_id TEXT,
                payload_json TEXT NOT NULL,
                incomplete INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS ring_checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                saved_at TEXT NOT NULL,
                payload_json TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self { conn })
    }

    pub fn load_latest_ring_checkpoint(&self) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT payload_json FROM ring_checkpoints ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
    }

    pub fn save_ring_checkpoint(&self, frames_json: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO ring_checkpoints (saved_at, payload_json) VALUES (datetime('now'), ?1)",
            [frames_json],
        )?;
        self.conn.execute(
            "DELETE FROM ring_checkpoints WHERE id NOT IN (SELECT id FROM ring_checkpoints ORDER BY id DESC LIMIT 12)",
            [],
        )?;
        Ok(())
    }

    pub fn update_incident(&self, incident: &IncidentRecord) -> rusqlite::Result<()> {
        let payload = serde_json::to_string(incident).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
        self.conn.execute(
            "UPDATE incidents SET payload_json = ?1, incomplete = ?2 WHERE id = ?3",
            params![
                payload,
                incident.incomplete as i32,
                incident.id.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn count_incidents(&self) -> rusqlite::Result<u64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM incidents", [], |r| r.get(0))
    }

    pub fn insert_incident(&self, incident: &IncidentRecord) -> rusqlite::Result<()> {
        let payload = serde_json::to_string(incident).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
        self.conn.execute(
            "INSERT OR REPLACE INTO incidents (id, detected_at, severity, kind, title_key, summary, rule_id, payload_json, incomplete)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                incident.id.to_string(),
                incident.detected_at.to_rfc3339(),
                severity_str(incident.severity),
                incident.kind,
                incident.title_key,
                incident.summary,
                incident.rule_id,
                payload,
                incident.incomplete as i32,
            ],
        )?;
        Ok(())
    }

    pub fn trim_incidents(&self, keep: u32) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM incidents WHERE id NOT IN (
                SELECT id FROM incidents ORDER BY detected_at DESC LIMIT ?1
            )",
            [keep as i64],
        )?;
        Ok(())
    }

    pub fn list_incidents(&self, limit: usize) -> rusqlite::Result<Vec<IncidentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT payload_json FROM incidents ORDER BY detected_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;
        let mut out = Vec::new();
        for row in rows {
            let json = row?;
            if let Ok(record) = serde_json::from_str::<IncidentRecord>(&json) {
                out.push(record);
            }
        }
        Ok(out)
    }

    pub fn get_incident(&self, id: Uuid) -> rusqlite::Result<Option<IncidentRecord>> {
        let mut stmt =
            self.conn
                .prepare("SELECT payload_json FROM incidents WHERE id = ?1 LIMIT 1")?;
        let mut rows = stmt.query([id.to_string()])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            return Ok(serde_json::from_str(&json).ok());
        }
        Ok(None)
    }
}