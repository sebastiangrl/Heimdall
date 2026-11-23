use heimdall_core::{Confidence, Diagnosis, Evidence};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalHintKind {
    Oom,
    Nvrm,
    IoError,
}

#[derive(Debug, Clone)]
pub struct JournalHint {
    pub kind: JournalHintKind,
    pub message: String,
}

pub fn enrich_diagnoses(diagnoses: &mut [Diagnosis], hints: &[JournalHint]) {
    if hints.is_empty() {
        return;
    }
    for d in diagnoses.iter_mut() {
        for hint in hints {
            match (&d.rule_id as &str, hint.kind) {
                ("memory_pressure", JournalHintKind::Oom) => {
                    d.evidence.push(Evidence {
                        kind: "journal".into(),
                        reference: "journald:oom".into(),
                        summary: truncate_summary(&hint.message, 200),
                    });
                    d.confidence = boost_confidence(d.confidence);
                }
                ("gpu_vram_pressure" | "thermal_gpu", JournalHintKind::Nvrm) => {
                    d.evidence.push(Evidence {
                        kind: "journal".into(),
                        reference: "journald:nvrm".into(),
                        summary: truncate_summary(&hint.message, 200),
                    });
                    d.confidence = boost_confidence(d.confidence);
                }
                ("io_contention" | "storage_pressure", JournalHintKind::IoError) => {
                    d.evidence.push(Evidence {
                        kind: "journal".into(),
                        reference: "journald:io".into(),
                        summary: truncate_summary(&hint.message, 200),
                    });
                    d.confidence = boost_confidence(d.confidence);
                }
                _ => {}
            }
        }
    }
}

fn boost_confidence(c: Confidence) -> Confidence {
    match c {
        Confidence::Low => Confidence::Medium,
        Confidence::Medium => Confidence::High,
        Confidence::High => Confidence::High,
    }
}

fn truncate_summary(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!("{}…", s.chars().take(max).collect::<String>())
}