# Area Spec Reference

Load after `bee-capturing` is selected and a spec must be written or
merged. The judgment lives in SKILL.md; the template, per-section
rules, and protocols live here.

## Where the truth is written

One predicate decides the state layer: a repo has a **bundle** when
`docs/knowledge/` holds at least one concept that actually parses (a
directory holding only `.gitkeep` is not a bundle).

- **With a bundle:** area truth lives in
  `docs/knowledge/areas/<area>/*.md`, one concept per subject. Ask the
  CLI for the write target — it returns the exact path and action, and
  refuses forks, blank subjects, and duplicate authorities with the
  fix named in the refusal. Frontmatter is always emitted by the CLI's
  frontmatter helper, never typed by hand. After a new concept or
  area, regenerate the indexes (`bee knowledge index`) and confirm the
  bundle grades clean (`bee knowledge check`).
- **With no bundle:** area truth lives in `docs/specs/<area>.md` plus
  `system-overview.md`, `reading-map.md`, and `visuals/`. One area =
  one file, forever. `docs/specs/` holds only this layer's content —
  scripts, exports, or notes found there are flagged for grooming to
  relocate.

The nine sections, the rebuild test, the tech-agnostic rule, and every
rule below are the same body contract in both modes — only the file
layout and frontmatter mechanics differ.

## Gather Sources — what each may feed

| Source | May feed | Never feeds |
|---|---|---|
| capped `behavior_change` cells + verification evidence | Entry Points & Triggers, Data Dictionary, Behaviors & Operations, Actors & Access | — |
| gate-locked `CONTEXT.md` + active decisions | Business Rules (cited); Terms seed the Data Dictionary | Behaviors stated as current reality, unless also evidenced |
| worker reports, UAT records in `docs/history/<feature>/reports/` | Behaviors ("what each actor sees") | — |
| code reading (harvest) | observable behavior, field inventory | field *meanings* and rules — code shows what, not why |
| user answers (harvest/capture) | any section, after confirmation | — |

**Never invent.** A claim backed by neither verification evidence nor
an approved decision enters the spec only as an Open Gap (or becomes a
question in interactive mode). Plans describe intent, not reality —
never copy from `plan.md`.

## Area Shapes

An area is any long-lived unit with observable behavior. The template
fits all of them — the sections stay, the content shifts:

| Section | UI area | Backend/job/API area |
|---|---|---|
| Entry Points & Triggers | links, menu paths, buttons | schedules, events, queue messages, endpoints, CLI invocations |
| Data Dictionary | form fields, display order | inputs, outputs, stored elements, config values, message payloads |
| Behaviors & Operations | user actions (Save, Publish…) | operations and runs (nightly expiry pass, webhook received, import batch) |
| Actors & Access | roles × what they see/do | roles AND consuming/producing systems × what they may call/receive |

A section with genuinely no content for the area's shape gets one
line — "Not applicable — <why>" — never silently deleted, so absence
reads as a statement, not an oversight.

## Area Spec Template

Path (no bundle): `docs/specs/<area>.md`. Area name: kebab-case,
chosen at first write, stable forever. Overwrite/merge freely — this
file always describes *now*; history lives in git and `docs/history/`.
With a bundle, these nine sections are the body contract of the area
concept; the per-section rules below are unchanged.

```markdown
---
area: <area-slug>
updated: YYYY-MM-DD
sources: [<feature-slugs that shaped current behavior>]
decisions: [<active decision ids cited below>]
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
| — | Expiry window (config, not shown) | How long a posting stays `active` before the expiry pass closes it | days, chosen per the expiry decision (`b9b9fee3`) | — | 60 |

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

### Nightly expiry pass (system operation)

- **Runs when:** every night at 02:00; skipped entirely if the previous night's
  pass is still running (never two passes at once).
- **What changes:** `active` postings older than the expiry window become `closed`.
- **Side effects:** the owner receives one summary notification per night;
  applicants with in-flight applications are notified their application is frozen.
- **On failure:** the pass stops at the first error, already-closed postings stay
  closed, and the failure is retried the next night; the owner sees nothing partial.

## Actors & Access

<Matrix: every actor — human roles AND consuming/producing systems — × what
they can see, do, call, or receive. Include anonymous visitors when relevant.>

| Capability | Owner | Company admin | Applicant | Visitor | Job-board partner (system) |
|---|---|---|---|---|---|
| See `draft` postings | ✓ | ✓ | — | — | — |
| Apply | — | — | ✓ (active only) | — | — |
| Receive posting feed | — | — | — | — | ✓ (active only, hourly) |

## Business Rules

<Numbered, one sentence each, citing the deciding decision by short8 id
(see references/citations.md). Rules live here even when the code enforces
them only implicitly.>

- **R1.** A posting can never return from `closed` to any other status (per `b9b9fee3`).
- **R2 (not yet implemented — backlog b-12).** …

## Edge Cases Settled

<Edge cases with a decided answer. An open question does not belong here — it
belongs in Open Gaps (harvest) or in exploring (new work).>

## Open Gaps

<Only in `coverage: partial` specs. One line per unknown: what is unknown, and
who/what could answer it. Empty section + `coverage: full` = the rebuild bar is met.>

## Diagrams

<Whenever a documented behavior is flow-shaped — states, sequences,
containment, routing — it is drawn here in Mermaid, always. One diagram per
question, business vocabulary only. A behavior with no drawable shape: omit,
never force a list into a picture.>

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
- **Entry Points & Triggers:** if a link, screen, schedule, event, or
  call exists that this table doesn't explain, the spec fails the
  rebuild test.
- **Data Dictionary:** display order is part of the spec for UI areas.
  Validation limits live in the Meaning/Values cells in business terms
  ("≤120 chars"), not as regexes. Config values whose numbers were
  *chosen* (thresholds, windows, retry counts) cite the deciding
  decision — a tuned number without its why is half-lost knowledge.
- **Behaviors & Operations:** four sub-answers mandatory for every
  action and operation — blocked-when (or runs-when) / what changes /
  side effects / afterwards-per-actor; "afterwards" names what EACH
  affected actor or consuming system observes, not just the acting
  user. System operations additionally state failure behavior (what
  happens mid-run, what retries, what stays consistent).
- **Actors & Access:** prefer one matrix; consuming/producing systems
  are actors too; footnote row-level subtleties ("owner of THIS
  posting, not any owner").
- **Business Rules vs Behaviors:** a Behavior is what the system
  observably does; a Rule is the policy behind it. A rule approved but
  not yet shipped is marked "not yet implemented" with a backlog id —
  never written as a Behavior.
- **Diagrams:** drawable means drawn — when a behavior is flow-shaped
  (a lifecycle of states, a sequence of actors, a containment of
  parts, a routing of cases), a Mermaid diagram section is mandatory,
  not decoration. `stateDiagram` for lifecycles, `sequenceDiagram` for
  who-talks-to-whom, `flowchart` for routing and containment; labels
  use the spec's pinned business terms, never code identifiers. A
  contradicted diagram is replaced in place like a contradicted line.
  The full craft lives in `.bee/expertise/documentation.md` ("Draw
  what is drawable").
- **Visuals:** the snapshot preserves what the spec cannot say — the
  settled *look*. One current image per screen, stable filename,
  replaced in place. Ask the user for a screenshot when you cannot
  capture one; an absent snapshot is an Open Gap with a stated reason.
- **Pointers:** load-bearing few, not a file listing. This section is
  allowed to rot slightly; everything above it is not.

## Merge Rules

- **Locate before create:** resolve every delta to an existing spec
  (via the reading map and a scan of spec frontmatter/Pointers — or,
  with a bundle, by asking the CLI for the owner) before considering a
  new file. A renamed screen, moved route, or refactored module is
  still the SAME area. Creating is the exception, reserved for
  genuinely new surfaces. Never create `-v2`, `-new`, or
  date-suffixed spec files.
- Deltas come from `behavior_change` cells + verification evidence,
  UAT records, and worker reports — never from plan.md, never from
  memory.
- A delta that contradicts an existing line **replaces** it; never
  keep both.
- Present tense only. "Was", "previously", "changed from" are banned —
  history lives in git and `docs/history/`.
- Update `updated`, append the feature to `sources`, reconcile
  `decisions` against the active set, set `coverage` honestly. With a
  bundle, re-emit the whole frontmatter block through the CLI helper —
  never hand-edit a line of it.
- If the feature added/removed an area, or changed shared entities,
  the role model, or a cross-area flow: sync `system-overview.md` (or
  the area's overview concept and generated index) in the same pass.
- UI areas: when a delta made a screen visibly different, refresh its
  snapshot under `visuals/<area>/`; cannot produce one → Open Gap with
  the reason.
- Standard commands are a Pointers-level fact: when a synced change
  alters how the project is set up, started, tested, or verified,
  update the recorded commands in the same pass — one record, never a
  second location.
- After merging, run the Rebuild Checklist on every touched spec.

## Harvest Interview

Three steps: (1) inventory the area from code and running behavior —
screens, fields, actions, roles; for backend areas: triggers, inputs,
outputs, consumers, failure paths; (2) draft the spec with everything
code can *prove*; every meaning or rule code cannot prove becomes a
question; (3) unanswered questions → Open Gaps, `coverage: partial`.

For each meaning or rule code cannot prove, ask one question per
message, outcome-framed, single-choice preferred:

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

Budget the interview: batch the inventory first, then ask only the
questions whose answers change the spec. Confirmed answers are
decisions — log them and cite the new id in the spec.

## Bootstrap Mode

No-bundle repos only (a bundle's indexes are regenerated with
`bee knowledge index`, never hand-bootstrapped). When `docs/specs/`
lacks `system-overview.md` or `reading-map.md`, offer — never
auto-run — a skeleton pass over the missing file(s) only:

- Sources are code/tree inspection and verbatim README extracts.
  Nothing else — no plan.md, no memory, no inference from names.
- Every meaning code cannot mechanically prove is an `[unknown]` gap
  line, never a written claim; every output carries
  `coverage: partial`.
- No interviews: bootstrap is inventory, harvest is meaning.
- A completed bootstrap announces its gap count and offers harvest as
  the next step.

## Rebuild Checklist

Cover the Pointers section and verify:

1. Every entry point and trigger is listed with what appears or runs.
2. Every visible field, input, output, and chosen config value appears
   in the dictionary — display order for UI, meanings everywhere;
   every enum value has a stated business meaning.
3. Every user action and system operation has a Behavior block with
   all four sub-answers (operations also state failure behavior).
4. Every actor — human role or consuming system — appears in the
   access matrix.
5. No sentence requires reading the code to be understood.
6. No technology name appears above Pointers.
7. `coverage` and Open Gaps are honest.
8. UI areas: every settled screen has a current snapshot — or an Open
   Gap saying why not.
9. If this area is new, removed, or changed shared entities/roles/
   flows: the system overview reflects it.

Any failure: fix it now, or file it as an Open Gap with
`coverage: partial` — silently shipping a hole is the defect, not
having one.

## System Overview

Path (no bundle): `docs/specs/system-overview.md`. One file,
singular — the cross-area glue no per-area spec owns. Same write
discipline as any spec. Fresh sessions read it FIRST, before any area
spec. (With a bundle, the glue is the area's own overview concept plus
the generated `docs/knowledge/areas/index.md`, kept current by
regenerating — never hand-edited.)

```markdown
---
area: system-overview
updated: YYYY-MM-DD
decisions: [<active decision ids cited below>]
coverage: full | partial
---

# Spec: System Overview

<One paragraph: what the product is, for whom, in business terms.>

## Area Map

<One line per area: what it is for, where its spec lives. This is the
completeness ledger — an area with shipped behavior and no line here is a gap.>

## Shared Entities

<Business entities two or more areas read or write, with their meaning and
which areas touch them. Per-area field detail stays in the area specs.>

## Actors & Roles (global)

<The role model stated ONCE: every human role and consuming system, one line
on what it is. Area specs reference these names; they never redefine them.>

## Cross-Area Flows

<One block per flow spanning two or more areas: trigger → step per area →
outcome each actor observes. Single-area behavior stays in the area spec.>

## Open Gaps

## Pointers (implementation)
```

Sync triggers: a feature adds or removes an area; a shared entity's
meaning changes; the role model changes; a cross-area flow is created,
removed, or rerouted. Anything else NOOPs — the overview is glue, not
a duplicate of the area specs.

## Reading Map

Path (no bundle): `docs/specs/reading-map.md`. One line per location,
grep-friendly:

```markdown
# Reading Map

- `src/auth/` — session middleware and guards; spec: docs/specs/auth.md
- `scripts/build.mjs` — single build entry point; run with `node scripts/build.mjs`
```

At sync time: add lines for locations the feature created or
repurposed, fix lines it made wrong, delete lines for removed
locations. Keep it a map, not documentation — one line each, no prose
blocks.
