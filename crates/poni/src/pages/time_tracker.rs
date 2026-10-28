//! Today's private foreground-app time.
use crate::commands::Command;
use crate::gfx::TextStyle;
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, wid_n, Ui, PillStyle, PILL_SMALL};
use poni_core::datetime::{civil_from_days, days_from_civil, days_in_month, fmt_duration};
use poni_core::geom::{palette, Rect};

const HEIGHT: f32 = 448.0;
const DATE_Y: f32 = 104.0;
const PLAN_TITLE_Y: f32 = 142.0;
const PLAN_TOP: f32 = 164.0;
const PLAN_H: f32 = 166.0;
const PLAN_ROW_H: f32 = 34.0;
const WHEEL_STEP: f32 = PLAN_ROW_H * 1.5;

pub fn height(_st: &AppState) -> f32 { HEIGHT }

fn clear_editor(st: &mut AppState) {
    st.ui.time_editing = None;
    st.ui.timer_comment.clear();
    st.ui.time_plan_date.clear();
}

fn save_plan(st: &mut AppState) {
    let planned = match parse_planned(&st.ui.time_plan_date, st) {
        Ok(value) => value,
        Err(message) => {
            st.notify(Icon::Calendar, palette::ORANGE, "Planned time wasn't saved", message, 6.0, None);
            return;
        }
    };
    let comment = st.ui.timer_comment.trim().to_string();
    if let Some(id) = st.ui.time_editing {
        if let Some(timer) = st.store.time_tracker.timers.iter_mut().find(|timer| timer.id == id) {
            timer.comment = comment;
            timer.planned_unix = planned;
            timer.reminder_mask = 0;
            st.save();
        }
    } else {
        st.store.time_tracker.plan_timer(comment, planned);
        st.save();
    }
    clear_editor(st);
}

/// `28.10 14:30`, `2026-10-28 14:30`, `14:30` (today), or `in 30 min`.
fn parse_planned(input: &str, st: &AppState) -> Result<Option<i64>, &'static str> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(None);
    }
    if input.eq_ignore_ascii_case("in 30 min") {
        return Ok(Some(st.unix + 30 * 60));
    }
    let mut parts = input.split_whitespace();
    let first = parts.next().ok_or("Use 28.10 14:30")?;
    let second = parts.next();
    if parts.next().is_some() { return Err("Use 28.10 14:30"); }
    let (date, clock) = match second {
        Some(clock) => (first, clock),
        None if first.contains(':') => ("", first),
        None => return Err("Add a time, for example 14:30"),
    };
    let (hour, minute) = parse_clock(clock).ok_or("Time must look like 14:30")?;
    let (year, month, day) = if date.is_empty() {
        (st.local.year, st.local.month, st.local.day)
    } else {
        parse_date(date, st.local.year).ok_or("Date must look like 28.10 or 2026-10-28")?
    };
    if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
        return Err("That date doesn't exist");
    }
    let current_day = days_from_civil(st.local.year, st.local.month, st.local.day);
    let target_day = days_from_civil(year, month, day);
    let at = st.day_start_unix() + (target_day - current_day) * 86_400 + i64::from(hour) * 3_600 + i64::from(minute) * 60;
    if at <= st.unix { return Err("Choose a future date and time"); }
    Ok(Some(at))
}

fn parse_clock(s: &str) -> Option<(u8, u8)> {
    let (hour, minute) = s.split_once(':')?;
    let hour = hour.parse::<u8>().ok()?;
    let minute = minute.parse::<u8>().ok()?;
    (hour < 24 && minute < 60).then_some((hour, minute))
}

fn parse_date(s: &str, default_year: i32) -> Option<(i32, u8, u8)> {
    let parts: Vec<_> = s.split(['.', '-']).collect();
    match parts.as_slice() {
        [day, month] => Some((default_year, month.parse().ok()?, day.parse().ok()?)),
        [year, month, day] => Some((year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)),
        _ => None,
    }
}

fn local_parts(st: &AppState, at: i64) -> (i32, u8, u8, i64) {
    let seconds = at - st.day_start_unix();
    let shift = seconds.div_euclid(86_400);
    let seconds = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days_from_civil(st.local.year, st.local.month, st.local.day) + shift);
    (year, month, day, seconds)
}

fn planned_label(st: &AppState, at: i64) -> String {
    let (year, month, day, seconds) = local_parts(st, at);
    let time = format!("{:02}:{:02}", seconds / 3_600, (seconds / 60) % 60);
    if year == st.local.year && month == st.local.month && day == st.local.day {
        format!("Today {time}")
    } else {
        format!("{day:02}.{month:02} {time}")
    }
}

fn planned_input(st: &AppState, at: i64) -> String {
    let (_year, month, day, seconds) = local_parts(st, at);
    format!("{day:02}.{month:02} {:02}:{:02}", seconds / 3_600, (seconds / 60) % 60)
}

fn fit_text(ui: &mut Ui, text: &str, style: &TextStyle, max_width: f32) -> String {
    if ui.c.measure(text, style, max_width).0 <= max_width {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        out.push(ch);
        if ui.c.measure(&(out.clone() + "…"), style, max_width).0 > max_width {
            out.pop();
            break;
        }
    }
    format!("{out}…")
}

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
    let editing = st.ui.time_editing.is_some();
    let input = Rect::new(area.x, area.y + 64.0, area.w, 34.0);
    let add = ui.text_input_row(wid("time.comment"), input, &mut st.ui.timer_comment, if editing { "Edit work item" } else { "Add to work plan — start it when you begin" }, 120, st.accent, Some(if editing { Icon::Sliders } else { Icon::Plus }));
    let date_r = Rect::new(area.x, area.y + DATE_Y, area.w - 112.0, PILL_SMALL);
    let date = ui.text_input_row(wid("time.date"), date_r, &mut st.ui.time_plan_date, "Planned: 28.10 14:30", 20, st.accent, Some(Icon::Calendar));
    let save_r = Rect::new(date_r.right() + 8.0, date_r.y, 76.0, PILL_SMALL);
    let save = ui.pill(wid("time.plan.save"), save_r, if editing { "Save" } else { "Add" }, Some(Icon::Check), PillStyle::Primary(st.accent)).clicked;
    let cancel_r = Rect::new(save_r.right() + 4.0, date_r.y, 24.0, PILL_SMALL);
    let cancel = editing && ui.icon_button(wid("time.plan.cancel"), cancel_r, Icon::Close, palette::TEXT_DIM, 13.0).clicked;
    if cancel {
        clear_editor(st);
    } else if (add.submitted || date.submitted || save) && !st.ui.timer_comment.trim().is_empty() {
        save_plan(st);
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
        let text_style = TextStyle::body();
        let text_r = Rect::new(row.x + 29.0, row.y, (row.w - 340.0).max(40.0), row.h);
        let shown = fit_text(ui, &timer.comment, &text_style, text_r.w);
        ui.c.text(&shown, text_r, &text_style);
        if let Some(at) = timer.planned_unix {
            ui.c.text(&planned_label(st, at), Rect::new(row.right() - 276.0, row.y, 98.0, row.h), &TextStyle::footnote().color(palette::TEXT_FAINT).right());
        }
        ui.c.text(&fmt_duration(timer.seconds(st.unix) as f64), Rect::new(row.right() - 170.0, row.y, 62.0, row.h), &TextStyle::body().color(if running { st.accent } else { palette::TEXT_DIM }).right().tabular());
        if ui.icon_button(wid_n("time.play", timer.id), Rect::new(row.right() - 102.0, row.y + 3.0, 24.0, 24.0), if running { Icon::Pause } else { Icon::Play }, st.accent, 13.0).clicked {
            if let Some(t) = st.store.time_tracker.timers.iter_mut().find(|t| t.id == timer.id) { if running { t.pause(st.unix); } else { t.resume(st.unix); } }
            st.save();
        }
        if ui.pill(wid_n("time.edit", timer.id), Rect::new(row.right() - 74.0, row.y + 3.0, 48.0, PILL_SMALL), "Edit", None, PillStyle::Utility).clicked {
            st.ui.time_editing = Some(timer.id);
            st.ui.timer_comment = timer.comment.clone();
            st.ui.time_plan_date = timer.planned_unix.map(|at| planned_input(st, at)).unwrap_or_default();
        }
        if ui.icon_button(wid_n("time.delete", timer.id), Rect::new(row.right() - 22.0, row.y + 3.0, 20.0, 24.0), Icon::Trash, palette::TEXT_DIM, 12.0).clicked {
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
