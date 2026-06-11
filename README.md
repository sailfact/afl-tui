# afl-tui

Live AFL scores and stats in your terminal — fixture browser, live scoreboard, team-stats comparison, and sortable player stats, similar to the AFL app. Built with [ratatui](https://ratatui.rs).

Data comes from the AFL's own public web APIs (the same ones that power afl.com.au). No API key needed.

## Run

```sh
cargo run --release
```

The app opens on the current round of the current season. Games that are in progress show a live quarter clock and auto-refresh every 20 seconds.

## Keys

### Fixture screen

| Key | Action |
| --- | --- |
| `←` / `→` (or `h` / `l`) | Previous / next round |
| `↑` / `↓` (or `k` / `j`) | Select match |
| `Enter` | Open match |
| `r` | Refresh round |
| `q` / `Esc` | Quit |

### Match screen

| Key | Action |
| --- | --- |
| `Tab` | Cycle Team Stats → Home Players → Away Players |
| `↑` / `↓` | Scroll players |
| `d` `g` `k` `h` `m` `t` `c` `f` | Sort by disposals, goals, kicks, handballs, marks, tackles, clearances, fantasy points |
| `r` | Refresh now |
| `Esc` | Back to fixture |
| `q` | Quit |

Player columns: G goals, B behinds, K kicks, HB handballs, D disposals, M marks, T tackles, HO hitouts, CLR clearances, I50 inside 50s, CP contested possessions, DE% disposal efficiency, MG metres gained, FP AFL Fantasy (dream team) points.

## Development

```sh
cargo test                    # deserialization tests against captured API payloads
cargo clippy --all-targets    # keep warning-free
```

This uses unofficial AFL endpoints; if they change shape, see CLAUDE.md for how to re-capture test fixtures.
