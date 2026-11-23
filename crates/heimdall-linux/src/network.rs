use std::time::Instant;

use heimdall_core::{MetricValue, Unit};
use sysinfo::Networks;

#[derive(Debug, Default)]
pub struct NetworkCounterState {
    last_rx: u64,
    last_tx: u64,
    last_at: Option<Instant>,
}

impl NetworkCounterState {
    pub fn sample(&mut self) -> heimdall_core::NetworkSnapshot {
        let networks = Networks::new_with_refreshed_list();
        let (rx, tx) = networks.iter().fold((0u64, 0u64), |acc, (_name, data)| {
            (
                acc.0 + data.received(),
                acc.1 + data.transmitted(),
            )
        });

        let now = Instant::now();
        let (rx_rate, tx_rate) = match self.last_at {
            Some(prev) => {
                let secs = now.duration_since(prev).as_secs_f64().max(0.001);
                (
                    (rx.saturating_sub(self.last_rx)) as f64 / secs,
                    (tx.saturating_sub(self.last_tx)) as f64 / secs,
                )
            }
            None => (0.0, 0.0),
        };

        self.last_rx = rx;
        self.last_tx = tx;
        self.last_at = Some(now);

        heimdall_core::NetworkSnapshot {
            rx_bytes_per_sec: if self.last_at.is_some() && rx_rate == 0.0 && tx_rate == 0.0 {
                MetricValue::supported(0.0, Unit::BytesPerSecond, "sysinfo:network")
            } else {
                MetricValue::supported(rx_rate, Unit::BytesPerSecond, "sysinfo:network")
            },
            tx_bytes_per_sec: MetricValue::supported(tx_rate, Unit::BytesPerSecond, "sysinfo:network"),
        }
    }
}