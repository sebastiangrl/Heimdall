use chrono::{DateTime, Utc};

pub const PENDING_MAX_AGE_SECS: i64 = 3_600;

#[derive(Debug, Clone)]
pub struct PendingAssistantRequest {
    pub question: String,
    pub model: String,
    pub queued_at: DateTime<Utc>,
}

impl PendingAssistantRequest {
    pub fn expired(&self) -> bool {
        (Utc::now() - self.queued_at).num_seconds() > PENDING_MAX_AGE_SECS
    }
}