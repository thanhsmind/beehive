promote proposal for work item "agent-activity-hook" (docs/history/agent-activity-hook/CONTEXT.md + docs/history/agent-activity-hook/plan.md) — 4 capped cell(s): aah-1, aah-2, aah-3, aah-4
anchor: history — docs/history/agent-activity-hook/CONTEXT.md, docs/history/agent-activity-hook/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/agent-activity-hook/delivery.md

---
type: bee.delivery
title: agent-activity-hook — delivery
description: "Delivery record proposed by bee knowledge promote for work item agent-activity-hook: 4 capped cell(s), 17 recorded deviation(s)."
timestamp: 2026-08-22
bee:
  id: agent-activity-hook-delivery
  lifecycle: active
  areas: [hook-runtime]
  required_context: [docs/history/agent-activity-hook/CONTEXT.md, docs/history/agent-activity-hook/plan.md]
  sources: [docs/history/agent-activity-hook/CONTEXT.md, docs/history/agent-activity-hook/plan.md, .bee/cells/archive/agent-activity-hook/aah-1.json, .bee/cells/archive/agent-activity-hook/aah-2.json, .bee/cells/archive/agent-activity-hook/aah-3.json, .bee/cells/archive/agent-activity-hook/aah-4.json]
---

# agent-activity-hook — Delivery

## What shipped

- **aah-1** — bee hook activity records per-session agent state (D1/D2/D3/D5) with sticky blocked/waiting_input and a hook-owned waiting mark (5 file(s) changed)
- **aah-2** — Activity hook declared in both Claude renderers on eight events, never SubagentStop; Codex projections byte-unchanged (6 file(s) changed)
- **aah-3** — session list and status worker rows now carry a derived live/no_signal answer, and the activity record is documented as a knowledge concept (7 file(s) changed)
- **aah-4** — The activity record now stamps the session's feature and held cell, fail-open on both (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **aah-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml activity hook_contracts`
- **aah-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml hook_manifests hooks_wiring && .bee/bin/bee dev render-hook-manifests --check && .bee/bin/bee dev release-manifest --check`
- **aah-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml session_signal sessions && .bee/bin/bee knowledge check`
- **aah-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml hooks::activity`

## Deviations

- **aah-1** — The cell's verify string chained two filters in one cargo invocation, which cargo rejects; ran cargo test ... hooks::activity and cargo test ... --test hook_contracts, then the wider `hook` filter, all green.
- **aah-1** — Widened `mod store` and `mod waiting_on` to pub(crate) in verbs/state_group/mod.rs (one word each, no behavior) so the hook can call the SAME waiting_on setters the CLI verb uses instead of becoming a second writer; file reserved under w-aah-1.
- **aah-1** — Blocked->blocked counts as a transition only when the NEW payload carries a tool_use_id that differs — an id-less Notification right after a PermissionRequest would otherwise append a duplicate row for one block.
- **aah-1** — sync-ack: The workflow-state touch is a two-word visibility widening (mod store / mod waiting_on -> pub(crate)) so the new hook calls the existing waiting_on setters instead of becoming a second writer — no verb, contract, or agent-facing behavior of that area changes, so none of its skills has anything to say. The new behavior lives in the hook-runtime area (docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md), which cells aah-2/aah-3 own.
- **aah-2** — cargo test refuses two filters in one call: ran hook_manifests and hooks_wiring as separate invocations, plus onboard/doctor/plugin_distribution for fallout
- **aah-2** — ran the freshly built packages/bee-rs/target/release/bee for render-hook-manifests and release-manifest: .bee/bin/bee symlinks to the main checkout binary, which carries the pre-change catalog
- **aah-2** — reserved and edited packages/bee-rs/crates/bee/src/onboard/tests.rs (not in cell files): two hook-count assertions went red purely from the new rows
- **aah-2** — reserved and committed docs/history/codex-harness-hardening/release-manifest.json (not in cell files): bee dev regen output, per the cell regen obligation
- **aah-2** — reverted .bee/onboarding.json — onboard/regen only rewrote its updated_at timestamp
- **aah-2** — affects_specs docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md needs no edit: it names no per-event list, only the module and ALLOWED_DIFFERENCES by name
- **aah-3** — the cell verify line `cargo test ... session_signal sessions` is malformed — cargo takes one positional TESTNAME and refused `sessions`; ran the same filters after `--` instead
- **aah-3** — the cell asked for the helper signature `(record, now: DateTime)`; this crate carries no DateTime in its session paths, so it takes `now_ms: f64` like every sibling staleness helper (date_parse_val/heartbeat_stale)
- **aah-3** — unit tests live in a new `#[cfg(test)] mod tests` inside sessions.rs (the workers.rs/ledger.rs sibling pattern) rather than state_group/tests.rs, which the cell did not name
- **aah-3** — `bee knowledge index` also regenerated docs/knowledge/areas/index.md (a concept count); reserved it under w-aah-3 before the write
- **aah-3** — capped with --sync-ack: affects_skills is empty and no owned skill describes session liveness
- **aah-3** — sync-ack: the cell declares affects_skills: [] — the change adds two read-only projection fields and one knowledge concept; no bee-planning/swarming/reviewing/capturing skill states how a session's liveness is read, so there is no skill sentence this makes stale
- **aah-4** — status_full::cells::is_claim_active is private to its module, so the same rule is re-stated locally in activity.rs (a hook has no Ctx anyway); the orchestrator note allowed a direct fail-open read

## Provenance

Proposed by `bee knowledge promote --work agent-activity-hook` from 4 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/agent-activity-hook/CONTEXT.md`, `docs/history/agent-activity-hook/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "agent-activity-hook" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-22T14:20:38.249Z), the work item declares no bee.areas.

area hook-runtime:
  - [aah-1] bee hook activity records per-session agent state (D1/D2/D3/D5) with sticky blocked/waiting_input and a hook-owned waiting mark — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/archive/agent-activity-hook/aah-1.json)
  - [aah-2] Activity hook declared in both Claude renderers on eight events, never SubagentStop; Codex projections byte-unchanged — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/archive/agent-activity-hook/aah-2.json)
  - [aah-3] session list and status worker rows now carry a derived live/no_signal answer, and the activity record is documented as a knowledge concept — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/archive/agent-activity-hook/aah-3.json)
  - [aah-4] The activity record now stamps the session's feature and held cell, fail-open on both — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/archive/agent-activity-hook/aah-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell aah-1 — save as docs/knowledge/patterns/agent-activity-hook-aah-1-pitfall.md

---
type: bee.pattern
title: agent-activity-hook cell aah-1 — pitfall candidate
description: "Pitfall candidate mined from cell aah-1's capped trace: The cell's verify string chained two filters in one cargo invocation, which cargo rejects; ran cargo test ... hooks::activity and cargo test ... --test hook_co…"
timestamp: 2026-08-22
bee:
  id: agent-activity-hook-aah-1-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/agent-activity-hook/aah-1.json]
  polarity: pitfall
---

# agent-activity-hook cell aah-1 — pitfall candidate

## What the cell did

bee hook activity records per-session agent state (D1/D2/D3/D5) with sticky blocked/waiting_input and a hook-owned waiting mark

## Recorded evidence (verbatim from .bee/cells/archive/agent-activity-hook/aah-1.json)

- **deviation** — The cell's verify string chained two filters in one cargo invocation, which cargo rejects; ran cargo test ... hooks::activity and cargo test ... --test hook_contracts, then the wider `hook` filter, all green.
- **deviation** — Widened `mod store` and `mod waiting_on` to pub(crate) in verbs/state_group/mod.rs (one word each, no behavior) so the hook can call the SAME waiting_on setters the CLI verb uses instead of becoming a second writer; file reserved under w-aah-1.
- **deviation** — Blocked->blocked counts as a transition only when the NEW payload carries a tool_use_id that differs — an id-less Notification right after a PermissionRequest would otherwise append a duplicate row for one block.
- **deviation** — sync-ack: The workflow-state touch is a two-word visibility widening (mod store / mod waiting_on -> pub(crate)) so the new hook calls the existing waiting_on setters instead of becoming a second writer — no verb, contract, or agent-facing behavior of that area changes, so none of its skills has anything to say. The new behavior lives in the hook-runtime area (docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md), which cells aah-2/aah-3 own.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aah-2 — save as docs/knowledge/patterns/agent-activity-hook-aah-2-pitfall.md

---
type: bee.pattern
title: agent-activity-hook cell aah-2 — pitfall candidate
description: "Pitfall candidate mined from cell aah-2's capped trace: cargo test refuses two filters in one call: ran hook_manifests and hooks_wiring as separate invocations, plus onboard/doctor/plugin_distribution for fallout"
timestamp: 2026-08-22
bee:
  id: agent-activity-hook-aah-2-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/agent-activity-hook/aah-2.json]
  polarity: pitfall
---

# agent-activity-hook cell aah-2 — pitfall candidate

## What the cell did

Activity hook declared in both Claude renderers on eight events, never SubagentStop; Codex projections byte-unchanged

## Recorded evidence (verbatim from .bee/cells/archive/agent-activity-hook/aah-2.json)

- **deviation** — cargo test refuses two filters in one call: ran hook_manifests and hooks_wiring as separate invocations, plus onboard/doctor/plugin_distribution for fallout
- **deviation** — ran the freshly built packages/bee-rs/target/release/bee for render-hook-manifests and release-manifest: .bee/bin/bee symlinks to the main checkout binary, which carries the pre-change catalog
- **deviation** — reserved and edited packages/bee-rs/crates/bee/src/onboard/tests.rs (not in cell files): two hook-count assertions went red purely from the new rows
- **deviation** — reserved and committed docs/history/codex-harness-hardening/release-manifest.json (not in cell files): bee dev regen output, per the cell regen obligation
- **deviation** — reverted .bee/onboarding.json — onboard/regen only rewrote its updated_at timestamp
- **deviation** — affects_specs docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md needs no edit: it names no per-event list, only the module and ALLOWED_DIFFERENCES by name

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aah-3 — save as docs/knowledge/patterns/agent-activity-hook-aah-3-pitfall.md

---
type: bee.pattern
title: agent-activity-hook cell aah-3 — pitfall candidate
description: "Pitfall candidate mined from cell aah-3's capped trace: the cell verify line `cargo test ... session_signal sessions` is malformed — cargo takes one positional TESTNAME and refused `sessions`; ran the same filters a…"
timestamp: 2026-08-22
bee:
  id: agent-activity-hook-aah-3-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/agent-activity-hook/aah-3.json]
  polarity: pitfall
---

# agent-activity-hook cell aah-3 — pitfall candidate

## What the cell did

session list and status worker rows now carry a derived live/no_signal answer, and the activity record is documented as a knowledge concept

## Recorded evidence (verbatim from .bee/cells/archive/agent-activity-hook/aah-3.json)

- **deviation** — the cell verify line `cargo test ... session_signal sessions` is malformed — cargo takes one positional TESTNAME and refused `sessions`; ran the same filters after `--` instead
- **deviation** — the cell asked for the helper signature `(record, now: DateTime)`; this crate carries no DateTime in its session paths, so it takes `now_ms: f64` like every sibling staleness helper (date_parse_val/heartbeat_stale)
- **deviation** — unit tests live in a new `#[cfg(test)] mod tests` inside sessions.rs (the workers.rs/ledger.rs sibling pattern) rather than state_group/tests.rs, which the cell did not name
- **deviation** — `bee knowledge index` also regenerated docs/knowledge/areas/index.md (a concept count); reserved it under w-aah-3 before the write
- **deviation** — capped with --sync-ack: affects_skills is empty and no owned skill describes session liveness
- **deviation** — sync-ack: the cell declares affects_skills: [] — the change adds two read-only projection fields and one knowledge concept; no bee-planning/swarming/reviewing/capturing skill states how a session's liveness is read, so there is no skill sentence this makes stale

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell aah-4 — save as docs/knowledge/patterns/agent-activity-hook-aah-4-pitfall.md

---
type: bee.pattern
title: agent-activity-hook cell aah-4 — pitfall candidate
description: "Pitfall candidate mined from cell aah-4's capped trace: status_full::cells::is_claim_active is private to its module, so the same rule is re-stated locally in activity.rs (a hook has no Ctx anyway); the orchestrator…"
timestamp: 2026-08-22
bee:
  id: agent-activity-hook-aah-4-pitfall
  lifecycle: draft
  areas: [hook-runtime]
  sources: [.bee/cells/archive/agent-activity-hook/aah-4.json]
  polarity: pitfall
---

# agent-activity-hook cell aah-4 — pitfall candidate

## What the cell did

The activity record now stamps the session's feature and held cell, fail-open on both

## Recorded evidence (verbatim from .bee/cells/archive/agent-activity-hook/aah-4.json)

- **deviation** — status_full::cells::is_claim_active is private to its module, so the same rule is re-stated locally in activity.rs (a hook has no Ctx anyway); the orchestrator note allowed a direct fail-open read

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 4 capped cell(s) mined, 1 delivery draft, 4 area bullet(s), 4 pattern candidate(s), 0 file(s) written.