# Paseo 0.13 compatibility — Context

**Feature slug:** paseo-013-compat
**Date:** 2026-10-27
**Shaping session:** complete (gate bypass full; user picked option B after the Paseo changelog review)
**Scope:** Standard
**Domain types:** CALL | RUN

## Feature Boundary

Paseo 0.12.0 deprecated `paseo run --background` (`-d`) in favor of
`--no-wait`, and `paseo send` gained `--behavior steer` for every provider.
bee must start workers with the supported flag, require a Paseo CLI that
has both, and steer running workers through the CLI instead of a Node
helper that loads the npm package by hand.

Evidence (Paseo 0.13.2 desktop + CLI on this machine, 2026-10-27):
`paseo run -d --json ...` prints `Warning: --background (-d) is deprecated.
Use --no-wait instead.` before the JSON document; `paseo send --help` shows
`--behavior <behavior>  Active-turn behavior (agent default: steer;
otherwise: interrupt) (choices: "interrupt", "steer")`.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | `bee herding run` starts a Paseo worker with `paseo run --no-wait --json ...`; the short flag `-d` is never passed. | 0.12.0 deprecated `--background`/`-d`; a future release can remove it and every worker spawn would fail. |
| D2 | `bee doctor` requires Paseo CLI 0.12.0 or newer for the Paseo channel (was 0.10.3); the FIX lines name 0.12.0. | `--no-wait` and `--behavior steer` are the 0.12+ surface bee relies on. |
| D3 | `bee herding steer` on a Paseo job steers every provider through the CLI: `paseo send <agent-id> --behavior steer --no-wait <text>`. The Node helper `paseo_steer.mjs`, the `@getpaseo/cli` package-dir lookup, the `SteerRunner` seam, the provider allow-list and the pi-specific mailbox steer file on Paseo jobs are removed. The send goes through the `PaseoCli` seam so tests use a fake CLI. Non-Paseo jobs keep writing the mailbox `steer-N.json` file unchanged. | Pi steering is native in Paseo since 0.7.2; no bee code reads the pi steer file on a Paseo job; the helper needed Node and the npm package directory. |
| D4 | The steer verb fails with code `paseo_steer_failed` and the FIX line `check the Paseo daemon` when `paseo send` returns an error; `paseo_steer_unsupported` stays only for a job whose agent has no paseo config. | The provider check is gone, so its refusal code goes with it. |

| D5 | The 0.12.0 floor holds at every check: `bee doctor` AND the pre-spawn checks in `bee herding run` (`run.rs`) with their refusal texts; one number in the runtime. | `run.rs` carried its own (0, 10, 3) floor; a 0.10.3–0.11.x CLI would pass it and then fail on `--no-wait`. |
| D6 | The steer argv places the text after a lone `--` separator: `paseo send <agent-id> --behavior steer --no-wait -- <text>`. | Live on 0.13.2, text starting with `-x` fails with `error: unknown option`; with `--` the daemon queues it. |
| D7 | Paseo owns the steer capability check: bee keeps no provider allow-list. A steer that reaches an idle worker opens a new turn; it is not dropped. The never-interrupt-and-replace rule (paseo-pi D5, store `b6dd8b33`) still holds. | Paseo steers claude, codex, opencode and pi natively; the old pi steer file had no reader, so the steer was lost silently. |

Decision store ids: D1 `5477c172`, D2 `4236f83d`, D3 `ad803822`, D4
`dc38fecc`, D5 `5ccde9fb`, D6 `11719b08`, D7 `87defaa3` (logged 2026-10-27).

### Environment facts

- Installed: Paseo desktop 0.13.2; `~/.local/bin/paseo` is the AppImage wrapper (prints `[desktop] app startup` lines before output). `herding.paseo.command` is unset, so bee resolves the running daemon's bundled CLI (0.13.2). The npm `@getpaseo/cli` under mise node 24.19.0 is still 0.10.3 (no `--no-wait`, no `--behavior`); it must be upgraded before it is ever configured as the command.
- `paseo run --help` (0.13.2) lists `--no-wait` and no `-d`/`--background`; `-d` still works with a deprecation warning.
- `paseo permit allow|deny` exits non-zero when the daemon does not confirm within 15 s (0.12.0); bee's 15 s call timeout and `permit_failed` mapping already cover it — no change.
- `paseo workspace create --background` and `workspace ls --background` exist (0.12/0.13); out of scope here.
