promote proposal for work item "herding-orchestration" (docs/history/herding-orchestration/CONTEXT.md + docs/history/herding-orchestration/plan.md) — 17 capped cell(s): ho-1, ho-2, ho-3, ho-4, ho-5, ho-6, ho-7, ho-8, ho-9, ho-10, ho-11, ho-12, ho-13, ho-14, ho-15, ho-16, ho-17
anchor: history — docs/history/herding-orchestration/CONTEXT.md, docs/history/herding-orchestration/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/herding-orchestration/delivery.md

---
type: bee.delivery
title: herding-orchestration — delivery
description: "Delivery record proposed by bee knowledge promote for work item herding-orchestration: 17 capped cell(s), 0 recorded deviation(s)."
timestamp: 2026-08-19
bee:
  id: herding-orchestration-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/herding-orchestration/CONTEXT.md, docs/history/herding-orchestration/plan.md]
  sources: [docs/history/herding-orchestration/CONTEXT.md, docs/history/herding-orchestration/plan.md, .bee/cells/ho-1.json, .bee/cells/ho-2.json, .bee/cells/ho-3.json, .bee/cells/ho-4.json, .bee/cells/ho-5.json, .bee/cells/ho-6.json, .bee/cells/ho-7.json, .bee/cells/ho-8.json, .bee/cells/ho-9.json, .bee/cells/ho-10.json, .bee/cells/ho-11.json, .bee/cells/ho-12.json, .bee/cells/ho-13.json, .bee/cells/ho-14.json, .bee/cells/ho-15.json, .bee/cells/ho-16.json, .bee/cells/ho-17.json]
---

# herding-orchestration — Delivery

## What shipped

- **ho-1** — Repaired the dispatch spawn line for herdr 0.8.0: section 8 is split-then-start with --kind/--pane, carrying the stray-pane cleanup and shell-readiness settle rules, and operational-invariants maps agent_command token 0 to --kind per D14 (3 file(s) changed)
- **ho-2** — Re-recorded spawn-proof.md from a live herdr 0.8.0 start/prompt/read round trip and fixed role-dispatch.md's three stale spots (proof pointer, ratio sentence, timeout value); regen chain green (3 file(s) changed)
- **ho-3** — Corrected the bee-herding Pointers section to name the live Rust implementation (herding.rs/router.rs/catalog.rs) and the real skill script paths, replacing five dead Node-file citations (1 file(s) changed)
- **ho-4** — Create the fleet coordination crate and prove its boundary against bee (6 file(s) changed)
- **ho-5** — Defined fleet's Wave value, five-state WorkerStatus model, synchronous WorkerBackend trait, Baseline/CompletionSignal types, and a fault-injecting FakeBackend test seam; closed the manifest boundary test's path-alias gap. (5 file(s) changed)
- **ho-6** — Implemented the five-phase choreography as fleet::choreography::run_wave over the WorkerBackend trait, with concurrent std::thread waiting and named failure buckets, and pinned all eight ordering invariants by tests proven against deliberate mutation (5 file(s) changed)
- **ho-7** — Documented the two proxy limits the ho-6 judge left open: invariant 5's ordering premise is not itself pinned, and invariant 8 holds only for identical name strings per D15 (1 file(s) changed)
- **ho-8** — Implemented the herdr backend behind the worker-backend trait: fail-closed status mapping, canonical identity per D15, the three recorded hazards, and caller-supplied kind and arguments per D14 — every behaviour pinned by pure tests that run on Windows (7 file(s) changed)
- **ho-9** — Folded the wave ledger by wave_id at read time and made occupancy cross unresolved pane ids against an injectable live pane list, with the one-hour timer as a tagged fallback only (1 file(s) changed)
- **ho-10** — Added bee herding wave and bee herding occupancy: the config-driven entry point that resolves herding.agent_command per D14 and appends one ledger row, and the CLI bridge that exposes occupancy with its live-versus-fallback distinction intact (5 file(s) changed)
- **ho-11** — Section 4's occupancy count now reads bee herding occupancy's live/fallback-tagged JSON instead of enumerating panes; fallback answers refuse dispatch for the iteration; anomaly scan and section 8 untouched (7 file(s) changed)
- **ho-12** — Closed the ledger loop: bee herding record-worker writes a row on the dispatch path per D18, section 8 calls it after each spawn, and section 4's fallback and command-failure branches are complete (5 file(s) changed)
- **ho-13** — Replaced control-loop.sh with a native bee herding control-loop verb: argv byte-identical to the bash default per D13, per-token config substitution, and a pid-based terminate-then-kill ceiling with no shell and no GNU timeout (4 file(s) changed)
- **ho-14** — Rewired bootstrap-cockpit.sh to invoke bee herding control-loop via the vendored binary, updated all comment/doc references, and deleted the retired shell script across canonical tree and all five mirrors. (6 file(s) changed)
- **ho-15** — Moved all nine live control-loop.sh references in skills/bee-herding/ to bee herding control-loop, brought openai.yaml back into parity with SKILL.md by hand, and recounted overview.md's Pointers verb count to nine from the router. (5 file(s) changed)
- **ho-16** — Rewrote the four-slot Edge Case from what the code now does, and resolved the trigger that guarded it (1 file(s) changed)
- **ho-17** — Derive an herdr-legal agent slug from the pane id (0 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **ho-1** — `.bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-2** — `.bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-3** — `.bee/bin/bee knowledge check`
- **ho-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-8** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-9** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-10** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-11** — `.bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-12** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-13** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
- **ho-14** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-15** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check`
- **ho-16** — `.bee/bin/bee knowledge check`
- **ho-17** — `PATH="$HOME/.cargo/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml`

## Deviations

None recorded in the capped cell traces.

## Provenance

Proposed by `bee knowledge promote --work herding-orchestration` from 17 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/herding-orchestration/CONTEXT.md`, `docs/history/herding-orchestration/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "herding-orchestration" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-08-19T10:55:09.337Z), the work item declares no bee.areas.

area bee-herding:
  - [ho-1] Repaired the dispatch spawn line for herdr 0.8.0: section 8 is split-then-start with --kind/--pane, carrying the stray-pane cleanup and shell-readiness settle rules, and operational-invariants maps agent_command token 0 to --kind per D14 — feature-wide sync per the scribing stamp, 3 file(s) changed (trace .bee/cells/ho-1.json)
  - [ho-6] Implemented the five-phase choreography as fleet::choreography::run_wave over the WorkerBackend trait, with concurrent std::thread waiting and named failure buckets, and pinned all eight ordering invariants by tests proven against deliberate mutation — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ho-6.json)
  - [ho-8] Implemented the herdr backend behind the worker-backend trait: fail-closed status mapping, canonical identity per D15, the three recorded hazards, and caller-supplied kind and arguments per D14 — every behaviour pinned by pure tests that run on Windows — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/ho-8.json)
  - [ho-9] Folded the wave ledger by wave_id at read time and made occupancy cross unresolved pane ids against an injectable live pane list, with the one-hour timer as a tagged fallback only — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/ho-9.json)
  - [ho-10] Added bee herding wave and bee herding occupancy: the config-driven entry point that resolves herding.agent_command per D14 and appends one ledger row, and the CLI bridge that exposes occupancy with its live-versus-fallback distinction intact — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ho-10.json)
  - [ho-11] Section 4's occupancy count now reads bee herding occupancy's live/fallback-tagged JSON instead of enumerating panes; fallback answers refuse dispatch for the iteration; anomaly scan and section 8 untouched — feature-wide sync per the scribing stamp, 7 file(s) changed (trace .bee/cells/ho-11.json)
  - [ho-12] Closed the ledger loop: bee herding record-worker writes a row on the dispatch path per D18, section 8 calls it after each spawn, and section 4's fallback and command-failure branches are complete — feature-wide sync per the scribing stamp, 5 file(s) changed (trace .bee/cells/ho-12.json)
  - [ho-13] Replaced control-loop.sh with a native bee herding control-loop verb: argv byte-identical to the bash default per D13, per-token config substitution, and a pid-based terminate-then-kill ceiling with no shell and no GNU timeout — feature-wide sync per the scribing stamp, 4 file(s) changed (trace .bee/cells/ho-13.json)
  - [ho-14] Rewired bootstrap-cockpit.sh to invoke bee herding control-loop via the vendored binary, updated all comment/doc references, and deleted the retired shell script across canonical tree and all five mirrors. — feature-wide sync per the scribing stamp, 6 file(s) changed (trace .bee/cells/ho-14.json)
  - [ho-17] Derive an herdr-legal agent slug from the pane id — feature-wide sync per the scribing stamp, 0 file(s) changed (trace .bee/cells/ho-17.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell ho-1 — save as docs/knowledge/patterns/herding-orchestration-ho-1-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-1 — pitfall candidate
description: "Pitfall candidate mined from cell ho-1's capped trace: T2: spawn-proof.md (all six copies) still carries agent start --cwd/--workspace/--tab invocations and a takeaway forbidding split-first, while role-dispatch.md…"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-1.json]
  polarity: pitfall
---

# herding-orchestration cell ho-1 — pitfall candidate

## What the cell did

Repaired the dispatch spawn line for herdr 0.8.0: section 8 is split-then-start with --kind/--pane, carrying the stray-pane cleanup and shell-readiness settle rules, and operational-invariants maps agent_command token 0 to --kind per D14

## Recorded evidence (verbatim from .bee/cells/ho-1.json)

- **failure_signature** — T2: spawn-proof.md (all six copies) still carries agent start --cwd/--workspace/--tab invocations and a takeaway forbidding split-first, while role-dispatch.md:269 already claims it is re-recorded — ho-2 is still open, and spawn-proof.md was never in ho-1's declared files, so this check was mis-scoped by planning. X1: the rewritten section 8 uses §2 at lines 296 and 334 to mean step 2 of section 8, colliding with the document's own §N-means-section convention used four lines away, so a cold agent can resolve the cleanup and confirm references to section 2.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-6 — save as docs/knowledge/patterns/herding-orchestration-ho-6-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-6 — pitfall candidate
description: "Pitfall candidate mined from cell ho-6's capped trace: invariant-5-survives-its-own-mutation: adding sent.clear() to the send-Err arm at choreography.rs:238 abandons already-dispatched targets and all nine choreogr…"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-6-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-6.json]
  polarity: pitfall
---

# herding-orchestration cell ho-6 — pitfall candidate

## What the cell did

Implemented the five-phase choreography as fleet::choreography::run_wave over the WorkerBackend trait, with concurrent std::thread waiting and named failure buckets, and pinned all eight ordering invariants by tests proven against deliberate mutation

## Recorded evidence (verbatim from .bee/cells/ho-6.json)

- **failure_signature** — invariant-5-survives-its-own-mutation: adding sent.clear() to the send-Err arm at choreography.rs:238 abandons already-dispatched targets and all nine choreography tests stay green, because tests/choreography.rs:205 puts the failing send first. Plus invariant-8 pinned only on same-string duplicates while CONTEXT.md:87 and plan.md:242 state name-versus-pane-id, with the deferral recorded only in a private rustdoc and publicly contradicted at wave.rs:13-16.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-8 — save as docs/knowledge/patterns/herding-orchestration-ho-8-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-8 — pitfall candidate
description: "Pitfall candidate mined from cell ho-8's capped trace: The herdr backend's behaviour is entirely unproven on Windows: 17 of 57 fleet tests sit behind a file-level cfg(unix), and 7 of 7 backend mutations SURVIVE a r…"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-8-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-8.json]
  polarity: pitfall
---

# herding-orchestration cell ho-8 — pitfall candidate

## What the cell did

Implemented the herdr backend behind the worker-backend trait: fail-closed status mapping, canonical identity per D15, the three recorded hazards, and caller-supplied kind and arguments per D14 — every behaviour pinned by pure tests that run on Windows

## Recorded evidence (verbatim from .bee/cells/ho-8.json)

- **failure_signature** — The herdr backend's behaviour is entirely unproven on Windows: 17 of 57 fleet tests sit behind a file-level cfg(unix), and 7 of 7 backend mutations SURVIVE a run restricted to the tests Windows compiles, against D4's required-outcome and the Windows workflow's explicit no-skip-list policy. Separately, HerdrBackend::start hardcodes --kind claude and a new test pins that constant green, contradicting D14 and the very D17 boundary the cell claims to defer.
- **failure_signature** — The unix gate still hides one real behaviour and two unproven wiring arms: canonical_id's D15 name-to-pane-id resolution has no pure function and no lib test, so collapsing it survives the Windows-compilable set and dies only in the gated file; the Refuse-to-error arm likewise survives that set; and replacing the constructed agent_args with an empty slice survives all 71 fleet tests, so nothing proves the constructed arguments reach the argv. Separately, the constructor's doc omits the D14 obligation entirely, deferring to a private field doc that rustdoc never renders on the struct page.
- **failure_signature** — Windows-set survivors on the production path: the production constructor can discard its caller-resolved kind and arguments with both the lib set and the full suite staying green, because nothing calls it anywhere; the start timeout constant and read_output's transcript argv are pinned by no test on any platform, and the latter is the documented mitigation for the alternate-screen hazard; and the pane-id early return plus the spill-path join are proven only by the cfg(unix) file, which also falsifies the module header's claim that those tests cover nothing the pure tests do not.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-9 — save as docs/knowledge/patterns/herding-orchestration-ho-9-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-9 — pitfall candidate
description: "Pitfall candidate mined from cell ho-9's capped trace: The wave ledger's one-hour timer combined with its total absence of a close path makes live_worker_count wrong in both directions — measured: a 90-minute live …"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-9-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-9.json]
  polarity: pitfall
---

# herding-orchestration cell ho-9 — pitfall candidate

## What the cell did

Folded the wave ledger by wave_id at read time and made occupancy cross unresolved pane ids against an injectable live pane list, with the one-hour timer as a tagged fallback only

## Recorded evidence (verbatim from .bee/cells/ho-9.json)

- **failure_signature** — The wave ledger's one-hour timer combined with its total absence of a close path makes live_worker_count wrong in both directions — measured: a 90-minute live worker reads 0, a finished worker reads 1 — so the occupancy answer D10 exists for is not usable by role-dispatch section 4, and would be worse than the pane count it was meant to replace.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-10 — save as docs/knowledge/patterns/herding-orchestration-ho-10-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-10 — pitfall candidate
description: "Pitfall candidate mined from cell ho-10's capped trace: The wave verb's constructor wiring is untested in the bee crate: mutating its single call site to discard the D14-resolved kind and arguments leaves the ENTIRE…"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-10-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-10.json]
  polarity: pitfall
---

# herding-orchestration cell ho-10 — pitfall candidate

## What the cell did

Added bee herding wave and bee herding occupancy: the config-driven entry point that resolves herding.agent_command per D14 and appends one ledger row, and the CLI bridge that exposes occupancy with its live-versus-fallback distinction intact

## Recorded evidence (verbatim from .bee/cells/ho-10.json)

- **failure_signature** — The wave verb's constructor wiring is untested in the bee crate: mutating its single call site to discard the D14-resolved kind and arguments leaves the ENTIRE workspace suite green, and gutting the constructor itself is caught only by the fleet crate's own test while the bee crate stays at 1901 passed. The entry point is a compile-time caller, not a proven one — the fourth instance in this feature of a join between two separately-tested things being itself untested.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-12 — save as docs/knowledge/patterns/herding-orchestration-ho-12-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-12 — pitfall candidate
description: "Pitfall candidate mined from cell ho-12's capped trace: The recording verb's argv-to-row wiring is untested: swapping the pane-id and path arguments inside the CLI wrapper passes the entire suite at 1908 passed, sil…"
timestamp: 2026-08-18
bee:
  id: herding-orchestration-ho-12-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-12.json]
  polarity: pitfall
---

# herding-orchestration cell ho-12 — pitfall candidate

## What the cell did

Closed the ledger loop: bee herding record-worker writes a row on the dispatch path per D18, section 8 calls it after each spawn, and section 4's fallback and command-failure branches are complete

## Recorded evidence (verbatim from .bee/cells/ho-12.json)

- **failure_signature** — The recording verb's argv-to-row wiring is untested: swapping the pane-id and path arguments inside the CLI wrapper passes the entire suite at 1908 passed, silently reproducing the inert-ledger failure the cell exists to remove. The crossing test exercises only the pure append-and-count pair and never enters through the verb with real flags, so it crosses a boundary production does not.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-13 — save as docs/knowledge/patterns/herding-orchestration-ho-13-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-13 — pitfall candidate
description: "Pitfall candidate mined from cell ho-13's capped trace: The argv builder and the spawn are each tested, but the JOIN between them is not: replacing the spawn's argument with a hardcoded vector leaves the whole works…"
timestamp: 2026-08-19
bee:
  id: herding-orchestration-ho-13-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-13.json]
  polarity: pitfall
---

# herding-orchestration cell ho-13 — pitfall candidate

## What the cell did

Replaced control-loop.sh with a native bee herding control-loop verb: argv byte-identical to the bash default per D13, per-token config substitution, and a pid-based terminate-then-kill ceiling with no shell and no GNU timeout

## Recorded evidence (verbatim from .bee/cells/ho-13.json)

- **failure_signature** — The argv builder and the spawn are each tested, but the JOIN between them is not: replacing the spawn's argument with a hardcoded vector leaves the whole workspace suite green at 1932 passed. Every test spawner discards argv — one declares the parameter underscore-prefixed and builds its own command, the other records into a field written once and read nowhere, despite that field's doc comment claiming it exists to assert on the full value crossing into the spawner. Ninth instance of this feature's recurring untested-crossing shape, on the D13 path that holds the cell at standard lane.

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell ho-17 — save as docs/knowledge/patterns/herding-orchestration-ho-17-pitfall.md

---
type: bee.pattern
title: herding-orchestration cell ho-17 — pitfall candidate
description: "Pitfall candidate mined from cell ho-17's capped trace: sanitize_agent_slug mutation survival: removing the leading-letter insert ('slug.insert(0, 'a')' block) and removing 'slug.truncate(32);' each leave the entire…"
timestamp: 2026-08-19
bee:
  id: herding-orchestration-ho-17-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/ho-17.json]
  polarity: pitfall
---

# herding-orchestration cell ho-17 — pitfall candidate

## What the cell did

Derive an herdr-legal agent slug from the pane id

## Recorded evidence (verbatim from .bee/cells/ho-17.json)

- **failure_signature** — sanitize_agent_slug mutation survival: removing the leading-letter insert ('slug.insert(0, 'a')' block) and removing 'slug.truncate(32);' each leave the entire fleet suite green (53+11+17+6 tests ok) — the sole rule-test input w4:pG never exercises those two clauses

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 17 capped cell(s) mined, 1 delivery draft, 10 area bullet(s), 8 pattern candidate(s), 0 file(s) written.