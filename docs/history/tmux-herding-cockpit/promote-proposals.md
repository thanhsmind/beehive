promote proposal for work item "tmux-herding-cockpit" (docs/history/tmux-herding-cockpit/CONTEXT.md + docs/history/tmux-herding-cockpit/plan.md) — 8 capped cell(s): thc-1, thc-2, thc-3, thc-4, thc-5, thc-6, thc-7, thc-8
anchor: history — docs/history/tmux-herding-cockpit/CONTEXT.md, docs/history/tmux-herding-cockpit/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/tmux-herding-cockpit/delivery.md

---
type: bee.delivery
title: tmux-herding-cockpit — delivery
description: "Delivery record proposed by bee knowledge promote for work item tmux-herding-cockpit: 8 capped cell(s), 33 recorded deviation(s)."
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/tmux-herding-cockpit/CONTEXT.md, docs/history/tmux-herding-cockpit/plan.md]
  sources: [docs/history/tmux-herding-cockpit/CONTEXT.md, docs/history/tmux-herding-cockpit/plan.md, .bee/cells/thc-1.json, .bee/cells/thc-2.json, .bee/cells/thc-3.json, .bee/cells/thc-4.json, .bee/cells/thc-5.json, .bee/cells/thc-6.json, .bee/cells/thc-7.json, .bee/cells/thc-8.json]
---

# tmux-herding-cockpit — Delivery

## What shipped

- **thc-1** — Occupancy lists live panes through tmux and the control allowlist swaps Bash(herdr:*) for Bash(tmux:*), both keyed by herding.transport (2 file(s) changed)
- **thc-2** — fleet gains TmuxBackend and the one shared screen classifier; bee's wave picks the backend by herding.transport (7 file(s) changed)
- **thc-3** — bee herding pane/agent-start/pane-id/result verbs run on both transports over a new CockpitTransport trait (4 file(s) changed)
- **thc-4** — bootstrap-cockpit.sh and role-bootstrap.md act on panes only through bee herding pane verbs (2 file(s) changed)
- **thc-5** — Dispatch, merge and wave role docs drive panes through the bee herding pane verbs; every parsed field follows the pane_verbs.rs envelope (6 file(s) changed)
- **thc-6** — The invariants reference, the config sample and the bee-herding knowledge bundle now state the tmux cockpit; regen re-rendered the skill trees, onboarding and the release manifest (14 file(s) changed)
- **thc-7** — pane layout rows carry x/y and pane list rows carry foreground_cwd, agent_status and agent_session on both transports; new pane list --with-status classifies tmux panes through the shared classifier (3 file(s) changed)
- **thc-8** — RealHerdr::tab_create now passes --no-focus so the fresh-tab fallback never steals the human's focus (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **thc-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **thc-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p fleet && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **thc-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::pane_verbs`
- **thc-4** — `bash -n skills/bee-herding/scripts/bootstrap-cockpit.sh && bash skills/bee-herding/scripts/bootstrap-cockpit.sh --dry-run --main-root . --workspace dry 2>&1 | grep -q 'bee herding pane' && ! bash skills/bee-herding/scripts/bootstrap-cockpit.sh --dry-run --main-root . --workspace dry 2>&1 | grep -q 'herdr '`
- **thc-5** — `test "$(rg -c 'herdr (pane|tab|agent|workspace) ' skills/bee-herding/references/role-dispatch.md skills/bee-herding/references/role-merge.md skills/bee-herding/references/wave-runs.md | awk -F: '{s+=$2} END {print s+0}')" = 0 && .bee/bin/bee knowledge check`
- **thc-6** — `.bee/bin/bee knowledge check && .bee/bin/bee knowledge index --check && .bee/bin/bee dev release-manifest --check`
- **thc-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::`
- **thc-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding::run`

## Deviations

- **thc-1** — Spelled the transport lookup super::transport_kind_at / super::TransportKind rather than crate::herding::… — the same items, matching the super:: idiom already used in both files
- **thc-1** — Amended one clause of the live_pane_ids_via_herdr DOC comment (it claimed to be the only process spawn in the file, which the new tmux lister made false); the function body stays byte-identical
- **thc-1** — A malformed herding.transport value now propagates as an Err out of resolve_iteration_argv (the typed refusal transport_kind defines) instead of silently arming the herdr allowlist
- **thc-1** — sync-ack: docs land in thc-6
- **thc-2** — TmuxSettings became a Deref newtype over fleet::screen::ScreenSettings, not a bare type alias: the alias forced from_config onto an extension trait, which broke the sibling worker in-flight files run.rs and pane_verbs.rs that call TmuxSettings::from_config. The newtype keeps their call sites byte-identical and keeps config parsing on a bee-owned type.
- **thc-2** — bee tmux.rs re-exports classify and Screen at its own path (pub(crate) use) so sibling use super::tmux::{classify, Screen} imports keep resolving after the move.
- **thc-2** — wrap_task_with_spill_instruction and recover_transcript are copied into fleet/src/backend/tmux.rs with a pointer to herdr.rs rather than widened to pub(crate) there: herdr.rs is outside this cell files list. The cell named copy-with-a-pointer as the allowed alternative.
- **thc-2** — sync-ack: docs land in thc-6
- **thc-3** — CockpitTransport carries a 7th method pane_context() -> {pane_id, tab_id?, workspace_id?}: the cell requires pane current to render tab/workspace and tab-create to fall back to the caller workspace, and PaneTransport::pane_current returns only a pane id
- **thc-3** — pane layout renders {panes:[{pane_id,width,height}]} instead of the cell listed {} — role-dispatch reads layout to pick split geometry, so an empty result would make the verb unusable for its only caller
- **thc-3** — tmux pane listing uses list-panes -s (one session) instead of -a plus a client-side filter: the pinned 5-column format carries no session column, so parse_pane_rows drops the unused session parameter
- **thc-3** — run.rs RealHerdr::call and read_main_config widened to pub(crate) alongside RealHerdr itself, so pane_verbs reuses the one herdr spawn-and-decode helper instead of a second copy
- **thc-3** — pane read accepts and ignores herdr flag --source, so one role line reads the same on both transports
- **thc-3** — the D3 blocked-preflight reads TmuxSettings from a module OnceLock set by cockpit_transport_for, because RealTmux keeps its settings private and tmux.rs is owned by another cell
- **thc-3** — sync-ack: docs land in thc-5/thc-6
- **thc-4** — tab-create answers with a root PANE id on both transports, so the closing line now names the runtime tab by its root pane instead of a tab_id; the chat pane branches on herding.transport because D3 fixes the tmux chat pane to the caller's own pane while herdr parity needs the cockpit tab's root pane
- **thc-4** — tab-create answers with a root pane id on both transports, so the closing line names the runtime tab by its root pane instead of a tab_id
- **thc-4** — the chat pane branches on herding.transport: D3 fixes the tmux chat pane to the caller own pane, while herdr parity needs the cockpit tab root pane
- **thc-5** — pane current renders pane_id/tab_id/workspace_id only (context_result in pane_verbs.rs) and carries no label key, so section 1 of role-dispatch.md and role-merge.md now names bee herding pane list as where the label is read; the self-name rule itself is unchanged
- **thc-5** — The cell's mandated phrase 'the pane workspace (herdr workspace or tmux session)' itself matches the cell verify regex, so all four places read '(a herdr workspace, or a tmux session)' and the verify counts 0
- **thc-5** — The never-let-the-verb-pick-its-own-anchor rule is kept in dispatch section 3 and merge section 2 but reworded: the bee verb has no --current form, so the hazard now reads 'always pass --pane' with the same reason and the same refusal
- **thc-5** — Prose that named herdr where both transports apply (herdr's live pane list, herdr closes the pane, the width herdr needs) now says the transport; the herdr 0.8.0 live-evidence citations in section 8 stay as recorded history
- **thc-5** — SKILL.md frontmatter now carries two dependency entries, herdr-cli and tmux-cli, both missing_effect degraded, each required only when herding.transport names it
- **thc-5** — sync-ack: dispatch-prompt.md and merge-prompt.md are named in the cell's own files list and its action text (the 'herdr workspace' rewording), but were left out of affects_skills when the cell was written — the touched set matches the assignment, the prediction field was short
- **thc-6** — Added the four new pane verbs and status to overview.md Pointers verb list (it said ten verbs; the router serves fifteen) — drift found while syncing, inside the cell files
- **thc-6** — Updated overview.md frontmatter description from herdr-driven to transport-neutral so the regenerated index stops naming one transport
- **thc-7** — Named the tmux layout format string as a PANE_GEOM_FORMAT const so the argv and the test that pins it cannot drift — the cell asked for the widened format, not for where it lives
- **thc-7** — Replaced tmux.rs stale not-wired-yet header with a truthful one instead of deleting it outright, because the #![allow(dead_code)] under it still needs a stated reason
- **thc-7** — extract_pane_layout defaults a missing rect x/y to 0 rather than dropping the row: the split-parent rule is pure over area and must not lose a spawn to an absent origin
- **thc-7** — bee is a binary crate so its tests are in-module #[cfg(test)]; the untested-lines advisory fires on path shape, but 12 tests were added or widened inside the three touched files
- **thc-7** — sync-ack: docs land in thc-5/thc-6
- **thc-8** — No fake could pin RealHerdr's argv (call() spawns herdr directly), so the argv was split into a pure tab_create_argv helper — the same split-out-for-test idiom as parse_herdr_body and extract_tab_create_root_pane in this file. Argv content is unchanged apart from the appended --no-focus.
- **thc-8** — sync-ack: docs land in thc-6

## Provenance

Proposed by `bee knowledge promote --work tmux-herding-cockpit` from 8 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/tmux-herding-cockpit/CONTEXT.md`, `docs/history/tmux-herding-cockpit/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "tmux-herding-cockpit" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-23T02:34:58.080Z), the work item declares no bee.areas.

area bee-herding:
  - [thc-1] Occupancy lists live panes through tmux and the control allowlist swaps Bash(herdr:*) for Bash(tmux:*), both keyed by herding.transport — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/thc-1.json)
  - [thc-2] fleet gains TmuxBackend and the one shared screen classifier; bee's wave picks the backend by herding.transport — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/thc-2.json)
  - [thc-3] bee herding pane/agent-start/pane-id/result verbs run on both transports over a new CockpitTransport trait — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/thc-3.json)
  - [thc-4] bootstrap-cockpit.sh and role-bootstrap.md act on panes only through bee herding pane verbs — feature-wide sync per the scribing stamp, 2 file(s) changed (trace .bee/cells/thc-4.json)
  - [thc-7] pane layout rows carry x/y and pane list rows carry foreground_cwd, agent_status and agent_session on both transports; new pane list --with-status classifies tmux panes through the shared classifier — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/thc-7.json)
  - [thc-8] RealHerdr::tab_create now passes --no-focus so the fresh-tab fallback never steals the human's focus — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/thc-8.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell thc-1 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-1-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-1 — pitfall candidate
description: "Pitfall candidate mined from cell thc-1's capped trace: Spelled the transport lookup super::transport_kind_at / super::TransportKind rather than crate::herding::… — the same items, matching the super:: idiom already…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-1.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-1 — pitfall candidate

## What the cell did

Occupancy lists live panes through tmux and the control allowlist swaps Bash(herdr:*) for Bash(tmux:*), both keyed by herding.transport

## Recorded evidence (verbatim from .bee/cells/thc-1.json)

- **deviation** — Spelled the transport lookup super::transport_kind_at / super::TransportKind rather than crate::herding::… — the same items, matching the super:: idiom already used in both files
- **deviation** — Amended one clause of the live_pane_ids_via_herdr DOC comment (it claimed to be the only process spawn in the file, which the new tmux lister made false); the function body stays byte-identical
- **deviation** — A malformed herding.transport value now propagates as an Err out of resolve_iteration_argv (the typed refusal transport_kind defines) instead of silently arming the herdr allowlist
- **deviation** — sync-ack: docs land in thc-6

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-2 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-2-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-2 — pitfall candidate
description: "Pitfall candidate mined from cell thc-2's capped trace: TmuxSettings became a Deref newtype over fleet::screen::ScreenSettings, not a bare type alias: the alias forced from_config onto an extension trait, which brok…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-2.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-2 — pitfall candidate

## What the cell did

fleet gains TmuxBackend and the one shared screen classifier; bee's wave picks the backend by herding.transport

## Recorded evidence (verbatim from .bee/cells/thc-2.json)

- **deviation** — TmuxSettings became a Deref newtype over fleet::screen::ScreenSettings, not a bare type alias: the alias forced from_config onto an extension trait, which broke the sibling worker in-flight files run.rs and pane_verbs.rs that call TmuxSettings::from_config. The newtype keeps their call sites byte-identical and keeps config parsing on a bee-owned type.
- **deviation** — bee tmux.rs re-exports classify and Screen at its own path (pub(crate) use) so sibling use super::tmux::{classify, Screen} imports keep resolving after the move.
- **deviation** — wrap_task_with_spill_instruction and recover_transcript are copied into fleet/src/backend/tmux.rs with a pointer to herdr.rs rather than widened to pub(crate) there: herdr.rs is outside this cell files list. The cell named copy-with-a-pointer as the allowed alternative.
- **deviation** — sync-ack: docs land in thc-6

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-3 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-3-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-3 — pitfall candidate
description: "Pitfall candidate mined from cell thc-3's capped trace: CockpitTransport carries a 7th method pane_context() -> {pane_id, tab_id?, workspace_id?}: the cell requires pane current to render tab/workspace and tab-creat…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-3-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-3.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-3 — pitfall candidate

## What the cell did

bee herding pane/agent-start/pane-id/result verbs run on both transports over a new CockpitTransport trait

## Recorded evidence (verbatim from .bee/cells/thc-3.json)

- **deviation** — CockpitTransport carries a 7th method pane_context() -> {pane_id, tab_id?, workspace_id?}: the cell requires pane current to render tab/workspace and tab-create to fall back to the caller workspace, and PaneTransport::pane_current returns only a pane id
- **deviation** — pane layout renders {panes:[{pane_id,width,height}]} instead of the cell listed {} — role-dispatch reads layout to pick split geometry, so an empty result would make the verb unusable for its only caller
- **deviation** — tmux pane listing uses list-panes -s (one session) instead of -a plus a client-side filter: the pinned 5-column format carries no session column, so parse_pane_rows drops the unused session parameter
- **deviation** — run.rs RealHerdr::call and read_main_config widened to pub(crate) alongside RealHerdr itself, so pane_verbs reuses the one herdr spawn-and-decode helper instead of a second copy
- **deviation** — pane read accepts and ignores herdr flag --source, so one role line reads the same on both transports
- **deviation** — the D3 blocked-preflight reads TmuxSettings from a module OnceLock set by cockpit_transport_for, because RealTmux keeps its settings private and tmux.rs is owned by another cell
- **deviation** — sync-ack: docs land in thc-5/thc-6

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-4 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-4-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-4 — pitfall candidate
description: "Pitfall candidate mined from cell thc-4's capped trace: tab-create answers with a root PANE id on both transports, so the closing line now names the runtime tab by its root pane instead of a tab_id; the chat pane br…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-4-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-4.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-4 — pitfall candidate

## What the cell did

bootstrap-cockpit.sh and role-bootstrap.md act on panes only through bee herding pane verbs

## Recorded evidence (verbatim from .bee/cells/thc-4.json)

- **deviation** — tab-create answers with a root PANE id on both transports, so the closing line now names the runtime tab by its root pane instead of a tab_id; the chat pane branches on herding.transport because D3 fixes the tmux chat pane to the caller's own pane while herdr parity needs the cockpit tab's root pane
- **deviation** — tab-create answers with a root pane id on both transports, so the closing line names the runtime tab by its root pane instead of a tab_id
- **deviation** — the chat pane branches on herding.transport: D3 fixes the tmux chat pane to the caller own pane, while herdr parity needs the cockpit tab root pane

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-5 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-5-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-5 — pitfall candidate
description: "Pitfall candidate mined from cell thc-5's capped trace: pane current renders pane_id/tab_id/workspace_id only (context_result in pane_verbs.rs) and carries no label key, so section 1 of role-dispatch.md and role-mer…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-5-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-5.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-5 — pitfall candidate

## What the cell did

Dispatch, merge and wave role docs drive panes through the bee herding pane verbs; every parsed field follows the pane_verbs.rs envelope

## Recorded evidence (verbatim from .bee/cells/thc-5.json)

- **deviation** — pane current renders pane_id/tab_id/workspace_id only (context_result in pane_verbs.rs) and carries no label key, so section 1 of role-dispatch.md and role-merge.md now names bee herding pane list as where the label is read; the self-name rule itself is unchanged
- **deviation** — The cell's mandated phrase 'the pane workspace (herdr workspace or tmux session)' itself matches the cell verify regex, so all four places read '(a herdr workspace, or a tmux session)' and the verify counts 0
- **deviation** — The never-let-the-verb-pick-its-own-anchor rule is kept in dispatch section 3 and merge section 2 but reworded: the bee verb has no --current form, so the hazard now reads 'always pass --pane' with the same reason and the same refusal
- **deviation** — Prose that named herdr where both transports apply (herdr's live pane list, herdr closes the pane, the width herdr needs) now says the transport; the herdr 0.8.0 live-evidence citations in section 8 stay as recorded history
- **deviation** — SKILL.md frontmatter now carries two dependency entries, herdr-cli and tmux-cli, both missing_effect degraded, each required only when herding.transport names it
- **deviation** — sync-ack: dispatch-prompt.md and merge-prompt.md are named in the cell's own files list and its action text (the 'herdr workspace' rewording), but were left out of affects_skills when the cell was written — the touched set matches the assignment, the prediction field was short

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-6 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-6-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-6 — pitfall candidate
description: "Pitfall candidate mined from cell thc-6's capped trace: Added the four new pane verbs and status to overview.md Pointers verb list (it said ten verbs; the router serves fifteen) — drift found while syncing, inside t…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-6-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-6.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-6 — pitfall candidate

## What the cell did

The invariants reference, the config sample and the bee-herding knowledge bundle now state the tmux cockpit; regen re-rendered the skill trees, onboarding and the release manifest

## Recorded evidence (verbatim from .bee/cells/thc-6.json)

- **deviation** — Added the four new pane verbs and status to overview.md Pointers verb list (it said ten verbs; the router serves fifteen) — drift found while syncing, inside the cell files
- **deviation** — Updated overview.md frontmatter description from herdr-driven to transport-neutral so the regenerated index stops naming one transport

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-7 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-7-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-7 — pitfall candidate
description: "Pitfall candidate mined from cell thc-7's capped trace: Named the tmux layout format string as a PANE_GEOM_FORMAT const so the argv and the test that pins it cannot drift — the cell asked for the widened format, not…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-7-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-7.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-7 — pitfall candidate

## What the cell did

pane layout rows carry x/y and pane list rows carry foreground_cwd, agent_status and agent_session on both transports; new pane list --with-status classifies tmux panes through the shared classifier

## Recorded evidence (verbatim from .bee/cells/thc-7.json)

- **deviation** — Named the tmux layout format string as a PANE_GEOM_FORMAT const so the argv and the test that pins it cannot drift — the cell asked for the widened format, not for where it lives
- **deviation** — Replaced tmux.rs stale not-wired-yet header with a truthful one instead of deleting it outright, because the #![allow(dead_code)] under it still needs a stated reason
- **deviation** — extract_pane_layout defaults a missing rect x/y to 0 rather than dropping the row: the split-parent rule is pure over area and must not lose a spawn to an absent origin
- **deviation** — bee is a binary crate so its tests are in-module #[cfg(test)]; the untested-lines advisory fires on path shape, but 12 tests were added or widened inside the three touched files
- **deviation** — sync-ack: docs land in thc-5/thc-6

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell thc-8 — save as docs/knowledge/patterns/tmux-herding-cockpit-thc-8-pitfall.md

---
type: bee.pattern
title: tmux-herding-cockpit cell thc-8 — pitfall candidate
description: "Pitfall candidate mined from cell thc-8's capped trace: No fake could pin RealHerdr's argv (call() spawns herdr directly), so the argv was split into a pure tab_create_argv helper — the same split-out-for-test idiom…"
timestamp: 2026-08-23
bee:
  id: tmux-herding-cockpit-thc-8-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/thc-8.json]
  polarity: pitfall
---

# tmux-herding-cockpit cell thc-8 — pitfall candidate

## What the cell did

RealHerdr::tab_create now passes --no-focus so the fresh-tab fallback never steals the human's focus

## Recorded evidence (verbatim from .bee/cells/thc-8.json)

- **deviation** — No fake could pin RealHerdr's argv (call() spawns herdr directly), so the argv was split into a pure tab_create_argv helper — the same split-out-for-test idiom as parse_herdr_body and extract_tab_create_root_pane in this file. Argv content is unchanged apart from the appended --no-focus.
- **deviation** — sync-ack: docs land in thc-6

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 8 capped cell(s) mined, 1 delivery draft, 6 area bullet(s), 8 pattern candidate(s), 0 file(s) written.