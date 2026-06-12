use afl_tui::{api, app, poller, ui};

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc;

use api::models::StatKey;
use app::{App, Screen, StatsTab};
use poller::{Cmd, DataEvent};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Arc::new(api::AflClient::new()?);
    let (cmd_tx, event_rx) = poller::spawn(client);
    cmd_tx.send(Cmd::Init)?;

    let terminal = ratatui::init();
    let result = run(terminal, cmd_tx, event_rx).await;
    ratatui::restore();
    result
}

async fn run(
    mut terminal: ratatui::DefaultTerminal,
    cmd_tx: mpsc::UnboundedSender<Cmd>,
    mut event_rx: mpsc::UnboundedReceiver<DataEvent>,
) -> Result<()> {
    let mut app = App::new();
    let mut input = EventStream::new();
    // Redraw tick keeps the live clock / "updated Xs ago" fresh.
    let mut redraw = tokio::time::interval(Duration::from_millis(500));

    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        tokio::select! {
            maybe_event = input.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event
                    && key.kind == KeyEventKind::Press {
                        handle_key(&mut app, &cmd_tx, key.code, key.modifiers);
                    }
            }
            Some(data) = event_rx.recv() => {
                app.apply(data);
                // Drain any queued updates before redrawing.
                while let Ok(data) = event_rx.try_recv() {
                    app.apply(data);
                }
            }
            _ = redraw.tick() => {}
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(
    app: &mut App,
    cmd_tx: &mpsc::UnboundedSender<Cmd>,
    code: KeyCode,
    modifiers: KeyModifiers,
) {
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }
    match app.screen {
        Screen::Fixture => match code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                app.selected = app.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if app.selected + 1 < app.matches.len() => {
                app.selected += 1;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(round) = app.change_round(-1) {
                    let _ = cmd_tx.send(Cmd::LoadRound(round));
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(round) = app.change_round(1) {
                    let _ = cmd_tx.send(Cmd::LoadRound(round));
                }
            }
            KeyCode::Char('r') => {
                app.loading = true;
                let _ = cmd_tx.send(Cmd::LoadRound(app.round));
            }
            KeyCode::Enter => {
                if let Some(id) = app.open_match() {
                    let _ = cmd_tx.send(Cmd::Watch(id));
                }
            }
            _ => {}
        },
        Screen::Match => match code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Esc | KeyCode::Backspace => {
                app.close_match();
                let _ = cmd_tx.send(Cmd::Unwatch);
            }
            KeyCode::Tab => {
                app.tab = app.tab.next();
                app.player_row = 0;
                app.feed_row = 0;
            }
            KeyCode::Up if app.tab == StatsTab::Feed => {
                app.feed_row = app.feed_row.saturating_sub(1)
            }
            KeyCode::Down if app.tab == StatsTab::Feed => app.feed_row += 1, // clamped at render
            KeyCode::Up => app.player_row = app.player_row.saturating_sub(1),
            KeyCode::Down => app.player_row += 1, // clamped against roster size at render
            KeyCode::Char('r') => {
                if let Some(md) = &app.match_data {
                    let _ = cmd_tx.send(Cmd::Watch(md.fixture.provider_id.clone()));
                }
            }
            KeyCode::Char(c) => {
                if matches!(app.tab, StatsTab::HomePlayers | StatsTab::AwayPlayers)
                    && let Some(key) = sort_key(c)
                {
                    app.sort = key;
                }
            }
            _ => {}
        },
    }
}

fn sort_key(c: char) -> Option<StatKey> {
    Some(match c {
        'd' => StatKey::Disposals,
        'g' => StatKey::Goals,
        'k' => StatKey::Kicks,
        'h' => StatKey::Handballs,
        'm' => StatKey::Marks,
        't' => StatKey::Tackles,
        'c' => StatKey::Clearances,
        'f' => StatKey::DreamTeamPoints,
        _ => return None,
    })
}
