# Provenance — bee-herding body rules

The body states its rules bare (provenance exile, skill-token-diet D8). This
table maps each body rule to the internal design-decision tag(s) that
authorize it and the rationale in one line. These `D…`/`i54-closeout-4` tags
are this skill's own shorthand, settled during its own build — they do not
resolve to `.bee/decisions.jsonl` or `docs/decisions/` entries; this file is
their only durable record.

| Body rule | Tags | Rationale |
|---|---|---|
| `bee-` prefix mandatory on the skill directory | (preflight law) | Distribution preflight refuses a non-matching dir; plugin render only copies `bee-*` — a misnamed skill installs for nobody |
| Bootstrap is one-shot, human-invoked, no `--role` | D13 | Cockpit layout fixes two control panes; bootstrap builds the layout once, not a loop |
| Bootstrap pre-flight requires `gate_bypass_level` `full`/`total` | D6 | An auto-created worktree inherits this repo's bypass level; dispatch would refuse every cycle below `full` anyway |
| Dispatch self-names every iteration via `herdr pane current` | D17 | herdr assigns no name of its own; a label is pane metadata that must be claimed, not assumed |
| Dispatch refuses below `gate_bypass: full` | D6 | An unattended agent must never inherit `normal`'s latitude for hard-gate work |
| Occupancy count + anomaly reporting, once per condition | D5, D18, D20 | D5 fixes the 4-slot cap; D18 forbids a registry file so scrollback is deduped by reading it; D20 forbids `agent_status` as finished-proof |
| Dispatchable set built only past the enable interlock | D1, D10 | D1 is the four-condition test; D10 is the owner marker gating whether the loop runs at all — ordinary post-exploring state would otherwise look dispatchable |
| Lane-safety is a two-key gate (script + own reading) | D6 | The classifier is fail-open on unmatched keywords (proven: 8/8 adversarial rows passed); only independent human-language reading catches what the wordlist can't |
| Rank and announce into the chat pane before acting | D16 | No priority field exists by design; the announcement is the audit trail for "why this one" |
| Spawn sequence: worktree new, then `agent start` (never split-first, never `-p`) | D14, D9, D22, D4 | D14: worktrees are created from main only; D9: herdr-go's own config stays untouched, flags travel as spawn-time argv; D22/D4: proven live that split-first and `-p` both corrupt the pane/session shape |
| Merge finds finished worktrees from bee's own state only | D2, D20 | D2 is the four-condition finished test; D20 forbids herdr's `agent_status` as evidence — an idle agent looks identical mid-item, waiting, or crashed |
| Red-stop marker checked before any merge; durable file, not chat text | D3, D18 | D3: stop cold, never retry; D18 forbids an occupancy registry but a red-attempt marker records a different fact with no other durable home |
| Merge/cleanup per worktree; stop cold, no retry, on red | D3, D15, D19 | D15: pane close only happens on a successful merge; D19: one worktree's failure never aborts the rest of the pass |
| Merge is a single-shot owner gesture, never looped | D11 | The single highest-authority action (landing work in main) requires a human present |
| Working agents run `bypassPermissions`, no allowlist — accepted risk | D7-FINAL | A narrowed working agent stalls forever on the first no-TTY permission prompt, defeating unattended dispatch; worktree/branch confinement is a git boundary, not a sandbox |
| Control panes run an enumerated `--allowedTools` list, never `bypassPermissions`, never "read-only" | D7-FINAL | Both control roles genuinely write (`bee worktree new`; `git merge --abort`, `.bee/tmp/` markers, `bee worktree merge --cleanup`); "read-only" would silently stall every interval |
| Runtime adapter: config-driven spawn argv, byte-equivalent default, per-token substitution only | D4, i54-closeout-4 | No `herding` config keys reproduces today's hardcoded strings exactly; per-token substitution (never join-then-split, never `eval`) is the shell-injection-safe shape |
| Lane classifier is advisory, never containment | D6 | Measured false: 8/8 adversarial backlog rows passed a keyword-based classifier, including a "delete the entire JS runtime" story |
| Containment ladder: enable interlock, owner-gesture merge, worktree isolation, slot cap + stop file, Key-2 reading | D10, D11, D14, D5, D6 | Five independent layers, descending load-bearing order; the classifier itself is not one of them |
| Stop file halts the control loop only, not running working agents | (stop/resume design) | Each working agent is its own `claude` session that never reads the stop file; only closing its pane stops it |
