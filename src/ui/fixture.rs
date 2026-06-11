use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use crate::api::models::{FixtureMatch, SimpleScore};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_header(frame, app, header);
    draw_matches(frame, app, body);

    let hints = " ←/→ round   ↑/↓ select   Enter open   r refresh   q quit";
    frame.render_widget(
        Paragraph::new(hints).style(Style::new().fg(Color::DarkGray)),
        footer,
    );
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let season = app
        .season
        .as_ref()
        .map(|s| s.name.as_str())
        .unwrap_or("AFL");
    let mut spans = vec![
        Span::styled(
            " AFL Fixture ",
            Style::new().fg(Color::Black).bg(Color::Red).bold(),
        ),
        Span::raw(format!("  {season}  ")),
        Span::styled(format!("Round {}", app.round), Style::new().bold()),
    ];
    if app.loading {
        spans.push(Span::styled("  loading…", Style::new().fg(Color::Yellow)));
    }
    if let Some(err) = &app.error {
        spans.push(Span::styled(
            format!("  {err}"),
            Style::new().fg(Color::Red),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_matches(frame: &mut Frame, app: &mut App, area: Rect) {
    let rows: Vec<Row> = app.matches.iter().map(match_row).collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(18), // home
            Constraint::Length(11), // home score
            Constraint::Length(1),  // v
            Constraint::Length(18), // away
            Constraint::Length(11), // away score
            Constraint::Length(16), // status / clock
            Constraint::Min(12),    // venue + time
        ],
    )
    .header(
        Row::new(["HOME", "", "", "AWAY", "", "STATUS", "VENUE"]).style(
            Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(Color::DarkGray)),
    )
    .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)).bold())
    .highlight_symbol("▶ ");

    let mut state = TableState::default().with_selected(if app.matches.is_empty() {
        None
    } else {
        Some(app.selected)
    });
    frame.render_stateful_widget(table, area, &mut state);

    if app.matches.is_empty() && !app.loading {
        let msg =
            Paragraph::new("No matches in this round.").style(Style::new().fg(Color::DarkGray));
        let inner = Rect {
            y: area.y + 2,
            height: 1,
            ..area
        };
        frame.render_widget(msg, inner);
    }
}

fn match_row(m: &FixtureMatch) -> Row<'_> {
    let (status, status_style) = status_cell(m);
    let when = crate::ui::format_start_time(&m.utc_start_time);
    Row::new(vec![
        Cell::from(m.home.team.name.clone()),
        Cell::from(score_text(m.home.score)).style(Style::new().bold()),
        Cell::from("v").style(Style::new().fg(Color::DarkGray)),
        Cell::from(m.away.team.name.clone()),
        Cell::from(score_text(m.away.score)).style(Style::new().bold()),
        Cell::from(status).style(status_style),
        Cell::from(format!("{} · {}", m.venue.name, when)),
    ])
}

fn score_text(score: Option<SimpleScore>) -> String {
    match score {
        Some(s) => format!("{}.{} ({})", s.goals, s.behinds, s.total_score),
        None => String::new(),
    }
}

fn status_cell(m: &FixtureMatch) -> (String, Style) {
    if m.is_live() {
        ("● LIVE".to_string(), Style::new().fg(Color::Green).bold())
    } else if m.is_concluded() {
        ("Full Time".to_string(), Style::new().fg(Color::DarkGray))
    } else {
        ("Upcoming".to_string(), Style::new().fg(Color::Cyan))
    }
}
