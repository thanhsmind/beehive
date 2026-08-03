# prompt-diet — plan

Route: class=refactor lane=standard flags=[public-contracts,multi-domain]
files=10. CONTEXT.md decisions D1–D7 govern; D5 is amended here (see
Constraints) to honor repo decision 8f63adb4 / budget-fence-removal.

## Shape

One slice, five cells. pd-1 runs first (it authors the canonical
Outstanding Questions home that pd-2's cites point at — citation
dependency, the named reason for serial); pd-2/pd-3/pd-4 then run in
parallel (disjoint file sets); pd-5 integrates.

- **pd-1** — Diet bee-hive, bee-shaping, bee-planning, bee-swarming
  SKILL.md: apply D1 (boundary-rule restatements → one-line cite of
  AGENTS.md + skill delta), D2 (bodies open imperative, no frontmatter
  re-narration), D3 (bee-hive gains the one canonical
  headless/Outstanding-Questions statement under a stable heading).
- **pd-2** — Same D1/D2 diet for bee-capturing, bee-reviewing,
  bee-grooming, bee-researching SKILL.md; their headless boilerplate
  becomes a cite of bee-hive's canonical statement + per-skill delta.
- **pd-3** — bee-herding SKILL.md: cut the intro paragraph duplicating
  "The three roles"; References table converted to "when to load";
  D2 trim.
- **pd-4** — Author
  docs/knowledge/areas/doctrine-layer/prompt-writing-standard.md and add
  it to the area index: the 4-question line filter, add-on-failure,
  one-rule-one-home (generalizing the existing duplication boundary,
  cited), verifiable-imperative style, deterministic-backstop
  preference. Research citations from CONTEXT.md.
- **pd-5** — Re-run skill sync (`bee onboard` → `--apply` when
  changes_needed) so every rendered projection and manifest hash
  matches skills/, then full `commands.test` green.

## Constraints (bind every cell)

- **No size ceilings anywhere** — decision 8f63adb4 + budget-fence-removal
  D1/D6: a diet is a one-off event; density is judged per edit. D5's
  drafted "size ceilings" clause is dropped; the standard records the
  research numbers as evidence, never as a gate. instruction_laws.rs
  fails the build on any reintroduced ceiling.
- **Pointer integrity** — every kept or added citation
  (`references/x.md ("Heading")` or named-set form) must resolve, and no
  heading cited by AGENTS.md or any other doc may be deleted
  (`cargo test --test pointer_integrity` is the fast check).
- **Duplication boundary R4** — a restatement may be dropped only when
  AGENTS.md verifiably carries the rule (check the real text, in the
  BEE block, before cutting).
- **Pinned wording** — never reword a surviving boundary-rule sentence;
  cut whole duplicates or keep them verbatim. AGENTS.md itself is not
  edited (CONTEXT D3).
- **No retired stage names** introduced (scan-set hygiene E8).
- **Meaning-preserving only** (CONTEXT D6): when a cut would change what
  a rule demands, keep the line and note it in the cell report.

## Verify

`commands.test`: `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test
--release --manifest-path packages/bee-rs/Cargo.toml` — run at every
cell finish; pd-5 additionally proves the render manifest is in sync.

## Smaller-path check

Considered: single mega-cell (rejected — one worker touching 10 files
serially, no parallelism, harder review); docs-lane no-pipeline
(rejected — skills/ are shipped product files with contract tests, not
knowledge). Five cells with one serial dependency is the smallest shape
honoring every decision. PASS.
