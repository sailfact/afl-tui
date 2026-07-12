use serde::Deserialize;

// ---------- aflapi.afl.com.au (v2, no auth) ----------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompSeasonsResponse {
    #[serde(default)]
    pub comp_seasons: Vec<CompSeason>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompSeason {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub current_round_number: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchesResponse {
    #[serde(default)]
    pub matches: Vec<FixtureMatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LadderResponse {
    #[serde(default)]
    pub ladders: Vec<Ladder>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ladder {
    #[serde(default)]
    pub entries: Vec<LadderEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LadderEntry {
    #[serde(default, alias = "position")]
    pub rank: u32,
    pub team: Team,
    #[serde(default, alias = "points")]
    pub premiership_points: f64,
    #[serde(default)]
    pub percentage: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureMatch {
    pub provider_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub utc_start_time: String,
    pub round: Round,
    pub home: MatchSide,
    pub away: MatchSide,
    pub venue: Venue,
}

impl FixtureMatch {
    pub fn is_live(&self) -> bool {
        status_is_live(&self.status)
    }
    pub fn is_concluded(&self) -> bool {
        status_is_concluded(&self.status)
    }
}

pub fn status_is_live(status: &str) -> bool {
    matches!(status, "LIVE" | "IN_PROGRESS" | "PERIOD_BREAK")
}

pub fn status_is_concluded(status: &str) -> bool {
    matches!(status, "CONCLUDED" | "FULL_TIME" | "POST_MATCH")
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    #[serde(default)]
    pub round_number: u32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchSide {
    pub team: Team,
    #[serde(default)]
    pub score: Option<SimpleScore>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub name: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub nickname: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleScore {
    #[serde(default)]
    pub goals: i64,
    #[serde(default)]
    pub behinds: i64,
    #[serde(default)]
    pub total_score: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Venue {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub abbreviation: String,
    #[serde(default)]
    pub timezone: String,
}

// ---------- api.afl.com.au/cfs (X-media-mis-token) ----------

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchItem {
    #[serde(rename = "match")]
    pub match_info: CfsMatch,
    #[serde(default)]
    pub score: Option<CfsScore>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfsMatch {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub venue_local_start_time: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfsScore {
    #[serde(default)]
    pub status: String,
    pub home_team_score: CfsTeamScore,
    pub away_team_score: CfsTeamScore,
    #[serde(default)]
    pub match_clock: Option<MatchClock>,
    #[serde(default)]
    pub weather: Option<Weather>,
    #[serde(default)]
    pub score_worm: Option<ScoreWorm>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreWorm {
    #[serde(default)]
    pub scoring_events: Vec<ScoringEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoringEvent {
    #[serde(default)]
    pub period_number: u32,
    #[serde(default)]
    pub period_seconds: i64,
    #[serde(default)]
    pub score_type: String,
    #[serde(default)]
    pub score_value: i64,
    #[serde(default)]
    pub aggregate_home_score: i64,
    #[serde(default)]
    pub aggregate_away_score: i64,
    #[serde(default)]
    pub team_name: Option<WormTeam>,
    #[serde(default)]
    pub player_score: Option<WormPlayerScore>,
}

impl ScoringEvent {
    /// Home lead (negative when away is in front).
    pub fn margin(&self) -> i64 {
        self.aggregate_home_score - self.aggregate_away_score
    }
    pub fn is_goal(&self) -> bool {
        self.score_type.eq_ignore_ascii_case("GOAL")
    }
    pub fn team_abbr(&self) -> &str {
        self.team_name
            .as_ref()
            .map(|t| t.team_abbr.as_str())
            .unwrap_or("")
    }
    pub fn player_name(&self) -> String {
        self.player_score
            .as_ref()
            .map(|p| {
                let n = &p.player.player_name;
                format!("{}. {}", initial(&n.given_name), n.surname)
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WormTeam {
    #[serde(default)]
    pub team_abbr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WormPlayerScore {
    pub player: WormPlayer,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WormPlayer {
    pub player_name: PlayerName,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CfsTeamScore {
    pub match_score: ScoreLine,
    #[serde(default)]
    pub period_score: Vec<PeriodScore>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreLine {
    #[serde(default)]
    pub goals: i64,
    #[serde(default)]
    pub behinds: i64,
    #[serde(default)]
    pub total_score: i64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodScore {
    pub period_number: u32,
    pub score: ScoreLine,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchClock {
    #[serde(default)]
    pub periods: Vec<Period>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub period_number: u32,
    #[serde(default)]
    pub period_seconds: i64,
    #[serde(default)]
    pub period_completed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Weather {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub temp_in_celsius: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsResponse {
    #[serde(default)]
    pub home_team_player_stats: Vec<PlayerEntry>,
    #[serde(default)]
    pub away_team_player_stats: Vec<PlayerEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerEntry {
    pub player: PlayerOuter,
    pub player_stats: PlayerStatsWrap,
}

impl PlayerEntry {
    pub fn name(&self) -> String {
        let p = &self.player.player.player;
        format!(
            "{}. {}",
            initial(&p.player_name.given_name),
            p.player_name.surname
        )
    }
    pub fn jumper(&self) -> i64 {
        self.player.jumper_number
    }
    pub fn position(&self) -> &str {
        &self.player.player.position
    }
    pub fn stats(&self) -> &Stats {
        &self.player_stats.stats
    }
}

fn initial(given: &str) -> String {
    given
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerOuter {
    pub player: PlayerInner,
    #[serde(default)]
    pub jumper_number: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInner {
    #[serde(default)]
    pub position: String,
    pub player: PlayerCore,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerCore {
    pub player_name: PlayerName,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerName {
    #[serde(default)]
    pub given_name: String,
    #[serde(default)]
    pub surname: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsWrap {
    pub stats: Stats,
}

/// All values arrive as JSON numbers-or-null; unofficial API, so default everything.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    #[serde(default)]
    pub goals: Option<f64>,
    #[serde(default)]
    pub behinds: Option<f64>,
    #[serde(default)]
    pub kicks: Option<f64>,
    #[serde(default)]
    pub handballs: Option<f64>,
    #[serde(default)]
    pub disposals: Option<f64>,
    #[serde(default)]
    pub marks: Option<f64>,
    #[serde(default)]
    pub tackles: Option<f64>,
    #[serde(default)]
    pub hitouts: Option<f64>,
    #[serde(default)]
    pub clearances: Option<Clearances>,
    #[serde(default)]
    pub inside50s: Option<f64>,
    #[serde(default)]
    pub rebound50s: Option<f64>,
    #[serde(default)]
    pub contested_possessions: Option<f64>,
    #[serde(default)]
    pub uncontested_possessions: Option<f64>,
    #[serde(default)]
    pub clangers: Option<f64>,
    #[serde(default)]
    pub frees_for: Option<f64>,
    #[serde(default)]
    pub frees_against: Option<f64>,
    #[serde(default)]
    pub disposal_efficiency: Option<f64>,
    #[serde(default)]
    pub dream_team_points: Option<f64>,
    #[serde(default)]
    pub metres_gained: Option<f64>,
    #[serde(default)]
    pub goal_assists: Option<f64>,
    #[serde(default)]
    pub marks_inside50: Option<f64>,
    #[serde(default)]
    pub contested_marks: Option<f64>,
    #[serde(default)]
    pub intercepts: Option<f64>,
    #[serde(default)]
    pub score_involvements: Option<f64>,
    #[serde(default)]
    pub turnovers: Option<f64>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clearances {
    #[serde(default)]
    pub centre_clearances: Option<f64>,
    #[serde(default)]
    pub stoppage_clearances: Option<f64>,
    #[serde(default)]
    pub total_clearances: Option<f64>,
}

impl Stats {
    pub fn get(&self, key: StatKey) -> f64 {
        use StatKey::*;
        let v = match key {
            Goals => self.goals,
            Behinds => self.behinds,
            Kicks => self.kicks,
            Handballs => self.handballs,
            Disposals => self.disposals,
            Marks => self.marks,
            Tackles => self.tackles,
            Hitouts => self.hitouts,
            Clearances => self.clearances.and_then(|c| c.total_clearances),
            Inside50s => self.inside50s,
            Rebound50s => self.rebound50s,
            ContestedPossessions => self.contested_possessions,
            UncontestedPossessions => self.uncontested_possessions,
            Clangers => self.clangers,
            FreesFor => self.frees_for,
            FreesAgainst => self.frees_against,
            DisposalEfficiency => self.disposal_efficiency,
            DreamTeamPoints => self.dream_team_points,
            MetresGained => self.metres_gained,
            GoalAssists => self.goal_assists,
            MarksInside50 => self.marks_inside50,
            ContestedMarks => self.contested_marks,
            Intercepts => self.intercepts,
            ScoreInvolvements => self.score_involvements,
            Turnovers => self.turnovers,
        };
        v.unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatKey {
    Goals,
    Behinds,
    Kicks,
    Handballs,
    Disposals,
    Marks,
    Tackles,
    Hitouts,
    Clearances,
    Inside50s,
    Rebound50s,
    ContestedPossessions,
    UncontestedPossessions,
    Clangers,
    FreesFor,
    FreesAgainst,
    DisposalEfficiency,
    DreamTeamPoints,
    MetresGained,
    GoalAssists,
    MarksInside50,
    ContestedMarks,
    Intercepts,
    ScoreInvolvements,
    Turnovers,
}
