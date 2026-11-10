---
type: bee.area
title: "Bee OKF Profile — freshness, anchors and the document homes"
description: "Which kinds of truth live in docs/knowledge and which live in code, how an area concept names the files it depends on and is graded against git, the cap re-confirm step, the one-time host seed, the pull-rate measure that tells whether any of it is read, the read set, the templates rule and its three writing warnings, and the decision-link error."
timestamp: 2026-11-07
bee:
  id: okf-profile-freshness-and-document-homes
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: ["knowledge-freshness-mechanic D1 (2b11f025): knowledge is pulled on demand, never pushed into briefs", "knowledge-freshness-mechanic D2 (93e1001d): the document homes rule", "knowledge-freshness-mechanic D3 (27ad96a5): freshness is derived from git through verified_at and verified_commit", "knowledge-freshness-mechanic D4 (e7e80d78): updates ride the cap sync door and promote", "knowledge-freshness-mechanic D5 (28fb074d): delete before write, work/ leaves the bundle", "knowledge-freshness-mechanic D6 (e7e133dc): pull rate is the measure", "knowledge-freshness-mechanic D7 (6a502ad0): the mechanic is portable and reaches hosts through onboarding", "knowledge-freshness-mechanic D8 (4148025d): bee.anchors is an explicit, seeded path list", "knowledge-docs-in-the-flow D1 (5c5ae146): documentation is a working part of the flow", "knowledge-docs-in-the-flow D2 (f3c5ad0f): the read set lives in CONTEXT.md and shape approval checks it", "knowledge-docs-in-the-flow D3 (2bc56fb6): every template's sections open with their question and sources, plus a deletion line", "knowledge-docs-in-the-flow D4 (e2f6e8c3): bee knowledge check refuses an unlinked area concept as missing_decision_link", "knowledge-docs-in-the-flow D5: the read set is pulled, nothing new is pushed into briefs (restates knowledge-freshness-mechanic D1)", "knowledge-docs-in-the-flow D6: the read-set gate and the decision-link check live in the binary and skills and reach hosts through onboarding (restates knowledge-freshness-mechanic D7)", "knowledge-useful-rules CONTEXT.md D2, D4 (check warns on missing Read-when, directory anchors, and long concepts; check and pack share read_when_line)"]
  sources: [docs/history/knowledge-freshness-mechanic/CONTEXT.md, docs/history/knowledge-freshness-mechanic/plan.md, docs/history/knowledge-docs-in-the-flow/CONTEXT.md, docs/history/knowledge-docs-in-the-flow/plan.md, docs/history/knowledge-useful-rules/CONTEXT.md]
  authoritative_for: "okf-profile: the document homes rule, the anchor and verified fields, the cap re-confirm step, the host seed step, the pull-rate measure, the read set, the templates rule and its three writing warnings, and the decision-link error"
  anchors: [packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs, packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs, packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs, packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs, packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs, packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs, packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs]
  verified_at: 2026-11-10
  verified_commit: f49dc03cd38d1e2df53a161680ecc37a570a10c6
---

# Bee OKF Profile (freshness, anchors and the document homes)

Read when: deciding whether a truth belongs in docs/knowledge or code, anchoring an area concept, or fixing freshness check warnings.

## Purpose

A knowledge entry that describes code can go on saying the same thing long after the code changed, and nothing
tells the reader. This concept records the rule that decides what belongs in `docs/knowledge/` at all, and the
mechanic that keeps what does belong there honest beside the code it describes. It is portable: the binary
carries the mechanic, this area and `skills/bee-capturing/SKILL.md` document it, and a host project receives
all three through onboarding without configuring anything.

## The document homes rule

Five kinds of truth live in `docs/knowledge/`: the vision, the glossary, the area map (an area's boundaries
and invariants), decisions, and patterns. The glossary is named here as a home; it is not yet authored, and
neither is an event catalogue — both wait on the pull-rate measure below.

Three kinds live in code, where a change to the behaviour changes them in the same commit: verify features and
tests (what the product does), generated contracts (the shapes a verb or file exchanges), and runnable runbooks
(scripts that perform a procedure instead of describing it).

Every area entry anchors to at least one code file or verify feature. An entry that names nothing it depends on
cannot be graded and has no home in the bundle. Behaviour narrative that a test already states is the reader's
cue to delete the narrative, not to keep two copies.

## The fields and the grade

An area concept carries three fields under `bee`:

- `anchors` — a flat list of repo-relative paths the concept's truth depends on;
- `verified_at` — the ISO date the concept was last confirmed;
- `verified_commit` — the full sha of the commit it was confirmed against.

`bee knowledge check` grades each area concept `confirmed`, `suspect`, `orphan` or `unverified` from git. The
rule, including how a squashed or rebased sha and a commit that edits both an anchor and the concept are
treated, lives in `docs/history/knowledge-freshness-mechanic/plan.md` under Approach, "Grade rule"; it is
cited here and not restated. Search, show, context and the session preamble print the grade wherever a concept
is read.

Patterns carry a date and no grade: they are neither anchored, seeded nor graded.

## The cap step

The cap sync door adds a fourth check beside the three it already had. When a cell's touched files intersect a
concept's `anchors`, the cap refuses while that concept grades `suspect` or `orphan`. The worker either edits
the concept in the same cell or runs `bee knowledge confirm --id <bee.id>` (or `--path <file>` for every
concept anchoring that file), reserves the concept file and commits it with the cell. For an orphan the worker
fixes `bee.anchors` instead, because confirm never clears an orphan.

A concept that grades `unverified` passes: a host that never seeded, or a concept outside git, is never
blocked. `--sync-ack "<reason>"` remains the recorded escape and is stored on the cell's trace.

## Host steps

The binary and the skills arrive in a host through onboarding. The owner then runs `bee knowledge seed-anchors`
once: it fills `bee.anchors` from the repo paths that resolve in each area concept's body and stamps a baseline
`verified_commit` and `verified_at` from the concept file's own last commit, so the first grade is honest.
It writes only concepts that have no `bee.anchors` and never touches `patterns/`. Until the seed has run,
`bee knowledge check` prints a NOTE that no concept carries `bee.anchors`. New area entries carry `bee.anchors`
from the start; the seed is not re-run to author them.

## The pull-rate measure

Every timings row carries the feature bound to the session that ran the command. `bee knowledge report` prints,
per feature, how many `knowledge search`, `knowledge context` and `knowledge show` calls ran against all calls,
and the ratio. That ratio is the measure of whether the bundle is read. A flat rate means the lean layer
carries no weight with the agents it was written for, and the lean layer is deleted next, not extended.

## The read set

Documentation is a working part of the flow rather than an afterthought. During shaping and planning, agents record what knowledge concepts and code entry points they read before making decisions. This record lives in `docs/history/<feature>/CONTEXT.md` under the heading `## Read set`.

The section holds two markdown tables:

- `### Knowledge` — lists each concept read, its freshness grade at read time, and one line on why it matters. The header row must start with `| Concept |` (`| Concept | Grade at read | Why it matters |`). First-column cells contain repo-relative paths beginning with `docs/knowledge/` (backtick-trimmed). A host with no knowledge bundle writes `| (no bundle) | — | this host has no docs/knowledge/ |`. The parser skips rows whose cell begins with an open parenthesis `(`.
- `### Code` — lists each code entry point and what it decides (`| Path | What it decides |`). The header row must start with `| Path |`. This table informs readers and planning; gate verification checks only the knowledge table.

The parser (`parse_read_set` in `packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs`) is heading-scoped and prefix-tolerant. It locates `## Read set`, scans lines until the next `## ` heading, finds the first pipe table whose header starts with `| Concept |`, and collects its first-column paths.

Gate approval for `shape` or `--merge` gates validates the read set through `read_set_refusal` (`packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs`) at the pre-lock peek, immediately after the plan claims check. Plain `--name execution` approvals never run the check. Features running on `tiny`, `small`, or `docs` lanes are exempt.

The check enforces three conditions:
1. **Readable context file:** `docs/history/<feature>/CONTEXT.md` must exist and be readable. If the file cannot be read, approval is refused with:
   `gate: approval refused — docs/history/<feature>/CONTEXT.md could not be read at <path> (<reason>). FIX: run \`bee knowledge context --work <feature>\` and \`bee knowledge search --text "<topic>"\`, write the section (skills/bee-shaping/references/context-template.md), or add \`<!-- bee:no-read-set: <reason> -->\` for a CONTEXT.md written before the read set existed.`
2. **Recorded escape marker:** If the file contains `<!-- bee:no-read-set:` anywhere, the check passes immediately. The marker records the rationale for legacy context files created before the read set existed.
3. **Section presence and resolving paths:** When no escape marker is present, `parse_read_set` must return a list of paths. If the `## Read set` section or the knowledge table is missing, approval is refused with:
   `gate: approval refused — docs/history/<feature>/CONTEXT.md has no "## Read set" section. FIX: run \`bee knowledge context --work <feature>\` and \`bee knowledge search --text "<topic>"\`, write the section (skills/bee-shaping/references/context-template.md), or add \`<!-- bee:no-read-set: <reason> -->\` for a CONTEXT.md written before the read set existed.`
   An empty knowledge table (zero rows) passes. When rows are present, every path must start with `docs/knowledge/` and resolve to an existing file on disk under the repo root. If any path fails this check, approval is refused with:
   `gate: approval refused — docs/history/<feature>/CONTEXT.md read set names \`<path>\`, which is not a docs/knowledge/ path on disk. FIX: fix the path or remove the row.`

Refusals mutate no state. Knowledge is pulled on demand; briefs do not push the read set.

## The templates rule

Document templates ask for information, not structure. Every template section opens with one sentence that states the single question the section answers for a rebuilder, followed by the sources that feed it from the Gather Sources table.

Each section includes a deletion line that tells the writer when to remove a line instead of keeping unneeded prose:
- In entry points: delete a line whose trigger the Behaviors section already names.
- In data dictionaries: delete an element that code names and nothing explains.
- In behaviors: delete a behavior that a test already states.
- In actor matrices: delete a row whose capabilities match the row above it.
- In business rules: delete a bullet that a test already states.
- In edge cases: delete a case that the Behaviors section already answers.
- In open gaps: delete a gap an answer has closed.
- In diagrams: delete a diagram that repeats a list.
- In visuals: delete a line whose screen no longer exists.
- In pointers: delete a pointer nobody would search for.

A business rule bullet may carry an optional status and verification tail:
`— status: <active|temporary|retired>; proves: <test or verify feature path>`
Use this tail only where a rule's status or verification differs from its concept. The decision link stays on the concept.

The cut area template provides a lean five-section structure for area concepts during content trimming: `## Map`, `## Boundaries`, `## Invariants`, `## Entry Points`, and `## Pointers`. Each section carries its question line, sources, and deletion line. Host repositories use the standard nine-section area template until instructed otherwise.

The template the writer fills is also a check: three writing rules are machine-visible, and `bee knowledge check` names the file that breaks one. Every area and pattern body opens with one line `Read when: <situation>` inside its first ten non-blank, non-heading lines, so the pack can print that line in place of the title (knowledge-useful-rules D2); a concept whose anchors name a directory rather than a file, or whose body passes 300 lines, is split or re-anchored before it is filed (knowledge-useful-rules D4). The three findings — `missing_read_when`, `directory_anchor`, `long_concept` — are profile warnings on the same finding shape the older profile warnings use. They never block: `bee knowledge check` stays green without `--strict`, and the close knowledge-freshness door ignores codes it does not name, so a directory anchor cannot stop a cap.

The check and the pack read the line through one function, `read_when_line` in `verbs/knowledge/check.rs`, and no second copy of its prefix or its ten-line window exists (knowledge-useful-rules D2). Two reads of one rule drift, and the drift is silent: the pack would print a line the check did not see, or the reverse.

Two writing rules have no machine check and live only in the rule list the worker brief prints. The title and description carry the words a reader searches for — the error text, the symptom, the mechanism — because search and the relevance ranker score those tokens and the pack shows one line per concept (knowledge-useful-rules D1). An entry keeps the why, the boundary and the rejected option with how it failed, and drops the how that code or a test already shows, because only the knowledge layer can hold what was decided against (knowledge-useful-rules D3).

## The decision-link error

`bee knowledge check` requires every area concept (`type: bee.area`) to link at least one decision through a non-empty `bee.decisions` list in its frontmatter. This rule applies to area concepts and area `overview.md` files. Generated `index.md` files carry no frontmatter and are excluded. Patterns, work items, and decision concepts are never checked.

The check arms conditionally. It verifies area concepts only when the bundle contains at least one area concept with a non-empty `bee.decisions` list. When no area concept in the bundle carries a decision link (such as in a newly bootstrapped repository), `bee knowledge check` outputs a single note and zero errors:
`no area concept links a decision (bee.decisions) — the missing_decision_link rule arms on the first link`

Once armed, any area concept whose `bee.decisions` field is absent or empty produces a profile error finding with code `missing_decision_link`:
`area concept "<path>" carries no decision link — run bee decisions search --text "<topic>", then bee decisions log, and put the decision id in this concept's bee.decisions list`

A `missing_decision_link` profile error makes `bee knowledge check` exit non-zero in every mode; `--strict` only adds warnings to what fails the command.

## Pointers (implementation)

- Grade, bundle root, `seed-anchors` and `confirm`: `packages/bee-rs/crates/bee/src/verbs/knowledge/freshness.rs`.
- The cap door's anchored-concept check: `packages/bee-rs/crates/bee/src/verbs/cells/sync_door.rs`.
- Read set table parser: `packages/bee-rs/crates/bee/src/verbs/drivers/context_table.rs`.
- Read set gate refusal: `packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs`.
- Decision-link check error: `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs`.
- The three writing warnings (missing_read_when, directory_anchor, long_concept) and shared read_when_line: `packages/bee-rs/crates/bee/src/verbs/knowledge/check.rs` (knowledge-useful-rules CONTEXT.md D2, D4).
- The pull-rate section of the report: `packages/bee-rs/crates/bee/src/verbs/knowledge/report.rs`.
- The locked decisions and the grade rule: `docs/history/knowledge-freshness-mechanic/CONTEXT.md` and
  `docs/history/knowledge-freshness-mechanic/plan.md`.
- Read set, template, and decision check decisions: `docs/history/knowledge-docs-in-the-flow/CONTEXT.md` and
  `docs/history/knowledge-docs-in-the-flow/plan.md`.
- Context template with read set: `skills/bee-shaping/references/context-template.md`.
- Area spec and cut area templates: `skills/bee-capturing/references/area-spec.md`.
- The capture-side step that cites this concept: `skills/bee-capturing/SKILL.md`.
