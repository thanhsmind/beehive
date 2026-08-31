---
type: bee.area
title: Advisor Protocol — blind lanes and the convergence dossier
description: "Isolated advisor lanes on one hard decision, the leaning guard the brief passes at the dispatch door, and the single dossier a convergence is checked against."
timestamp: 2026-08-28
bee:
  id: advisor-protocol-blind-lanes-and-the-convergence-dossier
  lifecycle: active
  areas: [advisor-protocol]
  required_context: [areas/advisor-protocol/overview.md, areas/advisor-protocol/slots-and-tiers.md]
  decisions: ["slp-blind-lanes D1 (the agent opens 2-3 blind lanes on its OWN judgment when a decision is both high-stakes and ambiguous, logging the reason at open time; the user may also order lanes; a deadlocked convergence always hands the user the dossier)", "slp-blind-lanes D2 (blind lanes are a PROCEDURE over the existing dispatch door: one leaning-linted brief, parallel advisor dispatches carrying it byte-identical with an explicit read diet, cross-critique as a second advisor round, convergence as a dossier plus one decision entry with a registered revisit trigger, deadlock handed to the human)", "slp-blind-lanes D3 (a lane NEVER runs as a cell, because the cell template injects shared prior-round context and leaks the very thing blindness protects)", "slp-blind-lanes D4 (convergence carries an anti-fabrication check: every dossier citation must resolve against the verbatim lane proposals)", "slp-blind-lanes D5 (an objection is valid only when it names the specific missing context)", "slp-blind-lanes D6 (the 5-Layer rubric, the Truth Table Test and the CRUD Lifecycle check join the reviewer/judge checklist material)", "slp-blind-lanes D7 (hats and lanes stay distinct instruments: lanes GENERATE designs from one brief, hats CRITIQUE one request from fixed perspectives)", "slp-blind-lanes D1/D2/D3/D4/D5/D6/D7 are the locked set in docs/history/slp-blind-lanes/CONTEXT.md", "slp-blind-lanes f0f21142 (2026-08-28 — shipped shape: no new store and no new command family; the brief rides the existing dispatch door, the lane-opening reason rides the existing decision log, the dossier document itself holds every proposal verbatim, and ONE new verb checks that document)", "slp-blind-lanes 79b5437b (2026-08-28 — the citation check claims PROVENANCE, never faithfulness: it proves a quoted span is a whole sentence of the named lane's own bytes, and cross-sentence framing stays a NAMED LIMIT pinned by its own green probe)"]
  sources: ["docs/history/slp-blind-lanes/CONTEXT.md (locked decisions D1-D7, terms, boundary)", "docs/history/slp-blind-lanes/plan.md (the slice queue 1a, 1b, 2, 3, 4, 5)", "slp-blind-lanes cell bln-1 (the brief carried into the advisor payload and its stamped digest; trace .bee/cells/bln-1.json, commit 2f1d57ae)", "slp-blind-lanes cell bln-2 (the leaning guard refusing at the dispatch door; trace .bee/cells/bln-2.json, commit 5f16ef6f)", "slp-blind-lanes cell bln-3 (the dossier's fixed section contract, refusing by section name; trace .bee/cells/bln-3.json, commit 5678b2f8)", "slp-blind-lanes cell bln-4 (the three evidence checks: citation, brief digest, read diet; trace .bee/cells/bln-4.json, commit c47fd241)", "slp-blind-lanes cells bln-5, bln-6, bln-7 (three judge rounds over the sentence-boundary rule: the abbreviation dot, the enumerator dot, the curly-quote panic; traces in .bee/cells/, commits 6eaf4955, 417fff74, a5ad7864)", "docs/history/slp-blind-lanes/blind/example-run.md (the worked dossier shape, pinned green by the door's own test)", "docs/knowledge/patterns/20260812-a-guard-and-its-tests-are-one-model-so-green-proves-only-that-the-model-agrees-with-itself.md"]
  authoritative_for: "advisor-protocol: blind lanes and the convergence dossier"
---

# Advisor Protocol — Blind Lanes and the Convergence Dossier

A blind lane is one isolated advisor consult that designs an answer to a
single hard question without seeing a sibling lane's work or the
orchestrator's own leaning. Two to three lanes run on one question, critique
each other's answers, and converge into ONE document. The whole feature is a
procedure over machinery that already exists — the dispatch door and the
decision log — plus one verb that checks the document at the end.

## Behaviors & Operations

**B1 — The agent opens lanes on its own judgment.** Lanes open when a decision
is both high-stakes AND ambiguous, and the reason is logged at open time — no
approve-each-lane wait. The human may also order lanes directly at any point.
A convergence that produces no chosen answer hands the human the dossier
unchanged; it never resolves itself by coin flip (slp-blind-lanes D1).

**B2 — A lane is an advisor consult, never a work cell.** Every lane receives
the same brief, byte for byte, plus an explicit list of the paths it may read.
Advisor consults are read-only and ephemeral by construction, which is what
makes them isolated. A lane never runs as a cell, because the cell form
carries prior-round history and assembled prior knowledge into the worker —
shared memory across lanes is exactly what blindness protects against
(slp-blind-lanes D2, D3).

**B3 — The brief passes a leaning guard at the dispatch door.** The guard
reads the brief and refuses wording that hands the lanes the orchestrator's
own verdict. It is named for what it does: it refuses LEANING LANGUAGE, and it
does not certify neutrality. A word list cannot certify neutrality, and
calling it a neutrality proof would turn "unlinted" into "certified" — false
confidence at the one door the feature rests on (slp-blind-lanes D2).

**B4 — The convergence record is one document plus one decision entry.**
Nothing new is stored anywhere else. The document holds every lane's proposal
verbatim, the cross-critiques, the chosen answer, the rejected set with its
reasons, and the citations. The decision entry carries a registered revisit
condition, so a reversal has a named trigger rather than a memory
(slp-blind-lanes D2, f0f21142).

**B5 — One verb checks that document.** Its sections are fixed and ordered,
and every refusal names the offending section, so a reader always learns which
part of the record is wrong. The verb re-runs the SAME leaning guard over the
brief the document recorded, so a convergence built on an unchecked brief
still refuses here — even though the door it bypassed never saw it. It then
runs three evidence checks, each reading a source outside the sentence making
the claim: every citation against the proposal of the lane it names, every
lane's brief digest against the dispatch log, and every reported path against
the diet the brief declared. A zero count refuses instead of passing:
"checked nothing" must never render as "checked" (slp-blind-lanes D2, D4).

**B6 — Proposals ride inside fenced blocks.** A lane proposal is arbitrary
prose and may quote a heading. Outside a fence, a lane's own text would move
the record's section boundaries, and the record would be checking itself
against whatever it happened to say.

## Business Rules

- **The citation check proves PROVENANCE, never faithfulness.** A resolved
  citation proves the quoted span is a whole sentence of the named lane's own
  bytes, and nothing more. A quote whose meaning is governed by the sentence
  BEFORE it still passes; that gap is a recorded limit with its own green
  probe, not a silent hole (slp-blind-lanes 79b5437b).
- **An objection counts only when it names the missing context.** Pushback
  without a named gap does not stand (slp-blind-lanes D5).
- **The guard reads the brief and nothing else.** It never sees the dispatch
  purpose, the reading list, or any other dispatch text. A false refusal on
  those would block the advisor consult that the high-risk gate itself
  requires, and deadlock the workflow that approves guards.
- **The guard's word list is frozen in both directions.** An addition needs
  its own recorded reason, and the list may NEVER be shrunk to make a test
  pass — shrinking a guard's list to satisfy its own corpus is the guard
  agreeing with itself.
- **Lanes and hats are different instruments.** Lanes GENERATE designs from
  one shared brief; hats CRITIQUE one request from fixed disjoint
  perspectives. Neither replaces the other (slp-blind-lanes D7).

## Open Gaps

- **Shipped today:** the brief on the dispatch door with its leaning guard and
  its stamped digest; the dispatch reading list rendered into every non-cell
  prompt, and refused beside a brief; the tagged fence that lets a round-2
  brief quote a rival proposal verbatim without disarming the guard; the
  dossier-checking verb with its section contract and its three evidence
  checks; the rejected set as a structured list on the decision record; the
  deadlock hand-off as a question mark or, unattended, a blocker letter that
  asks the human something; the reviewer/judge checklist material
  (slp-blind-lanes D6); and the procedure prose itself, whose single home is
  `skills/bee-hive/references/gates-and-delegation.md` ("Blind lanes and
  convergence") — lane-opening now has a written rule to follow.
- **Round two sits outside the evidence chain.** Round-2 briefs differ per lane
  by construction, so the brief-digest check, the recorded-brief re-lint and
  the citation check cover round ONE only. The dossier carries the round-2
  dispatch ids in its cross-critique section as text; nothing mechanically
  resolves them, and closing that gap is unbuilt work, not a hidden defect.
- **The framing limit is open work, not a defect to hide:** a citation whose
  preceding sentence negates or forward-references it resolves and passes. The
  acceptance for closing it is recorded as a backlog item.
- **Still deferred, not pending:** heterogeneous lane models, which break the
  one-name advisor slot (decision `4faf1de9`).

## Pointers (implementation)

- The dossier door and its checks:
  `packages/bee-rs/crates/bee/src/verbs/blind/mod.rs` (`bee blind check
  --dossier <path>`).
- The one leaning guard both doors call:
  `packages/bee-rs/crates/bee/src/verbs/drivers/brief_lint.rs`; the dispatch
  door's `--brief-file` arm is in
  `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`, and the advisor
  prompt's brief block is `packages/bee/prompts/advisor.md`.
- The worked dossier shape, pinned green by the door's own test:
  `docs/history/slp-blind-lanes/blind/example-run.md`.
- The open framing limit: backlog item `p-e09a0b7e`.
- The procedure — when lanes open, the four moves, the rule that convergence
  runs the checker green before it logs, and the three named limits — has ONE
  home: `skills/bee-hive/references/gates-and-delegation.md` ("Blind lanes and
  convergence"). This concept describes what shipped; it never restates that
  procedure.
- A second, sibling plan-step consult now lives beside blind lanes at that
  same home: the **hat wave** — five fixed-perspective advisor dispatches at
  the plan step, prompt-carried perspective, blue hat = orchestrator, the
  human at Lock is the checker. Shipped as a docs-only procedure (no new
  code); its home is the same file, section "Hat wave"
  (`skills/bee-hive/references/gates-and-delegation.md`). The foreign-origin
  spec-drop intake (corr-id as PBI id, provenance in CoS, proposed-until-shaped)
  is documented separately, in `skills/bee-shaping/SKILL.md` ("Qualify").
