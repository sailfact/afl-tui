use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Tabs};

use crate::api::models::{
    CfsScore, PlayerEntry, ScoreLine, StatKey, status_is_concluded, status_is_live,
};
use crate::app::{App, StatsTab};

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

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [scoreboard, tabs, body, footer] = Layout::vertical([
        Constraint::Length(6),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_scoreboard(frame, app, scoreboard);
    draw_tabs(frame, app, tabs);
    match app.tab {
        StatsTab::TeamStats => draw_team_stats(frame, app, body),
        StatsTab::HomePlayers | StatsTab::AwayPlayers => draw_players(frame, app, body),
    }

    let hints = match app.tab {
        StatsTab::TeamStats => " Tab switch view   r refresh   Esc back   q quit",
        _ => {
            " Tab switch view   ↑/↓ scroll   sort: d g k h m t c f   r refresh   Esc back   q quit"
        }
    };
    frame.render_widget(
        Paragraph::new(hints).style(Style::new().fg(Color::DarkGray)),
        footer,
    );
}

fn draw_scoreboard(frame: &mut Frame, app: &App, area: Rect) {
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

    let lines = vec![
        team_score_line(&fixture.home.team.name, home_line, home_winning),
        team_score_line(&fixture.away.team.name, away_line, away_winning),
        Line::from(""),
        status_line,
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
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

fn team_score_line(name: &str, score: ScoreLine, winning: bool) -> Line<'static> {
    let style = if winning {
        Style::new().bold().fg(Color::White)
    } else {
        Style::new().fg(Color::Gray)
    };
    Line::from(Span::styled(
        format!(
            "{name:<22} {:>2}.{:<2} {:>4}",
            score.goals, score.behinds, score.total_score
        ),
        style,
    ))
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
