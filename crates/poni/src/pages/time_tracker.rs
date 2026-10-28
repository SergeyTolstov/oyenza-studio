//! Today's private foreground-app time.
use crate::commands::Command;
use crate::gfx::TextStyle;
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, wid_n, Ui, PillStyle, PILL_SMALL};
use poni_core::datetime::fmt_duration;
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 430.0;
const PLAN_TITLE_Y: f32 = 106.0;
const PLAN_TOP: f32 = 128.0;
const PLAN_H: f32 = 184.0;
const PLAN_ROW_H: f32 = 34.0;
const WHEEL_STEP: f32 = PLAN_ROW_H * 1.5;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

/// Running task timers repaint once a second only while their page is open.
pub fn next_wake(st: &AppState) -> Option<f64> {
    st.store.time_tracker.timers.iter().any(|t| t.running()).then_some(st.now + 1.0)
}

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let enabled = st.settings.time_tracker;
    let label = if enabled { "Tracking is on" } else { "Start tracking" };
    if ui.pill(wid("time.toggle"), Rect::new(area.x, area.y, 160.0, PILL_SMALL), label, Some(if enabled { Icon::Pause } else { Icon::Play }), if enabled { PillStyle::Primary(st.accent) } else { PillStyle::Neutral }).clicked {
        st.settings.time_tracker = !enabled;
        if !enabled { st.store.time_tracker.observe(st.local.day_key(), st.unix, None); }
        st.cmd(Command::SaveSettings);
        st.save();
    }
    let note = if enabled { "Only the foreground app name is kept on this PC." } else { "Off: Oyenza Studio does not record app time." };
    ui.c.text(note, Rect::new(area.x, area.y + 38.0, area.w, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));
    let input = Rect::new(area.x, area.y + 64.0, area.w, 34.0);
    let add = ui.text_input_row(wid("time.comment"), input, &mut st.ui.timer_comment, "Add to work plan — start it when you begin", 120, st.accent, Some(Icon::Plus));
    if add.submitted && !st.ui.timer_comment.trim().is_empty() {
        let comment = std::mem::take(&mut st.ui.timer_comment);
        st.store.time_tracker.plan_timer(comment);
        st.save();
    }
    let title = format!("Work plan ({})", st.store.time_tracker.timers.len());
    ui.c.text(&title, Rect::new(area.x, area.y + PLAN_TITLE_Y, area.w, 18.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    let view = Rect::new(area.x, area.y + PLAN_TOP, area.w, PLAN_H);
    let timers = st.store.time_tracker.timers.clone();
    let max_scroll = (timers.len() as f32 * PLAN_ROW_H - view.h).max(0.0);
    if max_scroll > 0.0 {
        st.ui.time_timer_scroll -= ui.wheel_in(view) * WHEEL_STEP;
    }
    st.ui.time_timer_scroll = st.ui.time_timer_scroll.clamp(0.0, max_scroll);
    if timers.is_empty() {
        super::empty_state(ui, view, Icon::Timer, "Your work plan is empty", "Add work above; start its timer when you begin");
    }
    ui.c.clip(view);
    for (i, timer) in timers.iter().rev().enumerate() {
        let row = Rect::new(area.x, view.y + i as f32 * PLAN_ROW_H - st.ui.time_timer_scroll, area.w, PLAN_ROW_H);
        if row.bottom() <= view.y || row.y >= view.bottom() { continue; }
        let running = timer.running();
        ui.c.icon(Icon::Timer, Rect::new(row.x + 4.0, row.y + 6.0, 18.0, 18.0), if running { st.accent } else { palette::TEXT_DIM });
        ui.c.text(&timer.comment, Rect::new(row.x + 29.0, row.y, row.w - 150.0, row.h), &TextStyle::body());
        ui.c.text(&fmt_duration(timer.seconds(st.unix) as f64), Rect::new(row.right() - 112.0, row.y, 62.0, row.h), &TextStyle::body().color(if running { st.accent } else { palette::TEXT_DIM }).right().tabular());
        if ui.icon_button(wid_n("time.play", timer.id), Rect::new(row.right() - 42.0, row.y + 3.0, 24.0, 24.0), if running { Icon::Pause } else { Icon::Play }, st.accent, 13.0).clicked {
            if let Some(t) = st.store.time_tracker.timers.iter_mut().find(|t| t.id == timer.id) { if running { t.pause(st.unix); } else { t.resume(st.unix); } }
            st.save();
        }
        if ui.icon_button(wid_n("time.delete", timer.id), Rect::new(row.right() - 18.0, row.y + 3.0, 18.0, 24.0), Icon::Trash, palette::TEXT_DIM, 12.0).clicked {
            st.store.time_tracker.timers.retain(|t| t.id != timer.id);
            st.save();
        }
    }
    ui.c.unclip();
    let y = area.y + PLAN_TOP + PLAN_H + 14.0;
    ui.c.text("Automatic app time", Rect::new(area.x, y, area.w, 18.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    let y = y + 24.0;
    let mut apps: Vec<_> = st.store.time_tracker.apps.iter().collect();
    apps.sort_by_key(|(_, s)| std::cmp::Reverse(**s));
    if apps.is_empty() {
        super::empty_state(ui, Rect::new(area.x, y, area.w, HEIGHT - (y - area.y)), Icon::App, "No automatic time yet", "Keep an app active for a minute");
        return;
    }
    for (i, (name, secs)) in apps.into_iter().take(3).enumerate() {
        let row_y = y + i as f32 * 28.0;
        ui.c.text(name, Rect::new(area.x + 8.0, row_y, area.w - 120.0, 24.0), &TextStyle::body());
        ui.c.text(&fmt_duration(*secs as f64), Rect::new(area.right() - 100.0, row_y, 100.0, 24.0), &TextStyle::body().color(st.accent).right().tabular());
    }
}
