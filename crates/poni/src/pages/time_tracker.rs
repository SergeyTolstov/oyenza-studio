//! Today's private foreground-app time.
use crate::commands::Command;
use crate::gfx::TextStyle;
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, Ui, PillStyle, PILL_SMALL};
use poni_core::datetime::fmt_duration;
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 220.0;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

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
    let mut apps: Vec<_> = st.store.time_tracker.apps.iter().collect();
    apps.sort_by_key(|(_, s)| std::cmp::Reverse(**s));
    if apps.is_empty() {
        super::empty_state(ui, Rect::new(area.x, area.y + 68.0, area.w, HEIGHT - 68.0), Icon::Timer, "No time yet", "Keep an app active for a minute");
        return;
    }
    for (i, (name, secs)) in apps.into_iter().take(5).enumerate() {
        let y = area.y + 66.0 + i as f32 * 28.0;
        ui.c.text(name, Rect::new(area.x + 8.0, y, area.w - 120.0, 24.0), &TextStyle::body());
        ui.c.text(&fmt_duration(*secs as f64), Rect::new(area.right() - 100.0, y, 100.0, 24.0), &TextStyle::body().color(st.accent).right().tabular());
    }
}
