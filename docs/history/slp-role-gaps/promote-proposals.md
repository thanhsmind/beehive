promote proposal for work item "slp-role-gaps" (docs/history/slp-role-gaps/CONTEXT.md + docs/history/slp-role-gaps/plan.md) — 6 capped cell(s): rig-1, rig-2, rig-3, rig-5, rig-6, rig-7
anchor: history — docs/history/slp-role-gaps/CONTEXT.md, docs/history/slp-role-gaps/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-role-gaps/delivery.md

---
type: bee.delivery
title: slp-role-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-role-gaps: 6 capped cell(s), 16 recorded deviation(s)."
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-delivery
  lifecycle: active
  areas: [workflow-state, bee-herding, doctrine-layer]
  required_context: [docs/history/slp-role-gaps/CONTEXT.md, docs/history/slp-role-gaps/plan.md]
  sources: [docs/history/slp-role-gaps/CONTEXT.md, docs/history/slp-role-gaps/plan.md, .bee/cells/rig-1.json, .bee/cells/rig-2.json, .bee/cells/rig-3.json, .bee/cells/rig-5.json, .bee/cells/rig-6.json, .bee/cells/rig-7.json]
---

# slp-role-gaps — Delivery

## What shipped

- **rig-1** — bee cells dissent refuses a repeat dissent after a verdict unless --new-evidence is given; evidence is stored on the appended entry (4 file(s) changed)
- **rig-2** — Five optional finish-report keys validated, stored on the trace and cleared on release (4 file(s) changed)
- **rig-3** — Herding results carry the handback fields and dissent new_evidence into the run envelope, on_model_mismatch refuse stops the agent while polling with outcome model_mismatch, and the envelope carries model_mismatch in both modes (5 file(s) changed)
- **rig-5** — herding status reads a write job with a frozen worktree fingerprint as stalled with no_diff_progress and appends one progress message per stretch, observe-only (2 file(s) changed)
- **rig-6** — Optional result keys, partial stop, dissent once and four-layer report land in Result Formats; SKILL.md and worker-cell.md point at them (4 file(s) changed)
- **rig-7** — Re-cut advisor and reviewer dispatch goldens and pinned the rig-1 key_link span (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **rig-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee dissent` — cell verify over dissent.rs (45 passed); also -p bee catalog (17 passed) and --test registry_contracts (12 passed) for the flag table and registry; full suite not run
- **rig-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee cells::` — leader re-run after the herding worker: 388 passed, 0 failed, 10 ignored; covers the nine new D1/D2 tests
- **rig-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::` — the cell verify filter over every touched herding module, 765 passed. The full suite was not run. verbs::drivers:: has 6 reds from the wave-1 advisor falsifier prompt line, outside this cell
- **rig-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee herding::tests` — the cell verify, run with the cargo binary path direct: herding::tests 35 passed; also herding:: across all herding modules 753 passed; full suite not run
- **rig-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws` — cell verify over the instruction text touched, 9 passed; native_cell_prompt_render golden green after update; NOT green: every_template_carries_the_original_request_block_and_matches_disk stays red u…
- **rig-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --test instruction_laws` — cell verify 9/9 green; also ran cargo test --release -p bee drivers:: (441 passed, 0 failed) and -p bee no_shipped_command_spelling (1 passed); full suite not run

## Deviations

- **rig-1** — catalog.rs had no per-flag table entry to add; the change there is the pinned flag-count bump 224->225, with the why in the commit body instead of a code comment — the plan was wrong about a fact
- **rig-1** — the catalog flag registry is read from registry_payload.json, so the hand edit there is what registers --new-evidence — the plan was wrong about a fact
- **rig-1** — capped with --sync-ack because the cell names no skill and the worker skill text already landed in 7b0357b54 — the plan was wrong about a fact
- **rig-1** — sync-ack: cell affects_skills is empty by plan; the worker-facing dissent-once teaching already landed in skills via commit 7b0357b54; this cell only adds the CLI refusal and its concept
- **rig-2** — followed the plan
- **rig-2** — sync-ack: the worker-facing doctrine for these keys landed in skills/bee-swarming/references/swarming-reference.md under rig-6 (commit 7b0357b54)
- **rig-3** — The job record also stores paseo_model_configured and paseo_thinking_configured on a mismatch — the envelope needs the configured side and the job record is the one place both the refusal message and the envelope read — found a better route
- **rig-3** — The five handback fields live in one MailboxHandback sub-struct with Default instead of five flat fields — each of ten MailboxResult literals gains one line, not five — found a better route
- **rig-3** — The refuse path ends the poll loop through a blocked tick plus a new unit PollDecision::ModelMismatch, with unreachable arms at the three herdr and tmux match sites — PollDecision is Copy so it cannot carry the message, which is built at the paseo outcome site from job.json — hit an unforeseen obstacle
- **rig-3** — sync-ack: The bee-herding skill names no result or envelope field and no on_model_mismatch knob. The plan set affects_skills to empty and the facts live in the two bee-herding knowledge concepts this cell updated
- **rig-5** — Added insert_stall_reason so herding status shows stall_reason beside a stalled word — the done-criterion says status shows stalled with no_diff_progress and the reason lived only on job.json — found a better route
- **rig-5** — compute_job_status now computes the existing word before the stall check so the fingerprint runs only when the status would be working — the action limits the fingerprint to would-be-working jobs — found a better route
- **rig-5** — sync-ack: plan predicts affects_skills empty: the stall reason surfaces through the existing herding status read and is documented in the bee-herding overview concept; no skill instruction changes
- **rig-6** — updated the worker-cell golden string in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs (reserved first) — native_cell_prompt_render_is_byte_identical_to_golden went red on the new prompt block — something else had to be fixed first
- **rig-7** — the REVIEWER golden in no_fallback_chain_config_leaves_the_whole_dispatch_payload_byte_identical also failed on the two new reviewer.md lines, which the rework list did not name — updated the REVIEWER literal too — reviewer.md is this cell own change, same red cause
- **rig-7** — sync-ack: both predicted skill files were touched by this cell first cap commit 4ebbc2edd; this reopen only re-cuts test goldens and a cli_shape pin

## Provenance

Proposed by `bee knowledge promote --work slp-role-gaps` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-role-gaps/CONTEXT.md`, `docs/history/slp-role-gaps/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "slp-role-gaps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-15T12:53:12.237Z), the work item declares no bee.areas.

area workflow-state:
  - [rig-1] bee cells dissent refuses a repeat dissent after a verdict unless --new-evidence is given; evidence is stored on the appended entry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-1.json)
  - [rig-2] Five optional finish-report keys validated, stored on the trace and cleared on release — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-2.json)
  - [rig-3] Herding results carry the handback fields and dissent new_evidence into the run envelope, on_model_mismatch refuse stops the agent while polling with outcome model_mismatch, and the envelope carries model_mismatch in both modes — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rig-3.json)
  - [rig-5] herding status reads a write job with a frozen worktree fingerprint as stalled with no_diff_progress and appends one progress message per stretch, observe-only — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-5.json)
  - [rig-6] Optional result keys, partial stop, dissent once and four-layer report land in Result Formats; SKILL.md and worker-cell.md point at them — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-6.json)
  - [rig-7] Re-cut advisor and reviewer dispatch goldens and pinned the rig-1 key_link span — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-7.json)

area bee-herding:
  - [rig-1] bee cells dissent refuses a repeat dissent after a verdict unless --new-evidence is given; evidence is stored on the appended entry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-1.json)
  - [rig-2] Five optional finish-report keys validated, stored on the trace and cleared on release — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-2.json)
  - [rig-3] Herding results carry the handback fields and dissent new_evidence into the run envelope, on_model_mismatch refuse stops the agent while polling with outcome model_mismatch, and the envelope carries model_mismatch in both modes — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rig-3.json)
  - [rig-5] herding status reads a write job with a frozen worktree fingerprint as stalled with no_diff_progress and appends one progress message per stretch, observe-only — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-5.json)
  - [rig-6] Optional result keys, partial stop, dissent once and four-layer report land in Result Formats; SKILL.md and worker-cell.md point at them — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-6.json)
  - [rig-7] Re-cut advisor and reviewer dispatch goldens and pinned the rig-1 key_link span — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-7.json)

area doctrine-layer:
  - [rig-1] bee cells dissent refuses a repeat dissent after a verdict unless --new-evidence is given; evidence is stored on the appended entry — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-1.json)
  - [rig-2] Five optional finish-report keys validated, stored on the trace and cleared on release — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-2.json)
  - [rig-3] Herding results carry the handback fields and dissent new_evidence into the run envelope, on_model_mismatch refuse stops the agent while polling with outcome model_mismatch, and the envelope carries model_mismatch in both modes — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/rig-3.json)
  - [rig-5] herding status reads a write job with a frozen worktree fingerprint as stalled with no_diff_progress and appends one progress message per stretch, observe-only — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-5.json)
  - [rig-6] Optional result keys, partial stop, dissent once and four-layer report land in Result Formats; SKILL.md and worker-cell.md point at them — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/rig-6.json)
  - [rig-7] Re-cut advisor and reviewer dispatch goldens and pinned the rig-1 key_link span — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/rig-7.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell rig-1 — save as docs/knowledge/patterns/slp-role-gaps-rig-1-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-1 — pitfall candidate
description: "Pitfall candidate mined from cell rig-1's capped trace: catalog.rs had no per-flag table entry to add; the change there is the pinned flag-count bump 224->225, with the why in the commit body instead of a code comme…"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-1-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-1.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-1 — pitfall candidate

## What the cell did

bee cells dissent refuses a repeat dissent after a verdict unless --new-evidence is given; evidence is stored on the appended entry

## Recorded evidence (verbatim from .bee/cells/rig-1.json)

- **deviation** — catalog.rs had no per-flag table entry to add; the change there is the pinned flag-count bump 224->225, with the why in the commit body instead of a code comment — the plan was wrong about a fact
- **deviation** — the catalog flag registry is read from registry_payload.json, so the hand edit there is what registers --new-evidence — the plan was wrong about a fact
- **deviation** — capped with --sync-ack because the cell names no skill and the worker skill text already landed in 7b0357b54 — the plan was wrong about a fact
- **deviation** — sync-ack: cell affects_skills is empty by plan; the worker-facing dissent-once teaching already landed in skills via commit 7b0357b54; this cell only adds the CLI refusal and its concept

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rig-2 — save as docs/knowledge/patterns/slp-role-gaps-rig-2-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-2 — pitfall candidate
description: "Pitfall candidate mined from cell rig-2's capped trace: followed the plan"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-2-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-2.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-2 — pitfall candidate

## What the cell did

Five optional finish-report keys validated, stored on the trace and cleared on release

## Recorded evidence (verbatim from .bee/cells/rig-2.json)

- **deviation** — followed the plan
- **deviation** — sync-ack: the worker-facing doctrine for these keys landed in skills/bee-swarming/references/swarming-reference.md under rig-6 (commit 7b0357b54)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rig-3 — save as docs/knowledge/patterns/slp-role-gaps-rig-3-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-3 — pitfall candidate
description: "Pitfall candidate mined from cell rig-3's capped trace: The job record also stores paseo_model_configured and paseo_thinking_configured on a mismatch — the envelope needs the configured side and the job record is th…"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-3-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-3.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-3 — pitfall candidate

## What the cell did

Herding results carry the handback fields and dissent new_evidence into the run envelope, on_model_mismatch refuse stops the agent while polling with outcome model_mismatch, and the envelope carries model_mismatch in both modes

## Recorded evidence (verbatim from .bee/cells/rig-3.json)

- **deviation** — The job record also stores paseo_model_configured and paseo_thinking_configured on a mismatch — the envelope needs the configured side and the job record is the one place both the refusal message and the envelope read — found a better route
- **deviation** — The five handback fields live in one MailboxHandback sub-struct with Default instead of five flat fields — each of ten MailboxResult literals gains one line, not five — found a better route
- **deviation** — The refuse path ends the poll loop through a blocked tick plus a new unit PollDecision::ModelMismatch, with unreachable arms at the three herdr and tmux match sites — PollDecision is Copy so it cannot carry the message, which is built at the paseo outcome site from job.json — hit an unforeseen obstacle
- **deviation** — sync-ack: The bee-herding skill names no result or envelope field and no on_model_mismatch knob. The plan set affects_skills to empty and the facts live in the two bee-herding knowledge concepts this cell updated

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rig-5 — save as docs/knowledge/patterns/slp-role-gaps-rig-5-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-5 — pitfall candidate
description: "Pitfall candidate mined from cell rig-5's capped trace: Added insert_stall_reason so herding status shows stall_reason beside a stalled word — the done-criterion says status shows stalled with no_diff_progress and t…"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-5-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-5.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-5 — pitfall candidate

## What the cell did

herding status reads a write job with a frozen worktree fingerprint as stalled with no_diff_progress and appends one progress message per stretch, observe-only

## Recorded evidence (verbatim from .bee/cells/rig-5.json)

- **deviation** — Added insert_stall_reason so herding status shows stall_reason beside a stalled word — the done-criterion says status shows stalled with no_diff_progress and the reason lived only on job.json — found a better route
- **deviation** — compute_job_status now computes the existing word before the stall check so the fingerprint runs only when the status would be working — the action limits the fingerprint to would-be-working jobs — found a better route
- **deviation** — sync-ack: plan predicts affects_skills empty: the stall reason surfaces through the existing herding status read and is documented in the bee-herding overview concept; no skill instruction changes

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rig-6 — save as docs/knowledge/patterns/slp-role-gaps-rig-6-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-6 — pitfall candidate
description: "Pitfall candidate mined from cell rig-6's capped trace: updated the worker-cell golden string in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs (reserved first) — native_cell_prompt_render_is_byte_identical_t…"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-6-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-6.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-6 — pitfall candidate

## What the cell did

Optional result keys, partial stop, dissent once and four-layer report land in Result Formats; SKILL.md and worker-cell.md point at them

## Recorded evidence (verbatim from .bee/cells/rig-6.json)

- **deviation** — updated the worker-cell golden string in packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs (reserved first) — native_cell_prompt_render_is_byte_identical_to_golden went red on the new prompt block — something else had to be fixed first

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell rig-7 — save as docs/knowledge/patterns/slp-role-gaps-rig-7-pitfall.md

---
type: bee.pattern
title: slp-role-gaps cell rig-7 — pitfall candidate
description: "Pitfall candidate mined from cell rig-7's capped trace: the REVIEWER golden in no_fallback_chain_config_leaves_the_whole_dispatch_payload_byte_identical also failed on the two new reviewer.md lines, which the rework…"
timestamp: 2026-10-15
bee:
  id: slp-role-gaps-rig-7-pitfall
  lifecycle: draft
  areas: [workflow-state, bee-herding, doctrine-layer]
  sources: [.bee/cells/rig-7.json]
  polarity: pitfall
---

# slp-role-gaps cell rig-7 — pitfall candidate

## What the cell did

Re-cut advisor and reviewer dispatch goldens and pinned the rig-1 key_link span

## Recorded evidence (verbatim from .bee/cells/rig-7.json)

- **deviation** — the REVIEWER golden in no_fallback_chain_config_leaves_the_whole_dispatch_payload_byte_identical also failed on the two new reviewer.md lines, which the rework list did not name — updated the REVIEWER literal too — reviewer.md is this cell own change, same red cause
- **deviation** — sync-ack: both predicted skill files were touched by this cell first cap commit 4ebbc2edd; this reopen only re-cuts test goldens and a cli_shape pin

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 18 area bullet(s), 6 pattern candidate(s), 0 file(s) written.