# Repository Guidelines

## Project

`afl-tui` is a Rust terminal app (ratatui) that shows the AFL fixture and live match stats, similar to the AFL mobile app. It pulls data from the AFL's own (unofficial, unauthenticated-signup) backends.

## Project Structure & Module Organization

Single binary plus a lib target (`src/lib.rs`) so integration tests can use the models.

- `src/main.rs` — terminal setup, key handling, and the main `tokio::select!` event loop over (a) crossterm `EventStream`, (b) data events from the poller, (c) a 500ms redraw tick. The UI task owns all `App` state; the poller never touches it.
- `src/app.rs` — `App` state machine: `Screen::{Fixture, Match}`, round/match selection, `StatsTab`, sort key, and `apply(DataEvent)` which folds poller results into state (ignoring stale round responses).
- `src/poller.rs` — background tokio task. Receives `Cmd::{Init, LoadRound, Watch, Unwatch}` over an unbounded channel, emits `DataEvent`s back. While a watched match is live it refetches matchItem + playerStats every 20s; the fixture refreshes every 60s only when the round has a live game.
- `src/api/client.rs` — reqwest client. Caches the cfs token in a `Mutex<Option<String>>`; on 401/403 it refreshes the token and retries once.
- `src/api/models.rs` — serde models. Only fields the UI uses are modeled; everything is `#[serde(default)]`/`Option` because the APIs are unofficial and shapes drift. `StatKey` + `Stats::get()` is the single mapping used by both sorting and rendering.
- `src/teams.rs` — per-club identity (pure data, no ratatui/image deps): a readable score-highlight colour and a declarative block-art `Emblem` (guernsey stripes/bands, sash, Swans' V, sun, anchor, or monogram letter) for each of the 18 clubs, keyed for lookup by API `nickname` (with `name` fallback). Dark club colours are nudged brighter to stay legible on a dark terminal.
- `src/ui/` — `fixture.rs` (round list); `match_view.rs` (scoreboard with club emblems flanking team-coloured scores, Team Stats comparison, sortable player tables); `logo.rs` (rasterises `Emblem`s to in-memory `image::RgbaImage`s and renders them via `ratatui-image`, which auto-selects a terminal graphics protocol — Kitty in Ghostty, etc. — and falls back to Unicode half-blocks in terminals like Alacritty; encoded protocols are cached per club + cell-area).

Integration tests are in `tests/`, with captured API payloads under `tests/fixtures/`. The `examples/dump_logos.rs` example renders all 18 emblems to a `/tmp/afl-logos.png` contact sheet for eyeballing logo changes.

## Build, Test, and Development Commands

- `cargo run` starts the interactive TUI. It needs network access to AFL endpoints.
- `cargo build` compiles a debug build.
- `cargo build --release` builds an optimized executable.
- `cargo test` runs integration tests (deserialization) against the captured payloads in `tests/fixtures/`.
- `cargo test parses_player_stats` runs a single targeted test.
- `cargo run --example dump_logos` writes a contact sheet of all club emblems to `/tmp/afl-logos.png`.
- `cargo clippy --all-targets` checks for lints and should stay warning-free.
- `cargo fmt` formats Rust code using rustfmt.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt`; keep imports tidy and avoid broad refactors unrelated to the change. Prefer explicit, domain-oriented names such as `DataEvent`, `StatsTab`, and `status_is_live`. API models in `src/api/models.rs` should stay defensive: use `#[serde(default)]` and `Option` for unofficial fields that may drift.

## Testing Guidelines

Tests use Rust integration tests in `tests/*.rs` and fixture JSON in `tests/fixtures/`. Add or update fixtures when API response shapes change, and keep tests focused on deserialization, state updates, or rendering-sensitive logic. Run `cargo test` before submitting changes; run `cargo clippy --all-targets` when touching shared logic.

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

## Commit & Pull Request Guidelines

Existing commits use concise imperative messages, for example `Add Feed tab with score worm and scoring timeline` and `Fix worm x-axis bound when the last period runs long`. Follow that style: describe the user-visible or behavioral change first. Pull requests should include a short summary, verification commands run, and notes for any API fixture updates or UI behavior changes. Include screenshots or terminal captures when changing the ratatui layout.

## Security & Configuration Tips

The app uses unofficial AFL APIs and caches a temporary CFS token in memory only. Do not commit live tokens, captured personal data, or generated artifacts from `target/`.
