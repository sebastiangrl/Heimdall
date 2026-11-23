//! Integración opcional con Ollama (Fase 5). Heimdall funciona sin Ollama.

mod config;
mod ollama;
mod pending;

#[cfg(test)]
mod tests;

pub use config::{IntelligenceSettings, IntelligenceStore, DEFAULT_OLLAMA_URL};
pub use ollama::{ask, build_context, probe, AssistantReply, OllamaStatus};
pub use pending::{PendingAssistantRequest, PENDING_MAX_AGE_SECS};