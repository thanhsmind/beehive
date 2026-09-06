# Concept Authoring — the write loop for the bundle

Load when a lesson, a behavior, or a settled fact is about to become a
concept file under `docs/knowledge/`. The judgment (what earns a record)
lives in SKILL.md and `references/promotion.md` ("Promotion Decision
Tree"); this file is the mechanical loop that turns an earned record
into a canonical, linked, validated file — and nothing else.

Five steps, always in this order: **search → decide → write → relate →
validate**. Skipping the first produces a duplicate; skipping the fourth
produces an orphan; skipping the last ships a file `knowledge check`
will refuse next session.

## 1. Search before you write

The bundle is the memory. A concept written without a search is a
second home for a fact that may already have one — and every later
reader pays for both.

- `bee knowledge search --text "<symptom or topic words>" --json` —
  OR-substring over patterns and area concepts, ranked. Try the
  symptom phrasing first, then the mechanism phrasing; one search
  proves nothing.
- `bee knowledge list --area <area>` — every concept the owning area
  already holds. Read the `description` column before any body.
- `bee knowledge show --id <id> --json` on each hit — the whole file
  plus `links_out` and `links_in`. This is the read; never `cat` or
  grep the bundle wholesale, and never open a body whose description
  already rules it out.

Stop searching when the neighbourhood is clear: the owning area
concept, the closest existing pattern, and whether either already
states the fact.

## 2. Decide: extend, replace, or new

One fact, one home (`.bee/expertise/knowledge.md`). The decision tree,
in order:

| Finding | Action |
|---|---|
| An existing concept states the fact | Nothing to write. Add the new evidence to its `bee.sources` if it strengthens the claim. |
| An existing concept states the *opposite* | Replace that line in place; never keep both. Cite the evidence in `bee.sources`. A superseded concept gets `bee.superseded_by`, never a silent edit. |
| An existing concept is the natural home but lacks the fact | Extend it — a new bullet, rule, or edge case under the right heading. Same file, same id. |
| No home exists, and the fact has its own lifecycle | New concept (step 3). "Own lifecycle" means it will change, retire, or be cited on its own — a sibling of the nearest concept, never a fork of it. |

Split a concept only on independent lifecycle, never on length.
A `-v2`, `-new`, or `-fixed` twin of an existing id is the duplicate
the authoring gate names `duplicate_authority`; the fix is the edit
above, not a second file.

Keep uncertainty as written uncertainty. "Inferred from the cell trace,
unconfirmed by a run" is a valid line; a confident sentence over thin
evidence is not. A claim with no evidence and no decision behind it is
an Open Gap, not a rule.

Chat noise never enters the body: no "we discussed", no "the user
asked", no timestamps of the conversation. The body reads as the truth
of the system; the provenance rides in `bee.sources` and
`bee.decisions`.

## 3. Write through the CLI

Draft the body in a scratch file first — the same headings the type
expects (`docs/knowledge/areas/okf-profile/concept-model-and-authoring.md`
("Templates")). Then let the verb name the path and emit the
frontmatter:

```
bee knowledge new --type pattern|area --title "<title>" \
  --summary "<one line, the description field>" --area <area> \
  [--tags a,b] [--lifecycle draft] --file <scratch.md> --json
```

What the verb owns: the path (`patterns/YYYYMMDD-<slug>.md` or
`areas/<area>/<slug>.md`), the id (`pattern-YYYYMMDD-<slug>` or
`<area>-<slug>`), today's timestamp, the canonical key order, and the
index re-render. It refuses an existing path and a claimed id — a
refusal here means step 1 was skipped; go back, do not rename around it.

What the verb does not own — add these by hand after the write, on a
pattern:

- `bee.polarity: practice|pitfall` — every pattern carries one.
- `bee.critical: true` — only when all three legs of
  `docs/knowledge/areas/okf-profile/critical-bar.md` hold. Default is
  absent; the pool is selective on purpose.
- `bee.sources` — the evidence, quoted precisely enough to re-find:
  cell id and commit, a review finding, a verification output path.
- `bee.decisions` — the store ids or D-ids the concept applies.

Hand edits reorder nothing: keep the emitted key order (`id`,
`lifecycle`, `areas`, `required_context`, `decisions`, `sources`,
`lane`, `polarity`, `critical`, …) or `not_canonical` fires at
validation. Title and description are your words about the fact,
never invented from the search results.

The profile records trust only through `bee.sources` and
`bee.decisions`; there is no generated/verified field. State the
evidence in the source line instead of inventing a flag.

## 4. Relate — no orphans

A concept nobody links to is found by search alone, and search is a
fallback. Every new concept gets at least one inbound link before the
close:

1. `bee.required_context` — the area overview
   (`areas/<area>/overview.md`) at minimum; bundle-relative paths, and
   the verb's `dangling_required_context` check catches a typo.
2. One body link *from* the owning area concept or the nearest sibling
   pattern *to* the new file — a relative `.md` link in the sentence
   where the fact belongs, never a bare "see also" list.
3. Verify: `bee knowledge show --id <new id> --json` — `links_in`
   must be non-empty. `knowledge check` does not test for orphans;
   this read is the only guard.

A pattern that generalizes an area rule links both ways: the area
concept names the pattern where the rule is stated, the pattern's
`bee.areas` names the area.

## 5. Validate before the capture line

```
bee knowledge check --strict
bee knowledge index --check
```

Both green, or the write is not done. `check` reads the file back
through the same emitter `new` used, so a clean hand edit passes and a
reordered or misspelled key does not. `index --check` proves the
re-rendered `index.md` files match the tree.

## End-of-task review

Before the capture line of any task that touched the bundle — the
docs lane and quick work included — answer four questions in one
breath:

1. Did I search before every write?
2. Does every fact I added have exactly one home?
3. Does every new concept have an inbound link?
4. Are `check --strict` and `index --check` green in this message?

A "no" on any of them is the next action, not a note.

## Which verb, when

| Situation | Verb |
|---|---|
| A symptom, an error text, a "have we seen this?" | `bee knowledge search --text` |
| What does area X already hold? | `bee knowledge list --area X` |
| Read one concept and its neighbourhood | `bee knowledge show --id` |
| Write a new concept file | `bee knowledge new` |
| Mine a closed feature for candidates | `bee knowledge promote --work <feature>` — proposes only; each surviving proposal is reviewed against the bundle, then lands through `new` or an in-place edit |
| Prove the bundle is well-formed | `bee knowledge check --strict` |
| Prove the indexes match the tree | `bee knowledge index --check` |
| The curated read-set for a work item | `bee knowledge context --work <feature>` |
