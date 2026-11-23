use std::path::PathBuf;

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::config::InvestigationConfig;
use crate::types::{InvestigationDossier, InvestigationState, InvestigationSummary};
use crate::InvestigationEngine;
use heimdall_blackbox::TelemetryFrame;

struct PostCapture {
    id: Uuid,
    remaining: usize,
}

pub struct InvestigationStore {
    conn: Connection,
    config: InvestigationConfig,
    post_captures: Vec<PostCapture>,
}

impl InvestigationStore {
    pub fn open_default() -> Result<Self, String> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("heimdall");
        Self::open(data_dir.join("blackbox.db"))
    }

    pub fn open(db_path: PathBuf) -> Result<Self, String> {
        let config = InvestigationConfig::from_env();
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
        Self::ensure_schema(&conn)?;
        Ok(Self {
            conn,
            config,
            post_captures: Vec::new(),
        })
    }

    fn ensure_schema(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS investigations (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                state TEXT NOT NULL,
                level TEXT NOT NULL,
                rule_id TEXT NOT NULL,
                identity_json TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                incomplete INTEGER NOT NULL DEFAULT 0
            );
            ",
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn upsert(&self, dossier: &InvestigationDossier) -> Result<(), String> {
        let payload = serde_json::to_string(dossier).map_err(|e| e.to_string())?;
        let identity = serde_json::to_string(&dossier.identity).map_err(|e| e.to_string())?;
        let state = serde_json::to_string(&dossier.state)
            .map_err(|e| e.to_string())?
            .trim_matches('"')
            .to_string();
        let level = serde_json::to_string(&dossier.level)
            .map_err(|e| e.to_string())?
            .trim_matches('"')
            .to_string();
        let incomplete = dossier.state == InvestigationState::Incomplete;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO investigations
                (id, created_at, updated_at, state, level, rule_id, identity_json, payload_json, incomplete)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    dossier.id.to_string(),
                    dossier.created_at.to_rfc3339(),
                    dossier.updated_at.to_rfc3339(),
                    state,
                    level,
                    dossier.rule_id,
                    identity,
                    payload,
                    incomplete as i32,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn trim(&self) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM investigations WHERE id NOT IN (
                    SELECT id FROM investigations ORDER BY updated_at DESC LIMIT ?1
                )",
                [self.config.max_investigations as i64],
            )
            .map_err(|e| e.to_string())?;
        if self.config.max_investigation_age_days > 0 {
            let cutoff = (Utc::now()
                - chrono::Duration::days(self.config.max_investigation_age_days as i64))
            .to_rfc3339();
            self.conn
                .execute(
                    "DELETE FROM investigations WHERE created_at < ?1",
                    [cutoff],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<InvestigationDossier>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload_json FROM investigations ORDER BY updated_at DESC LIMIT ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([limit as i64], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in rows {
            let json: String = row.map_err(|e| e.to_string())?;
            if let Ok(d) = serde_json::from_str(&json) {
                out.push(d);
            }
        }
        Ok(out)
    }

    pub fn get(&self, id: Uuid) -> Result<Option<InvestigationDossier>, String> {
        let optional = self
            .conn
            .query_row(
                "SELECT payload_json FROM investigations WHERE id = ?1 LIMIT 1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(optional.and_then(|json| serde_json::from_str(&json).ok()))
    }

    pub fn export_json(&self, id: Uuid) -> Result<String, String> {
        let dossier = self
            .get(id)?
            .ok_or_else(|| "investigación no encontrada".to_string())?;
        serde_json::to_string_pretty(&dossier).map_err(|e| e.to_string())
    }

    pub fn apply_step(
        &mut self,
        dossiers: &[InvestigationDossier],
        closed_post: &[(Uuid, usize)],
    ) -> Result<(), String> {
        for d in dossiers {
            self.upsert(d)?;
        }
        self.trim()?;
        for (id, frames) in closed_post {
            self.post_captures.push(PostCapture {
                id: *id,
                remaining: *frames,
            });
        }
        Ok(())
    }

    pub fn append_post_frames(&mut self, frame: &TelemetryFrame) -> Result<(), String> {
        let mut done = Vec::new();
        let mut updates = Vec::new();
        for post in &mut self.post_captures {
            if post.remaining == 0 {
                done.push(post.id);
                continue;
            }
            let last = post.remaining == 1;
            post.remaining -= 1;
            updates.push((post.id, last));
            if last {
                done.push(post.id);
            }
        }
        for (id, last) in updates {
            if let Some(mut dossier) = self.get(id)? {
                dossier.post_frames.push(frame.clone());
                dossier.updated_at = Utc::now();
                if last {
                    dossier.state = InvestigationState::Closed;
                }
                self.upsert(&dossier)?;
            }
        }
        self.post_captures.retain(|p| !done.contains(&p.id));
        Ok(())
    }

    pub fn load_open_into_engine(&self, engine: &mut InvestigationEngine) -> Result<(), String> {
        let all = self.list(500)?;
        for dossier in all {
            if dossier.state == InvestigationState::Open {
                engine.restore_open(dossier);
            }
        }
        Ok(())
    }

    pub fn summaries(&self, limit: usize) -> Result<Vec<InvestigationSummary>, String> {
        let dossiers = self.list(limit)?;
        Ok(InvestigationEngine::summaries_from_dossiers(&dossiers))
    }
}

pub fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("heimdall")
        .join("blackbox.db")
}

pub fn dossier_context_json(dossier: &InvestigationDossier) -> String {
    serde_json::json!({
        "investigationId": dossier.id,
        "level": dossier.level,
        "deterministicExplanation": dossier.explanation,
        "attributionConfidence": dossier.attribution.confidence,
        "impactSeverity": dossier.impact.severity,
        "evidence": dossier.attribution,
        "impact": dossier.impact,
        "limitations": dossier.limitations,
        "instruction": "Explica el expediente solo con estos datos. No modifiques confianza ni crees diagnósticos."
    })
    .to_string()
}