use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table, TableState,
};

use crate::api::models::{MatchClock, ScoringEvent, status_is_live};
use crate::app::App;

/// Fallback quarter length (seconds) before the clock tells us the real one.
/// 20 minutes plus typical time-on.
const NOMINAL_PERIOD_SECS: i64 = 2000;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let events = app.scoring_events();
    if events.is_empty() {
        let msg = if app.match_data.as_ref().is_some_and(|md| md.item.is_some()) {
            "No scores yet — the worm starts wriggling at the first goal or behind."
        } else {
            "Loading match feed…"
        };
        frame.render_widget(
            Paragraph::new(msg).style(Style::new().fg(Color::Yellow)),
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
        return;
    }

    let [worm_area, list_area] =
        Layout::vertical([Constraint::Percentage(45), Constraint::Min(4)]).areas(area);
    draw_worm(frame, app, worm_area);
    draw_timeline(frame, app, list_area);
}

/// (match seconds, home margin) points for the worm, anchored at (0, 0).
/// X positions place each event at its period's offset plus seconds into the
/// period; period lengths come from the match clock when available.
pub fn worm_points(events: &[ScoringEvent], clock: Option<&MatchClock>) -> Vec<(f64, f64)> {
    let bounds = period_bounds(events, clock);
    let mut pts = vec![(0.0, 0.0)];
    for e in events {
        let p = e.period_number.max(1) as usize;
        let x = bounds.get(p - 1).copied().unwrap_or(0) + e.period_seconds;
        pts.push((x as f64, e.margin() as f64));
    }
    pts
}

/// Cumulative period boundaries (seconds): element p-1 is period p's start
/// and the final element is the end of the last period, so every worm point
/// lies within the boundaries. Each period's length is the longest of: the
/// match clock's elapsed seconds, the latest scoring event seen in it, or
/// the nominal quarter length.
pub fn period_bounds(events: &[ScoringEvent], clock: Option<&MatchClock>) -> Vec<i64> {
    let max_period = events
        .iter()
        .map(|e| e.period_number)
        .chain(
            clock
                .iter()
                .flat_map(|c| c.periods.iter().map(|p| p.period_number)),
        )
        .max()
        .unwrap_or(1)
        .max(1);

    let mut bounds = Vec::with_capacity(max_period as usize + 1);
    let mut acc = 0i64;
    for p in 1..=max_period {
        bounds.push(acc);
        let from_clock = clock
            .and_then(|c| c.periods.iter().find(|x| x.period_number == p))
            .map(|x| x.period_seconds)
            .unwrap_or(0);
        let from_events = events
            .iter()
            .filter(|e| e.period_number == p)
            .map(|e| e.period_seconds)
            .max()
            .unwrap_or(0);
        acc += from_clock.max(from_events).max(NOMINAL_PERIOD_SECS);
    }
    bounds.push(acc);
    bounds
}

fn draw_worm(frame: &mut Frame, app: &App, area: Rect) {
    let Some(md) = &app.match_data else { return };
    let score = md.item.as_ref().and_then(|i| i.score.as_ref());
    let events = app.scoring_events();
    let clock = score.and_then(|s| s.match_clock.as_ref());

    let pts = worm_points(events, clock);
    let bounds = period_bounds(events, clock);
    let x_max = bounds.last().copied().unwrap_or(NOMINAL_PERIOD_SECS) as f64;
    let y_max = pts
        .iter()
        .map(|(_, y)| y.abs())
        .fold(6.0f64, f64::max)
        .ceil()
        + 2.0;

    let margin = events.last().map(|e| e.margin()).unwrap_or(0);
    let home = md.fixture.home.team.abbreviation.clone();
    let away = md.fixture.away.team.abbreviation.clone();
    let worm_color = match margin.signum() {
        1 => Color::Cyan,
        -1 => Color::Magenta,
        _ => Color::Gray,
    };

    let zero = [(0.0, 0.0), (x_max, 0.0)];
    // Vertical quarter-break markers (interior boundaries only).
    let breaks: Vec<[(f64, f64); 2]> = bounds[1..bounds.len() - 1]
        .iter()
        .map(|&x| [(x as f64, -y_max), (x as f64, y_max)])
        .collect();

    let mut datasets = vec![
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(Color::DarkGray))
            .data(&zero),
    ];
    for b in &breaks {
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::new().fg(Color::DarkGray))
                .data(b),
        );
    }
    datasets.push(
        Dataset::default()
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(worm_color).bold())
            .data(&pts),
    );

    let leader = match margin.signum() {
        1 => format!("{home} by {margin}"),
        -1 => format!("{away} by {}", -margin),
        _ => "scores level".to_string(),
    };
    let mut title = vec![Span::styled(
        format!(" Worm · ▲ {home}  ▼ {away} · {leader} "),
        Style::new().fg(Color::Gray),
    )];
    let status = score
        .map(|s| s.status.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&md.fixture.status);
    if status_is_live(status) {
        title.push(live_indicator(md.last_updated));
    }

    let x_labels: Vec<Span> = (1..bounds.len())
        .map(|p| {
            Span::styled(
                crate::ui::period_label(p as u32),
                Style::new().fg(Color::DarkGray),
            )
        })
        .collect();
    let y_labels = vec![
        Span::styled(
            format!("{away} +{:.0}", y_max),
            Style::new().fg(Color::Magenta),
        ),
        Span::raw("0"),
        Span::styled(
            format!("{home} +{:.0}", y_max),
            Style::new().fg(Color::Cyan),
        ),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(Color::DarkGray))
                .title(Line::from(title)),
        )
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(x_labels)
                .style(Style::new().fg(Color::DarkGray)),
        )
        .y_axis(
            Axis::default()
                .bounds([-y_max, y_max])
                .labels(y_labels)
                .style(Style::new().fg(Color::DarkGray)),
        );
    frame.render_widget(chart, area);
}

/// Pulsing "● LIVE" — blinks on the app's 500ms redraw tick.
fn live_indicator(last_updated: Option<std::time::Instant>) -> Span<'static> {
    let on = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() % 1000 < 600)
        .unwrap_or(true);
    let dot = if on { "● " } else { "  " };
    let updated = last_updated
        .map(|at| format!(" · updated {}s ago", at.elapsed().as_secs()))
        .unwrap_or_default();
    Span::styled(
        format!(" {dot}LIVE{updated} "),
        Style::new().fg(Color::Green).bold(),
    )
}

fn draw_timeline(frame: &mut Frame, app: &mut App, area: Rect) {
    let count = app.scoring_events().len();
    app.feed_row = app.feed_row.min(count.saturating_sub(1));
    let events = app.scoring_events();

    // Newest first so the latest score is always at the top of the feed.
    let rows: Vec<Row> = events
        .iter()
        .rev()
        .enumerate()
        .map(|(i, e)| {
            let goal = e.is_goal();
            let type_style = if goal {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new().fg(Color::DarkGray)
            };
            let latest = i == 0;
            let marker = if latest { "▸" } else { " " };
            Row::new(vec![
                Cell::from(marker).style(Style::new().fg(Color::Green)),
                Cell::from(format!(
                    "{} {}",
                    crate::ui::period_label(e.period_number),
                    crate::ui::format_clock(e.period_seconds)
                ))
                .style(Style::new().fg(Color::DarkGray)),
                Cell::from(e.score_type.clone()).style(type_style),
                Cell::from(e.player_name()),
                Cell::from(e.team_abbr().to_string()).style(Style::new().fg(Color::DarkGray)),
                Cell::from(format!(
                    "{}–{}",
                    e.aggregate_home_score, e.aggregate_away_score
                ))
                .style(if latest {
                    Style::new().bold().fg(Color::White)
                } else {
                    Style::new().fg(Color::Gray)
                }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(1),
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Min(16),
            Constraint::Length(5),
            Constraint::Length(9),
        ],
    )
    .header(
        Row::new(vec!["", "TIME", "SCORE", "PLAYER", "TEAM", "TOTAL"])
            .style(Style::new().bold().fg(Color::Red)),
    )
    .block(
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(Color::DarkGray))
            .title(
                Line::from(Span::styled(
                    " Scoring timeline ",
                    Style::new().fg(Color::Gray),
                ))
                .alignment(Alignment::Left),
            ),
    )
    .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)));

    let mut state = TableState::default().with_selected(Some(app.feed_row));
    frame.render_stateful_widget(table, area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{MatchClock, Period, ScoringEvent};

    fn ev(period: u32, secs: i64, home: i64, away: i64) -> ScoringEvent {
        ScoringEvent {
            period_number: period,
            period_seconds: secs,
            score_type: "GOAL".into(),
            score_value: 6,
            aggregate_home_score: home,
            aggregate_away_score: away,
            team_name: None,
            player_score: None,
        }
    }

    #[test]
    fn worm_starts_at_origin_and_tracks_margin() {
        let events = vec![ev(1, 100, 6, 0), ev(1, 400, 6, 6), ev(2, 50, 6, 13)];
        let pts = worm_points(&events, None);
        assert_eq!(pts[0], (0.0, 0.0));
        assert_eq!(pts[1], (100.0, 6.0));
        assert_eq!(pts[2], (400.0, 0.0));
        // Q2 event lands after the nominal Q1 length.
        assert_eq!(pts[3], ((NOMINAL_PERIOD_SECS + 50) as f64, -7.0));
    }

    #[test]
    fn period_bounds_use_clock_lengths() {
        let clock = MatchClock {
            periods: vec![
                Period {
                    period_number: 1,
                    period_seconds: 2105,
                    period_completed: true,
                },
                Period {
                    period_number: 2,
                    period_seconds: 30,
                    period_completed: false,
                },
            ],
        };
        let events = vec![ev(1, 2100, 6, 0), ev(2, 20, 12, 0)];
        let bounds = period_bounds(&events, Some(&clock));
        assert_eq!(bounds, vec![0, 2105, 2105 + NOMINAL_PERIOD_SECS]);
        let pts = worm_points(&events, Some(&clock));
        assert_eq!(pts[2], (2125.0, 12.0));
    }

    #[test]
    fn last_period_longer_than_previous_stays_in_bounds() {
        // Final quarter runs longer than the earlier ones; its events must
        // still fall within the last period boundary used for the x-axis.
        let clock = MatchClock {
            periods: vec![
                Period {
                    period_number: 1,
                    period_seconds: 2000,
                    period_completed: true,
                },
                Period {
                    period_number: 2,
                    period_seconds: 2400,
                    period_completed: false,
                },
            ],
        };
        let events = vec![ev(1, 1990, 6, 0), ev(2, 2350, 12, 0)];
        let bounds = period_bounds(&events, Some(&clock));
        assert_eq!(bounds, vec![0, 2000, 4400]);
        let pts = worm_points(&events, Some(&clock));
        let end = *bounds.last().unwrap() as f64;
        assert!(pts.iter().all(|&(x, _)| x <= end));
        assert_eq!(pts[2], (4350.0, 12.0));
    }
}
