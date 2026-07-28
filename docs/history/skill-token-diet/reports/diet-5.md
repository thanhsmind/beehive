# diet-5 — bee-scribing thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-scribing/SKILL.md` rewritten to the D7 thin-body doctrine:
24,472 → **8,151 bytes** (≤ 8,192). All exiled text landed in
`skills/bee-scribing/references/scribing-reference.md` (new sections: Delegation,
Gather Sources — What Each May Feed, Map Deltas — bundleMode Routing
(`scribingTarget`, the three refusal answers, the anti-fork gate's three layers),
Capture Mode in full (debt signal, flush protocol), and a 3-step harvest summary
folded into Harvest Interview Protocol); new `references/provenance.md` maps
every body rule to its decision IDs (decisions 0002, 0003, 0007, 0011, 0017,
D2/D3 delegation, D2 of harness10, D4 goal-check, D8). Baseline lowered
24472→8151, `bee-scribing` appended to `migrated[]` (provenance grep now
active — 0 hits), `notes["bee-scribing"]` deleted. Regen obligation ran
in-cell: plugin mirror render, onboard `--apply`, release manifest
`--write`/`--check`. Full verify chain green.

**Files:** `skills/bee-scribing/SKILL.md`,
`skills/bee-scribing/references/scribing-reference.md`,
`skills/bee-scribing/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-5.json`.

## Side-by-side behavior checks (P5 acceptance)

### s1 — the SELF-TRIGGERING capture law and its lane-scaled cost table survive verbatim in meaning

Before (§4 "Capture Mode — Settled Outcomes from the Vibe Loop"):

> **Detection is the scribe's duty, unprompted (decision 0007).** The explicit
> signal is the *loud* case; most settlements are silent — the user confirms a
> behavior works, accepts an explanation, picks an option, moves on. The agent
> watches for these itself, every turn, and captures without being asked. Do
> not ask "should I document this?" — announce in one line what settled and
> where it goes … then do it in the same turn. **The close-audit habit, zero
> exceptions:** at every task close … ask "what settled here?" and either
> capture it or state "nothing settled" …
> 1. Log it … 2. **High-risk lane:** merge the settled truth into the area's
> spec now … **Every other lane (decision 0017):** append a stub instead …

After ("Capture — the self-triggering law"):

> **Detection is the scribe's own duty, unprompted.** Most settlements are
> silent — the user confirms a behavior, accepts an explanation, picks an
> option, moves on — and the agent watches for these itself, every turn,
> unasked. Do not ask "should I document this?" — announce what settled and
> where in one line, then do it same turn. **Close-audit, zero exceptions:**
> at every task close … ask "what settled here?" and either capture it or
> state "nothing settled" …
> 1. Log: … 2. Lane-scaled (never memory-scaled): **high-risk** → full spec
> merge now, never queued. **Everything else** → one-line stub … merge
> deferred to flush.

Same detection duty (unprompted, every turn, no asking), same close-audit
habit ("what settled here?" / "nothing settled"), same two-tier lane-scaled
cost (high-risk merges now vs. every other lane queues a stub to flush) —
only the decision-ID citations (0007, 0017) moved to `references/provenance.md`
per D8, and the numbered steps compressed. The law itself is unchanged and
still lives in the body, as required.

### s2 — bundleMode routing decides where meaning is written, identically

Before (§2 "Map Deltas to Areas"):

> **Never decide by eye where the area's truth lives, and never re-state the
> rule in your own words.** One predicate answers it: `bundleMode(root)` in
> `.bee/bin/lib/knowledge.mjs` — true only when `docs/knowledge/` exists **and
> at least one concept in it actually parses**. A directory alone is not a
> bundle: a repo whose `docs/knowledge/` holds nothing but a `.gitkeep` is
> **not** in bundle mode.

After (intro paragraph):

> **Where meaning is written: `bundleMode(root)`** (`.bee/bin/lib/knowledge.mjs`)
> — true only when `docs/knowledge/` exists AND a concept in it parses (a
> `.gitkeep`-only dir is NOT a bundle).

Same predicate, same module, same two conditions (`docs/knowledge/` exists AND
a concept parses), same `.gitkeep`-only counter-example. The full routing
mechanics (`scribingTarget()` call, the seven-key return shape) moved to
`references/scribing-reference.md` ("Map Deltas") — the pointer resolves to a
new section that reproduces the original `node -e` command and the returned
key list verbatim.

### s3 — the anti-fork gate's refusal answers (rare branch) are reachable unchanged

Before (§2, "A `path: null` answer is a refusal" table, inline in body):

> | `action` | Means | Do |
> |---|---|---|
> | `fork_denied` | the subject is already owned by the concept in `owner` |
> update the owner in place, or declare the split inside it |
> | `subject_required` | `intent:'new-concept'` was passed with no subject
> (empty, blank, `null`, punctuation-only) | name the subject and ask again —
> "no subject" is not a new subject, and it is **never** routed to
> `overview.md` |
> | `duplicate_authority` | two or more concepts already claim this subject,
> every claimant is listed on `owner.conflicts` | fix the bundle first —
> collapse the rival claims to one authority, then re-ask |

After (`references/scribing-reference.md` § "Map Deltas — bundleMode Routing
(scribingTarget)", reached via the body's "Map Deltas" pointer):

> | `action` | Means | Do |
> |---|---|---|
> | `fork_denied` | the subject is already owned by the concept in `owner` |
> update the owner in place, or declare the split inside it |
> | `subject_required` | `intent:'new-concept'` with no subject (empty,
> blank, `null`, punctuation-only) | name the subject and ask again — never
> routed to `overview.md` |
> | `duplicate_authority` | two or more concepts already claim this subject,
> listed on `owner.conflicts` | fix the bundle first — collapse the rival
> claims to one authority, then re-ask |

Identical three rows, same three refusal semantics, same "never routed to
`overview.md`" prohibition — the table moved location (body → reference,
reached one hop away via the body's "Map Deltas" pointer) with zero change in
meaning. The malformed-`bee.authoritative_for`-throws rule and the three-layer
anti-fork gate description that followed it in the original body are
similarly reproduced verbatim in the new reference section.

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` matches in the body (confirmed by `skill_budget_fence.mjs --selftest`'s live-repo check); the rule → decision-ID map is `references/provenance.md`.
- No `skill_lint`-required pointers are registered for bee-scribing (unlike bee-hive/bee-exploring/bee-validating); the body's multi-heading pointers (e.g. `("Map Deltas", "Bundle-mode gate and frontmatter")`) follow the same convention already shipped in bee-swarming's body (diet-4) — `skill_lint`'s advisory anchor check does not pattern-match comma-joined multi-heading pointers, consistent with existing precedent; it never blocks the chain regardless.
- Content newly added to the reference beyond a straight move: a "Delegation" section (preserves the original body's per-step delegation-tier note, D2/D3) and a "Gather Sources — What Each May Feed" section (the original §1 table) — both reachable from the reference file directly, additive, meaning unchanged.
- Deviation (recorded in trace): none beyond the additive reference sections above — all inside the declared file set.
