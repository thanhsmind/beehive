# Herding transport bypassed silently — field report (2026-08-22)

Source: sibling waggledance session (herdr pane w5:p5), bee 2.18.3. Observed, not inferred.

## 1. Reachability is not reported
- `bee herding status` → "not built into this binary …". Reads as "herding unavailable". Still listed in `--help --names`.
- `bee herding run --help` → "Splits a pane off the caller's own runtime pane". Cold agent reads: "I have no pane, cannot work".
- `bee dispatch prepare --cell <id> --kind cell --runtime claude --json` returns BOTH a `herding run … --agent pi-agy-flash-3.7` command and `fallback:{model:"sonnet"}`, with `economics.channel:"herding-exec"`. Nothing says which applies. Wrong guess is silent: fallback runs, cells go green, configured routing unused.
- Ground truth was live: `HERDR_PANE_ID=w5:p5`, `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, `herdr agent list` showed the session. bee does not point at them.

Suggested: `dispatch prepare` probes HERDR_ENV/HERDR_PANE_ID (optionally socket) and emits `transport_ready: bool` + `transport_reason`; `fallback` means "only when transport_ready is false". Build `herding status` or make its message say it does not answer availability and name the env vars. State the env check in the bee-swarming Delegation contract.

## 2. Rejected model id surfaces as a mailbox failure
`bee herding run --task-file - --json --cwd <wt> --agent "pi-agy-flash-3.7"` →
`outcome:"spawn_failed"`, error: "brief prompt failed after start: resent the pointer 10 time(s) (the agent kept going ready with no ack) but neither the round's ack file nor its result file ever appeared".
Pane (`herdr pane read w5:pS`) answered every round with "No response from agy."; status bar `agy/gemini-3.7-flash [high]`.

Isolation:
- `agy -p "…"` → ok.
- `agy -p "…" --model "agy/gemini-3.7-flash:high"` → "model … is not recognized" (list includes "Gemini 3.7 Flash (High)").
- `agy -p "…" --model gemini-3.7-flash --effort high` → ok. Model and effort are two flags.
- `pi -p "…" --model "agy/gemini-3.7-flash:high"` → hung, killed at 90s.

Suggested: preflight the configured agent once before the pane split (trivial prompt, short timeout), fail fast with provider error text. When rounds return with no ack, include a pane tail in the error.

## 3. Cleanup gap on spawn_failed
Cell stays claimed, files stay reserved, pane left open as forensics. Orchestrator unwinds by hand. Want a documented one-liner or auto-release on spawn_failed.

## 4. The recorded cell tier does not reach dispatch prepare
Same session, next cell. `bee cells tier --id ctk-6 --tier ceiling` recorded; `cells show` → `"tier":"ceiling"`. Then `bee dispatch prepare --cell ctk-6 --worker exec-ctk-6 --runtime claude --kind cell --claim --json` → `economics.logical_tier:"generation"`, `channel:"herding-exec"`, payload command `bee herding run … --agent "agy-flash"`. `dispatch prepare --help` has no `--tier` flag. `cells tier` rations ceiling (40% budget, decision 0012) and prepare discards the choice one verb later. A ceiling cell has no herding slot (ceiling IS the session model), so the downgrade is invisible, not an error.

Suggested, in order: (1) prepare --kind cell reads the cell's recorded tier and resolves from it; (2) else a --tier flag; (3) at minimum a tier_source field plus a warning when the resolved slot disagrees with the recorded tier; (4) a ceiling cell with no transport slot reports `channel:"session-model"` plainly instead of another tier's herding command.
