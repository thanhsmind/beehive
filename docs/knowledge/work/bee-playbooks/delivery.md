---
type: bee.delivery
title: bee-playbooks — delivery
description: "Delivery record proposed by bee knowledge promote for work item bee-playbooks: 3 capped cell(s), 8 recorded deviation(s)."
timestamp: 2026-09-24
bee:
  id: bee-playbooks-delivery
  lifecycle: active
  required_context: [docs/history/bee-playbooks/CONTEXT.md, docs/history/bee-playbooks/plan.md]
  sources: [docs/history/bee-playbooks/CONTEXT.md, docs/history/bee-playbooks/plan.md, .bee/cells/bpb-1.json, .bee/cells/bpb-2.json, .bee/cells/bpb-3.json]
---

# bee-playbooks — Delivery

## What shipped

- **bpb-1** — AGENTS.md Deep contracts pointer sentence names the class-bound playbook and points at planning-reference.md Class playbooks (AGENTS.md:357-358) (1 file(s) changed)
- **bpb-2** — content added as the 9th route class with its playbook, perf plateau-pivot sentence, and migration note (commit cb027e239) (8 file(s) changed)
- **bpb-3** — Confirmed pstack playbook gap-check artifact (23 anchored verdicts) and two deferred-gap backlog rows from eb7bd2e96 (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **bpb-1** — `rg -n "class-bound playbook" AGENTS.md && rg -n "## Class playbooks" skills/bee-planning/references/planning-reference.md` — docs pointer check, the cell verify
- **bpb-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml --test route_class_parity --test class_playbook_parity --test principle_index_parity && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee -- route_set_ && .bee/bin/bee dev regen && .bee/bin/bee dev release-manifest --check` — parity tests 2+5+2 and 4 route_set_ unit tests pin the class vocabulary this cell changed, regen 3/3 and manifest 376 match; full suite not run
- **bpb-3** — `test -f docs/history/research/pstack-playbooks-gap-check.md && rg -c pstack-playbooks-gap-check .bee/backlog.jsonl` — docs cell: artifact presence plus backlog linkage (2 rows)

## Deviations

- **bpb-1** — Wrapped the inserted clause across two new lines inside the same sentence instead of one long line — keeps the paragraph line width; no other line changed — found a better route
- **bpb-1** — Capped with --sync-ack for rule agents-capture-line-at-close — the edit is a Deep contracts pointer only, the rule text is untouched — the plan was wrong about a fact
- **bpb-1** — sync-ack: edit is one pointer clause in the Deep contracts index; the agents-capture-line-at-close rule text is untouched, so its applied_at files need no change
- **bpb-2** — no new commit: prior worker commit cb027e239 holds the work; regen again reverted AGENTS.md in the working tree and I restored it to HEAD, so AGENTS.md has no diff
- **bpb-2** — sync-ack: must_haves artifact names skills/bee-planning/references/planning-reference.md; affects_skills predicted bee-planning/SKILL.md for the same skill, the reference file is the approved target
- **bpb-3** — Row 23 Bug fix added: pinned pstack SKILL.md lines 112-140 list 23 playbooks, plan counted 22
- **bpb-3** — sync-ack: sibling-cell skill edits on branch, none in eb7bd2e96
- **bpb-3** — sync-ack: bpb-3 commit eb7bd2e96 touches no skills/** path; the named skill files were changed by sibling cells bpb-1/bpb-2 on the same branch

## Provenance

Proposed by `bee knowledge promote --work bee-playbooks` from 3 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/bee-playbooks/CONTEXT.md`, `docs/history/bee-playbooks/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.

## Follow-on: a real bug this delivery found

`bee dev regen`'s `onboard --apply` step reverts a committed, unrelated
edit to `AGENTS.md` in the working tree (observed twice, in cells bpb-1
and bpb-2, on this same branch) — AGENTS.md is itself a generated file
(rendered from `packages/bee/AGENTS.block.md`), and the parity test
`agents_block_render_parity` is the actual guard; bpb-1's cell hand-edited
the generated copy instead of the template, which this delivery's own
final full-suite run on main caught and fixed (see commit `6891898e3`).
Recorded via `bee mailbox reflect` (fix-at: check) during this feature's
own close.
