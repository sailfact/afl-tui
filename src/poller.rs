use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::api::AflClient;
use crate::api::models::{CompSeason, FixtureMatch, LadderEntry, MatchItem, PlayerStatsResponse};

/// How often a watched live match is refreshed.
const MATCH_REFRESH: Duration = Duration::from_secs(20);
/// Fixture refresh runs every Nth match tick (20s * 3 = 60s).
const FIXTURE_REFRESH_TICKS: u32 = 3;

#[derive(Debug)]
pub enum Cmd {
    /// Fetch seasons, then the current round of the latest season.
    Init,
    LoadRound(u32),
    LoadLadder,
    /// Start watching a match (providerId); refreshes while it is live.
    Watch(String),
    Unwatch,
}

#[derive(Debug)]
pub enum DataEvent {
    Season(CompSeason),
    Round {
        round: u32,
        matches: Vec<FixtureMatch>,
    },
    Ladder(Vec<LadderEntry>),
    MatchUpdate {
        provider_id: String,
        item: Box<MatchItem>,
        stats: Box<PlayerStatsResponse>,
    },
    Error(String),
}

pub fn spawn(
    client: Arc<AflClient>,
) -> (
    mpsc::UnboundedSender<Cmd>,
    mpsc::UnboundedReceiver<DataEvent>,
) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    tokio::spawn(run(client, cmd_rx, event_tx));
    (cmd_tx, event_rx)
}

async fn run(
    client: Arc<AflClient>,
    mut cmd_rx: mpsc::UnboundedReceiver<Cmd>,
    tx: mpsc::UnboundedSender<DataEvent>,
) {
    let mut season: Option<CompSeason> = None;
    let mut round: Option<u32> = None;
    let mut round_has_live = false;
    let mut watching: Option<String> = None;
    let mut watching_live = false;
    let mut tick_count: u32 = 0;

    let mut ticker = tokio::time::interval(MATCH_REFRESH);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { return };
                match cmd {
                    Cmd::Init => {
                        match client.comp_seasons().await {
                            Ok(seasons) => {
                                // Seasons come newest-first; take the latest premiership season.
                                if let Some(s) = seasons.into_iter().next() {
                                    let current = s.current_round_number.max(1);
                                    season = Some(s.clone());
                                    let _ = tx.send(DataEvent::Season(s));
                                    round = Some(current);
                                    round_has_live = load_round(&client, &tx, season.as_ref(), current).await;
                                    load_ladder(&client, &tx, season.as_ref()).await;
                                } else {
                                    let _ = tx.send(DataEvent::Error("no seasons returned by AFL API".into()));
                                }
                            }
                            Err(e) => { let _ = tx.send(DataEvent::Error(format!("loading seasons: {e:#}"))); }
                        }
                    }
                    Cmd::LoadRound(r) => {
                        round = Some(r);
                        round_has_live = load_round(&client, &tx, season.as_ref(), r).await;
                    }
                    Cmd::LoadLadder => load_ladder(&client, &tx, season.as_ref()).await,
                    Cmd::Watch(id) => {
                        watching_live = fetch_match(&client, &tx, &id).await;
                        watching = Some(id);
                    }
                    Cmd::Unwatch => { watching = None; }
                }
            }
            _ = ticker.tick() => {
                tick_count = tick_count.wrapping_add(1);
                if let Some(id) = watching.clone() {
                    if watching_live {
                        watching_live = fetch_match(&client, &tx, &id).await;
                    }
                } else if round_has_live && tick_count.is_multiple_of(FIXTURE_REFRESH_TICKS)
                    && let Some(r) = round {
                        round_has_live = load_round(&client, &tx, season.as_ref(), r).await;
                    }
            }
        }
    }
}

async fn load_ladder(
    client: &AflClient,
    tx: &mpsc::UnboundedSender<DataEvent>,
    season: Option<&CompSeason>,
) {
    let Some(season) = season else { return };
    match client.ladder(season.id).await {
        Ok(entries) => {
            let _ = tx.send(DataEvent::Ladder(entries));
        }
        Err(e) => {
            let _ = tx.send(DataEvent::Error(format!("loading ladder: {e:#}")));
        }
    }
}

/// Returns whether any match in the round is live.
async fn load_round(
    client: &AflClient,
    tx: &mpsc::UnboundedSender<DataEvent>,
    season: Option<&CompSeason>,
    round: u32,
) -> bool {
    let Some(season) = season else { return false };
    match client.round_matches(season.id, round).await {
        Ok(matches) => {
            let has_live = matches.iter().any(|m| m.is_live());
            let _ = tx.send(DataEvent::Round { round, matches });
            has_live
        }
        Err(e) => {
            let _ = tx.send(DataEvent::Error(format!("loading round {round}: {e:#}")));
            false
        }
    }
}

/// Returns whether the match is still live (keep refreshing).
async fn fetch_match(client: &AflClient, tx: &mpsc::UnboundedSender<DataEvent>, id: &str) -> bool {
    let (item, stats) = tokio::join!(client.match_item(id), client.player_stats(id));
    match (item, stats) {
        (Ok(item), Ok(stats)) => {
            let live = item
                .score
                .as_ref()
                .map(|s| crate::api::models::status_is_live(&s.status))
                .unwrap_or(false)
                || crate::api::models::status_is_live(&item.match_info.status);
            let _ = tx.send(DataEvent::MatchUpdate {
                provider_id: id.to_string(),
                item: Box::new(item),
                stats: Box::new(stats),
            });
            live
        }
        (Err(e), _) | (_, Err(e)) => {
            let _ = tx.send(DataEvent::Error(format!("loading match: {e:#}")));
            false
        }
    }
}
