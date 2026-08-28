# approved_gate2: <unset until approval>

# Plan: SLP Blind Lanes — Procedure (slices 2–5)

Feature: `slp-blind-lanes-procedure` · Lane: high-risk (4 flags: data-model,
public-contracts, covered-contract-change, multi-domain) · 8 product files.

Locked decisions: `docs/history/slp-blind-lanes/CONTEXT.md` D1–D7, plus store
decisions `f0f21142` (no new store, no new command family) and `79b5437b` (the
citation check proves provenance, never faithfulness). This plan cites them; it
reinterprets none.

## What research changed about the parent's slice queue

Two independent read-only passes over the shipped code (2026-08-28) moved three
items out of the work and one into it. Named here because the parent plan's
slice text is the input this plan is measured against.

| Parent plan said | Truth at HEAD | Effect |
|---|---|---|
| Slice 2 carries "the read-diet list into the advisor payload beside the brief" | The read diet is already a REQUIRED brief section (`brief_lint.rs:76`), and `bee blind check` already checks reported paths against it (`blind/mod.rs:1010`) | Dropped. A second carrier would be a second source of truth for one list. |
| Slice 2 carries "the `--kind cell` refusal for a blind brief (D3)" | Already ships, typed, and fires before the file is read (`prepare.rs:129-138`), pinned by `a_brief_is_refused_for_every_kind_but_advisor` | Dropped as built. |
| Slice 3 needs a `waiting-on --kind question` mark | The verb and the kind already exist (`record.rs:379`) and the supervisor already renders the mark | Reduced to procedure, not code. |
| — | `--expertise` is accepted at the door for every kind and DROPPED for every kind but `cell` (`prepare.rs:689` renders only `brief`; `:714` is the only site that passes `expertise`) | ADDED as Phase 1. A silent swallow at the door, on this feature's own path. |

## Approach

Three of the four remaining slices are procedure, not machinery — which is
exactly what `f0f21142` asks for. The plan spends its risk budget on the two
places where the door itself must change, and writes the rest as prose.

**The cross-critique round needs no new flag.** D2(c) asks that round two hand
each lane the rival proposal verbatim. A brief already carries arbitrary text,
and the round-2 brief is a brief. The one thing standing in the way is the
leaning guard: a rival proposal legitimately contains sentences like "I
recommend X", and the guard would refuse the brief that quotes it. The fix is
the trick the dossier parser already uses — read outside fenced blocks only
(`blind/mod.rs` ignores fenced lines when scanning headings, for the same
reason: a payload's own text must not move the record it rides in). With that,
round two is a procedure over the shipped door, and the parent's "round-2
payloads" item costs one guard change instead of a flag, a registry hand-edit
and a pinned-count bump.

## Shape

| Phase | What it delivers | Depends on | Why it exists |
|---|---|---|---|
| 1 | The reading list reaches the prompt it was passed for, or nothing accepts it | — | A flag that is parsed, rendered and thrown away is a lie at the door blindness depends on |
| 2 | The leaning guard reads outside fences only, so a round-2 brief can quote a rival proposal verbatim (D2(c)) | 1 | Without it there is no cross-critique round at all |
| 3a | A structured rejected set on the convergence decision (D2(d)) | — | The rejected lanes and their reasons are the dossier's other half |
| 3b | The deadlock hand-off: the question mark carrying the dossier, and the first real blocker letter for an unattended run (D2(e)) | — | Today an unattended deadlock has no channel; the letter section renders empty by construction |
| 4 | The blind-lane procedure prose (D1, D5, D7) and the rule that convergence runs the checker green before it logs | 2, 3a, 3b | Nothing today tells an agent to open lanes at all — the shipped door has no caller |
| 5 | Reviewer/judge checklist material (D6) | — | D6 is reviewer craft; it must not sit behind a stalled lane slice |

**Phase 1 — the swallowed reading list.** `--expertise` is declared, accepted,
parsed and rendered into a block, and then dropped for every non-cell kind.
Either it reaches the gather/reviewer/advisor prompts or the door refuses it
there. The plan chooses REACHES: refusing would break callers that pass it
today, while rendering costs the same `{{#if}}` mechanism the brief block
already proved, and the "absent renders byte-identical" property is already
pinned by `no_brief_leaves_every_runtime_and_kind_pair_exactly_as_it_was`.
Cost: three prompt templates, their vendored twins, the vars slice, the
release manifest, a rebuild.

**Phase 2 — fenced text is not the guard's business.** `lint_brief` scans the
whole brief. A round-2 brief quotes a rival LaneProposal, which is arbitrary
advisor prose and will contain leaning sentences. The guard learns the fence
rule; the frozen 17-stem list is NOT touched, and neither is the four-section
shape arm — the stems may never be shrunk to make a case pass. Red-first: a
brief whose ONLY leaning sentence sits inside a fence passes; the same sentence
outside the fence still refuses.

**Phase 3a — the rejected set.** A list-typed `--rejected` on `decisions log`,
comma-split like `--tags`. Costs the full flag chain: the handler allowlist,
the parse block, the params carrier, a hand-edit of the generated registry
payload, and `PINNED_FLAG_COUNT` 198 → 199 with its recorded reuse reason. The
cell also ships the drift net that does not exist: today a flag added to
`decisions log`'s allowlist but missing from the registry payload is
unreachable at the CLI and every test still passes — only `set_gate.rs` and
`workflows.rs` have that net.

**Phase 3b — the deadlock channel.** The question mark needs no code. The
letter does: the blocker entry kind exists and has no producer, and both
existing producers hard-code an empty "Needs your call" list, so the section is
dropped from every letter ever written. This phase wires the first real
producer. The test asserting the section is ABSENT for a plain cap stays green
— a cap still has nothing to ask.

**Phase 4 — the prose.** One new section in
`skills/bee-hive/references/gates-and-delegation.md`, between the delegation
contract and the judgment contract, carrying a single-home clause so the rule
is never restated on five surfaces. It also fixes two things the research
found: the generated help text still says the citation, digest and read-diet
checks "are not part of this door yet" when all six ship, and the one shipped
example dossier refuses in the main checkout because its placeholder dispatch
ids are not in this repo's log — the prose must name that constraint or the
rule has no worked example an agent can copy. The knowledge concept's Open Gaps
section names this slice as unbuilt and is corrected in the same cell.

**Phase 5 — the checklist material.** Into `expertise/review.md`, whose headings
`bee-reviewing` already cites by name. The Truth Table Test and the CRUD
Lifecycle check are genuinely new — neither concept appears anywhere in
`skills/`, `expertise/` or `docs/knowledge/`. The 5-Layer rubric lands as a
FRAME that cites its existing homes (happy path and failure-edge are already
the planning triad and the 12 edge dimensions; definition-of-done is already
the proof line), never as a restatement — a fourth spelling of
definition-of-done is the second home this import exists to avoid.

## Smaller path check

*Is there a cheaper shape that still honors every locked decision?* Yes, and it
is taken: Phase 2 replaces the parent's "round-2 payloads" flag with a guard
change, on the evidence that a brief already carries arbitrary text and the
fence rule already exists in the sibling module. What cannot be made cheaper:
Phase 3a's flag chain (a structured field is data-model work by definition) and
Phase 1 (a swallowed flag is a defect, not a scope choice).

## Test matrix

| Surface | Risk | Proof |
|---|---|---|
| Non-cell prompts gain a block | HIGH — every dispatch in the repo renders through this line | A briefless, expertise-less payload stays byte-identical for every runtime×kind pair; the existing pinned literal is extended, not weakened |
| The guard learns fences | HIGH — a guard and its tests are one model (`patterns/20260812-…`) | Red-first per arm: leaning inside a fence passes, the same bytes outside still refuse; the corpus run over every checked-in brief-shaped doc stays at zero fires; the 17 stems are unchanged, asserted by count |
| A new list flag | MEDIUM | Red-first on the drift net FIRST — the new net must fail against a registry payload that does not declare the flag, then pass |
| The blocker letter | MEDIUM — the authorship ban forbids any word not carried by an entry | The cap letter's "Needs your call" section stays absent; the new producer's letter carries the item and nothing invented |
| Prose | LOW | `pointer_integrity` and `instruction_laws` green; `bee dev release-manifest --check`; the regen obligation satisfied inside the same cell |

## Known red base

`p-624e2d7d` — the declared suite is RED on any machine running `opencode-ai`
newer than CI's pin (`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`).
Not this feature's work; CI is green on its pin. Cells prove themselves with
scoped runs.

## Out of scope

- Heterogeneous lane models — deferred by the map (`4faf1de9`), unchanged.
- Closing the citation check's cross-sentence framing gap — it is backlog item
  `p-e09a0b7e` with its own acceptance, and `79b5437b` holds that the check
  proves provenance only.
- Any new store, namespace or command family (`f0f21142`).
