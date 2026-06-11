use std::time::Instant;

use crate::api::models::{CompSeason, FixtureMatch, MatchItem, PlayerStatsResponse, StatKey};
use crate::poller::DataEvent;

pub const MIN_ROUND: u32 = 1;
pub const MAX_ROUND: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Fixture,
    Match,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsTab {
    TeamStats,
    HomePlayers,
    AwayPlayers,
}

impl StatsTab {
    pub fn next(self) -> Self {
        match self {
            Self::TeamStats => Self::HomePlayers,
            Self::HomePlayers => Self::AwayPlayers,
            Self::AwayPlayers => Self::TeamStats,
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::TeamStats => 0,
            Self::HomePlayers => 1,
            Self::AwayPlayers => 2,
        }
    }
}

pub struct MatchData {
    pub fixture: FixtureMatch,
    pub item: Option<MatchItem>,
    pub stats: Option<PlayerStatsResponse>,
    pub last_updated: Option<Instant>,
}

pub struct App {
    pub screen: Screen,
    pub season: Option<CompSeason>,
    pub round: u32,
    pub matches: Vec<FixtureMatch>,
    pub selected: usize,
    pub loading: bool,
    pub match_data: Option<MatchData>,
    pub tab: StatsTab,
    pub sort: StatKey,
    pub player_row: usize,
    pub error: Option<String>,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Fixture,
            season: None,
            round: 0,
            matches: Vec::new(),
            selected: 0,
            loading: true,
            match_data: None,
            tab: StatsTab::TeamStats,
            sort: StatKey::Disposals,
            player_row: 0,
            error: None,
            should_quit: false,
        }
    }

    pub fn apply(&mut self, event: DataEvent) {
        match event {
            DataEvent::Season(s) => {
                self.round = s.current_round_number.max(MIN_ROUND);
                self.season = Some(s);
            }
            DataEvent::Round { round, matches } => {
                // Ignore stale responses after the user moved to another round.
                if round == self.round {
                    self.loading = false;
                    self.error = None;
                    if self.selected >= matches.len() {
                        self.selected = matches.len().saturating_sub(1);
                    }
                    self.matches = matches;
                }
            }
            DataEvent::MatchUpdate {
                provider_id,
                item,
                stats,
            } => {
                if let Some(md) = self.match_data.as_mut()
                    && md.fixture.provider_id == provider_id
                {
                    md.item = Some(*item);
                    md.stats = Some(*stats);
                    md.last_updated = Some(Instant::now());
                    self.error = None;
                }
            }
            DataEvent::Error(e) => {
                self.loading = false;
                self.error = Some(e);
            }
        }
    }

    pub fn selected_match(&self) -> Option<&FixtureMatch> {
        self.matches.get(self.selected)
    }

    pub fn open_match(&mut self) -> Option<String> {
        let fixture = self.selected_match()?.clone();
        let id = fixture.provider_id.clone();
        self.match_data = Some(MatchData {
            fixture,
            item: None,
            stats: None,
            last_updated: None,
        });
        self.screen = Screen::Match;
        self.tab = StatsTab::TeamStats;
        self.player_row = 0;
        Some(id)
    }

    pub fn close_match(&mut self) {
        self.screen = Screen::Fixture;
        self.match_data = None;
        self.error = None;
    }

    pub fn change_round(&mut self, delta: i64) -> Option<u32> {
        let new = (self.round as i64 + delta).clamp(MIN_ROUND as i64, MAX_ROUND as i64) as u32;
        if new == self.round {
            return None;
        }
        self.round = new;
        self.matches.clear();
        self.selected = 0;
        self.loading = true;
        self.error = None;
        Some(new)
    }

    /// Player entries for the active players tab, sorted by the current sort key.
    pub fn sorted_players(&self) -> Vec<&crate::api::models::PlayerEntry> {
        let Some(md) = &self.match_data else {
            return Vec::new();
        };
        let Some(stats) = &md.stats else {
            return Vec::new();
        };
        let list = match self.tab {
            StatsTab::HomePlayers => &stats.home_team_player_stats,
            StatsTab::AwayPlayers => &stats.away_team_player_stats,
            StatsTab::TeamStats => return Vec::new(),
        };
        let mut players: Vec<_> = list.iter().collect();
        players.sort_by(|a, b| {
            b.stats()
                .get(self.sort)
                .partial_cmp(&a.stats().get(self.sort))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        players
    }
}
