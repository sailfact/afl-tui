use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs};

use crate::api::models::{
    CfsScore, PlayerEntry, ScoreLine, StatKey, status_is_concluded, status_is_live,
};
use crate::app::{App, StatsTab};
use crate::teams::{self, Rgb};
use crate::ui::LogoRenderer;

const PLAYER_COLUMNS: &[(&str, StatKey)] = &[
    ("G", StatKey::Goals),
    ("B", StatKey::Behinds),
    ("K", StatKey::Kicks),
    ("HB", StatKey::Handballs),
    ("D", StatKey::Disposals),
    ("M", StatKey::Marks),
    ("T", StatKey::Tackles),
    ("HO", StatKey::Hitouts),
    ("CLR", StatKey::Clearances),
    ("I50", StatKey::Inside50s),
    ("CP", StatKey::ContestedPossessions),
    ("DE%", StatKey::DisposalEfficiency),
    ("MG", StatKey::MetresGained),
    ("FP", StatKey::DreamTeamPoints),
];

const TEAM_COMPARISON: &[(&str, StatKey)] = &[
    ("Kicks", StatKey::Kicks),
    ("Handballs", StatKey::Handballs),
    ("Disposals", StatKey::Disposals),
    ("Marks", StatKey::Marks),
    ("Tackles", StatKey::Tackles),
    ("Hitouts", StatKey::Hitouts),
    ("Clearances", StatKey::Clearances),
    ("Inside 50s", StatKey::Inside50s),
    ("Rebound 50s", StatKey::Rebound50s),
    ("Contested Possessions", StatKey::ContestedPossessions),
    ("Uncontested Possessions", StatKey::UncontestedPossessions),
    ("Contested Marks", StatKey::ContestedMarks),
    ("Marks Inside 50", StatKey::MarksInside50),
    ("Intercepts", StatKey::Intercepts),
    ("Score Involvements", StatKey::ScoreInvolvements),
    ("Turnovers", StatKey::Turnovers),
    ("Clangers", StatKey::Clangers),
    ("Metres Gained", StatKey::MetresGained),
    ("Goal Assists", StatKey::GoalAssists),
    ("Frees For", StatKey::FreesFor),
    ("Frees Against", StatKey::FreesAgainst),
];

pub fn draw(frame: &mut Frame, app: &mut App, logos: &mut LogoRenderer) {
    let [scoreboard, tabs, body, footer] = Layout::vertical([
        Constraint::Length(9),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_scoreboard(frame, app, logos, scoreboard);
    draw_tabs(frame, app, tabs);
    match app.tab {
        StatsTab::TeamStats => draw_team_stats(frame, app, body),
        StatsTab::HomePlayers | StatsTab::AwayPlayers => draw_players(frame, app, body),
        StatsTab::Feed => super::feed::draw(frame, app, body),
    }

    let hints = match app.tab {
        StatsTab::TeamStats => " Tab switch view   r refresh   Esc back   q quit",
        StatsTab::Feed => " Tab switch view   ↑/↓ scroll   r refresh   Esc back   q quit",
        _ => {
            " Tab switch view   ↑/↓ scroll   sort: d g k h m t c f   r refresh   Esc back   q quit"
        }
    };
    frame.render_widget(
        Paragraph::new(hints).style(Style::new().fg(Color::DarkGray)),
        footer,
    );
}

fn draw_scoreboard(frame: &mut Frame, app: &App, logos: &mut LogoRenderer, area: Rect) {
    let Some(md) = &app.match_data else { return };
    let fixture = &md.fixture;
    let score = md.item.as_ref().and_then(|i| i.score.as_ref());

    let (home_line, away_line) = match score {
        Some(s) => (s.home_team_score.match_score, s.away_team_score.match_score),
        None => (
            simple_to_line(fixture.home.score),
            simple_to_line(fixture.away.score),
        ),
    };

    let status_line = status_text(app, score);
    let home_winning = home_line.total_score > away_line.total_score;
    let away_winning = away_line.total_score > home_line.total_score;

    let home_team = teams::lookup(&fixture.home.team.nickname, &fixture.home.team.name);
    let away_team = teams::lookup(&fixture.away.team.nickname, &fixture.away.team.name);

    let block = Block::bordered()
        .border_style(Style::new().fg(Color::DarkGray))
        .title(
            Line::from(vec![Span::raw(format!(
                " {} · {} ",
                fixture.round.name, fixture.venue.name
            ))])
            .style(Style::new().fg(Color::DarkGray)),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Flank the score block with each club's emblem when there's room, keeping
    // the logo / scores / logo trio centred. A square emblem needs ~2 columns
    // per row to look square (terminal cells are about twice as tall as wide).
    let logo_w = inner.height.saturating_mul(2);
    let score_w: u16 = 46;
    let center = if logo_w > 0 && inner.width >= logo_w * 2 + score_w {
        let [_, left, mid, right, _] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(logo_w),
            Constraint::Length(score_w),
            Constraint::Length(logo_w),
            Constraint::Min(0),
        ])
        .areas(inner);
        if let Some(t) = home_team {
            logos.draw(frame, t, square_in(left));
        }
        if let Some(t) = away_team {
            logos.draw(frame, t, square_in(right));
        }
        mid
    } else {
        inner
    };

    let lines = vec![
        team_score_line(
            &fixture.home.team.name,
            home_line,
            home_winning,
            home_team.map(|t| t.score),
        ),
        team_score_line(
            &fixture.away.team.name,
            away_line,
            away_winning,
            away_team.map(|t| t.score),
        ),
        Line::from(""),
        status_line,
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), center);
}

/// Largest aspect-correct (≈square) rect centred within `col`, assuming terminal
/// cells are about twice as tall as they are wide. Centring keeps the emblem
/// level with the score text instead of floating to the column's top-left.
fn square_in(col: Rect) -> Rect {
    let width = col.width.min(col.height.saturating_mul(2)).max(1);
    let height = (width / 2).min(col.height).max(1);
    Rect {
        x: col.x + (col.width - width) / 2,
        y: col.y + (col.height - height) / 2,
        width,
        height,
    }
}

fn simple_to_line(s: Option<crate::api::models::SimpleScore>) -> ScoreLine {
    match s {
        Some(s) => ScoreLine {
            goals: s.goals,
            behinds: s.behinds,
            total_score: s.total_score,
        },
        None => ScoreLine::default(),
    }
}

fn team_score_line(
    name: &str,
    score: ScoreLine,
    winning: bool,
    color: Option<Rgb>,
) -> Line<'static> {
    let team_color = color.map(|(r, g, b)| Color::Rgb(r, g, b));
    let name_color = team_color.unwrap_or(if winning { Color::White } else { Color::Gray });
    let total_color = team_color.unwrap_or(Color::White);

    let mut name_style = Style::new().fg(name_color);
    if winning {
        name_style = name_style.bold();
    }
    Line::from(vec![
        Span::styled(format!("{name:<20}"), name_style),
        Span::styled(
            format!(" {:>2}.{:<2} ", score.goals, score.behinds),
            Style::new().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("{:>4}", score.total_score),
            Style::new().fg(total_color).bold(),
        ),
    ])
}

fn status_text(app: &App, score: Option<&CfsScore>) -> Line<'static> {
    let Some(md) = &app.match_data else {
        return Line::from("");
    };
    let status = score
        .map(|s| s.status.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(&md.fixture.status);

    let mut spans: Vec<Span> = Vec::new();
    if status_is_live(status) {
        spans.push(Span::styled(
            "● LIVE ",
            Style::new().fg(Color::Green).bold(),
        ));
        if let Some(clock) = score.and_then(|s| s.match_clock.as_ref())
            && let Some(p) = clock.periods.iter().max_by_key(|p| p.period_number)
        {
            spans.push(Span::styled(
                format!(
                    "{} {}",
                    crate::ui::period_label(p.period_number),
                    crate::ui::format_clock(p.period_seconds)
                ),
                Style::new().fg(Color::Green),
            ));
        }
    } else if status_is_concluded(status) {
        spans.push(Span::styled(
            "Full Time",
            Style::new().fg(Color::Yellow).bold(),
        ));
    } else {
        spans.push(Span::styled(
            crate::ui::format_start_time(&md.fixture.utc_start_time),
            Style::new().fg(Color::Cyan),
        ));
    }

    if let Some(w) = score.and_then(|s| s.weather.as_ref())
        && let Some(t) = w.temp_in_celsius
    {
        spans.push(Span::styled(
            format!("   {} {:.0}°C", w.description, t),
            Style::new().fg(Color::DarkGray),
        ));
    }
    if let Some(at) = md.last_updated {
        spans.push(Span::styled(
            format!("   updated {}s ago", at.elapsed().as_secs()),
            Style::new().fg(Color::DarkGray),
        ));
    }
    Line::from(spans)
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let (home, away) = match &app.match_data {
        Some(md) => (
            md.fixture.home.team.nickname.clone(),
            md.fixture.away.team.nickname.clone(),
        ),
        None => ("Home".into(), "Away".into()),
    };
    let tabs = Tabs::new(vec![
        "Team Stats".to_string(),
        format!("{home} Players"),
        format!("{away} Players"),
        "Feed".to_string(),
    ])
    .select(app.tab.index())
    .highlight_style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, area);
}

fn draw_team_stats(frame: &mut Frame, app: &App, area: Rect) {
    let Some(md) = &app.match_data else { return };
    let Some(stats) = &md.stats else {
        draw_loading(frame, area);
        return;
    };

    let sum = |players: &[PlayerEntry], key: StatKey| -> i64 {
        players
            .iter()
            .map(|p| p.stats().get(key))
            .sum::<f64>()
            .round() as i64
    };

    let rows: Vec<Row> = TEAM_COMPARISON
        .iter()
        .map(|(label, key)| {
            let h = sum(&stats.home_team_player_stats, *key);
            let a = sum(&stats.away_team_player_stats, *key);
            // "Lower is better" stats shouldn't highlight the bigger number.
            let lower_better = matches!(
                key,
                StatKey::Clangers | StatKey::Turnovers | StatKey::FreesAgainst
            );
            let (h_wins, a_wins) = if lower_better {
                (h < a, a < h)
            } else {
                (h > a, a > h)
            };
            let hi = Style::new().bold().fg(Color::White);
            let lo = Style::new().fg(Color::Gray);
            Row::new(vec![
                Cell::from(ratatui::text::Text::from(h.to_string()).alignment(Alignment::Right))
                    .style(if h_wins { hi } else { lo }),
                Cell::from(ratatui::text::Text::from(*label).alignment(Alignment::Center))
                    .style(Style::new().fg(Color::DarkGray)),
                Cell::from(a.to_string()).style(if a_wins { hi } else { lo }),
            ])
        })
        .collect();

    let header = Row::new(vec![
        Cell::from(
            ratatui::text::Text::from(md.fixture.home.team.nickname.clone())
                .alignment(Alignment::Right),
        ),
        Cell::from(""),
        Cell::from(md.fixture.away.team.nickname.clone()),
    ])
    .style(Style::new().bold().fg(Color::Red));

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(25),
            Constraint::Fill(1),
        ],
    )
    .header(header)
    .block(
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(Color::DarkGray)),
    );
    frame.render_widget(table, area);
}

fn draw_players(frame: &mut Frame, app: &mut App, area: Rect) {
    let players = app.sorted_players();
    if players.is_empty() {
        draw_loading(frame, area);
        return;
    }

    let mut header_cells = vec![Cell::from("#"), Cell::from("PLAYER"), Cell::from("POS")];
    for (label, key) in PLAYER_COLUMNS {
        let style = if *key == app.sort {
            Style::new().fg(Color::Red).bold()
        } else {
            Style::new().fg(Color::DarkGray).bold()
        };
        header_cells.push(Cell::from(*label).style(style));
    }

    let rows: Vec<Row> = players
        .iter()
        .map(|p| {
            let s = p.stats();
            let mut cells = vec![
                Cell::from(p.jumper().to_string()).style(Style::new().fg(Color::DarkGray)),
                Cell::from(p.name()),
                Cell::from(p.position().to_string()).style(Style::new().fg(Color::DarkGray)),
            ];
            for (_, key) in PLAYER_COLUMNS {
                let v = s.get(*key);
                let text = if *key == StatKey::DisposalEfficiency {
                    format!("{v:.0}")
                } else {
                    format!("{}", v.round() as i64)
                };
                let style = if *key == app.sort {
                    Style::new().bold()
                } else {
                    Style::new()
                };
                cells.push(Cell::from(text).style(style));
            }
            Row::new(cells)
        })
        .collect();

    let mut widths = vec![
        Constraint::Length(3), // jumper
        Constraint::Min(16),   // name
        Constraint::Length(4), // position
    ];
    widths.extend(PLAYER_COLUMNS.iter().map(|_| Constraint::Length(4)));

    let row_count = players.len();
    app.player_row = app.player_row.min(row_count.saturating_sub(1));

    let table = Table::new(rows, widths)
        .header(Row::new(header_cells))
        .block(
            Block::new()
                .borders(Borders::TOP)
                .border_style(Style::new().fg(Color::DarkGray)),
        )
        .row_highlight_style(Style::new().bg(Color::Rgb(40, 40, 60)));

    let mut state = TableState::default().with_selected(Some(app.player_row));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_loading(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new("Loading stats…").style(Style::new().fg(Color::Yellow)),
        Rect {
            y: area.y + 1,
            height: 1,
            ..area
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_in_fills_an_aspect_correct_column() {
        // A 2:1 column (width == 2*height) should be filled exactly, no offset.
        let r = square_in(Rect::new(10, 3, 14, 7));
        assert_eq!(r, Rect::new(10, 3, 14, 7));
    }

    #[test]
    fn square_in_centres_within_an_oversized_column() {
        // A too-wide column keeps the ~square footprint and centres it.
        let r = square_in(Rect::new(0, 0, 30, 7));
        assert_eq!(r.width, 14); // height * 2
        assert_eq!(r.height, 7);
        assert_eq!(r.x, 8); // (30 - 14) / 2
        assert_eq!(r.y, 0);
    }

    #[test]
    fn square_in_is_height_limited_when_narrow() {
        // A narrow column shrinks height to keep the 2:1 ratio and centres it.
        let r = square_in(Rect::new(0, 0, 8, 7));
        assert_eq!(r.width, 8);
        assert_eq!(r.height, 4); // width / 2
        assert_eq!(r.y, 1); // (7 - 4) / 2
    }
}
