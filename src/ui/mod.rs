mod fixture;
mod match_view;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Fixture => fixture::draw(frame, app),
        Screen::Match => match_view::draw(frame, app),
    }
}

/// Parse an AFL API UTC timestamp ("2026-06-11T09:30:00.000+0000" or
/// "2026-06-11T09:30:00") into local time.
pub fn parse_utc(s: &str) -> Option<DateTime<Local>> {
    if let Ok(dt) = DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.3f%z") {
        return Some(dt.with_timezone(&Local));
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&naive).with_timezone(&Local));
    }
    None
}

pub fn format_start_time(utc: &str) -> String {
    parse_utc(utc)
        .map(|dt| dt.format("%a %d %b %l:%M%p").to_string())
        .unwrap_or_else(|| utc.to_string())
}

pub fn format_clock(seconds: i64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub fn period_label(period: u32) -> String {
    if period <= 4 {
        format!("Q{period}")
    } else {
        "ET".to_string()
    }
}
