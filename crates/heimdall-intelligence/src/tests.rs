use crate::config::IntelligenceSettings;
use crate::ollama::{ask, build_context};

#[test]
fn blocked_ask_does_not_require_network() {
    let r = ask(
        Some("http://127.0.0.1:1"),
        "test-model",
        "{}",
        "¿Estoy bien?",
        true,
    );
    assert!(r.text.is_empty());
    assert!(r.blocked_reason.is_some());
    assert!(!r.queued);
}

#[test]
fn build_context_includes_hostname_and_counts() {
    let ctx = build_context("asgard", "[]", "{}", 7);
    let v: serde_json::Value = serde_json::from_str(&ctx).expect("json");
    assert_eq!(v["hostname"], "asgard");
    assert_eq!(v["blackbox_incidents"], 7);
}

#[test]
fn settings_default_url() {
    let s = IntelligenceSettings::default();
    assert!(s.defer_when_blocked);
    assert!(s.ollama_base_url.contains("11434"));
}