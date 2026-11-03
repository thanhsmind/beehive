---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval>
---

# Plan: SLP Blind Lanes

Mode: `high-risk` — 4 risk flags: data-model, public-contracts,
covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the change adds a
REFUSAL to the dispatch door — the chokepoint every worker in the repo passes
through — and a new record type whose content is later cited as evidence. A
wrong refusal blocks all dispatch; a wrong citation check makes a dossier lie.

## Requirements (from CONTEXT.md)

- **D1** — the agent opens 2–3 blind lanes on its own judgment when a decision
  is high-stakes AND ambiguous, logging the reason at open time; the user may
  order lanes directly; deadlock always hands the user the dossier.
- **D2** — a procedure over the existing dispatch door: (a) one
  neutrality-linted LaneBrief, lint enforced at the door; (b) 2–3 parallel
  `--kind advisor` dispatches, byte-identical brief, explicit read-only path
  diet; (c) cross-critique as a second advisor round handing each lane the
  rival proposal verbatim; (d) convergence = dossier doc + one decisions-log
  entry + a registered revisit trigger; (e) deadlock hands the user the dossier
  via `waiting-on --kind question` or a human-mailbox letter.
- **D3** — blind lanes never run as `--kind cell` (learned_context leaks).
- **D4** — every citation in the dossier must resolve against the verbatim lane
  proposals, checked mechanically by string containment.
- **D5** — an objection is valid only when it names the specific missing context.
- **D6** — the 5-Layer rubric, the Truth Table Test and the CRUD Lifecycle check
  join the reviewer/judge checklist material.
- **D7** — hats critique one request from fixed perspectives; lanes generate
  designs from a byte-identical brief. Distinct instruments.

## Discovery

Two read-only reviewer dispatches re-verified the 2026-08-26 research digest
against HEAD `95d1273e`. Four findings change the shape:

1. **No flag carries free caller text into a non-cell prompt body.**
   `prepare.rs:564-567` renders every non-cell template with an EMPTY vars
   slice; `advisor.md`'s `Paths: <caller fills in…>` is literal prose, not a
   placeholder. `--purpose` is a 60-char label only (`prepare.rs:1106-1119`,
   `DESCRIPTION_TITLE_MAX = 60`, `prepare.rs:205`); `--expertise` is parsed then dropped for
   non-cell kinds (`prepare.rs:591`). A LaneBrief therefore needs a new flag
   AND the first non-empty vars slice at `prepare.rs:566`.
2. **`purpose_is_gather` does NOT enforce read-only.** `prepare.rs:115-117`
   returns `kind != "cell"`, and its only consumer is a model-slot gate for cli
   entries (`models.rs:394`); the `--claim` refusal gates on the kind string
   directly (`prepare.rs:1743-1750`). An advisor dispatch is read-only by PROSE
   (`packages/bee/prompts/advisor.md`), enforced nowhere. The `lane`-kind
   conclusion below survives — a new kind still buys nothing — but not for that
   reason: what makes a blind lane blind is that bee injects no store context
   into a non-cell payload, not a read guard.
3. **A new `lane` kind is expensive.** The kind string is matched at 25 references across 8 files
   and is itself the prompt filename (`prepare.rs:1733`), across three pinned
   constants: `DISPATCH_KINDS: [&str; 4]` (`prepare.rs:31`), `PROMPT_NAMES:
   [&str; 4]` (`devtools/prompts.rs:40`), and the registry payload's `kind`
   enum. `PINNED_FLAG_COUNT = 196` (`catalog.rs:632`) additionally gates any new
   flag.
4. **Nothing today carries a read diet into any payload.** Zero code hits for
   allowed-paths across the repo; `advisor.md`'s "Read-only." is prose bee does
   not enforce. D2(b)'s diet is new work, not a reuse.
5. **The lint precedent is exact and current**: `matches_supersession_prose`
   (`decisions/verbs_read.rs:313-353`) and `matches_deferral_prose` (`:365-430`)
   are word-bounded, case-insensitive, hand-scanned guards — no regex crate in
   the workspace. There is NO shared word-list constant: each guard inlines its
   own literal stems at the use site and calls the shared primitives in
   `decisions/scanners.rs` (`is_word:25`, `boundary_before:29`,
   `starts_with_ci:33`, `ws_run:51`). A neutrality lint defines its own set the
   same way and reuses those four primitives.
6. **The deadlock channels are thinner than the digest implied.**
   `waiting-on` has no field for a linked document — `subject` is free-form
   (`workflow_store/record.rs:396-401`), so the dossier path can only ride in
   the subject prose, and the CLI's own kind enum is `["gate","question"]`
   (`registry_payload.json`, enforced by `hooks/cli_shape.rs:362-380`). The
   mailbox is worse: `needs_you[]` exists in the letter contract
   (`mailbox.rs:444-448`) but `KIND_BLOCKER` (`mailbox.rs:511`) has NO producer
   outside tests, every real call site passes an empty vector
   (`handlers_close.rs:832`, `close.rs:2646`), and there is no `mailbox write`
   verb at all. A deadlock letter must WIRE that path, not just call it.
7. **`alternatives` is a flat `Option<String>`** (`verbs_read.rs:270`, written at
   `:694-697`) with six readers and no record versioning or schema check on read.
   A structured rejected set is therefore cheap in the store and costs exactly
   one `PINNED_FLAG_COUNT` bump plus `bee dev regen`.

Two in-repo docs carry stale anchors for the same matcher and will mislead
anyone who copies them — `docs/history/doc-deferral-baseline/CONTEXT.md:58` and
`docs/history/doc-impact-synthesis/plan.md:110`. Filed separately; not this
feature's work.

## Approach

**Recommended path** (shape B, settled by decision `f0f21142`). Build blind
lanes as a procedure over the EXISTING `--kind advisor`, with no new store and
no new command family. `dispatch prepare --kind advisor --brief-file <path>`
carries the LaneBrief into the payload and the lint refuses at that door. The
lane-opening reason (D1) is logged through the existing `bee decisions log`. The
dossier document itself holds every lane proposal verbatim, so a program has real
bytes to check. ONE new verb, `bee blind check <dossier>`, runs three mechanical
checks over that document's fixed sections: D4's citation check, brief-digest
equality across lanes, and the read-diet check. Byte-identity (D2b) is VERIFIED
rather than constructed — `dispatch prepare` stamps the brief's sha256 on the
dispatch record it already returns, the dossier records one digest per lane, and
`blind check` refuses when they differ.

**The lint, stated honestly.** One shared guard function runs at two callers —
`dispatch prepare --brief-file` (the chokepoint refusal D2(a) locks) and
`bee blind check`, which re-runs it over the dossier's recorded brief so a
convergence built on an unlinted brief refuses — one shared function, one test
asserting both callers use it. Its scope is EXACTLY the brief bytes, in both
callers. It never reads `--purpose`,
`--expertise`, or any other dispatch text: a false fire on those would block the
advisor consult Gate 3 itself requires (`high_risk_advisor_refusal`,
`set_gate.rs`) and deadlock the high-risk workflow. The guard has two arms: a
narrow verdict-stem scan ("I recommend", "the right answer", "we should pick"),
and a SHAPE rule that refuses a brief enumerating candidate answers — lanes
exist to generate options, so a brief that lists them has already led the
witness. The shape arm is where the real leaning lives; the lexical arm catches
only the lazy leak. Its refusal text and every skill line say "leaning language
refused", never "neutrality enforced". Adding the shape arm beside the lexical
one is delivery-plus within D2(a), not a supersession.

**Blindness is stated, not enforced — so breach becomes evidence.** bee's hooks
guard writes and secrets, never reads, so a lane can read anywhere the
orchestrator wrote its leaning. Shape B removes one such path by not creating a
run store, but it does not remove the hazard: D1's open reason now lands in
`.bee/decisions.jsonl`, which sits on the same disk. Two cheap teeth close it
without a read hook: every LaneBrief's read diet excludes `.bee/` by
construction, and `bee blind check` REFUSES a dossier whose lane sections report
a paths-read entry outside that diet or naming `.bee/` at all. The advisor prompt
already obliges a lane to return the paths it read, so a silent breach becomes a
typed refusal or a recorded lie. State the trust level honestly: this is NOT the
same trust level as D4's citation check. D4 checks bytes the checker holds, so a
fabricating lane is caught whether or not it cooperates; the diet check reads the
lane's OWN paths-read list, so a lane that reads `.bee/decisions.jsonl` and omits
it passes clean. The diet check is a prompt instruction plus a confession
requirement. What IS structural is that `prepare.rs:563-566` injects zero store
context into a non-cell payload, so a breach takes active defiance of the
prompt. Convergence renders a dossier and RECORDS — never prints — one
`bee decisions log` entry with a registered trigger. The rejected set rides
today's flat `--alternatives` string, in the fixed form
`<lane-id>: <one-line reason>; <lane-id>: <reason>`. That is the single answer to
CONTEXT.md's deferred question 2; the structured `--rejected` flag is a slice-3
UPGRADE of the same field, not a competing plan, and it carries its own flag
bump. Deadlock reuses
`waiting-on --kind question` and the human-mailbox letter.

**Rejected alternatives.**
- A new `--kind lane` — rejected: ~15 match sites and three pinned constants
  bought nothing D2 requires. `advisor` is already the blind shape because bee
  injects no store context into a non-cell payload (`prepare.rs:564-567`), and a
  new kind would inherit exactly the same non-enforcement of read-only-ness.
- Reusing `--expertise` as the brief carrier — rejected: it means "paths to
  read, with purpose", the read diet, not the question. Overloading it would
  make the diet and the brief one field that the lint cannot scan cleanly.
- A `bee blind` namespace over a `.bee/blind/` run store (shape A) — rejected by
  the user at the shape gate (`f0f21142`): byte-identity by construction did not
  justify a new command family, six new flag spellings, new registry entries, a
  new served-but-undeclared scanner arm, and a readable on-disk copy of the
  orchestrator's leaning.
- A shared forbidden-word constant across both prose guards and the new lint —
  rejected: neither existing guard has one, the vocabularies do not overlap
  (deferral stems versus leaning language), and merging them would put three
  refusals behind one list.
- Keeping proposals only in the orchestrator's message history — rejected: D4's
  citation check must be mechanical, and a check against remembered text is the
  fabrication it exists to catch.
- Heterogeneous lane models — stays out of scope by the map, but see Out of
  scope: the blocker the map cited has dissolved.

**SMALLER PATH check.** Asked and answered: reusing `--kind advisor` instead of
adding a `lane` kind removes ~15 match sites and two pinned array types while
honoring D2 in full — a `lane` kind would carry the same empty-vars, no-store
context payload the advisor kind already carries, and the same unenforced
read-only prose. No cheaper shape exists below this: a prose-only procedure
would drop D2(a)'s door refusal and D4's mechanical check, both locked.

**Risk map.**

| Component | Risk | Reason | Proof needed |
|---|---|---|---|
| The lint at the door | HIGH | A guard and its tests are one model (`docs/knowledge/patterns/20260812-a-guard-and-its-tests-are-one-model-so-green-proves-only-that-the-model-agrees-with-itself.md`). Worse here: a false fire on the wrong input blocks the advisor consult Gate 3 requires, deadlocking the high-risk workflow that approves guards | Red-first refusal per arm; a scope test proving the lint NEVER reads `--purpose`/`--expertise`; a corpus run over every `packages/bee/prompts/*.md` and every checked-in brief-shaped doc asserting zero fires; one test asserting both doors call the same function |
| Word list co-tuned with its corpus | MEDIUM | The zero-false-fire corpus and the stem list are authored together and will agree with each other rather than with reality | The shape arm carries the load; the lexical arm's stems are frozen in the plan and any later addition needs its own recorded reason |
| Leaning readable on disk | MEDIUM | D1 forces an open reason that states the orchestrator's suspicion, and it lands in `.bee/decisions.jsonl` on the same disk the lanes can read | Every diet excludes `.bee/` by construction; `bee blind check` refuses a dossier whose lane sections report a paths-read entry outside the diet or naming `.bee/` |
| Editing `advisor.md` without re-vendoring | HIGH | `prompts_match_disk` (`prompt.rs:72-89`) byte-compares the embedded template against BOTH `packages/bee/prompts/advisor.md` and `.bee/bin/prompts/advisor.md`; on skew `prepare.rs:1733-1735` returns `None`, and the Node delegate it would fall to was deleted at the R6 cutover. A source-only edit breaks EVERY `dispatch prepare` in this checkout | The prompt edit and `bee dev regen` land in ONE commit, with a probe asserting `prompts_match_disk(root, "advisor")` after the edit |
| First non-empty vars slice for a non-cell template | LOW | Proven feasible: `prompt.rs:156` treats an absent var as falsy and `render_conditional_blocks_leave_no_residue` pins it, so a `{{brief}}` INSIDE a `{{#if brief}}` block leaves today's bytes untouched when no brief is passed. A `{{brief}}` outside a block is a hard refusal (`prompt.rs:176-181`), and the block must not start at byte 0 (`prompt.rs:118`) | `embedded_prompts_are_the_checked_out_files_byte_for_byte`, `c4_embedded_prompts_match_disk`, `real_templates_render_end_to_end` and the runtime×kind label walk all green; a brief-absent render byte-identical to today |
| Empty-vs-whitespace brief | MEDIUM | `{{#if}}` truthiness is `!v.is_empty()` (`prompt.rs:156`), so a whitespace-only brief splices an empty block instead of rendering today's bytes | The carrier trims before it renders; the whitespace-only probe asserts the trimmed-empty render is byte-identical to no-brief |
| New flag vs `PINNED_FLAG_COUNT` | MEDIUM | `catalog.rs:271-282` demands a written per-flag reuse check before a bump, and `bee dev regen` does NOT write `src/generated/registry_payload.json` — that file is hand-maintained (`devtools/mod.rs:141-150`; both dissent commits edited it by hand) | The full chain in one commit: hand-edit `registry_payload.json` keeping `examples[0]` runnable (`tests/registry_dispatch.rs:50-80` executes it), bump `PINNED_FLAG_COUNT` with the recorded reuse reason, parse the flag, rebuild, `bee dev regen` to re-vendor. Gated by `distinct_flag_vocabulary_is_pinned_so_growth_is_a_decision`, `registry_dispatch.rs`, `registry_contracts.rs` |
| The dossier as the record | MEDIUM | A hand-written document is the only place proposals live, so a malformed section makes the whole check unrunnable | `bee blind check` parses the fixed section set and refuses a malformed dossier by NAME of the missing section, never by silently checking less |
| Citation check | HIGH | Plain containment has three separate holes, not one: a short generic quote ("the dispatch door") is inside every proposal; a NEGATION strip inverts meaning (proposal says "we should NOT cache the token", the dossier cites "cache the token" and passes); and containment against the concatenated set lets a citation attributed to lane A but written only by lane B pass — misattribution IS the fabrication D4 exists to catch | A citation is `<lane-id> :: <quote>`, checked against THAT lane's normalized bytes only, with a minimum quote span. Four red-first probes, one per hole: short-generic refuses, fabricated refuses, negation-strip refuses, cross-lane misattribution refuses; a real quote differing only in whitespace passes |
| Deadlock hand-off | MEDIUM | Only one of the two channels exists. `waiting-on --kind question` works but carries the dossier path as prose only; the mailbox's `needs_you[]` blocker path has no producer and must be wired | A `question` mark naming the dossier path, plus the first real `KIND_BLOCKER` producer with its own letter-render test |
| Skill prose (D1, D5, D6, D7) | MEDIUM | Instruction text is an untested code path (critical pattern, 2026-08-21) | `--test instruction_laws --test pointer_integrity` green |

## Answers to CONTEXT.md's deferred questions

1. **Where the lint attaches.** On the existing `--kind advisor`, at
   `dispatch prepare --brief-file`, as one shared guard function that
   `bee blind check` also calls, with one test asserting both callers use it. No
   new kind.
2. **The rejected set.** Today's flat `--alternatives` string, in the fixed form
   `<lane-id>: <one-line reason>; <lane-id>: <reason>`. The structured
   `--rejected` flag is a later upgrade of the same field.
3. **Where the dossier lives.** `docs/history/<feature>/blind/<run-id>.md` when
   the run carries a feature; `docs/history/blind/<run-id>.md` when it does not.
   One rule, one fallback. Fixed sections, in order: `# Blind lane run <run-id>`,
   `## Question` (the brief's Question verbatim), `## Lanes` (one `### <lane-id>`
   each, carrying its `dispatch_id`, its `brief_sha256`, its role, its paths
   read, and the proposal verbatim), `## Cross-critiques`, `## Chosen`,
   `## Rejected` (one line per rejected lane with its reason), `## Citations`
   (`<lane-id> :: <quote>` per line), `## Revisit trigger`.

   The per-lane `dispatch_id` is what gives `bee blind check` a chain of
   custody: without it the check compares the orchestrator's own transcribed
   digests against each other, which verifies the transcriber against itself. It
   reads the authoritative record instead — `.bee/logs/dispatch.jsonl`, written
   by `append_prepare_record` (`prepare.rs:601-613`) — and refuses when a lane's
   recorded digest does not match its dispatch_id's logged one. That log is
   FAIL-OPEN by design ("a log failure never blocks the payload",
   `prepare.rs:599-600`), so a dispatch_id absent from it refuses by name rather
   than passing silently.
4. **The lint's vocabulary, frozen here so it can be reviewed before it is
   written.** Two arms, both word-bounded and ASCII case-folded, reusing
   `decisions/scanners.rs` (`is_word:25`, `boundary_before:29`,
   `starts_with_ci:33`, `ws_run:51`).
   - *Verdict-stem arm*: `i recommend`, `we recommend`, `my recommendation`,
     `i prefer`, `we prefer`, `i lean`, `leaning toward`, `leaning towards`,
     `the correct answer`,
     `the obvious answer`, `the obvious choice`, `clearly the best`,
     `obviously better`, `we should pick`, `we should use`, `you should pick`,
     `you should use` — seventeen. `the right answer` and `the right approach`
     were cut at the re-consult: they collide with neutral interrogative
     phrasing ("What is the right approach for X?"), which the other impersonal
     stems have no natural use for. That set is frozen: a later addition needs its own
     recorded reason, so the list can never be quietly shrunk to make a corpus
     test pass.
   - *Shape arm*: the brief must carry exactly four `##` sections — Question,
     Constraints, Read diet, Digest contract — and the Question section must
     contain no enumerated list (a line starting `-`, `*`, or `<n>.`). A brief
     that lists candidate answers has already led the witness. This arm is where
     real leaning lives; the verdict-stem arm catches only the lazy leak.
   - *Cap*: a brief over 8192 bytes refuses.
   - *Red-first per arm*, plus a corpus test scoped to the VERDICT-STEM ARM
     ONLY: that arm runs over every file matching `packages/bee/prompts/*.md`
     and must fire zero times (verified today: zero of the stems appear in any
     of them). The shape arm is deliberately excluded from the corpus — no
     prompt file carries the four required sections, so a whole-guard corpus
     would fire on every one of them and force a silent re-scoping. That
     exclusion is the reason the corpus cannot be quietly co-tuned with the stem
     list.

## Shape

**Feature outcome.** One hard question goes in; 2–3 blind proposals, a
cross-critique round, and one dossier + decision + trigger come out — with the
brief's leaning language refused at the door and every dossier citation checked
against real proposal bytes.

**Repo-reality basis.** Advisor dispatches are already isolated by construction
(`purpose_is_gather`, `prepare.rs:115-117`). The four genuinely missing pieces
are the brief carrier, the lint, the run record, and the convergence check.

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| E1 | LaneBrief carrier + the lint at the dispatch door | D2(a); today no caller text reaches a non-cell prompt body and nothing lints it | 1a | Red-first typed refusal per arm, a lint-scope test, a zero-false-fire corpus run |
| E2 | The brief digest on the dispatch record | D2(b) byte-identity has to be checkable, and the dispatch record is the one artifact prepare already returns | 1a | Two prepares over one file give one digest; an edited file gives two |
| E3 | `bee blind check` — the dossier contract, citation check, digest equality, diet check | D2(d), D4, and the blindness teeth | 1b, 3 | Fabricated, short-generic, negation-strip and cross-lane citations all refuse red-first |
| E4 | Cross-critique round, the read diet, and the `--kind cell` refusal | D2(b) read diet, D2(c) round two, D3 | 2 | Round-2 payload carries the rival proposal verbatim; a blind brief on `--kind cell` refuses |
| E5 | Deadlock hand-off | D2(e) | 3 | `waiting-on --kind question` carries the dossier path; unattended writes a letter |
| E6 | Blind-lane procedure prose | D1, D5, D7 | 4 | `instruction_laws`, `pointer_integrity` green |
| E7 | Reviewer/judge checklist material | D6 | any | `instruction_laws`, `pointer_integrity` green — independent of E1–E6 |

**Slice queue.**

- **Slice 1a — the door (current).** `dispatch prepare --kind advisor
  --brief-file <path>` carries the brief into the advisor payload through a
  `{{#if brief}}` block in `advisor.md`, the lint refuses a leaning brief on
  either arm at that door, and the returned dispatch record gains the brief's
  sha256. One commit carries the prompt edit AND `bee dev regen`, because a
  source-only prompt edit breaks every `dispatch prepare` in this checkout. Ends
  end-to-end and usable: three lanes can be fired on one linted brief by hand
  today. Depends on nothing.
- **Slice 1b — `bee blind check`.** The dossier's fixed section contract, the
  lane-scoped citation check with its minimum quote span, brief-digest equality
  across lanes, and the read-diet check. Cut from 1a because the checks are their
  own risk surface with their own red-first proofs; welding them to the door
  change makes one big-bang gate out of two independent verifications. Depends on
  slice 1a for the digest.
- **Slice 2 — cross-critique and the read diet.** Round-2 payloads carrying the
  rival proposal verbatim; the read-diet list carried into the advisor payload
  beside the brief; the `--kind cell` refusal for a blind brief (D3). Depends on
  slice 1a.
- **Slice 3 — deadlock and the rejected set.** A `waiting-on --kind question`
  mark carrying the dossier path in its subject, wiring the first real
  `KIND_BLOCKER` producer so an unattended run files a letter whose "Needs your
  call" section actually renders, and a structured rejected set (`--rejected`,
  list-typed like `--tags`) on the convergence decision. Costs the full flag
  chain: a hand-edit to `registry_payload.json`, a `PINNED_FLAG_COUNT` bump with
  its recorded reuse reason, and `bee dev regen`. Depends on slice 1b.
- **Slice 4 — blind-lane procedure prose.** When the agent opens lanes and logs
  the reason (D1), pushback names the missing context (D5), hats are not lanes
  (D7), and — the rule that makes slice 1b bite — convergence RUNS
  `bee blind check` green before it logs the decision. No door forces that today;
  the prose is what forces it until one does. Depends on slices 1a–3 landing so the prose describes shipped behavior.
- **Slice 5 — reviewer/judge checklist material (D6).** The 5-Layer rubric, the
  Truth Table Test and the CRUD Lifecycle check. Depends on NOTHING: D6 is
  reviewer craft, not blind-lane behavior, so it must not sit undelivered behind
  a stalled slice 1.

Only slice 1a becomes cells at this gate.

## Test matrix

High-risk: probes per applicable edge dimension. Each cell's writer judges
existing coverage first and authors only the gap.

| Dimension | Probe | Applies to |
|---|---|---|
| 2 Input extremes | Empty brief, whitespace-only brief (trimmed-empty must render byte-identically to no brief), a brief at exactly 8192 bytes and one over, unicode/RTL text, a brief whose verdict stem appears inside a quoted code sample | E1 |
| 5 State transitions | `converge` before any proposal; `proposal add` after converge; the same lane id twice; converge with 1 proposal (below the 2–3 range) | E2, E3 |
| 3 Timing | Two `proposal add` calls racing on one run — append or clobber | E2 |
| 7 Error cascades | Lint refuses mid-batch: are the already-fired lanes reported, and is the run left readable | E1, E2 |
| 9 Data integrity | Two lanes rendered from one run id produce byte-identical payloads; a run id that names no record refuses | E1, E2 |
| 5 State transitions (lint scope) | `dispatch prepare` with `--purpose`/`--expertise` carrying every stem on the lint's list, and no `--blind` — the lint MUST NOT fire | E1 |
| 10 Integration | `dispatch prepare` with no brief renders byte-identically to today, on every runtime × kind pair | E1 |
| 6 Environment | The vendored `.bee/bin/prompts/advisor.md` matches the embedded template after the edit — `prompts_match_disk(root, "advisor")` is true, asserted as a unit probe rather than by the release-manifest verify obligation, which is not a `cargo test` target | E1 |
| 12 Business logic | Exactly 1, 2, 3 and 4 lanes — the 2–3 rule's boundaries | E2 |
| 1, 4, 8, 11 | Not applicable — no user tiers, no scale surface, no authorization boundary, no PII path beyond bee's existing secret scan, which the brief inherits | — |

Guard-specific, from the critical pattern: the lint's proof is NOT its own
fixture. Slice 1a carries a corpus test that runs the lint over every checked-in
prompt and brief-shaped doc in the repo and asserts zero fires, and a scope test
proving the lint never reads any dispatch text but the brief and the open reason.

## The scope question, settled

D2's rationale reads *"a PROCEDURE over the existing dispatch door, not new
machinery — the two genuinely missing pieces are the brief lint and a structured
rejected-set"*. An earlier draft of this plan added a `bee blind` namespace over
a `.bee/blind/` run store, which is machinery that rationale excluded. Planning
raised it rather than deciding it.

The user picked shape B, recorded as decision `f0f21142`: no new store, no new
command family. The brief rides `--brief-file`, the open reason rides the
existing decision log, the dossier holds the proposals, and one new verb runs the
checks. The accepted cost is named in that decision — a brief edited between lane
1 and lane 3 is caught by digest equality at convergence rather than made
impossible by construction.

## Advisor consult

`docs/history/slp-blind-lanes/advisor-consult.md` — two consults, both SAFE WITH
NAMED CHANGES.

Round 1 named four changes. Three are folded here: the lint scope, byte-identity
no longer left to a re-read path, and the honest "leaning language" claim plus
the shape arm. The fourth — "lint the open reason" — was deliberately DROPPED,
not folded: an open reason is inherently a statement of why the agent suspects
something, so linting it for leaning would refuse every honest one. The teeth
that remain on it are the diet exclusion and the paths-read check. Round 1's
recommended cut at `converge` and its D6-independence fix are both in.

Round 2 re-read the settled shape-B plan and the two drafted cells and named
three text-level changes, all folded: the dossier carries a per-lane
`dispatch_id` so `bee blind check` reads the authoritative dispatch log instead
of the orchestrator's own transcription; the corpus test is scoped to the
verdict-stem arm, because the shape arm would fire on every prompt file in it;
and `bln-1` states what `--brief-file` does on a non-advisor kind. It also cut
two stems from the frozen list and corrected the diet check's claimed trust
level. Round 2 confirmed every code anchor in both cells against HEAD.

## Known red base

`p-624e2d7d` — the declared suite is RED on any machine running opencode-ai
newer than CI's pinned 1.18.16 (`every_registered_write_or_read_capable_opencode_tool_is_mapped_or_named_as_a_gap`
panics on the 1.18.21 tool-id literal). Not this feature's work, and CI is green
on its pin. No blind-lanes cell may claim a full-suite green locally until it
lands; cells prove themselves with scoped runs and CI runs the declared command.

## Out of scope

- Heterogeneous lane models. The map defers them (`4faf1de9`), and that stands.
  Recorded finding, not a scope change: the cited blocker has dissolved —
  `--role` now keys the advisor branch (`prepare.rs:985`, pinned by
  `the_advisor_slot_follows_the_role_not_the_kind` at `prepare.rs:2617`), so
  three lanes CAN already differ by role. Reopening it is the user's call.
- A new `--kind lane` at the dispatch door.
- Enforcing the read diet as a hook-level read guard — bee's hooks guard writes
  and secrets, never reads. The diet ships as a carried, stated list.
- Building SLP as a standalone six-agent layer (`787a9eb0`).
- Relaxing R2, R3 or R4.
