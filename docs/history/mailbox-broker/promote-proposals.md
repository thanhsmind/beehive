promote proposal for work item "mailbox-broker" (docs/history/mailbox-broker/CONTEXT.md + docs/history/mailbox-broker/plan.md) — 5 capped cell(s): mb-1, mb-2, mb-3, mb-4, mb-5
anchor: history — docs/history/mailbox-broker/CONTEXT.md, docs/history/mailbox-broker/plan.md
PROPOSAL ONLY — nothing was written. Applying any section below is a human or agent decision.

(a) DELIVERY DRAFT — save as docs/knowledge/work/mailbox-broker/delivery.md

---
type: bee.delivery
title: mailbox-broker — delivery
description: "Delivery record proposed by bee knowledge promote for work item mailbox-broker: 5 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-10-03
bee:
  id: mailbox-broker-delivery
  lifecycle: active
  areas: [bee-herding]
  required_context: [docs/history/mailbox-broker/CONTEXT.md, docs/history/mailbox-broker/plan.md]
  sources: [docs/history/mailbox-broker/CONTEXT.md, docs/history/mailbox-broker/plan.md, .bee/cells/mb-1.json, .bee/cells/mb-2.json, .bee/cells/mb-3.json, .bee/cells/mb-4.json, .bee/cells/mb-5.json]
---

# mailbox-broker — Delivery

## What shipped

- **mb-1** — Herded workers can end a round with a question on the pane and the no-pane path; envelope and job.json carry the broker facts (3 file(s) changed)
- **mb-2** — Pi verdict tool accepts a question and the drain waits for the final round (3 file(s) changed)
- **mb-3** — Broker tick and answer verbs route a worker question and start the next round; the herding broker group is declared (5 file(s) changed)
- **mb-4** — The broker runs as a code-only control-loop role (1 file(s) changed)
- **mb-5** — The mailbox broker concept, the question outcome and the herding skill's Broker role are documented (4 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **mb-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding` — covered by the full declared suite after rework d35e76976 (cargo test --release --no-fail-fast: 37 binaries ok, 0 failed, 3958 in the bin incl. new no-pane question and blocked tests)
- **mb-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts` — 114 pi contract tests incl. new verdict question and drain-skip tests
- **mb-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- herding catalog supervisor && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test registry_contracts` — covered by the full declared suite run after rework cf7aff65c (cargo test --release --no-fail-fast, 37 binaries ok, 0 failed, registry_dispatch included)
- **mb-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee control_loop` — 65 control_loop tests incl. broker parse, code-only argv and no prompt file
- **mb-5** — `rg -n "mailbox-broker D6" docs/knowledge/areas/bee-herding/the-mailbox-broker.md && rg -n "question" docs/knowledge/areas/bee-herding/the-run-verb-and-worker-outcomes.md && rg -n "the-mailbox-broker.md" docs/knowledge/areas/bee-herding/index.md` — the three pointer checks the cell names, plus the full suite green after the docs commit

## Deviations

- **mb-1** — round 2: execute_no_pane no longer overwrites a worker-written result-1.json, found by the live sandbox run 20261003-195611-398120; this also stops Pi no-pane blocked verdicts being reported as done
- **mb-1** — sync-ack: the bee-herding skill text landed with mb-5
- **mb-2** — sync-ack: the bee-herding skill text lands once with mb-5, after the broker exists
- **mb-3** — round 2 fixed the registry_dispatch red: herding.broker.tick became one herding.broker group entry, the pane precedent; the child round is a new job id with its own Pi inbox marker, so the parent marker stays pending (backlog finding filed); registry_payload.json was restored to one line in 3e229f963
- **mb-3** — sync-ack: the bee-herding skill text for the broker landed with mb-5 (85fcdf42f)
- **mb-4** — sync-ack: the bee-herding skill text for the broker role lands once with mb-5
- **mb-5** — skills/bee-herding/SKILL.md gained the Broker paragraph promised by the mb-1 to mb-4 sync-acks; the worker also edited tests/registry_dispatch.rs to special-case the broker namespace, which the leader reverted in df0e068ad
- **mb-5** — sync-ack: skills/bee-herding/SKILL.md is the owned-skill sync the mb-1 to mb-4 caps promised; it was added to the brief by expertise, not to affects_skills

## Provenance

Proposed by `bee knowledge promote --work mailbox-broker` from 5 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/mailbox-broker/CONTEXT.md`, `docs/history/mailbox-broker/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

(b) AREA UPDATES — candidate spec-sync bullets, each citing its cell

areas: from the scribing stamp for "mailbox-broker" — .bee/logs/scribing-runs.jsonl's most recent entry (2026-10-03T13:08:40.259Z), the work item declares no bee.areas.

area bee-herding:
  - [mb-4] The broker runs as a code-only control-loop role — feature-wide sync per the scribing stamp, 1 file(s) changed (trace .bee/cells/mb-4.json)

(c) PATTERN CANDIDATES — candidate bee.pattern concepts, bee.polarity pitfall

from cell mb-1 — save as docs/knowledge/patterns/mailbox-broker-mb-1-pitfall.md

---
type: bee.pattern
title: mailbox-broker cell mb-1 — pitfall candidate
description: "Pitfall candidate mined from cell mb-1's capped trace: round 2: execute_no_pane no longer overwrites a worker-written result-1.json, found by the live sandbox run 20261003-195611-398120; this also stops Pi no-pane …"
timestamp: 2026-10-03
bee:
  id: mailbox-broker-mb-1-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/mb-1.json]
  polarity: pitfall
---

# mailbox-broker cell mb-1 — pitfall candidate

## What the cell did

Herded workers can end a round with a question on the pane and the no-pane path; envelope and job.json carry the broker facts

## Recorded evidence (verbatim from .bee/cells/mb-1.json)

- **deviation** — round 2: execute_no_pane no longer overwrites a worker-written result-1.json, found by the live sandbox run 20261003-195611-398120; this also stops Pi no-pane blocked verdicts being reported as done
- **deviation** — sync-ack: the bee-herding skill text landed with mb-5

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mb-2 — save as docs/knowledge/patterns/mailbox-broker-mb-2-pitfall.md

---
type: bee.pattern
title: mailbox-broker cell mb-2 — pitfall candidate
description: "Pitfall candidate mined from cell mb-2's capped trace: sync-ack: the bee-herding skill text lands once with mb-5, after the broker exists"
timestamp: 2026-10-03
bee:
  id: mailbox-broker-mb-2-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/mb-2.json]
  polarity: pitfall
---

# mailbox-broker cell mb-2 — pitfall candidate

## What the cell did

Pi verdict tool accepts a question and the drain waits for the final round

## Recorded evidence (verbatim from .bee/cells/mb-2.json)

- **deviation** — sync-ack: the bee-herding skill text lands once with mb-5, after the broker exists

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mb-3 — save as docs/knowledge/patterns/mailbox-broker-mb-3-pitfall.md

---
type: bee.pattern
title: mailbox-broker cell mb-3 — pitfall candidate
description: "Pitfall candidate mined from cell mb-3's capped trace: round 2 fixed the registry_dispatch red: herding.broker.tick became one herding.broker group entry, the pane precedent; the child round is a new job id with it…"
timestamp: 2026-10-03
bee:
  id: mailbox-broker-mb-3-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/mb-3.json]
  polarity: pitfall
---

# mailbox-broker cell mb-3 — pitfall candidate

## What the cell did

Broker tick and answer verbs route a worker question and start the next round; the herding broker group is declared

## Recorded evidence (verbatim from .bee/cells/mb-3.json)

- **deviation** — round 2 fixed the registry_dispatch red: herding.broker.tick became one herding.broker group entry, the pane precedent; the child round is a new job id with its own Pi inbox marker, so the parent marker stays pending (backlog finding filed); registry_payload.json was restored to one line in 3e229f963
- **deviation** — sync-ack: the bee-herding skill text for the broker landed with mb-5 (85fcdf42f)

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mb-4 — save as docs/knowledge/patterns/mailbox-broker-mb-4-pitfall.md

---
type: bee.pattern
title: mailbox-broker cell mb-4 — pitfall candidate
description: "Pitfall candidate mined from cell mb-4's capped trace: sync-ack: the bee-herding skill text for the broker role lands once with mb-5"
timestamp: 2026-10-03
bee:
  id: mailbox-broker-mb-4-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/mb-4.json]
  polarity: pitfall
---

# mailbox-broker cell mb-4 — pitfall candidate

## What the cell did

The broker runs as a code-only control-loop role

## Recorded evidence (verbatim from .bee/cells/mb-4.json)

- **deviation** — sync-ack: the bee-herding skill text for the broker role lands once with mb-5

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

from cell mb-5 — save as docs/knowledge/patterns/mailbox-broker-mb-5-pitfall.md

---
type: bee.pattern
title: mailbox-broker cell mb-5 — pitfall candidate
description: "Pitfall candidate mined from cell mb-5's capped trace: skills/bee-herding/SKILL.md gained the Broker paragraph promised by the mb-1 to mb-4 sync-acks; the worker also edited tests/registry_dispatch.rs to special-ca…"
timestamp: 2026-10-03
bee:
  id: mailbox-broker-mb-5-pitfall
  lifecycle: draft
  areas: [bee-herding]
  sources: [.bee/cells/mb-5.json]
  polarity: pitfall
---

# mailbox-broker cell mb-5 — pitfall candidate

## What the cell did

The mailbox broker concept, the question outcome and the herding skill's Broker role are documented

## Recorded evidence (verbatim from .bee/cells/mb-5.json)

- **deviation** — skills/bee-herding/SKILL.md gained the Broker paragraph promised by the mb-1 to mb-4 sync-acks; the worker also edited tests/registry_dispatch.rs to special-case the broker namespace, which the leader reverted in df0e068ad
- **deviation** — sync-ack: skills/bee-herding/SKILL.md is the owned-skill sync the mb-1 to mb-4 caps promised; it was added to the brief by expertise, not to affects_skills

## Status

Candidate only. `bee knowledge promote` proposes; naming the pattern, generalizing it beyond this cell, and moving `bee.lifecycle` to `active` are a human or agent decision.

knowledge promote: 5 capped cell(s) mined, 1 delivery draft, 1 area bullet(s), 5 pattern candidate(s), 0 file(s) written.