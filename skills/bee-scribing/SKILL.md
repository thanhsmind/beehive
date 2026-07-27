---
name: bee-scribing
description: >-
  Keep technology-agnostic BA specs of every area current, so a human understands the system without the code and an agent can rebuild it on another stack. SELF-TRIGGERING: invoke this yourself, unprompted, the moment any discussion-test-adjust loop settles a rule, behavior, or value — the user should never have to ask for knowledge to be recorded. Also use when execution completes (chain), when the user asks to document a screen/API/job/area, or when a legacy area has code but no spec.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    nodejs-runtime:
      kind: command
      command: node
      missing_effect: degraded
      reason: Reads cell traces and logs decisions via the vendored .bee/bin helpers.
---

# Scribing (scribe bees)

Scribing is bee's BA. It owns the state layer. An **area is domain-general**: a screen or form, an API, a background job, an integration, a data pipeline, a CLI command, a business process — any unit with observable behavior that outlives features. Code is the implementation; the spec is the *meaning* — it must survive a full rewrite on a different stack (decision 0002).

**Where that meaning is written depends on one thing only: whether this repo has a knowledge bundle** (§2). In a repo that has one, the state layer is `docs/knowledge/areas/<area>/*.md` — one `bee.area` concept per subject. In a repo that has none, the state layer is exactly what it has always been: `docs/specs/<area>.md` (one BA-grade functional spec per long-lived area), `docs/specs/system-overview.md` (the cross-area glue — area map, shared entities, global roles, cross-area flows; decision 0003), `docs/specs/visuals/<area>/` (settled screen snapshots for UI areas), and `docs/specs/reading-map.md`. Everything else in this skill — the rebuild bar, the tech-agnostic rule, the nine sections, the modes, the triggers, the never-invent rule — is identical in both.

§1 gather sources, §2 map deltas, §3 render sections, harvest inventory, and §7 reading-map refresh delegate as extraction/generation-tier I/O workers per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); any other ad-hoc subagent dispatch scribing makes (for example, a harvest research pass) defaults to the generation slot model, and ceiling requires the [bee-tier: ceiling] marker plus a one-line justification.

**The rebuild bar (acceptance test for every spec):** a competent agent given ONLY this spec — with the Pointers section deleted — rebuilds the same observable behavior on a different technology. A human reading it understands every field, behavior, rule, and role without opening the code.

**The tech-agnostic rule:** outside the final `Pointers (implementation)` section, a spec names NO language, framework, library, class, table, component, or file. Fields, screens, roles, actions, jobs, and messages are named in business vocabulary. "The React hook debounces and PATCHes /api/jobs" is a violation; "edits are saved automatically shortly after typing stops" is the spec. "A Celery beat task scans the `applications` table" is a violation; "every night, applications idle for 30 days are marked expired and the applicant is notified" is the spec.

## Modes

| Mode | Trigger | Does |
|---|---|---|
| **sync** (chain default) | execution completed with `behavior_change` cells capped (scribing follows execution directly — a feature may be scribed and closed while unreviewed; independent review is a separate, user-invoked session) | merge the feature's behavior deltas into the touched areas' specs |
| **capture** | any discuss → build → test → adjust loop **settles an outcome**, any phase — a rule agreed, a behavior confirmed by test, a threshold/tuning value chosen, an error policy adjusted; an explicit user settlement signal ("chốt", "final", "ok ship it") makes capture **mandatory in the same turn** (decision 0003) | log the decision same turn, then: **high-risk lane → merge into the spec immediately**; every other lane → append a capture stub (`node .bee/bin/bee.mjs capture add`) and keep working — the merge happens at flush (decision 0017) |
| **flush** | capture queue non-empty at a flush point — session wrap-up, the PreCompact/close warning, or the session-start offer (decision 0017) | drain the queue oldest-first: full merge of each stub into its area's spec, mark it flushed (`bee.mjs capture flush --id <id> --into <spec>`), record the scribing run |
| **harvest** | user asks to document an existing area, or grooming files a missing-spec item | write the first spec for an area built before/outside bee |
| **bootstrap** | **no-bundle repos only** — `docs/specs/` lacks `system-overview.md` or `reading-map.md`, typically right after onboarding. A repo WITH a bundle has no equivalent bootstrap: its `docs/knowledge/index.md` and `areas/index.md` are pure functions of the concepts and are regenerated (`bee.mjs knowledge index`), never skeletoned | **offer — never auto-run** (D2 of harness10) a bounded skeleton pass creating ONLY the missing map file(s) from mechanically provable facts; an existing map file is never touched. Full binding rules + skeleton shapes: the reference's Bootstrap section |

Bootstrap is inventory, harvest is meaning: bootstrap writes only what code, tree, and verbatim README extracts prove, marks every meaning as an Open Gap (`coverage: partial`), and asks no interview questions — its loudly stated gaps are harvest's worklist.

**Sync mode runs after goal-check, not instead of it:** `standard`/`high-risk` `behavior_change` cells are already judged by the semantic checklist judge (D4, table in `bee-hive/references/routing-and-contracts.md`) before scribing sees them — that judge is goal-check verification, not the separate user-invoked review session this mode's trigger already distinguishes.

## 1. Gather Sources — and What Each May Feed

| Source | May feed | Never feeds |
|---|---|---|
| capped `behavior_change` cells + `verification_evidence` (`node .bee/bin/bee.mjs cells list --feature <feature>`) | Entry Points & Triggers, Data Dictionary, Behaviors & Operations, Actors & Access | — |
| gate-locked `CONTEXT.md` + active decisions (`node .bee/bin/bee.mjs decisions active`) | Business Rules (cited by D-ID); the `Terms` section seeds the Data Dictionary | Behaviors stated as current reality, unless also evidenced |
| worker reports, UAT records in `docs/history/<feature>/reports/` | Behaviors ("what each actor sees") | — |
| code reading (harvest mode) | observable behavior, field inventory | field *meanings* and rules — code shows what, not why |
| user answers (harvest/capture) | any section, after confirmation | — |

**NEVER invent.** A claim backed by neither verification evidence nor an approved decision enters the spec only as an Open Gap (or becomes a question in interactive mode). Plans describe intent, not reality — never copy from `plan.md`.

## 2. Map Deltas to Areas — Update in Place, Never Fork

Map each delta to an area by the files/screens it touched. Area names are kebab-case, chosen at first write, stable forever.

**Never decide by eye where the area's truth lives, and never re-state the rule in your own words.** One predicate answers it: `bundleMode(root)` in `.bee/bin/lib/knowledge.mjs` — true only when `docs/knowledge/` exists **and at least one concept in it actually parses**. A directory alone is not a bundle: a repo whose `docs/knowledge/` holds nothing but a `.gitkeep` is **not** in bundle mode. Ask the module for the exact target rather than reasoning about paths:

```
node -e "import('./.bee/bin/lib/knowledge.mjs').then(m=>console.log(JSON.stringify(m.scribingTarget(process.cwd(),{area:'<area>',subject:'<area>: <subject>'}),null,1)))"
```

It returns `{bundle_mode, action, area, subject, path, owner, regenerate_index}` — the same seven keys in every mode and on every answer. Write to `path`, do exactly `action`, and regenerate the index when `regenerate_index` is true. Pass `intent:'new-concept'` when you believe the subject is new — if it is in fact already owned, the answer is `fork_denied` naming the owner, and there is nothing to write.

**A `path: null` answer is a refusal, and a refusal is never a licence to pick your own path.** Three answers refuse:

| `action` | Means | Do |
|---|---|---|
| `fork_denied` | the subject is already owned by the concept in `owner` | update the owner in place, or declare the split inside it |
| `subject_required` | `intent:'new-concept'` was passed with no subject (empty, blank, `null`, punctuation-only) | name the subject and ask again — "no subject" is not a new subject, and it is **never** routed to `overview.md` |
| `duplicate_authority` | two or more concepts already claim this subject; every claimant is listed on `owner.conflicts` | fix the bundle first — collapse the rival claims to one authority, then re-ask |

And the call itself **throws** when any concept in the bundle carries a malformed `bee.authoritative_for` (a list, a boolean, an empty or blank string), naming the file. That is not noise to route around: a claim bee cannot read is an owner the anti-fork gate cannot see. Fix the concept.

`docs/knowledge/` and `docs/specs/` are both **product** doc trees — the module resolves them through `resolveProductRoot`, so a repo whose `.bee/config.json` sets `product_root` (the repo-divorce topology) is graded on its real product docs, not the workshop root. Never join these paths yourself.

### 2a. Bundle mode — one subject, one concept, forever

Three paths, and the choice between the first two is gated on `bee.authoritative_for` — the field that names, in one line, the subject a concept is the single truth for:

| Situation | Action | Where |
|---|---|---|
| the subject is already owned by a concept (`bee.authoritative_for` matches, anywhere in the bundle) | update THAT concept in place | the owner's own file — never a second one |
| a new subject in an area that already exists | author a new concept in that area, then regenerate the index | `docs/knowledge/areas/<area>/<subject-slug>.md` |
| a brand-new area | create `docs/knowledge/areas/<area>/` with an `overview` concept, then regenerate the index | `docs/knowledge/areas/<area>/overview.md` |

**A new concept may NOT claim a subject an existing concept already owns.** This is the anti-fork gate, and it carries the weight the one-file-per-area rule used to carry alone: two concepts claiming one subject both parse, both list in the index, and no reader can tell which is true — the `-v2` failure in a new costume. Ownership is checked bundle-wide, not per area: a subject owned by a concept in another area still routes there. When a subject genuinely splits, the owning concept is rewritten and the split is declared in it (see `docs/knowledge/areas/doctrine-layer/overview.md` §"How this area is split") — never by quietly authoring a rival.

**The anti-fork gate has three layers** — skeleton matching (NFKC, casefold, confusable fold), malformed-input fail-closed, and the bundle-wide `duplicate_authoritative_for` chain-fail — and none may be softened to get a write through: a refused write means the bundle is wrong, not the gate. **Frontmatter is always produced by `emitFrontmatter`, never typed by hand** (the round-trip guard catches hand-written blocks `not_canonical`). Layer detail, the emit command, and the post-write index/check steps: the reference ("Bundle-mode gate and frontmatter").

### 2b. No bundle — the area's spec file

**One area = one file, forever.** A modified area is ALWAYS an in-place update to its existing spec — that is what keeps the doc permanently current. Before creating any spec, check `docs/specs/reading-map.md` and the existing `docs/specs/*.md` for an area that already covers this surface (it may be named differently than you'd name it today — search by what it describes, not by the name you expect). Only when no existing spec covers the surface, create one from the template in `references/scribing-reference.md`. Never create `-v2`, `-new`, `-updated`, or date-suffixed spec files: two documents describing one area is worse than a stale one — readers cannot tell which is true.

## 3. Merge — BA-Grade Sections

Sections (full template + per-section rules in the reference): **Purpose → Entry Points & Triggers → Data Dictionary → Behaviors & Operations → Actors & Access → Business Rules → Edge Cases Settled → Open Gaps → Pointers (implementation)**. The same sections fit every area shape — for a UI area the triggers are links and clicks and the data is form fields; for a backend area the triggers are schedules, events, and calls, and the data is inputs, outputs, and stored elements. **The same nine, in the same order, are the body contract for a `bee.area` concept** (§2a): a concept covers the sections its subject has content for and says nothing where it has nothing, but it never invents a different set of headings. Splitting an area into concepts must not quietly downgrade body quality to whatever the author felt like — format-green is not quality-green.

Merge rules — present tense only, contradictions replace, enums carry business meaning, behaviors answer the five questions, rules are numbered and cite D-IDs, UI snapshots and system-overview sync, frontmatter re-emitted whole: the reference ("Merge rules").

## 4. Capture Mode — Settled Outcomes from the Vibe Loop

The trigger is **settlement**, not subject matter: whenever a discuss → build → test → adjust loop lands on an outcome that is now "how it works" — a business rule agreed, a behavior confirmed by a test run, a retry/threshold/tuning value chosen after experiment, an error-handling policy adjusted — capture it in the same session. When the user says the settlement out loud — "chốt", "final", "ok ship it", any equivalent — capture happens **in that same turn**, never deferred (decision 0003). What "capture" costs in that turn is lane-scaled (decision 0017): high-risk = the full spec merge; every other lane = decision log + a one-line queue stub, with the merge at flush — the flow is never held hostage to the elaboration. The session-close hook warns when a decision exists that no spec update followed, and when queued stubs await their flush.

**The debt signal backs this up (decision 0011).** Every `behavior_change` cell capped since the last scribing run is counted as *scribing debt* and surfaced mechanically — in the session preamble, in `bee_status`, and in the chain-nudge fired when a worker returns during swarming. Debt > 0 means a settlement already landed in a capped cell and belongs in a spec **now**, not at feature close. Self-detection is still the first duty; the debt count is the backstop for the settlements the agent's own watching missed. Running capture (or sync) and recording the run in state clears it.

**Detection is the scribe's duty, unprompted (decision 0007).** The explicit signal is the *loud* case; most settlements are silent — the user confirms a behavior works, accepts an explanation, picks an option, moves on. The agent watches for these itself, every turn, and captures without being asked. Do not ask "should I document this?" — announce in one line what settled and where it goes ("chốt: X — ghi vào <the area's file, resolved per §2> + decision log"), then do it in the same turn. **The close-audit habit, zero exceptions:** at every task close — cell, docs-lane write, or a plain quick fix with no cell at all — ask "what settled here?" and either capture it or state "nothing settled"; task smallness is never the answer, and a close with neither is not a close. Capture writes only `docs/` and `.bee/` — allowed in every phase, no gate. A user having to say "ghi lại" means detection already failed once:

1. Log it first: `node .bee/bin/bee.mjs decisions log --decision "..." --rationale "..."` — the decision log is the durable anchor; the rationale records *why* this outcome won over what was tried. This is always same-turn, every lane.
2. **High-risk lane:** merge the settled truth into the area's spec now (Business Rules for policy; Behaviors & Operations for confirmed behavior; Data Dictionary for a value's meaning) citing the new D-ID, same message. A spec lagging high-risk behavior even briefly is dangerous — never queue it.
   **Every other lane (decision 0017):** append a stub instead — `node .bee/bin/bee.mjs capture add --outcome "..." --did <D-IDs> [--area <area>] [--files ...]` — one line, seconds, then keep working. Durability now, elaboration at flush.
3. If it contradicts current shipped behavior, record it as a rule with a note "not yet implemented — see backlog" and file a backlog item; do NOT state it as current behavior.

Litmus: if the session ended right now, would this outcome exist anywhere but the chat? If no — capture it now. (A queued stub passes the litmus — the chat can die and the stub survives into the next session's preamble.)

### Flush — draining the queue (decision 0017)

Flush points, whichever comes first: **wrap-up** (the working session is ending), the **PreCompact/close warning** (the hook fires when the queue is non-empty), or the **session-start offer** (bee-hive surfaces a non-empty queue before new work). At flush: `node .bee/bin/bee.mjs capture list`, then oldest-first give each stub the full capture treatment — merge into its area's spec per the section-3 rules, `bee.mjs capture flush --id <id> --into <spec>` — and record the scribing run in state (section 8). A stub is never dropped, summarized away, or flushed without its merge; if a stub's meaning is no longer reconstructable, ask the user rather than invent — that cost is the signal to flush earlier next time.

### Deferred requests

A request the user parks ("để sau", "not now") becomes a product-backlog row in the same turn — never a silent drop. Protocol: the reference ("Deferred requests").

## 5. Harvest Mode — Backfill Without Inventing

1. Inventory the area from code and running behavior: screens, fields, actions, roles — or for backend areas: triggers, inputs, outputs, consumers, failure paths.
2. Draft the spec with everything code can *prove*; every meaning or rule code cannot prove becomes a question — Socratic style, one question per message, outcome-framed.
3. Unanswered questions → `## Open Gaps`, `coverage: partial`. A partial spec that states its gaps beats an invented-complete one.

## 6. Rebuild Self-Check

Before finishing, re-read the spec with the Pointers section covered and ask: could a stranger rebuild this on another stack? Any "you'd have to look at the code" answer is a hole — fix it or file it as an Open Gap.

## 7. Refresh the Reading Map

**With no bundle** — `docs/specs/reading-map.md`: add lines for locations created or repurposed, fix lines made wrong, delete lines for removed locations. One line each; a map, not documentation.

In bundle mode, the generated indexes carry the per-area map instead — run `node .bee/bin/bee.mjs knowledge index` after any new concept or new area (the run is a pure function of the bundle, so re-running it is always safe), and keep the hand-written reading map pointing at the areas that exist.

## 8. Update State

Record the scribing run: `node .bee/bin/bee.mjs state scribing-run --feature <feature> --areas "<a,b>" --next-action "<next action>"`. This stamps `last_scribing_run` (`feature`, `date`, an **ISO-precise `at` timestamp**, `areas_synced`, `next_action`) and mirrors `next_action` plus advances `phase` to `compounding` at the top level. The `at` stamp is what clears **scribing debt** (decision 0011): the harness counts `behavior_change` cells capped *after* it, so a missing or day-only stamp leaves just-synced cells still showing as debt. No `behavior_change` cells and nothing to capture → still run it (`--areas "none"`, `--next-action` reflecting "scribing: no sync needed") so the debt signal resets.

## Hard Gates

- Do NOT skip scribing when `behavior_change` cells were capped — in ANY lane, tiny included; lanes scale ceremony, never memory (vision principle 11). An unsynced spec is measured entropy (grooming counts it).
- Do NOT name technology outside Pointers. The rebuild bar is the acceptance test, not a slogan.
- Do NOT state unverified claims as behavior. Evidence → behavior; approved decision → rule; neither → Open Gap.
- Do NOT create a second spec for an existing area. Modification = in-place update of the one true file; check the reading map before every create.
- Do NOT create a second concept for a subject an existing concept already claims via `bee.authoritative_for`. Ownership is checked bundle-wide, before authoring, every time (§2a).
- Do NOT decide bundle mode by looking at the tree, by `existsSync`, or by restating the rule in prose. `bundleMode` is the only answer; a `docs/knowledge/` holding only a `.gitkeep` is not a bundle.
- Do NOT hand-write concept frontmatter. `emitFrontmatter` produces every block, always.
- Do NOT let a settled outcome die in the chat log — capture mode exists precisely for it, whatever the domain (UI, backend, integration, process).
- Secrets and PII never appear in specs.

## Headless

`mode:headless`: apply mechanical merges (deltas straight from `behavior_change` cells + evidence) and reading-map fixes; log capture-mode decisions only when the user's wording is verbatim-quotable. Harvest questions, ambiguous merges, and any rewording beyond the delta go to an `Outstanding Questions` section of the structured terminal report.

## Red Flags

- a framework, library, or file path in any section above Pointers
- a status/enum value listed without its business meaning
- a Behavior block that never says what each actor or consumer observes
- spec content copied from plan.md or written from memory
- a `-v2`/`-new`/date-suffixed spec file, or a fresh spec created without checking the reading map for the existing one
- a new concept authored for a subject another concept already claims, or an `authoritative_for` line copied from a sibling concept
- bundle mode decided by eye or by an `existsSync` on `docs/knowledge/` instead of by `bundleMode`
- a concept whose frontmatter was typed by hand rather than emitted
- a new concept or new area left without regenerating the indexes
- harvest answers invented from field or symbol names instead of asked
- "I'll write the spec after compounding" — scribing runs first, while evidence is fresh
- a settled outcome (rule, confirmed behavior, chosen value) that exists nowhere but the chat
- the user said "chốt"/"final" and the turn ended with no decision logged and neither a spec merge nor a queued stub (decision 0017: the stub is the same-turn minimum outside high-risk)
- a high-risk settlement queued as a stub instead of synced inline
- a capture stub surviving past a flush point (wrap-up, PreCompact warning, session-start offer) without being flushed
- a capture that ran only because the user asked "ghi lại" — a silent settlement the agent should have caught itself (decision 0007)
- the user deferred work ("để sau", "phase 2", "later") and the turn ended with no `proposed` row appended to `docs/backlog.md` — the missed-capture failure applied to backlog items (D8)
- asking "should I document this?" instead of announcing the capture and doing it
- a UI screen that visibly changed while its snapshot did not, and no Open Gap says why — with no bundle the snapshot is `docs/specs/visuals/`; with a bundle there is no home yet, so the Open Gap IS the required output
- an area added or removed with `system-overview.md` left unsynced
- treating scribing as UI-only — backend jobs, APIs, integrations, and processes are areas too

Violating the letter of these rules is violating the spirit of these rules.

## Handoff

Scribing complete: <N> area specs synced (<coverage>), <M> open gaps, reading map refreshed. Invoke bee-compounding skill.

| Reference | When to Load |
|---|---|
| `references/scribing-reference.md` | full spec template, per-section rules, merge rules, bundle-mode gate + emitFrontmatter, deferred-requests protocol, field-dictionary and visibility-matrix formats, harvest interview protocol, bootstrap rules, rebuild checklist |
