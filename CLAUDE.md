# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`afl-tui` is a Rust terminal app (ratatui) that shows the AFL fixture and live match stats, similar to the AFL mobile app. It pulls data from the AFL's own (unofficial, unauthenticated-signup) backends.

## Commands

- `cargo run` — launch the TUI (needs network access to the AFL APIs)
- `cargo build` / `cargo build --release`
- `cargo test` — deserialization tests against captured payloads in `tests/fixtures/`
- `cargo test parses_player_stats` — run a single test
- `cargo clippy --all-targets` — must be warning-free
- `cargo fmt`

## Architecture

Single binary plus a lib target (`src/lib.rs`) so integration tests can use the models.

- `src/main.rs` — terminal setup, key handling, and the main `tokio::select!` event loop over (a) crossterm `EventStream`, (b) data events from the poller, (c) a 500ms redraw tick. The UI task owns all `App` state; the poller never touches it.
- `src/app.rs` — `App` state machine: `Screen::{Fixture, Match}`, round/match selection, `StatsTab`, sort key, and `apply(DataEvent)` which folds poller results into state (ignoring stale round responses).
- `src/poller.rs` — background tokio task. Receives `Cmd::{Init, LoadRound, Watch, Unwatch}` over an unbounded channel, emits `DataEvent`s back. While a watched match is live it refetches matchItem + playerStats every 20s; the fixture refreshes every 60s only when the round has a live game.
- `src/api/client.rs` — reqwest client. Caches the cfs token in a `Mutex<Option<String>>`; on 401/403 it refreshes the token and retries once.
- `src/api/models.rs` — serde models. Only fields the UI uses are modeled; everything is `#[serde(default)]`/`Option` because the APIs are unofficial and shapes drift. `StatKey` + `Stats::get()` is the single mapping used by both sorting and rendering.
- `src/ui/` — `fixture.rs` (round list) and `match_view.rs` (scoreboard, Team Stats comparison aggregated from player stats, sortable player tables).

## AFL API notes

Two backends (both power afl.com.au; no API key signup exists — be a polite client):

1. `https://aflapi.afl.com.au` — no auth.
   - `GET /afl/v2/competitions/1/compseasons` → seasons newest-first, each with `currentRoundNumber`
   - `GET /afl/v2/matches?competitionId=1&compSeasonId={id}&roundNumber={n}` → fixture incl. scores for finished/live games
2. `https://api.afl.com.au/cfs` — requires `X-media-mis-token` header.
   - Token: `POST /cfs/afl/WMCTok` with empty body → `{"token": ...}`. Tokens expire; refresh on 401/403.
   - `GET /cfs/afl/matchItem/{providerId}` → score, status, quarter clock (`matchClock.periods`), weather
   - `GET /cfs/afl/playerStats/match/{providerId}` → Champion Data per-player stats (note: `clearances` is a nested object, stats values are `f64`-or-null)

Match `providerId`s look like `CD_M20260141401`. Statuses observed: `UNCONFIRMED_TEAMS`, `CONFIRMED_TEAMS`, `LIVE`, `CONCLUDED` (see `status_is_live`/`status_is_concluded` in models.rs).

If a deserialization test fails after an API shape change, re-capture fixtures (replace the providerId with any recent match):

```sh
TOK=$(curl -s -X POST https://api.afl.com.au/cfs/afl/WMCTok -H "User-Agent: Mozilla/5.0" -d '' | python3 -c "import sys,json;print(json.load(sys.stdin)['token'])")
curl -s "https://api.afl.com.au/cfs/afl/playerStats/match/CD_M20260141401" -H "X-media-mis-token: $TOK" -H "User-Agent: Mozilla/5.0" -o tests/fixtures/player_stats.json
```

## Verifying UI changes

The TUI can be driven headlessly through a PTY (Python `pty.fork`, send `\r`/`\t`/`q`, strip ANSI, assert on content) — useful since `cargo run` needs an interactive terminal. Opening a CONCLUDED match exercises the same render path as a live one except the auto-refresh loop.
