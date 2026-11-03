promote proposal for work item "slp-followup-gaps" (docs/history/slp-followup-gaps/CONTEXT.md + docs/history/slp-followup-gaps/plan.md) — 6 capped cell(s): sfg-1, sfg-2, sfg-3, sfg-4, sfg-5, sfg-6
anchor: history — docs/history/slp-followup-gaps/CONTEXT.md, docs/history/slp-followup-gaps/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/slp-followup-gaps/delivery.md

---
type: bee.delivery
title: slp-followup-gaps — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-followup-gaps: 6 capped cell(s), 12 recorded deviation(s)."
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-delivery
  lifecycle: active
  areas: [hook-runtime, bee-herding, workflow-state]
  required_context: [docs/history/slp-followup-gaps/CONTEXT.md, docs/history/slp-followup-gaps/plan.md]
  sources: [docs/history/slp-followup-gaps/CONTEXT.md, docs/history/slp-followup-gaps/plan.md, .bee/cells/archive/slp-followup-gaps/sfg-1.json, .bee/cells/archive/slp-followup-gaps/sfg-2.json, .bee/cells/archive/slp-followup-gaps/sfg-3.json, .bee/cells/archive/slp-followup-gaps/sfg-4.json, .bee/cells/archive/slp-followup-gaps/sfg-5.json, .bee/cells/archive/slp-followup-gaps/sfg-6.json]
---

# slp-followup-gaps — Delivery

## What shipped

- **sfg-1** — An unbound session holding one live claim is judged against its claimed feature's lane record at both the write check and the git intake gate; the intake refusal names the session binding as the remedy (4 file(s) changed)
- **sfg-2** — Herding briefs teach a dissent object, the parser reads it leniently, and the run verb transcribes it through record_dissent with the outcome stamped on the envelope (2 file(s) changed)
- **sfg-3** — The claim readers are infallible so no malformed claim can fail the write guard open, and the ownership guard's claim-derived trigger set is stated honestly and pinned both ways (3 file(s) changed)
- **sfg-4** — The heartbeat, control-root and product-root readers are infallible, so no unparseable store or config byte can switch the write guard off through them (3 file(s) changed)
- **sfg-5** — The lease, hold and strict-session fail-opens are closed, two false comments repaired, and the heartbeat lockout made visible (8 file(s) changed)
- **sfg-6** — The last store read that could switch the write guard off now denies natively, and the module header describes the guard that exists (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **sfg-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`
- **sfg-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml herding`
- **sfg-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`
- **sfg-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`
- **sfg-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`
- **sfg-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml write_guard`

## Deviations

- **sfg-1** — Grouped the new source claim with default at the msn-21 workspace-ownership check instead of letting it skip that guard — the claim arm derives WHICH LANE a session works under and says nothing about who owns the checkout — found a better route
- **sfg-1** — The judge found two hunt defects after the cap: the claim readers could fail the guard open on a malformed timestamp, and the ownership comment asserted an invariance the code no longer held. Both were fixed in cell sfg-3 and re-judged PASS — something else had to be fixed first
- **sfg-2** — verbs/cells/dissent.rs was listed in the cell but needed no edit — record_dissent is already pub(crate) and re-exported through verbs::cells, so herding/run.rs calls it unchanged — the plan was wrong about a fact
- **sfg-2** — The retargeted negative pin bans any `bee <word>` command by scanning, instead of the literal `bee cells` plus a bare `bee ` substring ban — the brief's own prose already says "Ignore any bee or agent-workflow instructions", so a bare `bee ` ban would fail on text that names no command — hit an unforeseen obstacle
- **sfg-2** — sync-ack: skills/bee-herding documents the mailbox FILE LAYOUT (job.json, result-N.json, ack-N.json), not the result JSON field list — the layout is byte-unchanged here, and the StopAndAsk pair (options/leaning) added the same kind of field with no skill edit either. The cell scopes three source files and declares affects_skills: [].
- **sfg-3** — followed the plan
- **sfg-4** — The judge found the sweep incomplete: lease and hold timestamp readers and the read_session_strict path carried the same escape, and two hook_local.rs comments were left describing the old behavior. All were closed in cell sfg-5 — something else had to be fixed first
- **sfg-5** — Reserved and edited two files the cell did not list, jspath.rs and mod.rs — the warning queue the lockout line needs lives in jspath.rs and had no push helper, and mod.rs still listed the strict session read as delegated — something else had to be fixed first
- **sfg-5** — The judge found one surviving escape on the companion marker and a header bullet narrowed too far; both were closed in cell sfg-6 — something else had to be fixed first
- **sfg-6** — Kept resolve_verified_companion_mount_real as a thin wrapper over a new three-answer resolve_companion_mount instead of changing its signature — crate::nested_checkout maps its Err onto a native fail-closed refusal that had to be preserved — found a better route
- **sfg-6** — Removed the headers timestamp-strings delegated bullet and tightened the every-reader-is-infallible sentence to exempt guards.memory_root — the header sweep the cell asked for found both — something else had to be fixed first
- **sfg-6** — The judge found the rewritten delegated-branch list still claimed to be exhaustive while omitting two shape delegates; both were added in commit 85ead065 and the verdict re-recorded PASS — something else had to be fixed first

## Provenance

Proposed by `bee knowledge promote --work slp-followup-gaps` from 6 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-followup-gaps/CONTEXT.md`, `docs/history/slp-followup-gaps/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "slp-followup-gaps" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-29T06:03:31.499Z), the work item declares no bee.areas.

area hook-runtime:
  - [sfg-1] An unbound session holding one live claim is judged against its claimed feature's lane record at both the write check and the git intake gate; the intake refusal names the session binding as the remedy — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-1.json)
  - [sfg-2] Herding briefs teach a dissent object, the parser reads it leniently, and the run verb transcribes it through record_dissent with the outcome stamped on the envelope — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-2.json)
  - [sfg-3] The claim readers are infallible so no malformed claim can fail the write guard open, and the ownership guard's claim-derived trigger set is stated honestly and pinned both ways — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-3.json)
  - [sfg-4] The heartbeat, control-root and product-root readers are infallible, so no unparseable store or config byte can switch the write guard off through them — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-4.json)
  - [sfg-5] The lease, hold and strict-session fail-opens are closed, two false comments repaired, and the heartbeat lockout made visible — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-5.json)
  - [sfg-6] The last store read that could switch the write guard off now denies natively, and the module header describes the guard that exists — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-6.json)

area bee-herding:
  - [sfg-1] An unbound session holding one live claim is judged against its claimed feature's lane record at both the write check and the git intake gate; the intake refusal names the session binding as the remedy — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-1.json)
  - [sfg-2] Herding briefs teach a dissent object, the parser reads it leniently, and the run verb transcribes it through record_dissent with the outcome stamped on the envelope — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-2.json)
  - [sfg-3] The claim readers are infallible so no malformed claim can fail the write guard open, and the ownership guard's claim-derived trigger set is stated honestly and pinned both ways — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-3.json)
  - [sfg-4] The heartbeat, control-root and product-root readers are infallible, so no unparseable store or config byte can switch the write guard off through them — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-4.json)
  - [sfg-5] The lease, hold and strict-session fail-opens are closed, two false comments repaired, and the heartbeat lockout made visible — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-5.json)
  - [sfg-6] The last store read that could switch the write guard off now denies natively, and the module header describes the guard that exists — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-6.json)

area workflow-state:
  - [sfg-1] An unbound session holding one live claim is judged against its claimed feature's lane record at both the write check and the git intake gate; the intake refusal names the session binding as the remedy — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-1.json)
  - [sfg-2] Herding briefs teach a dissent object, the parser reads it leniently, and the run verb transcribes it through record_dissent with the outcome stamped on the envelope — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-2.json)
  - [sfg-3] The claim readers are infallible so no malformed claim can fail the write guard open, and the ownership guard's claim-derived trigger set is stated honestly and pinned both ways — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-3.json)
  - [sfg-4] The heartbeat, control-root and product-root readers are infallible, so no unparseable store or config byte can switch the write guard off through them — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-4.json)
  - [sfg-5] The lease, hold and strict-session fail-opens are closed, two false comments repaired, and the heartbeat lockout made visible — feature-wide sync per the scribing stamp, 8 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-5.json)
  - [sfg-6] The last store read that could switch the write guard off now denies natively, and the module header describes the guard that exists — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/archive/slp-followup-gaps/sfg-6.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell sfg-1 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-1-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-1 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-1's capped trace: Grouped the new source claim with default at the msn-21 workspace-ownership check instead of letting it skip that guard — the claim arm derives WHICH LANE a se…"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-1-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-1.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-1 — pitfall candidate

## What the cell did

An unbound session holding one live claim is judged against its claimed feature's lane record at both the write check and the git intake gate; the intake refusal names the session binding as the remedy

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-1.json)

- **deviation** — Grouped the new source claim with default at the msn-21 workspace-ownership check instead of letting it skip that guard — the claim arm derives WHICH LANE a session works under and says nothing about who owns the checkout — found a better route
- **deviation** — The judge found two hunt defects after the cap: the claim readers could fail the guard open on a malformed timestamp, and the ownership comment asserted an invariance the code no longer held. Both were fixed in cell sfg-3 and re-judged PASS — something else had to be fixed first
- **failure_signature** — A malformed claimed_at on the session's own claim makes the guard fail OPEN (Nd -> emit_undecidable) instead of reading the claim as active, and the claim-derived phase silently shifts which sessions the isolated-workspace ownership deny fires for — both unpinned, both contradicted by their own comments.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sfg-2 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-2-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-2 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-2's capped trace: verbs/cells/dissent.rs was listed in the cell but needed no edit — record_dissent is already pub(crate) and re-exported through verbs::cells, so herding/run.rs…"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-2-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-2.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-2 — pitfall candidate

## What the cell did

Herding briefs teach a dissent object, the parser reads it leniently, and the run verb transcribes it through record_dissent with the outcome stamped on the envelope

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-2.json)

- **deviation** — verbs/cells/dissent.rs was listed in the cell but needed no edit — record_dissent is already pub(crate) and re-exported through verbs::cells, so herding/run.rs calls it unchanged — the plan was wrong about a fact
- **deviation** — The retargeted negative pin bans any `bee <word>` command by scanning, instead of the literal `bee cells` plus a bare `bee ` substring ban — the brief's own prose already says "Ignore any bee or agent-workflow instructions", so a bare `bee ` ban would fail on text that names no command — hit an unforeseen obstacle
- **deviation** — sync-ack: skills/bee-herding documents the mailbox FILE LAYOUT (job.json, result-N.json, ack-N.json), not the result JSON field list — the layout is byte-unchanged here, and the StopAndAsk pair (options/leaning) added the same kind of field with no skill edit either. The cell scopes three source files and declares affects_skills: [].

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sfg-3 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-3-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-3 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-3's capped trace: followed the plan"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-3-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-3.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-3 — pitfall candidate

## What the cell did

The claim readers are infallible so no malformed claim can fail the write guard open, and the ownership guard's claim-derived trigger set is stated honestly and pinned both ways

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-3.json)

- **deviation** — followed the plan

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sfg-4 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-4-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-4 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-4's capped trace: The judge found the sweep incomplete: lease and hold timestamp readers and the read_session_strict path carried the same escape, and two hook_local.rs comments…"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-4-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-4.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-4 — pitfall candidate

## What the cell did

The heartbeat, control-root and product-root readers are infallible, so no unparseable store or config byte can switch the write guard off through them

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-4.json)

- **deviation** — The judge found the sweep incomplete: lease and hold timestamp readers and the read_session_strict path carried the same escape, and two hook_local.rs comments were left describing the old behavior. All were closed in cell sfg-5 — something else had to be fixed first
- **failure_signature** — The same store-data fail-open is still live on two paths the sweep missed: malformed lease/hold timestamps escape check_write via find_session_conflicts/find_conflicts/find_foreign_holds/hold_expiry to emit_undecidable, and the deliberately-left read_session_strict escape does fail open on a real Edit/Write because main.rs:427 runs is_shared_nested_checkout_target for every resolvable target, not only for Bash.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sfg-5 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-5-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-5 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-5's capped trace: Reserved and edited two files the cell did not list, jspath.rs and mod.rs — the warning queue the lockout line needs lives in jspath.rs and had no push helper,…"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-5-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-5.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-5 — pitfall candidate

## What the cell did

The lease, hold and strict-session fail-opens are closed, two false comments repaired, and the heartbeat lockout made visible

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-5.json)

- **deviation** — Reserved and edited two files the cell did not list, jspath.rs and mod.rs — the warning queue the lockout line needs lives in jspath.rs and had no push helper, and mod.rs still listed the strict session read as delegated — something else had to be fixed first
- **deviation** — The judge found one surviving escape on the companion marker and a header bullet narrowed too far; both were closed in cell sfg-6 — something else had to be fixed first
- **failure_signature** — A corrupt .bee/companion-session.json still switches the whole write guard off: hook_local.rs:1015 map_err(|_| Nd)? -> hook_local.rs:1149 -> main.rs:438 -> emit_undecidable -> exit 0, on any Edit/Write with a resolvable target once one other session is live — the same class and the same function sfg-5 closed at hook_local.rs:1140 — and mod.rs:40-41's rewritten delegated-branch bullet now excludes it.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell sfg-6 — save as docs/knowledge/patterns/slp-followup-gaps-sfg-6-pitfall.md

---
type: bee.pattern
title: slp-followup-gaps cell sfg-6 — pitfall candidate
description: "Pitfall candidate mined from cell sfg-6's capped trace: Kept resolve_verified_companion_mount_real as a thin wrapper over a new three-answer resolve_companion_mount instead of changing its signature — crate::nested_…"
timestamp: 2026-08-29
bee:
  id: slp-followup-gaps-sfg-6-pitfall
  lifecycle: draft
  areas: [hook-runtime, bee-herding, workflow-state]
  sources: [.bee/cells/archive/slp-followup-gaps/sfg-6.json]
  polarity: pitfall
---

# slp-followup-gaps cell sfg-6 — pitfall candidate

## What the cell did

The last store read that could switch the write guard off now denies natively, and the module header describes the guard that exists

## Recorded evidence (verbatim from .bee/cells/archive/slp-followup-gaps/sfg-6.json)

- **deviation** — Kept resolve_verified_companion_mount_real as a thin wrapper over a new three-answer resolve_companion_mount instead of changing its signature — crate::nested_checkout maps its Err onto a native fail-closed refusal that had to be preserved — found a better route
- **deviation** — Removed the headers timestamp-strings delegated bullet and tightened the every-reader-is-infallible sentence to exempt guards.memory_root — the header sweep the cell asked for found both — something else had to be fixed first
- **deviation** — The judge found the rewritten delegated-branch list still claimed to be exhaustive while omitting two shape delegates; both were added in commit 85ead065 and the verdict re-recorded PASS — something else had to be fixed first
- **failure_signature** — mod.rs:21 claims the DELEGATED BRANCHES list at mod.rs:61-82 is exhaustive, but two live delegates are missing from it: checks.rs:718 (bash tokenizer depth truncation, reached via main.rs:534) and detectors.rs:205 (non-ASCII AskUserQuestion header, reached via main.rs:105). Documentation-only; the defect class under sweep is fully closed.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 6 capped cell(s) mined, 1 delivery draft, 18 area bullet(s), 6 pattern candidate(s), 0 file(s) written.