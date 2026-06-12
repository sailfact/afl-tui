//! The feed must reflect each MatchUpdate the poller sends — this is the same
//! path the live 20s refresh takes, so it proves the feed auto-updates.

use afl_tui::api::models::*;
use afl_tui::app::App;
use afl_tui::poller::DataEvent;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}

fn open_fixture_match(app: &mut App) -> String {
    let res: MatchesResponse = serde_json::from_str(&fixture("matches_round.json")).unwrap();
    app.apply(DataEvent::Round {
        round: 0,
        matches: res.matches,
    });
    app.open_match().unwrap()
}

fn update(provider_id: &str, item_json: &str) -> DataEvent {
    DataEvent::MatchUpdate {
        provider_id: provider_id.to_string(),
        item: Box::new(serde_json::from_str(item_json).unwrap()),
        stats: Box::new(serde_json::from_str("{}").unwrap()),
    }
}

#[test]
fn feed_updates_when_new_scoring_events_arrive() {
    let mut app = App::new();
    let id = open_fixture_match(&mut app);
    assert!(app.scoring_events().is_empty());

    let item_json = fixture("match_item.json");
    app.apply(update(&id, &item_json));
    let before = app.scoring_events().len();
    assert!(before > 0);

    // Simulate the next live poll: same match, one more goal on the worm.
    let mut v: serde_json::Value = serde_json::from_str(&item_json).unwrap();
    let events = v["score"]["scoreWorm"]["scoringEvents"]
        .as_array_mut()
        .unwrap();
    let mut goal = events.last().unwrap().clone();
    goal["scoreType"] = "GOAL".into();
    goal["periodSeconds"] = 1900.into();
    events.push(goal);
    app.apply(update(&id, &v.to_string()));

    assert_eq!(app.scoring_events().len(), before + 1);
    assert_eq!(app.scoring_events().last().unwrap().score_type, "GOAL");
}

#[test]
fn updates_for_other_matches_are_ignored() {
    let mut app = App::new();
    let _id = open_fixture_match(&mut app);
    app.apply(update("CD_MSOMEOTHERGAME", &fixture("match_item.json")));
    assert!(app.scoring_events().is_empty());
}
