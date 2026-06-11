//! Deserialization tests against captured API responses in tests/fixtures/.
//! These are real payloads from the unofficial AFL APIs — if the shape drifts,
//! re-capture them (see CLAUDE.md) and adjust the models.

use afl_tui::api::models::*;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

#[test]
fn parses_comp_seasons() {
    let res: CompSeasonsResponse = serde_json::from_str(&fixture("compseasons.json")).unwrap();
    assert!(!res.comp_seasons.is_empty());
    let latest = &res.comp_seasons[0];
    assert!(latest.name.contains("Premiership"));
    assert!(latest.current_round_number >= 1);
}

#[test]
fn parses_round_matches() {
    let res: MatchesResponse = serde_json::from_str(&fixture("matches_round.json")).unwrap();
    assert_eq!(res.matches.len(), 7);
    let m = &res.matches[0];
    assert_eq!(m.provider_id, "CD_M20260141401");
    assert!(m.is_concluded());
    assert_eq!(m.home.team.abbreviation, "WB");
    assert_eq!(m.home.score.unwrap().total_score, 64);
    assert_eq!(m.venue.name, "Marvel Stadium");
    // Upcoming matches have no score.
    assert!(res.matches[1].home.score.is_none());
}

#[test]
fn parses_match_item() {
    let item: MatchItem = serde_json::from_str(&fixture("match_item.json")).unwrap();
    assert_eq!(item.match_info.name, "Western Bulldogs Vs Adelaide Crows");
    let score = item.score.unwrap();
    assert!(status_is_concluded(&score.status));
    assert_eq!(score.home_team_score.match_score.total_score, 64);
    assert_eq!(score.away_team_score.match_score.total_score, 121);
    assert_eq!(score.home_team_score.period_score.len(), 4);
    let clock = score.match_clock.unwrap();
    assert_eq!(clock.periods.len(), 4);
    assert!(clock.periods[0].period_completed);
}

#[test]
fn parses_player_stats() {
    let res: PlayerStatsResponse = serde_json::from_str(&fixture("player_stats.json")).unwrap();
    assert!(res.home_team_player_stats.len() >= 22);
    assert!(res.away_team_player_stats.len() >= 22);
    let bont = &res.home_team_player_stats[0];
    assert_eq!(bont.name(), "M. Bontempelli");
    assert_eq!(bont.jumper(), 4);
    assert_eq!(bont.stats().get(StatKey::Disposals), 29.0);
    assert_eq!(bont.stats().get(StatKey::Kicks), 16.0);
}
