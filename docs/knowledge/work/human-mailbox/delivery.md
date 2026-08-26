---
type: bee.delivery
title: human-mailbox — delivery
description: "Delivery record proposed by bee knowledge promote for work item human-mailbox: 10 capped cell(s), 17 recorded deviation(s)."
timestamp: 2026-08-26
bee:
  id: human-mailbox-delivery
  lifecycle: active
  areas: [human-mailbox]
  required_context: [docs/history/human-mailbox/CONTEXT.md, docs/history/human-mailbox/plan.md]
  sources: [docs/history/human-mailbox/CONTEXT.md, docs/history/human-mailbox/plan.md, .bee/cells/archive/human-mailbox/hm-1.json, .bee/cells/archive/human-mailbox/hm-2.json, .bee/cells/archive/human-mailbox/hm-3.json, .bee/cells/archive/human-mailbox/hm-4.json, .bee/cells/archive/human-mailbox/hm-5.json, .bee/cells/archive/human-mailbox/hm-6.json, .bee/cells/archive/human-mailbox/hm-7.json, .bee/cells/archive/human-mailbox/hm-8.json, .bee/cells/archive/human-mailbox/hm-9.json, .bee/cells/archive/human-mailbox/hm-10.json]
---

# human-mailbox — Delivery

## What shipped

- **hm-1** — Mailbox store and letter record shape land: one markdown letter per run, entry layer settled as one append-only JSONL per run (2 file(s) changed)
- **hm-2** — Every cap now appends its human-mailbox entry the moment it lands, with the plain sentence written then; arming reads the herding block plus the owner switch and gates only the letter (2 file(s) changed)
- **hm-3** — An armed run now composes its stored entries into one filed letter with D7's five sections, with a test that fails if composition invents a fact (2 file(s) changed)
- **hm-4** — The mailbox store is git-ignored, so a cap that appends an entry no longer dirties the checkout or blocks a sibling session's merge (1 file(s) changed)
- **hm-5** — An armed cap must state a departure in three parts or state that the plan was followed; an unarmed cap stays byte-identical (2 file(s) changed)
- **hm-6** — Worker prompt and swarming reference now describe the three-part departure line, the closed four kinds, the explicit plan-followed statement, and the armed-only door (2 file(s) changed)
- **hm-7** — worker-details now teaches the shipped departure contract: three parts, the closed four kinds, the explicit followed-the-plan statement recorded on trace.plan_followed, and the armed-only boundary (1 file(s) changed)
- **hm-8** — bee mailbox mark flips a filed letter's read state through the store, idempotently, and the registry declares it (3 file(s) changed)
- **hm-9** — A run that went silent gets its unfinished letter from the next session, detected from directory names alone and never swept while it is still alive (2 file(s) changed)
- **hm-10** — The feature-close letter carries architecture, behaviour and usage; the nightly letter never grows them (2 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **hm-1** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hm-2** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hm-3** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hm-4** — `git check-ignore -q .bee/human-mailbox/entries/probe.jsonl`
- **hm-5** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::cells`
- **hm-6** — `rg -n 'unforeseen obstacle|better route' packages/bee/prompts/worker-cell.md skills/bee-swarming/references/swarming-reference.md`
- **hm-7** — `rg -n 'hit an unforeseen obstacle' skills/bee-swarming/references/worker-details.md`
- **hm-8** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hm-9** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`
- **hm-10** — `cargo test --release --manifest-path packages/bee-rs/Cargo.toml mailbox`

## Deviations

- **hm-1** — Followed the plan for the record shape and the D11 filename.
- **hm-1** — Departure (what): registered the module in verbs/mod.rs as a library module instead of adding a try_native verb probe. Why: the one command this feature owes is the D6 read flip, which the cell assigns to phase 3, so there is no verb to serve yet and an empty probe would be dead argv handling. Kind: found a better route.
- **hm-1** — Departure (what): added a sha256 tail to any run slug that had to be truncated. Why: two long run ids sharing a 20-character prefix would otherwise collapse onto one letter filename and one entries file, overwriting a letter and merging two runs of entries - the exact loss D11 exists to prevent. Kind: hit an unforeseen obstacle.
- **hm-1** — Departure (what): capped with --inline-reason instead of a registered workers[] row. Why: the dispatch wrote no worker row and bee state worker add refuses from a granted worktree, so this session cannot add it; the orchestrator must reconcile state.workers. Kind: hit an unforeseen obstacle.
- **hm-2** — {"what":"Arming requires the owner enable marker as well as the config herding block","why":"The herding block alone answers only that the checkout CAN herd, so every attended session in a herding repo would have filed a letter, which D9 excludes; the marker is bee existing arming switch, not a new signal","kind":"the plan was wrong about a fact"}
- **hm-2** — {"what":"The run is the session span, not the herding job id","why":"D9 and D12 both describe a run in sessions, and one night dispatches many jobs, which would shatter D11 one-letter-per-run","kind":"found a better route"}
- **hm-2** — sync-ack: phase 1 changes no contract a skill documents: no flag, no Result form and no worker instruction moves here (the cell declares affects_skills []). plan.md puts the worker-prompt and bee-swarming re-wording in phase 2, with the departure contract that actually changes what a worker must type
- **hm-3** — The Next section of D7 renders empty and is therefore dropped: no Entry field carries a next step, and printing one would be authoring (D8). Recorded in compose_body rather than left implicit.
- **hm-3** — work.rs hooks the run end at a work record reaching a terminal status (done|dropped) — the one moment this verb group can see a session span end; bee state session release lives in state_group/sessions.rs, outside the cell's files.
- **hm-4** — Found while verifying hm-2 rather than planned: hm-2 flagged the untracked store path as a next-phase question, but it dirties main on every cap today, which is the standoff that blocked two sessions for hours earlier in the session. Kind: hit an unforeseen obstacle.
- **hm-5** — {"what":"Recorded D5's plan-followed statement on trace.plan_followed and kept it out of trace.deviations, instead of appending it like any other line","why":"trace.deviations is what bee knowledge promote mines for patterns, and a line saying nothing happened would teach it a pattern out of silence; the lift is armed-only so D10's byte-identical unarmed path is untouched","kind":"found a better route"}
- **hm-5** — sync-ack: the skill wording this contract needs (bee-swarming reference, worker prompt) is cell hm-6 by design, so the change reviews against shipped behaviour rather than beside it
- **hm-6** — followed the plan
- **hm-7** — followed the plan
- **hm-8** — Followed the plan — the deferred naming question was the cell's to settle and it is settled in the module header and the commit body
- **hm-9** — Followed the plan — the two files the cell named were enough, and no locked decision needed reinterpreting.
- **hm-10** — Stored the three lists as extra keys on the feature-close stop own JSONL line instead of as three new Entry fields — three new Entry fields would have forced an edit to verbs/cells/handlers_close.rs, which this cell does not name, and no other stop kind has an architecture — found a better route

## Provenance

Proposed by `bee knowledge promote --work human-mailbox` from 10 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/human-mailbox/CONTEXT.md`, `docs/history/human-mailbox/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

