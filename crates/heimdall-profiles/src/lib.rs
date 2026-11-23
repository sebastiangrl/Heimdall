mod baseline;
mod baseline_store;
mod classifier;
mod engine;
mod store;

#[cfg(test)]
mod tests;

pub use baseline::{BaselineTracker, ProfileBaselineSummary};
pub use classifier::{ActivityAssessment, ActivityProfile, ProfileSignal};
pub use engine::{ProfileEngine, ProfileState};
pub use store::{ProcessProfileOverride, ProfileStore};

pub fn profile_label(profile: ActivityProfile) -> &'static str {
    match profile {
        ActivityProfile::Gaming => "gaming",
        ActivityProfile::ContentCreation => "content_creation",
        ActivityProfile::Development => "development",
        ActivityProfile::General => "general",
        ActivityProfile::Idle => "idle",
    }
}