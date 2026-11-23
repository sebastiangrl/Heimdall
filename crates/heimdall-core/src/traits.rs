use std::time::Duration;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(String),
    #[error("unavailable: {0}")]
    Unavailable(String),
}

pub type CollectorResult<T> = Result<T, CollectorError>;

pub trait MetricCollector: Send + Sync {
    fn id(&self) -> &'static str;
    fn sample_interval(&self) -> Duration;
}