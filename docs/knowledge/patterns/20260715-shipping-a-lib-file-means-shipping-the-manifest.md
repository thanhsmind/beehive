---
type: bee.pattern
title: "Shipping a lib file means shipping the manifest: regen release-manifest inside the feature"
description: "Shipping a lib file means shipping the manifest: regen release-manifest inside the feature"
tags: [process, release-manifest, verify-chain, lib-files]
timestamp: 2026-07-23
bee:
  id: pattern-20260715-shipping-a-lib-file-means-shipping-the-manifest
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT37", "original feature: parallel-scheduler", "okf-switchover-f3 close-out (fourth recurrence — the managed-hash ledger layer; trace in `.bee/cells/f3-5.json`, 2026-07-22)", "compaction-hardening D24 (fifth recurrence — the trigger widens to skills/** and hooks/**; trace in `.bee/cells/cz-4.json`, 2026-07-23)"]
  polarity: pitfall
  critical: true
---

# Shipping a lib file means shipping the manifest: regen release-manifest inside the feature

**Any cell that adds/renames/changes a file the manifest hashes carries the regen in ITS OWN
work and both `--check`s in ITS OWN verify.** Not the slice's last cell, not the close step —
the cell that moves the guard owns it, because every cell between it and the close otherwise
caps green against a red shared baseline. **Do not memorize which roots are hashed, and do not
trust any list written here or anywhere else — `cells add` derives them from the scripts at
every write and refuses the cell if the obligation is missing** (see MECHANIZED below). Every
count ever recited for this trigger was wrong, including the ones this doc used to carry; the
current derivation reports twelve manifest roots and two ledger roots, and it will report a
different number the day a script grows one. `ledger_parity` covers strictly less than the
manifest does, so it must never stand in for it. Order matters:
`render_plugin_skill_trees.mjs` → `onboard --apply` → `release_manifest.mjs --write`, because
`onboard --apply` never renders the plugin trees and a manifest written first freezes stale
trees as authoritative with nothing going red. Same rule generalized: before capping a cell,
ask which standing repo-wide guards hash the files you touched, run their regen inside that
cell, and put their `--check` in that cell's verify.

> **SUPERSEDED — this headline previously read:** *"the regen is part of the FEATURE, owned by
> its last cell or its close step"*, scoped to `templates/lib/` or `.bee/bin/lib/` (the Node-era vendored mirror, retired 2026-08-02; the single `.bee/bin/bee` binary replaces it). Both halves
> were wrong and both were followed faithfully. `compaction-hardening` revision 1 cited this
> pattern **by name** and routed the whole chain to its last cell — exactly what the headline
> said and exactly what recurrence #2 had already condemned three paragraphs below it. Revision 2
> then inherited the `lib/`-only trigger from the same sentence and put the check that stays
> green on the two cells that needed the one that moves. **A pattern doc whose summary is
> authored once and whose corrections are appended transmits its own obsolete advice with the
> authority of a critical pattern** — under compaction and under skim-reading, an agent loads
> the summary and the citation, not the tail. Rewrite the headline on every recurrence; when the
> fix direction changes, the old sentence is a defect, not history.

**MECHANIZED 2026-07-23 (regen-obligation-derived, ro-1) — the recurrence log ends here.**
`cells add`/`cells update` now REFUSE a cell whose own `files` touch a hashed root unless its
own `verify` carries that guard's `--check` (and, for the manifest, unless `files` lists the
manifest path). The roots are **derived from the scripts at every write**, never hard-coded —
and that instruction was the load-bearing one, not the refusal logic. Every number anyone had
recited for this trigger was read from a summary or a window rather than from the definition:
D20 said one root (`templates/lib/`), D24 corrected it to two (`skills/**`, `hooks/**`), the
P75 brief re-asserted two, the orchestrator widened it to six from `release_manifest.mjs:130-136`
— and the derivation measures **twelve**, because `DISTRIBUTION_TOOLS`/`DISTRIBUTION_TESTS` are
hashed at `:141-142`, outside that window. A guard built on the recited six would have shipped
blind to five hashed roots. The escape hatch is `regen_obligation_ack`, a reason string, so a
skip is an act with a name in the trace. A present-but-yielding-nothing guard script throws
rather than passing silently; a repo with no `scripts/` owes nothing.

**Recurred 2026-07-19 (cnt-3):** the cell regenerated rendered plugin trees, deferred the
manifest regen to "the slice-closing cell", and its own cell verify ran neither check — it
capped green while the shared baseline sat red for every concurrent session until a fix-first
cell (cnt-6) repaired it. Prose alone did not hold under a multi-session checkout;
mechanization (manifest check derived into the verify of any cell whose files hit rendered
trees or `lib/`) is now the recorded fix direction, not a nice-to-have.

**Recurred 2026-07-20 (msh-1) — THIRD instance, new layer:** the cell added a lib file and
regenerated the manifest, but the committed plugin skill trees were never re-rendered
(`render_plugin_skill_trees.mjs`); red surfaced only at the feature-close full chain and cost a
fix-first cell (msh-8). The derived-artifact set for a `templates/lib/` touch is now known to be
THREE-deep: `.bee/bin` mirror + rendered plugin trees + release manifest. Until the mechanization
lands (backlog), any lib-touching cell's verify carries all three regen/checks explicitly.

**Recurred 2026-07-22 (okf-switchover-f3 close-out) — FOURTH instance, and the derived set is now
FOUR deep.** The session rewired one citation inside `templates/lib/inject.mjs` and ended with the
repo RED on four suites: lib-mirror parity, release manifest, plugin distribution, and a
preamble-contract suite. The new layer beyond the known three is the **onboarding managed-hash
ledger** — `.bee/onboarding.json` records a hash per managed `.bee/bin/**` file, so re-syncing the
mirror is not enough; self-onboard must re-run or `ledger_parity --check` stays red. Full set for
one `templates/lib/` touch: **mirror copy → plugin tree render → manifest write → self-onboard**.

The deeper cause is not the count. The edit was made OUTSIDE a tracked cell, AFTER that cell's
cap-time verify had already gone green. **A cap-time verify is evidence about the tree as it stood
at that moment, and it expires the instant anything else is edited** — run the full chain again
before ending the session, with no exception for "it was just a citation" or "the cell already
passed". The next session opened on the red baseline and paid the repair.

**Recurred 2026-07-23 (compaction-hardening D24) — FIFTH instance, and the trigger itself was
scoped wrong.** The earlier recorded trigger was "any file under `templates/lib/` or `.bee/bin/lib/` (the Node-era vendored mirror, retired 2026-08-02; the single `.bee/bin/bee` binary replaces it)
changes" — but `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs` (formerly scripts/release_manifest.mjs) was measured to actually hash `skills/**` and
`hooks/**` in full, including the rendered plugin/skill projection trees themselves, not only the
shared lib directory. A cell touching only `hooks/**` moves `release_manifest --check` and does
**not** move `ledger_parity --check` (which covers only `packages/bee-rs/crates/bee/src/` (formerly .bee/bin/lib) and the shared helpers) — so
two cells in the same feature each carried exactly the check that stays green for a hooks-only
change and omitted the one that actually goes red. The ordering matters too: the plugin/skill tree
render must run **before** the manifest write, because the onboarding apply step never renders
those trees on its own — a manifest written first freezes stale trees as authoritative with nothing
going red to say so. Fix direction, generalized again: before scoping a regen trigger to "the lib
directory," check what the manifest script's own hash list actually covers — a scope written from
assumption can pass green while hiding the exact bug it exists to catch.

**Recurred 2026-07-23 (herding-dispatch-lock-toggle, cell hdlt-1) — SIXTH instance, first in a
`small`-lane feature.** A single `small`-lane cell added `packages/bee-rs/crates/bee/src/herding.rs` (formerly skills/bee-hive/templates/lib/herding.mjs)
and wired it into `bee.mjs`; its own targeted verify (`test_herding_cli.mjs`) went green and the
cell capped clean. The orchestrator's mandatory wave-close full-chain run (`packages/bee-rs/crates/bee/src/verbs/test_runner.rs` (formerly scripts/run_verify.mjs),
required even for a one-cell `small`-lane wave) then failed the same four-deep set this pattern
already names: `.bee/bin` mirror, both plugin skill-route trees, and the release manifest — fixed
with the same three commands (`onboard_bee.mjs --apply`, `render_plugin_skill_trees.mjs`,
`release_manifest.mjs --write`) and committed as a separate fixup before the wave could close.
No new derived layer surfaced this time; the four-deep set (mirror → plugin tree render → manifest
write → self-onboard) held. What's new: this is the first confirmed recurrence in a `tiny`/`small`
lane, where the single-cell dispatch and the merged Gate 2+3 fast path give even less natural
surface area for a mid-feature "did I regen everything" check than a multi-cell standard/high-risk
plan does — the wave-close full-chain run remains the only backstop that catches it, and it did.
