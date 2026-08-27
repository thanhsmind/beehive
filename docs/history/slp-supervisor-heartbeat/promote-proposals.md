promote proposal for work item "slp-supervisor-heartbeat" (docs/history/slp-supervisor-heartbeat/CONTEXT.md + docs/history/slp-supervisor-heartbeat/plan.md) — 10 capped cell(s): sup-1, sup-2, sup-3, sup-5, sup-6, sup-7, sup-8, sup-9, sup-10, sup-11
anchor: history — docs/history/slp-supervisor-heartbeat/CONTEXT.md, docs/history/slp-supervisor-heartbeat/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-supervisor-heartbeat/delivery.md

---
type: bee.delivery
title: slp-supervisor-heartbeat — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-supervisor-heartbeat: 10 capped cell(s), 26 recorded deviation(s)."
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-delivery
  lifecycle: active
  areas: [bee-herding, patterns]
  required_context: [docs/history/slp-supervisor-heartbeat/CONTEXT.md, docs/history/slp-supervisor-heartbeat/plan.md]
  sources: [docs/history/slp-supervisor-heartbeat/CONTEXT.md, docs/history/slp-supervisor-heartbeat/plan.md, .bee/cells/sup-1.json, .bee/cells/sup-2.json, .bee/cells/sup-3.json, .bee/cells/sup-5.json, .bee/cells/sup-6.json, .bee/cells/sup-7.json, .bee/cells/sup-8.json, .bee/cells/sup-9.json, .bee/cells/sup-10.json, .bee/cells/sup-11.json]
---

# slp-supervisor-heartbeat — Delivery

## What shipped

- **sup-1** — Role::Supervisor lands in the herding control loop with an enumerated read-only tool surface, config-driven model fall-through, and a self-contained observer prompt (2 file(s) changed)
- **sup-2** — supervisor record/list verbs land the append-only observation store, validated before write and wired into the dispatcher + registry (4 file(s) changed)
- **sup-3** — Supervisor tick proven end to end; the prompt record command line now reads as a template, so the shipped-command-spelling guard is green (2 file(s) changed)
- **sup-5** — Intervention/escalation mailbox with a point-key frequency cap, plus pending and mark-delivered verbs (3 file(s) changed)
- **sup-6** — The UserPromptSubmit hook appends and stamps a session's pending supervisor questions at its next turn (2 file(s) changed)
- **sup-7** — urgent mailbox kind: cap-exempt, one best-effort notification, supervisor.notify opt-out (3 file(s) changed)
- **sup-8** — Presence away/back/presence verbs with exactly two effects: the report window pair and the quiet queue (3 file(s) changed)
- **sup-9** — back renders and stores exactly one 4-section WakeReport per window; supervisor report reads it; one best-effort notification through the sup-7 seam (3 file(s) changed)
- **sup-10** — supervisor metrics answers seven derived counters with two-sided bands, sample counts and a first-class not-measurable verdict; the WakeReport carries them on one line (3 file(s) changed)
- **sup-11** — Narrow opt-in silence-is-consent: fail-closed config, a named-refusal eligibility predicate, a deterministic sweep, and a prominent report marker (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sup-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml control_loop`
- **sup-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml control_loop`
- **sup-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml hooks`
- **sup-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`
- **sup-11** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml supervisor`

## Deviations

- **sup-1** — Added a per-role default interval (900s for supervisor, 60s kept for dispatch/merge) that the cell action did not list — locked decision 322695d6 on this cell names `--interval 900` as part of the role and no other cell owns the control-loop options — found a better route
- **sup-1** — Pane reading in the supervisor allowlist goes through the transport-neutral `bee herding pane list|read` verbs instead of a raw `Bash(herdr:*)`/`Bash(tmux:*)` entry, so both transport arms are the SAME string — a raw multiplexer client carries send-text and kill-pane, which is write scope wearing a read name (R4) — found a better route
- **sup-2** — added an optional --tick flag to supervisor record — the cell mandates a tick field in the row but no verb-visible tick source exists in the tree, so the control loop must pass its index — the plan was wrong about a fact
- **sup-2** — --signal is optional and records as none when omitted, instead of being required — a silence row and a signal-free observation are both ordinary, and the closed-set refusal still fires on any unknown value — found a better route
- **sup-3** — Ran bee dev regen and folded its output into the same commit — why: editing a shipped skill reference restaled the five skill-tree projections and the release manifest, turning render_matches_the_committed_trees and opencode_projection_matches_the_committed_tree red; the cell wave-barrier regen ack had already fired in the previous commit, so deferring again would have left the suite red — kind: something else had to be fixed first
- **sup-3** — Ran bee dev regen and folded its output into the same commit; editing a shipped skill reference restaled the five skill-tree projections and the release manifest, and the wave-barrier regen ack had already fired in the previous commit — kind: something else had to be fixed first
- **sup-5** — Interventions went into their own store .bee/supervisor/interventions.jsonl instead of the observation log — an intervention is stamped delivered while an observation is written once and never touched, and separate stores keep observation|silence behavior byte-identical — found a better route
- **sup-5** — KNOWN_KINDS stayed the two tick kinds and the mailbox kinds took their own const (ALL_KINDS is the union the refusal names) — herding::control_loop pins the shipped supervisor prompt to KNOWN_KINDS and that prompt file is outside this cell files — hit an unforeseen obstacle
- **sup-5** — supervisor.record registry required went from [kind, note] to [kind] — hooks/cli_shape.rs enforces required statically and cannot express note-only-for-observation, so leaving it would block every intervention record at the hook; the verb still refuses a missing note — the plan was wrong about a fact
- **sup-5** — Added a secret/control-token refusal on --question — that text is delivered into another session context, so it takes the same guard capture add puts on stored text — found a better route
- **sup-5** — The worker could not commit: the intake gate read the control root default record (phase idle) because the session was bound to no lane; the orchestrator bound the session, re-ran the verify green, and committed — hit an unforeseen obstacle
- **sup-6** — followed the plan
- **sup-6** — sync-ack: No doctrine or skill text changes: the delivered lines are runtime mailbox rows the hook prints, not a rule; the cell declares affects_skills empty and the supervisor prompt half is its own cell. Tests are inline #[cfg(test)] modules in the two touched files (no test-shaped path exists for this crate).
- **sup-7** — followed the plan
- **sup-8** — followed the plan
- **sup-9** — The report notification is a sibling of notifier_argv (report_notifier_argv) rather than notifier_argv called with a fabricated mailbox row — a WakeReport is not an urgent intervention, so a fake row would have lied to every reader of the store; it shares NOTIFIER, spawn_notifier and the same supervisor.notify opt-out, so no second transport exists — found a better route
- **sup-9** — Added the flag name --window and bumped catalog.rs PINNED_FLAG_COUNT 194 -> 195 with the reason recorded there: --id already means a mailbox row id on this same supervisor surface, and a stored report is keyed by a row of a different store (the presence window) — the ratchet's own documented process, same reasoning sup-2 recorded for --target-session — something else had to be fixed first
- **sup-9** — One sup-8 test (away_and_back_write_nothing_outside_the_supervisor_store) now expects reports.jsonl beside interventions.jsonl and presence.jsonl, because back writes the report inside the supervisor store — hit an unforeseen obstacle
- **sup-9** — One-way-door ranking for decisions reads the irreversible member the decision log actually carries (type supersede, or a non-empty supersedes) because the log has no explicit door field; the fuller confidence x door predicate is Phase 4 (a8f4b8ab) — the plan was wrong about a fact
- **sup-10** — The metrics line is a FIXED content line inside the first report section rather than a ranked item, which costs the report one spare item line (floor 8 -> 9); a readout truncation can drop is a readout the report does not carry, and the cell requires the line to be present either way — found a better route
- **sup-10** — the_report_sorts_by_impact_if_wrong_descending now proves urgent<escalation and escalation<intervention across two windows of two rows instead of one window of three, because the metrics line takes the content line the third row used to occupy; the empty-window and pure-renderer line counts moved 8 -> 9 for the same reason — hit an unforeseen obstacle
- **sup-10** — blocked-rate takes its denominator as the union of cells claimed in the window and cells blocked in it, not literally claimed cells: a real swept-blocked cell (.bee/cells/mdp-1.json) has claimed_at null, so blocked-over-claimed can exceed 1 and is not derivable — the plan was wrong about a fact
- **sup-10** — The not-measurable list on the report line puts counters carrying a named literal state first, so a clip at 110 characters can never eat overrun (no estimate recorded) — something else had to be fixed first
- **sup-11** — The dispatched worker died mid-cell on an API rate limit after writing the implementation and its tests but before committing; the orchestrator recovered the working tree rather than re-dispatching — hit an unforeseen obstacle
- **sup-11** — sup-10 test every_counter_computes_off_records_that_already_exist now expects 8 counters, not 7, because this cell adds auto-proceeded-without-you to the set — something else had to be fixed first
- **sup-11** — The worker never reached the registry declaration, so the orchestrator added supervisor.consent-sweep to generated/registry_payload.json by hand, the same route sup-5, sup-7, sup-8 and sup-9 took for their verbs — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work slp-supervisor-heartbeat` from 10 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-supervisor-heartbeat/CONTEXT.md`, `docs/history/slp-supervisor-heartbeat/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "slp-supervisor-heartbeat" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-27T20:56:17.194Z), the work item declares no bee.areas.

area bee-herding:
  - [sup-1] Role::Supervisor lands in the herding control loop with an enumerated read-only tool surface, config-driven model fall-through, and a self-contained observer prompt — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-1.json)
  - [sup-2] supervisor record/list verbs land the append-only observation store, validated before write and wired into the dispatcher + registry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/sup-2.json)
  - [sup-3] Supervisor tick proven end to end; the prompt record command line now reads as a template, so the shipped-command-spelling guard is green — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-3.json)
  - [sup-5] Intervention/escalation mailbox with a point-key frequency cap, plus pending and mark-delivered verbs — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-5.json)
  - [sup-6] The UserPromptSubmit hook appends and stamps a session's pending supervisor questions at its next turn — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-6.json)
  - [sup-7] urgent mailbox kind: cap-exempt, one best-effort notification, supervisor.notify opt-out — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-7.json)
  - [sup-8] Presence away/back/presence verbs with exactly two effects: the report window pair and the quiet queue — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-8.json)
  - [sup-9] back renders and stores exactly one 4-section WakeReport per window; supervisor report reads it; one best-effort notification through the sup-7 seam — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-9.json)
  - [sup-10] supervisor metrics answers seven derived counters with two-sided bands, sample counts and a first-class not-measurable verdict; the WakeReport carries them on one line — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-10.json)
  - [sup-11] Narrow opt-in silence-is-consent: fail-closed config, a named-refusal eligibility predicate, a deterministic sweep, and a prominent report marker — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-11.json)

area patterns:
  - [sup-1] Role::Supervisor lands in the herding control loop with an enumerated read-only tool surface, config-driven model fall-through, and a self-contained observer prompt — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-1.json)
  - [sup-2] supervisor record/list verbs land the append-only observation store, validated before write and wired into the dispatcher + registry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/sup-2.json)
  - [sup-3] Supervisor tick proven end to end; the prompt record command line now reads as a template, so the shipped-command-spelling guard is green — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-3.json)
  - [sup-5] Intervention/escalation mailbox with a point-key frequency cap, plus pending and mark-delivered verbs — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-5.json)
  - [sup-6] The UserPromptSubmit hook appends and stamps a session's pending supervisor questions at its next turn — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-6.json)
  - [sup-7] urgent mailbox kind: cap-exempt, one best-effort notification, supervisor.notify opt-out — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-7.json)
  - [sup-8] Presence away/back/presence verbs with exactly two effects: the report window pair and the quiet queue — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-8.json)
  - [sup-9] back renders and stores exactly one 4-section WakeReport per window; supervisor report reads it; one best-effort notification through the sup-7 seam — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-9.json)
  - [sup-10] supervisor metrics answers seven derived counters with two-sided bands, sample counts and a first-class not-measurable verdict; the WakeReport carries them on one line — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/sup-10.json)
  - [sup-11] Narrow opt-in silence-is-consent: fail-closed config, a named-refusal eligibility predicate, a deterministic sweep, and a prominent report marker — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/sup-11.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sup-1 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-1-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-1 — pitfall candidate
description: "Pitfall candidate mined from cell sup-1's capped trace: Added a per-role default interval (900s for supervisor, 60s kept for dispatch/merge) that the cell action did not list — locked decision 322695d6 on this cell …"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-1-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-1.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-1 — pitfall candidate

## What the cell did

Role::Supervisor lands in the herding control loop with an enumerated read-only tool surface, config-driven model fall-through, and a self-contained observer prompt

## Recorded evidence (verbatim from .bee/cells/sup-1.json)

- **deviation** — Added a per-role default interval (900s for supervisor, 60s kept for dispatch/merge) that the cell action did not list — locked decision 322695d6 on this cell names `--interval 900` as part of the role and no other cell owns the control-loop options — found a better route
- **deviation** — Pane reading in the supervisor allowlist goes through the transport-neutral `bee herding pane list|read` verbs instead of a raw `Bash(herdr:*)`/`Bash(tmux:*)` entry, so both transport arms are the SAME string — a raw multiplexer client carries send-text and kill-pane, which is write scope wearing a read name (R4) — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-2 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-2-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-2 — pitfall candidate
description: "Pitfall candidate mined from cell sup-2's capped trace: added an optional --tick flag to supervisor record — the cell mandates a tick field in the row but no verb-visible tick source exists in the tree, so the contr…"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-2-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-2.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-2 — pitfall candidate

## What the cell did

supervisor record/list verbs land the append-only observation store, validated before write and wired into the dispatcher + registry

## Recorded evidence (verbatim from .bee/cells/sup-2.json)

- **deviation** — added an optional --tick flag to supervisor record — the cell mandates a tick field in the row but no verb-visible tick source exists in the tree, so the control loop must pass its index — the plan was wrong about a fact
- **deviation** — --signal is optional and records as none when omitted, instead of being required — a silence row and a signal-free observation are both ordinary, and the closed-set refusal still fires on any unknown value — found a better route

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-3 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-3-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-3 — pitfall candidate
description: "Pitfall candidate mined from cell sup-3's capped trace: Ran bee dev regen and folded its output into the same commit — why: editing a shipped skill reference restaled the five skill-tree projections and the release …"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-3-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-3.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-3 — pitfall candidate

## What the cell did

Supervisor tick proven end to end; the prompt record command line now reads as a template, so the shipped-command-spelling guard is green

## Recorded evidence (verbatim from .bee/cells/sup-3.json)

- **deviation** — Ran bee dev regen and folded its output into the same commit — why: editing a shipped skill reference restaled the five skill-tree projections and the release manifest, turning render_matches_the_committed_trees and opencode_projection_matches_the_committed_tree red; the cell wave-barrier regen ack had already fired in the previous commit, so deferring again would have left the suite red — kind: something else had to be fixed first
- **deviation** — Ran bee dev regen and folded its output into the same commit; editing a shipped skill reference restaled the five skill-tree projections and the release manifest, and the wave-barrier regen ack had already fired in the previous commit — kind: something else had to be fixed first
- **failure_signature** — supervisor-prompt.md:92 fenced .bee/bin/bee supervisor record --kind observation|silence backslash line is extracted as a runnable spelling and refused for missing --note, turning hooks::cli_shape::documented_invocations::no_shipped_command_spelling_is_refused_by_the_widened_guard red

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-5 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-5-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-5 — pitfall candidate
description: "Pitfall candidate mined from cell sup-5's capped trace: Interventions went into their own store .bee/supervisor/interventions.jsonl instead of the observation log — an intervention is stamped delivered while an obse…"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-5-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-5.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-5 — pitfall candidate

## What the cell did

Intervention/escalation mailbox with a point-key frequency cap, plus pending and mark-delivered verbs

## Recorded evidence (verbatim from .bee/cells/sup-5.json)

- **deviation** — Interventions went into their own store .bee/supervisor/interventions.jsonl instead of the observation log — an intervention is stamped delivered while an observation is written once and never touched, and separate stores keep observation|silence behavior byte-identical — found a better route
- **deviation** — KNOWN_KINDS stayed the two tick kinds and the mailbox kinds took their own const (ALL_KINDS is the union the refusal names) — herding::control_loop pins the shipped supervisor prompt to KNOWN_KINDS and that prompt file is outside this cell files — hit an unforeseen obstacle
- **deviation** — supervisor.record registry required went from [kind, note] to [kind] — hooks/cli_shape.rs enforces required statically and cannot express note-only-for-observation, so leaving it would block every intervention record at the hook; the verb still refuses a missing note — the plan was wrong about a fact
- **deviation** — Added a secret/control-token refusal on --question — that text is delivered into another session context, so it takes the same guard capture add puts on stored text — found a better route
- **deviation** — The worker could not commit: the intake gate read the control root default record (phase idle) because the session was bound to no lane; the orchestrator bound the session, re-ran the verify green, and committed — hit an unforeseen obstacle

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-6 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-6-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-6 — pitfall candidate
description: "Pitfall candidate mined from cell sup-6's capped trace: followed the plan"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-6-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-6.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-6 — pitfall candidate

## What the cell did

The UserPromptSubmit hook appends and stamps a session's pending supervisor questions at its next turn

## Recorded evidence (verbatim from .bee/cells/sup-6.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: No doctrine or skill text changes: the delivered lines are runtime mailbox rows the hook prints, not a rule; the cell declares affects_skills empty and the supervisor prompt half is its own cell. Tests are inline #[cfg(test)] modules in the two touched files (no test-shaped path exists for this crate).

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-7 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-7-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-7 — pitfall candidate
description: "Pitfall candidate mined from cell sup-7's capped trace: followed the plan"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-7-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-7.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-7 — pitfall candidate

## What the cell did

urgent mailbox kind: cap-exempt, one best-effort notification, supervisor.notify opt-out

## Recorded evidence (verbatim from .bee/cells/sup-7.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-8 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-8-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-8 — pitfall candidate
description: "Pitfall candidate mined from cell sup-8's capped trace: followed the plan"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-8-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-8.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-8 — pitfall candidate

## What the cell did

Presence away/back/presence verbs with exactly two effects: the report window pair and the quiet queue

## Recorded evidence (verbatim from .bee/cells/sup-8.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-9 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-9-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-9 — pitfall candidate
description: "Pitfall candidate mined from cell sup-9's capped trace: The report notification is a sibling of notifier_argv (report_notifier_argv) rather than notifier_argv called with a fabricated mailbox row — a WakeReport is n…"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-9-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-9.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-9 — pitfall candidate

## What the cell did

back renders and stores exactly one 4-section WakeReport per window; supervisor report reads it; one best-effort notification through the sup-7 seam

## Recorded evidence (verbatim from .bee/cells/sup-9.json)

- **deviation** — The report notification is a sibling of notifier_argv (report_notifier_argv) rather than notifier_argv called with a fabricated mailbox row — a WakeReport is not an urgent intervention, so a fake row would have lied to every reader of the store; it shares NOTIFIER, spawn_notifier and the same supervisor.notify opt-out, so no second transport exists — found a better route
- **deviation** — Added the flag name --window and bumped catalog.rs PINNED_FLAG_COUNT 194 -> 195 with the reason recorded there: --id already means a mailbox row id on this same supervisor surface, and a stored report is keyed by a row of a different store (the presence window) — the ratchet's own documented process, same reasoning sup-2 recorded for --target-session — something else had to be fixed first
- **deviation** — One sup-8 test (away_and_back_write_nothing_outside_the_supervisor_store) now expects reports.jsonl beside interventions.jsonl and presence.jsonl, because back writes the report inside the supervisor store — hit an unforeseen obstacle
- **deviation** — One-way-door ranking for decisions reads the irreversible member the decision log actually carries (type supersede, or a non-empty supersedes) because the log has no explicit door field; the fuller confidence x door predicate is Phase 4 (a8f4b8ab) — the plan was wrong about a fact

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-10 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-10-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-10 — pitfall candidate
description: "Pitfall candidate mined from cell sup-10's capped trace: The metrics line is a FIXED content line inside the first report section rather than a ranked item, which costs the report one spare item line (floor 8 -> 9); …"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-10-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-10.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-10 — pitfall candidate

## What the cell did

supervisor metrics answers seven derived counters with two-sided bands, sample counts and a first-class not-measurable verdict; the WakeReport carries them on one line

## Recorded evidence (verbatim from .bee/cells/sup-10.json)

- **deviation** — The metrics line is a FIXED content line inside the first report section rather than a ranked item, which costs the report one spare item line (floor 8 -> 9); a readout truncation can drop is a readout the report does not carry, and the cell requires the line to be present either way — found a better route
- **deviation** — the_report_sorts_by_impact_if_wrong_descending now proves urgent<escalation and escalation<intervention across two windows of two rows instead of one window of three, because the metrics line takes the content line the third row used to occupy; the empty-window and pure-renderer line counts moved 8 -> 9 for the same reason — hit an unforeseen obstacle
- **deviation** — blocked-rate takes its denominator as the union of cells claimed in the window and cells blocked in it, not literally claimed cells: a real swept-blocked cell (.bee/cells/mdp-1.json) has claimed_at null, so blocked-over-claimed can exceed 1 and is not derivable — the plan was wrong about a fact
- **deviation** — The not-measurable list on the report line puts counters carrying a named literal state first, so a clip at 110 characters can never eat overrun (no estimate recorded) — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sup-11 — save as docs/knowledge/patterns/slp-supervisor-heartbeat-sup-11-pitfall.md

---
type: bee.pattern
title: slp-supervisor-heartbeat cell sup-11 — pitfall candidate
description: "Pitfall candidate mined from cell sup-11's capped trace: The dispatched worker died mid-cell on an API rate limit after writing the implementation and its tests but before committing; the orchestrator recovered the w…"
timestamp: 2026-08-27
bee:
  id: slp-supervisor-heartbeat-sup-11-pitfall
  lifecycle: draft
  areas: [bee-herding, patterns]
  sources: [.bee/cells/sup-11.json]
  polarity: pitfall
---

# slp-supervisor-heartbeat cell sup-11 — pitfall candidate

## What the cell did

Narrow opt-in silence-is-consent: fail-closed config, a named-refusal eligibility predicate, a deterministic sweep, and a prominent report marker

## Recorded evidence (verbatim from .bee/cells/sup-11.json)

- **deviation** — The dispatched worker died mid-cell on an API rate limit after writing the implementation and its tests but before committing; the orchestrator recovered the working tree rather than re-dispatching — hit an unforeseen obstacle
- **deviation** — sup-10 test every_counter_computes_off_records_that_already_exist now expects 8 counters, not 7, because this cell adds auto-proceeded-without-you to the set — something else had to be fixed first
- **deviation** — The worker never reached the registry declaration, so the orchestrator added supervisor.consent-sweep to generated/registry_payload.json by hand, the same route sup-5, sup-7, sup-8 and sup-9 took for their verbs — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 10 capped cell(s) mined, 1 delivery draft, 20 area bullet(s), 10 pattern candidate(s), 0 file(s) written.