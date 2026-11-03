# Research digest — contract_status + original_request surfaces in bee (SLP ticket 007)

- Date: 2026-08-26 · Tier: advisor (fable) — supersedes the same-day cheap-tier draft
- Context: `docs/discovery/slp-supervisor-lead-peer/tickets/007-contract-status-original-request.md`

## A. Per-contract CHỐT/CHƯA-CHỐT label — candidate surfaces

**1. Locked decisions in `docs/history/<feature>/CONTEXT.md`** — the settled/unsettled split already exists (Locked table = CHỐT; Agent's Discretion / Open items = CHƯA CHỐT), and the worker refusal reflex exists too (`packages/bee/prompts/worker-cell.md:21,38` — CONTEXT.md required reading, "Never reinterpret a locked decision"; `skills/bee-swarming/references/worker-details.md:262` — locked-decision conflict is an instant `[BLOCKED]`). Gap: per-feature prose, not per-contract; nothing enumerates interfaces; not machine-checkable; cross-feature contracts have no home.

**2. `docs/knowledge/` area records** — the only surface with a document-level `**Status:**` field today (e.g. `docs/knowledge/areas/worktree-parallelism/overview.md:24`), and learned-context lines already ride worker prompts mechanically (`prepare.rs` `learned_context_lines`). Gap: Status means doc-freshness, not contract settledness; area granularity; free prose.

**3. `bee decisions` — the VIEW candidate (strongest).** Records carry `tags[]`, `relation`, `supersedes[]`, `touches[]`, `feature`; `active_decisions()` excludes superseded ids (`verbs_read.rs:588`); query surface `bee decisions active --tag <t> --feature <f> --cell <id>` (`verbs_read.rs:82-105`); the deferral guard (`verbs_read.rs:432`) *forces* an unsettled decision to name a `--trigger`. CHỐT ≈ active decision tagged `contract:<name>` with no waiting trigger; CHƯA CHỐT ≈ decision whose trigger is waiting/due — derivable today with one tag convention and zero new storage; supersession keeps the label current for free. Gap: the absence problem — a never-logged contract reads the same as "no contract"; and nothing makes a worker query it before writing tests.

**4. Triggers** (`verbs/triggers/mod.rs:75-88`) — `{id, decision, condition, tier, status: waiting|due|resolved}`; IS the machine-readable "unsettled, revisit when X" marker, keyed to a decision id. Gap: no reverse index by contract name.

**5. Cell records** — `cell.decisions` (string array, `handlers_write.rs:348`) is exactly the spec's `contract_status_refs` slot, inlined verbatim into the worker prompt. Gap: no enforced semantics; nothing checks cited decisions are still active at claim time.

## B. Verbatim original_request — candidate surfaces

**1. `bee intent set/show` (`verbs/intent_group.rs`) — bee already has the spec's exact invariant.** Anchor schema (:182-205): `request` (required), `acceptance`, `next_action`, `do_not_reverse[]`, `stop_conditions[]`; stored per feature at `.bee/intent/<key>.json`; the compaction hook (`hooks/compaction.rs:71`) documents `--request "<the user's VERBATIM request>"` and the render header is literally `=== BEE INTENT ANCHOR — VERBATIM · DO NOT SUMMARIZE · DO NOT PARAPHRASE ===` (intent_group.rs:251). Gap: it survives *compaction*, not *dispatch* — `dispatch prepare` and `worker-cell.md` reference intent nowhere.

**2. Cell record** — whatever is ON the cell rides every dispatch verbatim, but the schema is closed (`handlers_write.rs:488` refuses unknown fields) and per-cell copies duplicate the same string N times.

**3. Dispatch prepare payload / worker prompt** — `prompt_body_for` (`drivers/prepare.rs:549`) renders `worker-cell.md` from cell JSON, learned context, feature, roots; adding one `{{#if original_request}}` template slot is mechanical. The single choke point every worker passes. Gather/reviewer/advisor templates need the same one-line slot.

**4. `docs/history/<feature>/CONTEXT.md` "Original ask"** — durable and already required reading, but a hand-copied prose convention; shaping summarizes by design, so the anti-drift guarantee cannot rest on it.

## Two cheapest workable shapes

**A — a derived view, no new registry.** Convention: every contract/interface decision is logged with tag `contract:<name>`; deferral wording already forces a trigger. Label = computed: active + no waiting trigger → CHỐT; waiting/due trigger → CHƯA CHỐT. Cells writing tests against a contract must list its decision id in `cell.decisions` (= `contract_status_refs`). New machinery: (i) one read verb or a `decisions active --tag contract:` recipe rendered into the worker contract; (ii) one claim-time or prepare-time check: any `cell.decisions` entry that is superseded or trigger-waiting refuses/warns the dispatch — the mint-trap tripwire, ~one function in `prepare.rs`/`claims.rs`. The absence problem is a refusal rule, not a registry: test-writing cell with an empty `decisions` list on a lane declaring a contract surface → `[BLOCKED]`, ask.

**B — intent rides the dispatch.** Keep `bee intent set --request` as the ONE verbatim store (per-feature key). In `prompt_body_for`, read the feature's intent anchor and render a new `{{#if original_request}}` block in `worker-cell.md` (and the gather/reviewer/advisor templates), framed with the existing VERBATIM/DO-NOT-PARAPHRASE header. No schema change, no per-cell copies; "layers only ADD" holds by construction.

## Opinion

A should be a derived view over the decision log, not a hand-kept registry: the log already has the three hard parts — supersession (labels never go stale in the active set), forced triggers on deferrals (CHƯA CHỐT with its revisit condition attached), and a query surface — and a second registry would drift from the log the first week nobody updates it. The never-logged contract is better handled by a refusal rule on test-writing cells than by pre-enumerating interfaces. B belongs in the dispatch payload, not on the cell record: the cell schema is deliberately closed, per-cell copies are N chances to truncate, and `bee intent` already holds the verbatim string under the exact "do not summarize" framing the spec demands — prepare.rs is the single door every worker passes, so injecting it there gives "rides every ticket untouched" with roughly ten lines of change. The spec's open question (d) "who edits contract_status" resolves naturally under the view: decisions are user-gated, so the label's authority is already the human's.
