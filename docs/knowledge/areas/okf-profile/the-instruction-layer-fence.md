---
type: bee.area
title: Bee OKF Profile — the instruction-layer fence
description: "Content guards protect where truth is written; nothing tested the prose that TELLS an agent where truth lives. This fence grades every instruction line, on the line, so a migration cannot leave its own instructions teaching the retired model."
timestamp: 2026-07-23
bee:
  id: okf-profile-instruction-layer-fence
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/specs-read-only-fence.md, areas/okf-profile/overview.md]
  decisions: [G1, G2, G12, G13, F4-D4]
  sources: ["okf-integration-close-f4 cells f4-1..f4-6 (three hand audits found 7, then 2, then 6 instruction gaps with the chain green throughout; the gate ends the sequence; traces in `.bee/cells/`, 2026-07-22/23)", red evidence `docs/history/okf-integration-close-f4/reports/red-instructions-fence.md`, CONTEXT.md `docs/history/okf-integration-close-f4/CONTEXT.md`]
  authoritative_for: "okf-profile: the instruction-layer fence"
---

# Bee OKF Profile — The Instruction-Layer Fence

## Purpose

A migration moves where truth lives. Its guards protect **content** — where new truth may be
written, whether it parses, whether coverage is complete. Nothing guards the **prose that tells an
agent where truth lives**, and prose has no test, so it rots while every check stays green. The rot
lands in the worst possible place: the surfaces that teach every future session.

The evidence for needing this is the sequence that produced it. Three successive hand audits of the
same question each found gaps the previous audit had missed — seven, then two, then six — and the
verify chain was green before, during, and after all three. The third audit's own conclusion was
that a fourth would find an eleventh. This fence exists so the fourth audit is a machine.

## Entry Points & Triggers

- The verify chain runs it on every green run, in two forms: a self-test proving the classifier
  bites, and a check of this repo's own instruction surfaces.
- It is the sibling of the read-only fence (`specs-read-only-fence.md`): that one governs where new
  truth may LAND, this one governs what the instructions SAY about where truth lives.

## Data Dictionary

The governed surfaces are the ones that instruct an agent: the skills' own documents, the hooks that
inject text into a session, and the root operating block. Every line naming the retired state layer
is classified as exactly one of:

| Verdict | What it means |
|---|---|
| branched | the line carries the branch **on the line** — a reader seeing it in isolation knows it is conditional |
| legacy-anchor | the line cites a numbered anchor in a migrated document, which resolves through that document's pointer stub to the concept owning it now |
| historical-record | a creation log or a dated history document — a record of what was true then, not an instruction about now |
| unbranched misroute | anything else — a failure, named by file and line, with the offending line quoted |
| inert | the whole check, in a repo with no bundle: it does not scan at all |

## Behaviors & Operations

**The rule is LINE-local, and that is the load-bearing design decision.** The obvious rule — "a
document that mentions the bundle somewhere is branched" — was implemented first, measured, and
discarded on evidence: it passed **four of six** misroutes that humans had already found by hand.
One skill document named the bundle four times and still called the retired pattern file mandatory
pre-work reading; one hook was bundle-aware in its logic and still printed the wrong destination in
its message. The reason is how the failure actually happens: an agent reads a bullet, a table row,
or an injected message **in isolation**, and a branch three paragraphs above does not travel with
it. A document-level rule grades the document; the damage is done by the line.

**Exemptions are structural, never a list of names.** Recognising exempt files by name is the
failure the read-only fence already refuses for the same reason: a name list rots the first time
something is added or renamed, and a rotted list stops guarding *silently*. Each exemption is
therefore a property of the line or its location, and each is asserted in the self-test **with a
negative twin** — a near-miss that must still fail — so an exemption cannot quietly widen.

**It is gated on bundle mode and inert without one.** A repo that never migrated keeps its
instructions exactly as they are and cannot tell this shipped. Same predicate as every other
consumer; never re-derived.

**Narrowed surfaces are declared, not hidden.** Where the governed set is narrower than the obvious
reading — machinery where a retired path is *data* rather than instruction, and the test suites
whose retired-layer strings are their own assertions — the narrowing is reported by the check
itself, with its reason, rather than living as an unstated assumption.

**The failure it caught that no human audit did: a guard sending an agent into another guard.** The
session-close capture nudge fires when settled work has not reached the state layer. Its *logic* had
been migrated — it measures staleness across both trees — so in a migrated repo it fires precisely
**because the bundle is stale**, and then told the agent to write into the compatibility surface,
which the read-only fence fails the chain for accepting. An agent obeying one guard would have been
stopped by another. The logic was migrated and the message was not, and nothing connected the two.

## Actors & Access

- **Anyone editing a skill, a hook, or the operating block** is the party this governs; the failure
  it prevents is authored, never inherited.
- **A downstream repo with no bundle** never sees it.

## Business Rules

- R1 — In bundle mode, a line on a governed surface that names the retired state layer must carry
  its branch **on that line**; a branch elsewhere in the same document does not exempt it.
- R2 — Exemptions are properties of the line or its location, never a filename list, and each
  carries a negative twin in the self-test.
- R3 — With no bundle the fence is inert and scans nothing.
- R4 — A narrowed surface set is reported by the check with its reason.
- R5 — A guard's *message* is part of that guard: migrating the logic without the message produces
  a guard that routes an agent into a different guard's refusal.

## Edge Cases Settled

- **A line pinned verbatim by an existing test** cannot be branched inline without weakening that
  assertion. Such a line is exempted by its section's branch rather than by editing it — the
  exemption is narrow, measured (it covers exactly one line), and preferred over loosening a test.
- **A cross-reference that names no path** — "startup step 5 is unchanged", pointing at a step that
  has since moved and changed — is invisible to a path-based fence. This limit is stated rather than
  papered over; such drift is still found by reading, and remains the class this fence does not
  cover.

## Open Gaps

- The fence proves a branch is *present* on the line, not that it is *correct*. A line naming both
  trees but describing them backwards passes. Grading meaning remains the rebuild bar's job.

## Pointers (implementation)

- `scripts/okf_instructions_fence.mjs` — classifier, exemptions with their reasons, `--check`,
  `--selftest`, `--json`.
- Wired as two chain entries in `scripts/run_verify.mjs` `EXTRA_SUITES`, pinned in
  `scripts/tests/test_verify_manifest.mjs` (`MANDATORY_SUITES`, `MANDATORY_SUITE_ARGS`).
- The nudge it caught: `hooks/bee-session-close.mjs`, asserted end-to-end in
  `hooks/test_hook_contracts.mjs`.
- Its sibling: `scripts/okf_specs_fence.mjs` (`specs-read-only-fence.md`).
