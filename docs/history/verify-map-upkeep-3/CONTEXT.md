# verify-map-upkeep-3 — context

The 2026-11-25 `bee-verify-upkeep` pass over `.bee/verify/verify-app/`. The
pass was due with reason `never_run` (no `verify-upkeep` audit row). Scope is
the verification skill's own source tree and its rendered runtime copies. No
product code changes.

## Locked Decisions

| # | Decision | Why |
|---|---|---|
| D1 | `control-bee` isolates the user-global config by default: every bee process it starts gets `BEEHIVE_CONFIG_DIR` set to an empty folder inside the run dir, unless the caller already exported `BEEHIVE_CONFIG_DIR`. | A live drive read the real user-global config: the sandbox got paseo agents, a `paseo_ready` doctor row, and hat seats without `--no-pane`. With an empty global folder the Pi recipes match the map. |
| D2 | `control-bee drive --fast` and `drive --full` are the deploy-facing entry points. Each prints one `ok` or `FAIL` line per mapped feature, exits 0 when all pass, 1 when any fails, and 2 for an unknown entry point. | The upkeep skill requires them; a deploy script calls `drive --full` and trusts the exit code. |
| D3 | Each feature drive runs in its own fresh sandbox on a private run id, never touches `current-run`, removes the sandbox on pass and keeps it on fail. Evidence is always kept. | Same isolation model as `smoke`; one feature's state cannot leak into the next. |
| D4 | `drive` never starts a paid model. Features whose full recipe needs a live agent (Pi leader, Codex canary, no-pane Pi worker) drive their CLI surface only and say so on their `ok` line. | `drive --full` runs from a deploy script; a model call there costs money and depends on credentials. |
| D5 | Map corrections fix only claims proven wrong by a live drive or a cited source line. Omitted sub-behaviours stay out unless a recipe needs them. | The unit of rigor is the feature, not every bullet. |

## Product gaps found (reported, not fixed here)

- The herded read-only worker guard refuses `rg` searches whose pattern holds tokens such as `sessionRuntime`, `--id` or `claim_ownership`. Two of thirteen source-wave digests came back incomplete.
- `commands.test: "none"` counts as a declared test in the onboard verify notice, but the retirement text tells users to set `"none"` for a test-free repo.

<!-- bee:no-read-set: harness and doc upkeep; the read set is the feature map itself -->
