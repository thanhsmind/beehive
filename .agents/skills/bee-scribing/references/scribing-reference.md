# Scribing Reference

Load after `bee-scribing` is selected. The workflow lives in SKILL.md; the template, per-section rules, and protocols live here.

**The templates below, through Reading Map, are written against the no-bundle spec tree** — `docs/specs/<area>.md`, `system-overview.md`, `reading-map.md` (SKILL.md §2 states the single predicate; do not re-derive it here). **With a bundle**, the equivalent state layer is `docs/knowledge/areas/<area>/*.md` — one `bee.area` concept per subject, authored per SKILL.md §2a via `scribingTarget()`, with `emitFrontmatter` producing every frontmatter block and the full worked examples living in `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md` §Templates. In that mode `docs/specs/` is the **read-only compatibility surface**: a legacy citation resolves through its pointer stub to the concept that owns the anchor now, and it is never written for new content. The nine sections, the rebuild bar, the tech-agnostic rule, and every per-section rule below are the same body contract in both modes (SKILL.md §3) — only the file layout and frontmatter mechanics differ, and each section below states its own bundle counterpart at the point where the two diverge.

## Delegation

Gather sources, map deltas, render sections, harvest inventory, and reading-map refresh delegate as extraction/generation-tier I/O workers per the Delegation contract (`bee-hive/references/routing-and-contracts.md`); any other ad-hoc subagent dispatch scribing makes (for example, a harvest research pass) defaults to the generation slot model, and ceiling requires the `[bee-tier: ceiling]` marker plus a one-line justification.

## Gather Sources — What Each May Feed

| Source | May feed | Never feeds |
|---|---|---|
| capped `behavior_change` cells + `verification_evidence` (`node .bee/bin/bee.mjs cells list --feature <feature>`) | Entry Points & Triggers, Data Dictionary, Behaviors & Operations, Actors & Access | — |
| gate-locked `CONTEXT.md` + active decisions (`node .bee/bin/bee.mjs decisions active`) | Business Rules (cited by D-ID); the `Terms` section seeds the Data Dictionary | Behaviors stated as current reality, unless also evidenced |
| worker reports, UAT records in `docs/history/<feature>/reports/` | Behaviors ("what each actor sees") | — |
| code reading (harvest mode) | observable behavior, field inventory | field *meanings* and rules — code shows what, not why |
| user answers (harvest/capture) | any section, after confirmation | — |

**NEVER invent.** A claim backed by neither verification evidence nor an approved decision enters the spec only as an Open Gap (or becomes a question in interactive mode). Plans describe intent, not reality — never copy from `plan.md`.

## Map Deltas — bundleMode Routing (scribingTarget)

Map each delta to an area by the files/screens it touched; area names are kebab-case, chosen at first write, stable forever. Never decide by eye where an area's truth lives, and never re-state the rule in your own words — one predicate answers it: `bundleMode(root)` in `.bee/bin/lib/knowledge.mjs`, true only when `docs/knowledge/` exists AND at least one concept in it actually parses (a directory holding only `.gitkeep` is NOT a bundle). Ask the module for the exact target:

```
node -e "import('./.bee/bin/lib/knowledge.mjs').then(m=>console.log(JSON.stringify(m.scribingTarget(process.cwd(),{area:'<area>',subject:'<area>: <subject>'}),null,1)))"
```

It returns `{bundle_mode, action, area, subject, path, owner, regenerate_index}` — the same seven keys every time. Write to `path`, do exactly `action`, regenerate the index when `regenerate_index` is true. Pass `intent:'new-concept'` when the subject is believed new — if it is already owned, the answer is `fork_denied` naming the owner, and there is nothing to write.

**A `path: null` answer is a refusal, never a licence to pick your own path.** Three answers refuse:

| `action` | Means | Do |
|---|---|---|
| `fork_denied` | the subject is already owned by the concept in `owner` | update the owner in place, or declare the split inside it |
| `subject_required` | `intent:'new-concept'` with no subject (empty, blank, `null`, punctuation-only) | name the subject and ask again — never routed to `overview.md` |
| `duplicate_authority` | two or more concepts already claim this subject, listed on `owner.conflicts` | fix the bundle first — collapse the rival claims to one authority, then re-ask |

The call **throws** when any concept in the bundle carries a malformed `bee.authoritative_for` (a list, a boolean, an empty/blank string), naming the file — a claim bee cannot read is an owner the anti-fork gate cannot see; fix the concept, do not route around it.

`docs/knowledge/` and `docs/specs/` are both product doc trees — the module resolves them through `resolveProductRoot`, so a repo whose `.bee/config.json` sets `product_root` (the repo-divorce topology) is graded on its real product docs, not the workshop root. Never join these paths yourself.

**Bundle mode — one subject, one concept, forever.** Three paths, gated on `bee.authoritative_for` (the field naming, in one line, the subject a concept is the single truth for):

| Situation | Action | Where |
|---|---|---|
| subject already owned (`bee.authoritative_for` matches, anywhere in the bundle) | update THAT concept in place | the owner's own file — never a second one |
| new subject in an existing area | author a new concept, then regenerate the index | `docs/knowledge/areas/<area>/<subject-slug>.md` |
| brand-new area | create the area with an `overview` concept, then regenerate the index | `docs/knowledge/areas/<area>/overview.md` |

A new concept may NOT claim a subject an existing concept already owns — checked bundle-wide, not per area. When a subject genuinely splits, the owning concept is rewritten and the split is declared inside it (see `docs/knowledge/areas/doctrine-layer/overview.md` §"How this area is split") — never by quietly authoring a rival.

**The anti-fork gate has three layers**, and none may be softened to get a write through — a refused write means the bundle is wrong, not the gate: skeleton matching (NFKC, casefold, confusable fold), malformed-input fail-closed (the throw above), and the bundle-wide `duplicate_authoritative_for` chain-fail. Frontmatter is always produced by `emitFrontmatter`, never typed by hand (see "Bundle-mode gate and frontmatter" below).

**No bundle — the area's spec file.** One area = one file, forever; a modified area is ALWAYS an in-place update. Before creating any spec, check `docs/specs/reading-map.md` and existing `docs/specs/*.md` for an area that already covers this surface (it may be named differently than expected — search by what it describes). Only when nothing covers the surface, create one from the Area Spec Template below. Never create `-v2`, `-new`, or date-suffixed spec files.

## Area Shapes

An area is any long-lived unit with observable behavior: a screen/form, an API, a background job, an integration with an external system, a data pipeline, a CLI command, a business process. The template below fits all of them — the sections stay, the content shifts:

| Section | UI area | Backend/job/API area |
|---|---|---|
| Entry Points & Triggers | links, menu paths, buttons | schedules, events, queue messages, endpoints, CLI invocations |
| Data Dictionary | form fields, display order | inputs, outputs, stored elements, config values, message payloads |
| Behaviors & Operations | user actions (Save, Publish…) | operations and runs (nightly expiry pass, webhook received, import batch) |
| Actors & Access | roles × what they see/do | roles AND consuming/producing systems × what they may call/receive |

A section with genuinely no content for the area's shape gets one line — "Not applicable — <why>" — never silently deleted, so absence reads as a statement, not an oversight.

## Area Spec Template (BA grade)

**With a bundle**, this template's nine sections are the body contract for a `bee.area` concept — write `docs/knowledge/areas/<area>/<subject-slug>.md` (or `overview.md` for a new area) per SKILL.md §2a instead of the file below, and skip straight to Per-Section Rules for the section-by-section content rules, which are unchanged. **With no bundle**, today's guidance stands, unchanged:

Path (no bundle): `docs/specs/<area>.md`. Area name: kebab-case, chosen at first write, stable thereafter. Overwrite/merge freely — this file always describes *now*; history lives in git and `docs/history/`.

In a no-bundle repo, `docs/specs/` holds ONLY this layer's content: area specs, `system-overview.md`, `reading-map.md`, `visuals/`. Never write other artifacts (scripts, exports, survey notes) here; when found, flag them for grooming to relocate — they pollute coverage counting and spec scans.

```markdown
---
area: <area-slug>
updated: YYYY-MM-DD
sources: [<feature-slugs that shaped current behavior>]
decisions: [<active D-IDs cited below>]
coverage: full | partial
---

# Spec: <Area Name>

<One paragraph: what this area is for and who uses it, in business terms.>

## Entry Points & Triggers

<One line per way this area is invoked: route/URL, menu path, link source,
schedule, event, incoming call → what appears or what runs. Business names,
not component or class names.>

- `/jobs/new` → the job posting form (empty)
- `/jobs/<id>/edit` → the same form, pre-filled; visible to the posting's owner only
- every night at 02:00 (posting timezone) → the expiry pass runs over all `active` postings

## Data Dictionary

<Every element a user sees, the area stores, or a consumer receives — form
fields in DISPLAY ORDER; inputs/outputs/config for backend areas.>

| # | Element | Meaning | Values | Required | Default |
|---|-------|---------|--------|----------|---------|
| 1 | Title | The headline applicants see in search results | free text, ≤120 chars | yes | — |
| 2 | Status | Lifecycle of the posting | `draft` — visible to the owner only, never searchable · `active` — publicly listed and accepting applications · `paused` — hidden from applicants, still editable by the owner · `closed` — read-only, kept for records | yes | `draft` |
| — | Expiry window (config, not shown) | How long a posting stays `active` before the expiry pass closes it | days, decided per D9 | — | 60 |

Rules: every enum value carries its business meaning inline; a value whose
meaning nobody can state goes to Open Gaps, not into the table. Derived,
hidden, and config elements get a row too, marked "(not shown)" in the # column.

## Behaviors & Operations

<One block per user action OR system operation. Given/when/then prose, no code.>

### Save (create)

- **Blocked when:** Title empty ("Title is required" shown at the field); …
- **What changes:** a posting is created in `draft`; the owner becomes its editor.
- **Side effects:** none. No notification is sent for drafts.
- **Afterwards:** the owner lands on the edit view with a "Saved" confirmation;
  applicants and other companies see nothing.

### Publish

- **Blocked when:** …
- **What changes:** status `draft` → `active`; the published date is set to today.
- **Side effects:** followers of the company receive a new-job notification (per R3).
- **Afterwards:** applicants find the posting in search; the owner sees it flagged "Live".

### Nightly expiry pass (system operation)

- **Runs when:** every night at 02:00; skipped entirely if the previous night's
  pass is still running (per R4 — never two passes at once).
- **What changes:** `active` postings older than the expiry window become `closed`.
- **Side effects:** the owner receives one summary notification per night, not one
  per posting; applicants with in-flight applications are notified their application
  is frozen.
- **On failure:** the pass stops at the first error, already-closed postings stay
  closed, and the failure is retried the next night; the owner sees nothing partial.

## Actors & Access

<Matrix: every actor — human roles AND consuming/producing systems — × what
they can see, do, call, or receive. Include anonymous visitors when relevant.>

| Capability | Owner | Company admin | Applicant | Visitor | Job-board partner (system) |
|---|---|---|---|---|---|
| See `draft` postings | ✓ | ✓ | — | — | — |
| Edit fields | ✓ | ✓ | — | — | — |
| Apply | — | — | ✓ (active only) | — | — |
| Receive posting feed | — | — | — | — | ✓ (active only, hourly) |

## Business Rules

<Numbered, one sentence each, citing the deciding D-ID and its short8 (Citation
Discipline). Rules live here even when the code enforces them only implicitly.>

- **R1.** A posting can never return from `closed` to any other status (per D4, `b9b9fee3`).
- **R2.** … (per D7, `e230444a`)
- **R3 (not yet implemented — backlog b-12).** …

## Edge Cases Settled

<Edge cases with a decided answer. An open question does not belong here — it
belongs in Open Gaps (harvest) or in exploring (new work).>

## Open Gaps

<Only in `coverage: partial` specs. One line per unknown: what is unknown, and
who/what could answer it. Empty section + `coverage: full` = the rebuild bar is met.>

## Visuals

<UI areas only. One line per settled screen:
`visuals/<area>/<screen>.png` — what it shows. Refreshed at sync when the screen
visibly changed. No snapshot available → say so here or in Open Gaps, never silently.
Backend areas: "Not applicable — no screen.">

## Pointers (implementation)

<THE ONLY technology-bound section. Key files/routes/tables: `path` — role.
Deleting this section must not remove any business meaning.>
```

## Per-Section Rules

- **Purpose:** who uses it and what for. No feature history.
- **Entry Points & Triggers:** if a link, screen, schedule, event, or call exists that this table doesn't explain, the spec fails the rebuild bar.
- **Data Dictionary:** display order is part of the spec for UI areas (which field comes before which). Validation limits live in the Meaning/Values cells in business terms ("≤120 chars"), not as regexes. Config values whose numbers were *chosen* (thresholds, windows, retry counts) cite the deciding D-ID and its short8 (Citation Discipline) — a tuned number without its why is half-lost knowledge.
- **Behaviors & Operations:** the four sub-answers (blocked-when or runs-when / what changes / side effects / afterwards-per-actor) are mandatory for every action and operation; "afterwards" must name what EACH affected actor or consuming system observes, not just the acting user. System operations additionally state their failure behavior (what happens mid-run, what retries, what stays consistent).
- **Actors & Access:** prefer one matrix; consuming/producing systems are actors too; footnote row-level subtleties ("owner of THIS posting, not any owner").
- **Business Rules vs Behaviors:** a Behavior is what the system observably does; a Rule is the policy behind it. A rule approved but not yet shipped is marked "not yet implemented" with a backlog id — never written as a Behavior.
- **Visuals:** the snapshot preserves what the spec cannot say — the settled *look* the vibe loop agreed on by eye. One current image per screen, stable filename, replaced in place (history lives in git). The agent asks the user for a screenshot when it cannot capture one; an absent snapshot is an Open Gap with a stated reason.
- **Pointers:** load-bearing few, not a file listing. This section is allowed to rot slightly; everything above it is not.

## Merge Rules (sync mode)

**With a bundle, "locate before create" resolves through `scribingTarget()`, never a spec scan** (SKILL.md §2): ask it for the area/subject, write exactly the `path`/`action` it returns, and regenerate the index when it says to. Everything else below — deltas never from plan.md, present tense only, a contradicting delta replaces rather than doubles, frontmatter reconciled (via `emitFrontmatter`, not by hand) — is the same discipline in both modes. **With no bundle**, today's guidance stands, unchanged:

- **Locate before create (no bundle):** resolve every delta to an existing spec via `docs/specs/reading-map.md` (and a scan of `docs/specs/*.md` frontmatter/Pointers) before considering a new file. A renamed screen, moved route, or refactored module is still the SAME area — update its spec and its reading-map line; do not fork a new one. Creating is the exception, reserved for genuinely new surfaces.
- Deltas come from `behavior_change` cells + `verification_evidence`, UAT records, and worker reports — never from plan.md, never from memory.
- A delta that contradicts an existing line **replaces** it; do not keep both.
- Update `updated`, append the feature to `sources`, reconcile `decisions` against the active set (`node .bee/bin/bee.mjs decisions active`) — cited by short8 id (see Citation Discipline below), so this reconcile step is itself sweepable.
- Present tense only. "Was", "previously", "changed from" are banned words.
- If the feature added/removed an area, or changed shared entities, the role model, or a cross-area flow: sync `system-overview.md` in the same pass. **With a bundle**, the same duty falls on the area's `overview` concept and the generated area index instead (SKILL.md §3).
- UI areas: when a delta made a screen visibly different, refresh its snapshot under `visuals/<area>/`; cannot produce one → Open Gap with the reason.
- Standard commands are a Pointers-level fact: when a synced change alters how the project is set up, started, tested, or verified, update `.bee/config.json` `commands` in the same pass — one record, never a second location.
- After merging, run the rebuild self-check (below) on every touched spec (or concept).

## Capture Mode in full

The trigger is **settlement**, not subject matter: whenever a discuss → build → test → adjust loop lands on an outcome that is now "how it works" — a business rule agreed, a behavior confirmed by a test run, a retry/threshold/tuning value chosen after experiment, an error-handling policy adjusted — capture it in the same session. When the user says the settlement out loud — "chốt", "final", "ok ship it", any equivalent — capture happens in that same turn, never deferred. What "capture" costs in that turn is lane-scaled: high-risk = the full spec merge; every other lane = decision log + a one-line queue stub, with the merge at flush — the flow is never held hostage to the elaboration. The session-close hook warns when a decision exists that no spec update followed, and when queued stubs await their flush.

**The debt signal backs this up.** Every `behavior_change` cell capped since the last scribing run is counted as *scribing debt* and surfaced mechanically — in the session preamble, in `bee_status`, and in the chain-nudge fired when a worker returns during swarming. Debt > 0 means a settlement already landed in a capped cell and belongs in a spec now, not at feature close. Self-detection is still the first duty; the debt count is the backstop for the settlements the agent's own watching missed. Running capture (or sync) and recording the run in state clears it.

**Flush — draining the queue.** Flush points, whichever comes first: wrap-up (the working session is ending), the PreCompact/close warning (the hook fires when the queue is non-empty), or the session-start offer (bee-hive surfaces a non-empty queue before new work). At flush: `node .bee/bin/bee.mjs capture list`, then oldest-first give each stub the full capture treatment — merge into its area's spec per the Merge Rules above, `bee.mjs capture flush --id <id> --into <spec>` — and record the scribing run in state. A stub is never dropped, summarized away, or flushed without its merge; if a stub's meaning is no longer reconstructable, ask the user rather than invent — that cost is the signal to flush earlier next time.

If a settlement contradicts current shipped behavior, record it as a rule with a note "not yet implemented — see backlog" and file a backlog item; do NOT state it as current behavior.

## Citation Discipline

Any artifact that encodes a decision — a spec's Business Rules line, a `docs/backlog.md` row's Story/CoS, a CONTEXT/plan passage — cites the decision's **short8 id** (the log entry's id, first 8 hex chars, e.g. `b9b9fee3`) alongside any CONTEXT-local label (`D4`, `D11b`); the label alone is not enough. The `decisions supersede` propagation sweep matches short8 word-boundary hits across `docs/**` — it finds only what is cited that way, so a passage carrying only a `D4`-style label is invisible to the scan. An uncited embodiment is the residual risk: the decision changes, but nothing points a sweep at the passage that assumed it.

## Harvest Interview Protocol

**Harvest mode, three steps:** (1) inventory the area from code and running behavior — screens, fields, actions, roles, or for backend areas: triggers, inputs, outputs, consumers, failure paths; (2) draft the spec with everything code can *prove*, every meaning or rule code cannot prove becomes a question; (3) unanswered questions → `## Open Gaps`, `coverage: partial` — a partial spec that states its gaps beats an invented-complete one.

For each meaning/rule code cannot prove, ask in the standard question format — one per message, outcome-framed, single-choice preferred:

```text
CONTEXT: The job form has a Status field with values draft/active/paused/closed.
  The code only shows that `paused` postings are excluded from search.
QUESTION: When a posting is paused, what should the applicant who already
  applied see?
RECOMMENDATION: (b) — matches the exclusion already enforced in search.
  (a) The posting stays visible to them — their application is in flight
  (b) The posting shows as "no longer available" — applications freeze
  (c) Something else (describe)
```

Budget the interview: batch the inventory first, then ask only the questions whose answers change the spec. Unanswered → Open Gaps + `coverage: partial`. Confirmed answers in harvest/capture mode are decisions — log them (`bee.mjs decisions log`) and cite the new D-ID in the spec.

## Bootstrap Mode

**No-bundle only.** A repo with a bundle has no equivalent bootstrap: `docs/knowledge/index.md` and `docs/knowledge/areas/index.md` are pure functions of the bundle's own concepts, regenerated on demand with `node .bee/bin/bee.mjs knowledge index` — never hand-bootstrapped from a skeleton. Bootstrap exists for one situation: `docs/specs/` lacks `system-overview.md` or `reading-map.md` — typically a repo fresh from onboarding, before any harvest has run. It is **offered, never auto-run**: the agent names the missing file(s) and asks; only user approval starts the pass. Bootstrap creates ONLY the missing map file(s) — an existing `system-overview.md` or `reading-map.md` is never touched by bootstrap (in-place-never-fork holds; improving an existing map belongs to sync or harvest).

Binding rules:

- **Sources:** code/tree inspection and verbatim README extracts only. Nothing else feeds a skeleton — no plan.md, no memory, no inference from file or symbol names.
- **Never invent:** every meaning, purpose, or rule that code cannot mechanically prove is an Open Gap line, never a written claim. A plausible-sounding guess is worse than a stated gap.
- **`coverage: partial`, always:** every bootstrap output carries `coverage: partial` in frontmatter — a skeleton by definition fails the rebuild bar, and says so.
- **No interviews:** bootstrap asks the user nothing about meaning. Meaning-filling belongs to harvest mode — bootstrap is inventory, harvest is meaning.
- **Loud gaps:** the output states its own gaps explicitly — a populated Open Gaps section plus `[unknown]` markers inline — so the Fresh Session Test probe (grooming) and harvest inherit a concrete worklist, never a silent hole.

**Tech-agnostic collision rule (binding):** directory paths live only in reading-map lines and Pointers sections. A system-overview area-map line whose purpose cannot be stated in business terms carries an `[unknown]` gap marker instead of a path-derived guess. A README quote that names technology goes to Pointers or becomes a gap — never into the Purpose paragraph.

**Skeleton shape — `system-overview.md`** (standard overview template, filled only where provable):

- Purpose: the README's first paragraph as a quoted extract with stated provenance ("README, opening paragraph, verbatim") when it speaks in business terms; otherwise one `[unknown]` gap line — never a paraphrase presented as fact.
- Area Map: one stub line per top-level structural unit and entry point the tree proves, phrased in business terms where provable; a line that cannot be carries `[unknown — see Open Gaps]`.
- Shared Entities, Actors & Roles, Cross-Area Flows: section headers kept, containing only what code proves — usually a single Open Gap pointer each.
- Open Gaps: one line per unfilled meaning, naming who or what could answer it (usually "harvest interview").
- Pointers: the entry points and technology facts the tree proves.

**Skeleton shape — `reading-map.md`:** one line per top-level location, each with a mechanically derived one-liner (manifest fields, script names, an unambiguous README statement) or an `[unknown]` gap marker — never an invented description. `spec:` cross-references appear only for spec files that actually exist.

A completed bootstrap announces its gap count and offers harvest as the next step for meaning-filling.

## Rebuild Checklist (self-check before finishing)

Cover the Pointers section and verify:

1. Every entry point and trigger (link, screen, schedule, event, call) is listed with what appears or runs.
2. Every visible field, input, output, and chosen config value appears in the dictionary — display order for UI, meanings everywhere; every enum value has a stated business meaning.
3. Every user action and system operation has a Behavior block with all four sub-answers (operations also state failure behavior).
4. Every actor — human role or consuming system — appears in the access matrix.
5. No sentence requires reading the code to be understood.
6. No technology name appears above Pointers.
7. `coverage` and Open Gaps are honest.
8. UI areas: every settled screen has a current snapshot under `visuals/<area>/` — or an Open Gap saying why not.
9. If this spec's area is new, removed, or changed shared entities/roles/flows: `system-overview.md` reflects it.

Any failure: fix it now, or file it as an Open Gap with `coverage: partial` — silently shipping a hole is the red flag, not having one.

## System Overview Spec

**With a bundle**, there is no separate system-overview file to author: the cross-area glue is the area's own `overview` concept plus the generated `docs/knowledge/areas/index.md`, kept current by regenerating the index (`bee.mjs knowledge index`) after any area or concept change — never hand-edited. Fresh sessions read the bundle's root index FIRST (`docs/knowledge/index.md`), before any area concept. **With no bundle**, today's guidance stands, unchanged:

Path (no bundle): `docs/specs/system-overview.md`. One file, singular — the cross-area glue no per-area spec owns. Same write discipline as any spec (present tense, overwrite to match reality, tech-agnostic above Pointers, never fork). Fresh sessions read it FIRST, before any area spec.

```markdown
---
area: system-overview
updated: YYYY-MM-DD
decisions: [<active D-IDs cited below>]
coverage: full | partial
---

# Spec: System Overview

<One paragraph: what the product is, for whom, in business terms.>

## Area Map

<One line per area: what it is for, where its spec lives. This is the
completeness ledger — an area with shipped behavior and no line here is a gap.>

- job-posting-form — where owners create and manage postings; spec: job-posting-form.md
- applicant-inbox — where applicants track applications; spec: applicant-inbox.md (partial)

## Shared Entities

<Business entities that two or more areas read or write, with their meaning and
which areas touch them. Per-area field detail stays in the area specs.>

| Entity | Meaning | Touched by |
|---|---|---|
| Posting | A job opening a company offers | job-posting-form (owns), applicant-inbox (reads), partner-feed (reads) |

## Actors & Roles (global)

<The role model stated ONCE: every human role and consuming system, one line on
what it is. Area specs reference these names; they never redefine them.>

## Cross-Area Flows

<One block per flow spanning two or more areas: trigger → step per area →
outcome each actor observes. Single-area behavior stays in the area spec.>

## Open Gaps

## Pointers (implementation)
```

Sync triggers: a feature adds or removes an area; a shared entity's meaning changes; the role model changes; a cross-area flow is created, removed, or rerouted. Anything else NOOPs — the overview is glue, not a duplicate of the area specs.

## Reading Map

**With a bundle**, the generated indexes carry the per-area map instead (SKILL.md §7) — run `node .bee/bin/bee.mjs knowledge index` after any new concept or new area (the run is a pure function of the bundle, so re-running it is always safe), and keep the hand-written reading map below pointing only at the areas that exist. **With no bundle**, today's guidance stands, unchanged:

Path (no bundle): `docs/specs/reading-map.md`. One line per location, grep-friendly:

```markdown
# Reading Map

- `src/auth/` — session middleware and guards; spec: docs/specs/auth.md
- `scripts/build.mjs` — single build entry point; run with `node scripts/build.mjs`
```

At sync time: add lines for locations the feature created or repurposed, fix lines it made wrong, delete lines for removed locations. Keep it a map, not documentation — one line each, no prose blocks.

## Product Backlog (`docs/backlog.md`)

`docs/backlog.md` is the **product backlog** — the human-first, priority-ordered view of product backlog items (PBIs): stories the product owner wants. It is a **generated view**, never a store: the ONE store is `.bee/backlog.jsonl`, where PBIs live as event-sourced records (`{ts, kind:"pbi", event:"add"|"status"|"amend", id, ...fields}`) in the same append-only stream that already holds friction/grooming events — one concept, one file, current state = fold by id, last-event-wins per field. `docs/backlog.md` is rendered from that fold by `node .bee/bin/bee.mjs backlog render --write`; it is never edited by hand and never edited by scribing directly — scribing's ownership of the product backlog is expressed entirely through the CLI verbs below, the same way it owns specs by writing through the tooling that owns the file.

**Structure — the rendered table, priority-ordered (highest first):**

```markdown
# Product Backlog

| ID | Story | CoS | Status | Feature |
|----|-------|-----|--------|---------|
| P1 | Owners can pause a posting without closing it | A paused posting is hidden from applicants but still editable by the owner | done | job-pause |
| p-3f9a2b11 | Applicants get a weekly digest of matching postings | One email per week lists new matches; opt-out honored | in-flight | applicant-digest |
| p-7c1e0a44 | Companies can archive closed postings out of the default list | Closed postings move to an Archive view, restorable within 30 days | proposed | — |
```

- **Columns:** `ID` (stable — legacy rows keep their `P<n>`; every new PBI gets a collision-free `p-<8hex>` generated by `backlog pbi add`, never a hand-picked next integer) · `Story` (one line, user-facing outcome) · `CoS` (Condition of Satisfaction — the one-line acceptance signal) · `Status` · `Feature` (the `docs/history/<feature>/` slug once opened, `—` while unstarted).
- **Status enum — `proposed | in-flight | parked | done | declined`** — five values, no others. This is `PBI_STATUSES` in `.bee/bin/lib/backlog.mjs`; do not invent a sixth status.
- **Priority order is the row order in the generated view** (`proposed`/`in-flight`/`parked` render as full rows; `done`/`declined` collapse to one-line links so the view stays short forever) — nothing here is hand-reordered.

**Verbs (scribing-owned, specs pattern — CLI-owned, never hand-edited):**

- **Append, never fork.** A new deferred request is captured with `node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` (prints the generated id) — an `add` event, `proposed` by default. There is never a second backlog file and never a hand-inserted row.
- **In place forever.** A PBI's fields are updated by appending a new event for its id (`pbi amend` for title/cos, `pbi status` for status/feature); history lives in the event stream and in git, never in a "was proposed" note.
- **Flip triggers are the only status writes, and they are prose-ruled, never hook-enforced:**
  - **(a) exploring opens a feature matching a row** → `node .bee/bin/bee.mjs backlog pbi status --id <id> --to in-flight --feature <slug>` (one move, status + slug together); if the request never passed through the backlog, exploring runs `pbi add` first, then the status flip (owned by exploring).
  - **(b) feature close** (scribing sync, or compounding when no `behavior_change` cell ran) → `node .bee/bin/bee.mjs backlog pbi status --id <id> --to done` once every CoS clause has cited evidence (owned by scribing at sync). Partial delivery never silently flips: leave the row `in-flight` and run `pbi amend --id <id> --cos "<original CoS> — Delivered: <clause(s) shipped>; Remaining: <clause(s) owed>"` instead; split the remainder into a new `pbi add` row when the delivered subset is independently shippable.
- **No validation coupling.** A cell may carry an optional `pbi` field naming a row ID; a missing or stale reference is a grooming find, not a cap blocker.
- **Rendering:** after any event, `node .bee/bin/bee.mjs backlog render --write` regenerates `docs/backlog.md` from the current fold (deterministic, no timestamp); `backlog render --check` reports drift without writing. The render owns the view.

**Runnable surfaces already exist — reference them, never re-describe machinery here:** `node .bee/bin/bee.mjs status --json` reports `pbi: { proposed, in_flight, parked, done, declined } | null`, and the session preamble carries one line naming the counts whenever `.bee/backlog.jsonl` holds `kind:"pbi"` events. The token-cheap query for current state is `node .bee/bin/bee.mjs backlog pbi list --json` (the fold — never a `docs/backlog.md` read). Drift (an `in-flight` row with no active feature, a `done` feature with no row, duplicate rows for one story) is caught by grooming's audit, not by any hook.

## State Record

```json
{
  "phase": "scribing",
  "summary": "Synced 2 area specs (job-posting-form full, applicant-inbox partial, 3 gaps)",
  "next_action": "Invoke bee-compounding."
}
```

`bee-compounding` checks this record as its state-layer guard; if scribing has not run for the feature, compounding invokes it rather than syncing inline.

## Bundle-mode gate and frontmatter

**The gate has three layers, because exact string matching on free text can never be sufficient.**

1. **The match is a skeleton, not a string.** Subjects are compared after NFKC, lowercasing, accent stripping, a cross-script confusable fold, and punctuation/whitespace collapse. Neither a trailing period nor a Cyrillic `е` can buy a rival concept.
2. **Malformed input fails closed** — the three refusals and the throw in §2 above. A silently skipped claim is a fork with extra steps.
3. **The bundle-wide backstop bites.** `duplicate_authoritative_for` is a chain-**failing** finding in `bee knowledge check` (no `--strict` needed), grouped by the same skeleton. Layer 1 cannot catch a genuine word-order paraphrase (`refunds and reversals` vs `reversals and refunds`) — nothing that compares strings can — so the bundle-wide check is what refuses to let two authorities coexist. `malformed_authoritative_for` fails the chain the same way.

Do not soften any layer to get a write through. A refused write means the bundle is wrong, not the gate.

**Frontmatter is ALWAYS produced by `emitFrontmatter`, never typed by hand** — hand-written blocks are caught `not_canonical` by the round-trip guard. Build the data object, emit, then write body under it:

```
node -e "import('./.bee/bin/lib/knowledge.mjs').then(m=>process.stdout.write(m.emitFrontmatter({type:'bee.area',title:'...',description:'...',tags:['...'],timestamp:'YYYY-MM-DD',bee:{id:'...',lifecycle:'active',areas:['<area>'],required_context:[],decisions:[],sources:[],authoritative_for:'<area>: <subject>'}})))"
```

Body sections, the rebuild bar, and the `bee.areas` vs `bee.authoritative_for` distinction: the `bee.area` template in `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md` §Templates. After any new concept or new area, regenerate the indexes — `node .bee/bin/bee.mjs knowledge index` — and confirm the bundle still grades clean: `node .bee/bin/bee.mjs knowledge check`.

## Merge rules


- Present tense only. "Was", "previously", "changed from" are banned — history lives in git and `docs/history/`.
- A delta that contradicts an existing line **replaces** it; never keep both.
- Every enum value in the Data Dictionary carries its business meaning ("`paused` — hidden from applicants, still editable by the owner"). A value without a meaning is an Open Gap, not a table row.
- Every Behavior block answers: what triggers it, what blocks it, what changes, what side effects fire, and **what each actor or consuming system observes afterwards**.
- Business Rules are numbered (R1, R2…) and cite the active D-ID that decided them.
- UI areas: refresh the settled snapshot when the screen visibly changed (ask the user for one if you cannot produce it); a UI area with no current snapshot records that as an Open Gap, never silently. **With no bundle** the snapshot lives under `docs/specs/visuals/<area>/`, unchanged. **With a bundle there is no snapshot home yet, and this skill does not invent one:** the compatibility surface is read-only for new content (`scripts/okf_specs_fence.mjs` fails the chain) and the bundle profile defines no visuals location. Until one is decided, record the missing snapshot as an **Open Gap** in the area's concept — naming the screen and stating that the bundle has no visuals home — and never write the image into the retired tree. The gap itself is tracked in `docs/knowledge/areas/okf-profile/concept-model-and-authoring.md` §Open Gaps.
- If the feature added or removed an area, or changed shared entities, the role model, or a cross-area flow: sync `docs/specs/system-overview.md` in the same pass (template in the reference). In bundle mode the same duty falls on the area's `overview` concept and the area index.
- Update frontmatter: `updated`, append to `sources`, reconcile `decisions`, set `coverage: full | partial` honestly. In bundle mode this means re-emitting the whole block through `emitFrontmatter` with `timestamp` refreshed and `bee.sources`/`bee.decisions` extended — never hand-editing a line of it.

## Deferred requests


The same unprompted-capture duty covers **deferred work**, not just settled truths. When the user pushes work out of the current scope — "để sau", "phase 2", "later", "not now" — or a Deferred Idea leaves exploring, the agent appends a `proposed` PBI **in the same turn, announce-then-do**: "ghi vào backlog: <story> (proposed)", then `node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` followed by `node .bee/bin/bee.mjs backlog render --write` so `docs/backlog.md` stays current. A user having to say "ghi vào backlog" means detection already failed once. `backlog pbi add`/`backlog render` are `.bee/`-layer writes through the CLI — allowed in every phase, no gate; `docs/backlog.md` itself is never hand-edited (it is CLI-owned, exact-path write-guard deny). The id/columns/verbs live in the reference's Product Backlog section; do not duplicate the table schema here. This is prose-ruled, never hook-enforced.

At sync, close the loop the other way: when this scribing run closes a feature that matches a backlog row, check the flip against the row's CoS before writing anything — enumerate every CoS clause and cite the delivered evidence per clause. Only when every clause has cited evidence does the row flip via `node .bee/bin/bee.mjs backlog pbi status --id <id> --to done` and link `docs/history/<feature>/` (added via `pbi amend --id <id> --cos "..."` if the link belongs in the CoS text) — the sync pass owns the done-flip. Any clause without evidence means the row does NOT flip: run `node .bee/bin/bee.mjs backlog pbi amend --id <id> --cos "<original CoS> — Delivered: <subset shipped>; Remaining: <subset owed>"` instead, leaving status `in-flight`; when the delivered subset is independently shippable, split the remainder into a new `pbi add` row rather than stranding it. Silent full-flip on partial delivery is never allowed. After any row flip, run `node .bee/bin/bee.mjs backlog render --write` so the generated view stays honest, and, when README carries the badge block, `node .bee/bin/bee.mjs backlog badges --write`.

