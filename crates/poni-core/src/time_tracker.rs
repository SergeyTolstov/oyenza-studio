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
    /// Manually started work timers, saved with their purpose.
    pub timers: Vec<TaskTimer>,
    pub next_timer_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTimer {
    pub id: u64,
    pub comment: String,
    /// Seconds completed before the current run.
    pub elapsed: u64,
    /// Unix time when the current run began; `None` when paused.
    pub started_at: Option<i64>,
}

impl TaskTimer {
    pub fn seconds(&self, now: i64) -> u64 {
        self.elapsed.saturating_add(self.started_at.map(|at| now.saturating_sub(at) as u64).unwrap_or(0))
    }
    pub fn running(&self) -> bool { self.started_at.is_some() }
    pub fn pause(&mut self, now: i64) {
        if let Some(at) = self.started_at.take() { self.elapsed = self.elapsed.saturating_add(now.saturating_sub(at) as u64); }
    }
    pub fn resume(&mut self, now: i64) { if self.started_at.is_none() { self.started_at = Some(now); } }
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

    pub fn start_timer(&mut self, comment: String, now: i64) {
        let comment = comment.trim();
        if comment.is_empty() || self.timers.len() >= 24 { return; }
        let id = self.next_timer_id.max(1);
        self.next_timer_id = id.saturating_add(1);
        self.timers.push(TaskTimer { id, comment: comment.to_string(), elapsed: 0, started_at: Some(now) });
    }
}

