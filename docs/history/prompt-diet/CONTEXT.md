# prompt-diet — CONTEXT

Research-backed diet of the skill prompt corpus. Source: four independent
studies on agent context files (Lulla 2601.20404, ETH SRI 2602.11988,
Khatri 2607.27250, NAIST 2511.12884) audited against this repo's
skills/*/SKILL.md by two gather passes on 2026-08-03. User approved the
resulting top-10 cuts and a prompt-writing standard ("ok", this session).

## Locked decisions

- D1 **One rule, one home.** Each boundary rule is stated in full exactly
  once — AGENTS.md is the canonical home. A SKILL.md may carry a one-line
  cite plus its skill-specific delta, never a near-verbatim restatement.
  Applies to: gate-approval boundary (6 copies today), worktree-first
  (4), cite-never-reinterpret (4), 65%-handoff (3).
- D2 **Body never re-narrates frontmatter.** The YAML `description` routes;
  once loaded, the body opens with an imperative or a table, not a
  definition of the skill. Trim the 3–6 opening descriptive lines in all
  9 SKILL.md bodies.
- D3 **Outstanding Questions boilerplate gets one canonical home** in
  bee-hive/SKILL.md (the router, always loaded when routing); the 5
  per-skill copies shrink to a one-line cite + skill-specific delta.
  AGENTS.md (templates.rs render) is NOT edited — no Rust change in
  this feature.
- D4 **bee-herding**: cut the `:21-33` intro paragraph that duplicates
  "The three roles" section; convert its References table from
  "File | Contents" to the "when to load" pattern the other 8 use.
- D5 **Prompt-writing standard** is authored as a knowledge doc under
  docs/knowledge/areas/doctrine-layer/ — the 4-question line filter,
  add-on-failure, one-rule-one-home, verifiable-imperative style,
  deterministic-backstop-preference, size ceilings (SKILL.md ≤ ~120
  lines, reference ≤ ~350).
- D6 **No behavior change.** Every cut removes duplication or narration
  only; no rule is weakened, no boundary reworded in meaning. When a cut
  would change meaning, keep the line and note it instead.
- D7 **Render + proof.** skills/ is the source of truth; after edits the
  skill sync must be re-run so .claude/skills/ and the manifest hashes
  match, and `commands.test` (cargo test --release) must be green.

## Out of scope

- AGENTS.md / templates.rs content changes (D3).
- The oversized swarming-reference.md (644 lines) — noted as follow-up
  debt, not cut here.
- permissions.deny / hook hardening (enforcement gaps from the audit) —
  separate feature if the user wants it.
