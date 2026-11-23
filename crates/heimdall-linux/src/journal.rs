use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JournalSignalKind {
    Oom,
    Nvrm,
    IoError,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalSignal {
    pub kind: JournalSignalKind,
    pub message: String,
}

/// Classify a journal line (unit-tested without calling journalctl).
pub fn classify_journal_line(line: &str) -> Option<JournalSignal> {
    let lower = line.to_ascii_lowercase();
    let kind = if lower.contains("out of memory")
        || lower.contains("oom-kill")
        || lower.contains("invoked oom-killer")
        || (lower.contains("killed process") && lower.contains("oom"))
    {
        JournalSignalKind::Oom
    } else if lower.contains("nvrm")
        || lower.contains("xid")
        || (lower.contains("nvidia")
            && (lower.contains("gpu has fallen off the bus") || lower.contains("rm init")))
    {
        JournalSignalKind::Nvrm
    } else if lower.contains("i/o error")
        || lower.contains("blk_update_request")
        || lower.contains("ext4-fs error")
    {
        JournalSignalKind::IoError
    } else {
        return None;
    };
    Some(JournalSignal {
        kind,
        message: line.trim().to_string(),
    })
}

pub fn fetch_recent_signals(max_lines: usize) -> Vec<JournalSignal> {
    let output = Command::new("journalctl")
        .args([
            "-b",
            "--no-pager",
            "-o",
            "short-iso",
            "-n",
            &max_lines.to_string(),
            "-p",
            "warning..alert",
        ])
        .output();

    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .filter_map(classify_journal_line)
        .take(12)
        .collect()
}

pub struct JournalCache {
    last_fetch: Option<Instant>,
    signals: Vec<JournalSignal>,
    ttl: Duration,
}

impl JournalCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            last_fetch: None,
            signals: Vec::new(),
            ttl,
        }
    }

    pub fn signals(&mut self) -> &[JournalSignal] {
        let stale = self
            .last_fetch
            .map(|t| t.elapsed() >= self.ttl)
            .unwrap_or(true);
        if stale {
            self.signals = fetch_recent_signals(120);
            self.last_fetch = Some(Instant::now());
        }
        &self.signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_oom_and_nvrm() {
        let oom = classify_journal_line("kernel: Out of memory: Killed process 1234 (stress)").unwrap();
        assert_eq!(oom.kind, JournalSignalKind::Oom);
        let nv = classify_journal_line("kernel: NVRM: Xid (PCI:0000:01:00): 13, ...").unwrap();
        assert_eq!(nv.kind, JournalSignalKind::Nvrm);
    }

    #[test]
    fn ignores_unrelated_lines() {
        assert!(classify_journal_line("systemd: Started Heimdall").is_none());
    }
}