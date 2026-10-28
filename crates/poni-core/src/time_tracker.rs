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
    /// Completed work for one local calendar day. The day key keeps the
    /// compact history honest when the app stays open across midnight.
    #[serde(default)]
    pub history_day: String,
    #[serde(default)]
    pub history: Vec<CompletedTimer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTimer {
    pub id: u64,
    pub comment: String,
    /// Seconds completed before the current run.
    pub elapsed: u64,
    /// Unix time when the current run began; `None` when paused.
    pub started_at: Option<i64>,
    /// Optional planned local date/time, kept separate from actual tracked time.
    #[serde(default)]
    pub planned_unix: Option<i64>,
    /// Bit set for each advance reminder already shown (60, 30, 5, 1 min).
    #[serde(default)]
    pub reminder_mask: u8,
}

/// A finished work timer retained for the current local day.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedTimer {
    pub id: u64,
    pub comment: String,
    pub elapsed: u64,
    pub finished_at: i64,
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

    /// Add a planned work item. It is deliberately paused: planning work and
    /// starting work are separate actions.
    pub fn plan_timer(&mut self, comment: String, planned_unix: Option<i64>) {
        let comment = comment.trim();
        if comment.is_empty() || self.timers.len() >= 24 { return; }
        let id = self.next_timer_id.max(1);
        self.next_timer_id = id.saturating_add(1);
        self.timers.push(TaskTimer { id, comment: comment.to_string(), elapsed: 0, started_at: None, planned_unix, reminder_mask: 0 });
    }

    /// Move a work item out of the plan and into today's completed history.
    /// A running timer is paused first so its final seconds are preserved.
    pub fn finish_timer(&mut self, id: u64, now: i64, day: String) -> bool {
        let Some(index) = self.timers.iter().position(|timer| timer.id == id) else { return false };
        let mut timer = self.timers.remove(index);
        timer.pause(now);
        if self.history_day != day {
            self.history_day = day;
            self.history.clear();
        }
        self.history.push(CompletedTimer {
            id: timer.id,
            comment: timer.comment,
            elapsed: timer.elapsed,
            finished_at: now,
        });
        // Keep the day view short and useful even after a busy day.
        if self.history.len() > 48 {
            self.history.drain(..self.history.len() - 48);
        }
        true
    }

    /// The closest future plan that is not currently running.
    pub fn next_planned(&self, now: i64) -> Option<&TaskTimer> {
        self.timers.iter()
            .filter(|timer| !timer.running())
            .filter_map(|timer| timer.planned_unix.filter(|at| *at > now).map(|at| (at, timer)))
            .min_by_key(|(at, _)| *at)
            .map(|(_, timer)| timer)
    }

    /// The next planned, not-yet-started task reminder due at `now`.
    /// The bitmask makes all four notices durable across restarts.
    pub fn due_reminder(&self, now: i64) -> Option<(u64, i64, String)> {
        self.timers.iter().filter(|timer| !timer.running()).flat_map(|timer| {
            timer.planned_unix.into_iter().flat_map(move |at| {
                REMINDER_LEADS.into_iter().enumerate().filter_map(move |(bit, lead)| {
                    (timer.reminder_mask & (1 << bit) == 0 && now >= at - lead).then(|| (timer.id, lead, timer.comment.clone()))
                })
            })
        }).min_by_key(|(_, lead, _)| *lead)
    }

    pub fn mark_reminder_shown(&mut self, id: u64, lead: i64) -> bool {
        if !REMINDER_LEADS.contains(&lead) { return false; }
        let Some(timer) = self.timers.iter_mut().find(|timer| timer.id == id) else { return false };
        // If the PC woke late, surface the most urgent passed milestone and
        // skip older ones rather than playing a stale burst of reminders.
        for (index, value) in REMINDER_LEADS.iter().enumerate() {
            if *value >= lead {
                timer.reminder_mask |= 1 << index;
            }
        }
        true
    }

    /// Unix time for the next advance reminder. A due one returns `now` so
    /// the caller wakes immediately and surfaces it.
    pub fn next_reminder_at(&self, now: i64) -> Option<i64> {
        self.timers.iter().filter(|timer| !timer.running()).flat_map(|timer| {
            timer.planned_unix.into_iter().flat_map(move |at| {
                REMINDER_LEADS.into_iter().enumerate().filter_map(move |(bit, lead)| {
                    (timer.reminder_mask & (1 << bit) == 0).then_some((at - lead).max(now))
                })
            })
        }).min()
    }
}

/// Seconds before a planned item: one hour, half an hour, five and one minute.
pub const REMINDER_LEADS: [i64; 4] = [3_600, 1_800, 300, 60];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planned_timer_waits_for_an_explicit_start() {
        let mut tracker = TimeTracker::default();
        tracker.plan_timer("  Prepare release notes  ".into(), Some(2_000));
        let timer = tracker.timers.first().unwrap();
        assert_eq!(timer.comment, "Prepare release notes");
        assert!(!timer.running());
        assert_eq!(timer.seconds(1_000), 0);
        assert_eq!(timer.planned_unix, Some(2_000));
        assert_eq!(tracker.next_reminder_at(0), Some(0));
    }

    #[test]
    fn planned_item_reminds_at_each_lead_once() {
        let mut tracker = TimeTracker::default();
        tracker.plan_timer("Call supplier".into(), Some(10_000));
        assert_eq!(tracker.due_reminder(6_399), None);
        let (id, lead, _) = tracker.due_reminder(6_400).unwrap();
        assert_eq!(lead, 3_600);
        assert!(tracker.mark_reminder_shown(id, lead));
        assert_eq!(tracker.due_reminder(8_200).map(|(_, lead, _)| lead), Some(1_800));
        let (id, lead, _) = tracker.due_reminder(9_700).unwrap();
        assert_eq!(lead, 300, "late wake chooses the current milestone");
        assert!(tracker.mark_reminder_shown(id, lead));
        assert_eq!(tracker.due_reminder(9_700).map(|(_, lead, _)| lead), None, "older notices are not replayed");
    }

    #[test]
    fn finished_timer_moves_to_todays_history() {
        let mut tracker = TimeTracker::default();
        tracker.plan_timer("Write brief".into(), None);
        tracker.timers[0].resume(1_000);
        assert!(tracker.finish_timer(1, 1_125, "2026-10-28".into()));
        assert!(tracker.timers.is_empty());
        assert_eq!(tracker.history_day, "2026-10-28");
        assert_eq!(tracker.history[0].comment, "Write brief");
        assert_eq!(tracker.history[0].elapsed, 125);
    }

    #[test]
    fn next_planned_ignores_past_and_running_items() {
        let mut tracker = TimeTracker::default();
        tracker.plan_timer("Past".into(), Some(90));
        tracker.plan_timer("Soon".into(), Some(150));
        tracker.plan_timer("Later".into(), Some(200));
        tracker.timers[1].resume(100);
        assert_eq!(tracker.next_planned(100).map(|timer| timer.comment.as_str()), Some("Later"));
    }
}

