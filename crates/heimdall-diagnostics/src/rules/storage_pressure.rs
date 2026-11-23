use chrono::Utc;
use heimdall_core::{Confidence, Diagnosis, DiagnosisSeverity, Evidence, RecommendedAction};
use uuid::Uuid;

use crate::config::DiagnosticConfig;
use crate::util::metric_u64;
use heimdall_core::TelemetrySnapshot;

pub fn evaluate_storage_pressure(
    config: &DiagnosticConfig,
    current: &TelemetrySnapshot,
) -> Option<Diagnosis> {
    let mut critical_disks = Vec::new();

    for disk in &current.storage {
        let total = metric_u64(&disk.total_bytes);
        let used = metric_u64(&disk.used_bytes);
        if total.is_none() || used.is_none() {
            continue;
        }
        let total = total.unwrap();
        let used = used.unwrap();
        if total == 0 {
            continue;
        }
        let ratio = used as f64 / total as f64;
        if ratio >= config.storage_used_ratio_warn {
            critical_disks.push((disk.name.clone(), disk.mount_point.clone(), ratio, used, total));
        }
    }

    if critical_disks.is_empty() {
        return None;
    }

    let (name, mount, ratio, used, total) = critical_disks
        .iter()
        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        .cloned()?;

    let mount_label = mount.unwrap_or_else(|| "—".into());

    Some(Diagnosis {
        id: Uuid::new_v4(),
        rule_id: "storage_pressure".into(),
        title: "storage_pressure".into(),
        explanation: format!("storage_pressure:{name}:{mount_label}:{:.3}", ratio),
        severity: if ratio >= 0.98 {
            DiagnosisSeverity::Critical
        } else {
            DiagnosisSeverity::Warning
        },
        observed_at: Utc::now(),
        first_detected_at: Utc::now(),
        last_seen_at: Utc::now(),
        lifecycle_state: heimdall_core::DiagnosisLifecycleState::Detected,
        active_seconds: 0,
        evidence: vec![Evidence {
            kind: "metric".into(),
            reference: format!("storage.{name}.used_bytes"),
            summary: format!("Used {used} / {total} bytes."),
        }],
        affected_resources: vec!["storage".into()],
        related_processes: vec![],
        confidence: Confidence::High,
        recommended_actions: vec![RecommendedAction {
            kind: "suggest_cleanup".into(),
            label: "Free disk space".into(),
            detail: Some("Remove caches, old builds, or move data to another volume.".into()),
        }],
        limitations: vec![if critical_disks.len() > 1 {
            format!("{} additional volume(s) are also above the threshold.", critical_disks.len() - 1)
        } else {
            String::new()
        }]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect(),
    })
}