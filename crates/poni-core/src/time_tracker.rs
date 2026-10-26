//! Private, daily foreground-app accounting. The Windows layer supplies an
//! app name only while the person opted in; no window titles or URLs enter it.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeTracker {
    pub day: String,
    pub apps: BTreeMap<String, u64>,
    pub active: Option<String>,
    pub sampled_at: i64,
}

impl TimeTracker {
    pub fn observe(&mut self, day: String, now: i64, active: Option<String>) {
        if self.day != day {
            self.day = day;
            self.apps.clear();
            self.active = None;
            self.sampled_at = now;
        }
        if let Some(name) = self.active.as_ref() {
            let elapsed = now.saturating_sub(self.sampled_at).min(90) as u64;
            *self.apps.entry(name.clone()).or_default() += elapsed;
        }
        self.active = active;
        self.sampled_at = now;
    }

    pub fn total(&self) -> u64 { self.apps.values().sum() }
}

