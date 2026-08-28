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

**The cross-critique round needs no new flag, but it does need a bounded
change to the guard.** D2(c) asks that round two hand each lane the rival
proposal verbatim. A brief already carries arbitrary text, and the round-2
brief is a brief. What stands in the way is the leaning guard: a rival proposal
legitimately contains sentences like "I recommend X", and the guard would
refuse the brief that quotes it.

The first draft of this plan proposed skipping fenced text. The advisor consult
broke that in one move: an unbounded skip is a bypass in EVERY round, because a
round-one brief could fence its own verdict and still reach every lane. That is
the frozen stem list being shrunk by scope instead of by deletion, which the
shipped rule forbids outright.

The bounded shape, which this plan takes: the guard skips a fenced block ONLY
when its opening info string is the one designated token that claims "this is a
quoted rival proposal". Untagged fenced text scans exactly as it does today. A
fence still open at the end of the brief is a typed refusal, never a skip —
otherwise one unmatched opener hides every following line from every scan. The
tag is forgeable, and that is accepted with its eyes open: a forged tag is a
named lie inside a recorded brief, which is the same trust posture D4 already
takes at the door and settles by evidence at convergence. It ships as a named
limit, in the shape `79b5437b` established.

The fence rule reaches all THREE scans, not just the stem scan. A quoted
proposal carries its own headings and its own bullet lists, so the section-shape
arm and the Question-enumeration arm fire on it today just as loudly. One
implementation serves all three, hoisted out of the dossier module rather than
copied — the guard's own banner already argues that a rule living in two places
drifts.

## Shape

| Phase | What it delivers | Depends on | Why it exists |
|---|---|---|---|
| 1 | The reading list reaches the prompt it was passed for, and never rides beside a brief | — | A flag that is parsed, rendered and thrown away is a lie at the door blindness depends on |
| 2 | A tagged fence lets a round-2 brief quote a rival proposal verbatim without disarming the guard (D2(c)) | — | Without it there is no cross-critique round at all |
| 3a | A structured rejected set on the convergence decision (D2(d)) | — | The rejected lanes and their reasons are the dossier's other half |
| 3b | The deadlock hand-off: the question mark carrying the dossier, and the first real blocker letter for an unattended run (D2(e)) | — | Today an unattended deadlock has no channel; the letter section renders empty by construction |
| 4 | The blind-lane procedure prose (D1, D5, D7), the rule that convergence runs the checker green before it logs, and round two's named evidence limit | 1, 2, 3a, 3b | Nothing today tells an agent to open lanes at all — the shipped door has no caller |
| 5 | Reviewer/judge checklist material (D6) | — | D6 is reviewer craft; it must not sit behind a stalled lane slice |

Phases 1, 2, 3a, 3b and 5 are mutually independent and may run concurrently;
only Phase 4 waits, because prose that describes unshipped behavior is a lie.

**Phase 1 — the swallowed reading list.** `--expertise` is declared, accepted,
parsed and rendered into a block, and then dropped for every non-cell kind.
This phase renders it for gather, reviewer and briefless advisor dispatches —
refusing it outright would break callers that pass it today — and REFUSES the
one combination that is unsafe: a reading list beside a brief. The guard reads
brief bytes only, by design, so a reading list riding along is an unlinted
leaning channel straight into a blind lane, and the digest would keep proving
the briefs were equal while the payloads diverged. The refusal names the
brief's own read-diet section as the one carrier, which is the same argument
this plan already used to drop the read-diet flag.

Proof: the absent case stays byte-identical for every runtime×kind pair, AND
each kind gains a positive case — an absent-variable block is silently falsy in
this grammar, so a template twin that misses the edit would swallow the list
again with every existing test green. Every edited template gains the
disk-match probe the parent used for the advisor prompt. Cost: three prompt
templates, their vendored twins, the vars slice, the release manifest, a
rebuild.

**Phase 2 — a tagged fence, and nothing wider.** The guard learns one rule: a
fenced block whose opening info string carries the designated tag is not
scanned; every other fence scans as today; an unclosed fence refuses. The
frozen 17-stem list is NOT touched, and neither is the four-section shape arm —
the stems may never be shrunk to make a case pass. Red-first PER SCAN, not per
guard: a tagged-fenced stem passes while the same bytes outside still refuse; a
tagged-fenced heading no longer trips the section arm; a tagged-fenced bullet
list under Question no longer trips the enumeration arm; an unclosed fence
earns its typed refusal; and the zero-fire corpus run over every checked-in
brief-shaped document stays at zero.

The byte cap is part of this phase, not a surprise for its first user. A
round-2 brief carries four required sections plus a whole rival proposal, and
will meet the 8 KB refusal — whose own remedy text contradicts D2(c)'s word
"verbatim". This phase names the recorded fallback in the refusal itself and in
the prose, so a real cross-critique round has a road when the proposal is
large.

**Phase 3a — the rejected set.** A list-typed `--rejected` on `decisions log`,
comma-split like `--tags`. Costs the full flag chain: the handler allowlist,
the parse block, the params carrier, a hand-edit of the generated registry
payload, and `PINNED_FLAG_COUNT` 198 → 199 with its recorded reuse reason. The
cell also ships the drift net that does not exist: today a flag added to
`decisions log`'s allowlist but missing from the registry payload is
unreachable at the CLI and every test still passes — only `set_gate.rs` and
`workflows.rs` have that net.

**Phase 3b — the deadlock channel.** The question mark needs no code. The
letter does: the blocker entry kind exists and has no producer, and all three
existing producers hard-code an empty "Needs your call" list, so the section is
dropped from every letter ever written. This phase wires the first real
producer. The test asserting the section is ABSENT for a plain cap stays green
— a cap still has nothing to ask.

**Phase 4 — the prose.** One new section in
`skills/bee-hive/references/gates-and-delegation.md`, between the delegation
contract and the judgment contract, carrying a single-home clause so the rule
is never restated on five surfaces. Beyond D1, D5 and D7 it carries three
things the research and the consult surfaced:

- The rule that gives the checker teeth — convergence RUNS it green before it
  logs the decision — together with the constraint that makes that runnable:
  the check reads the dispatch log of the root the lanes actually ran in.
- Round two's evidence limit, named in the `79b5437b` style: round-2 briefs
  differ per lane by construction, so the digest chain, the recorded-brief
  re-lint and the citation check cover round ONE only. The prose requires the
  round-2 dispatch ids in the cross-critique section, and states plainly what
  is not mechanically checked.
- The tagged-fence limit from Phase 2: the tag is a claim, not a proof.

It also fixes two false surfaces the research found: the generated help text
still says the citation, digest and read-diet checks "are not part of this door
yet" when all six ship, and the one shipped example dossier refuses in the main
checkout because its placeholder dispatch ids are not in this repo's log. The
knowledge concept's Open Gaps section, which names this slice as unbuilt, is
corrected in the same cell.

**Phase 5 — the checklist material.** Into `expertise/review.md`, whose headings
`bee-reviewing` already cites by name. The Truth Table Test and the CRUD
Lifecycle check are genuinely new — neither concept appears anywhere in
`skills/`, `expertise/` or `docs/knowledge/`. The 5-Layer rubric lands as a
FRAME that cites its existing homes (happy path and failure-edge are already
the planning triad and the 12 edge dimensions; definition-of-done is already
the proof line), never as a restatement — a fourth spelling of
definition-of-done is the second home this import exists to avoid.

## Smaller path check

*Is there a cheaper shape that still honors every locked decision?* Yes for the
cross-critique round, and it is taken: Phase 2 replaces the parent's "round-2
payloads" flag with a bounded guard change, on the evidence that a brief
already carries arbitrary text. The advisor consult then priced the cheapest
version of that change — skip every fence — and found it unsafe, so this plan
takes the next-cheapest bounded one rather than the cheapest one. What cannot
be made cheaper: Phase 3a's flag chain (a structured field is data-model work
by definition) and Phase 1 (a swallowed flag is a defect, not a scope choice).

## Test matrix

| Surface | Risk | Proof |
|---|---|---|
| Non-cell prompts gain a block | HIGH — every dispatch in the repo renders through this line | Absent case byte-identical for every runtime×kind pair; a POSITIVE case per kind; a disk-match probe per edited template; the brief+expertise combination refused, red-first |
| The guard learns a tagged fence | HIGH — a guard and its tests are one model (`patterns/20260812-…`) | Red-first per SCAN: tagged-fenced stem, tagged-fenced heading, tagged-fenced enumeration, untagged fence still scanned, unclosed fence refused; the 17 stems unchanged, asserted by count; the zero-fire corpus stays at zero |
| One fence implementation | MEDIUM — a rule in two places drifts | One test reads both call sites against the same shared function |
| A new list flag | MEDIUM | Red-first on the drift net FIRST — the new net must fail against a registry payload that does not declare the flag, then pass |
| The blocker letter | MEDIUM — the authorship ban forbids any word not carried by an entry | The cap letter's "Needs your call" section stays absent; the new producer's letter carries the item and nothing invented |
| Prose | LOW | `pointer_integrity` and `instruction_laws` green; `bee dev release-manifest --check`; the regen obligation satisfied inside the same cell |

## Known red base

`p-624e2d7d` — the declared suite is RED on any machine running `opencode-ai`
newer than CI's pin (`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`).
Not this feature's work; CI is green on its pin. Cells prove themselves with
scoped runs.

## Advisor consult

`docs/history/slp-blind-lanes-procedure/advisor-consult.md` — verdict SAFE WITH
NAMED CHANGES, nine changes, all nine folded into the text above before this
plan reached its gate. The consult's own finding is recorded there: the first
draft's unbounded fence skip was a bypass of the guard in every round.

## Out of scope

- Heterogeneous lane models — deferred by the map (`4faf1de9`), unchanged.
- Closing the citation check's cross-sentence framing gap — it is backlog item
  `p-e09a0b7e` with its own acceptance, and `79b5437b` holds that the check
  proves provenance only.
- Any new store, namespace or command family (`f0f21142`).
