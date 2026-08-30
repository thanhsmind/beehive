---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval; then a date stamp — the only permitted post-approval write>
---

# Plan: Existence Is Not Evidence

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: one new mechanical gate
refusal in the binary plus skill-text mandates, each landing in a rule's
existing single home; no new commands, no new artifact types.

Revision note: reshaped after the plan-step hat wave (3 seats; synthesis in
`reports/hat-wave-synthesis.md`) — firing sites corrected (B1/B2), membership
rule added (B3), 2 cells instead of 3, risk re-scored on counted evidence.

## Requirements (from CONTEXT.md)

- D1: plan.md gains a mandatory load-bearing claims table (claim, label
  `read`/`ran`/`guessed`, anchor, verbatim quote); tiny/small carry one
  evidence line per claim in the merged gate message.
- D2: the shape/merged gate refuses mechanically while the table is
  missing/malformed or a load-bearing row is `guessed`; remedy is upgrade or
  move to Open Questions; no waiver flag.
- D3: every lane makes one cheap reality touch per novel surface before the
  gate, output recorded (Discovery, or the gate message for tiny/small).
- D4: `hat-facts-gaps` audits the claims table — open each anchor, confirm
  the quote; mismatch = BLOCKER.
- D5: carrier is skill text + templates + bee-rs enforcement; the knowledge
  pattern is additive only (written at close capture, not as a cell).

## Load-bearing claims

Every row here is load-bearing: if the claim were false, the shape below
changes. The converse holds too: a load-bearing claim living only in prose is
a plan defect (B3). Labels: `read` = the author opened that file at that
line; `ran` = executed, output in hand; `guessed` = inferred (none may remain
at gate). Match rule for audits: the evidence column is a verbatim byte
substring of the anchored line(s), multi-line joined with " / " — reflow or
paraphrase is a mismatch.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Gate-approval preconditions live as fail-closed refusal helpers beside `run_gate` in set_gate.rs; the high-risk advisor precondition is the shape to copy | read | `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:598` | `fn high_risk_advisor_refusal(` |
| 2 | plan.md path resolution already exists and is reusable | read | `packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs:78-79` | `pub(crate) fn advisor_plan_path(root: &Path, feature: &str) -> PathBuf {` / `    root.join("docs").join("history").join(feature).join("plan.md")` |
| 3 | The D5 conflict precondition is the second precedent, same placement, fail-closed | read | `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:625-626` | `// The twin of \`high_risk_advisor_refusal\` above, in the same place and with` / `// the same fail-closed shape:` |
| 4 | Both existing preconditions fire ONLY under the exec guard — `--name shape` is not covered, and a test pins that by name | read | `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:824,830,1922-1924` | `let exec_component = merge || name == "execution";` / `if exec_component && approved {` / `fn the_conflict_precondition_covers_name_execution_but_not_name_shape()` |
| 5 | plan.md freezes at shape approval — a refusal firing after that cannot be remedied by editing the plan | read | `skills/bee-planning/references/planning-reference.md:25-26` | `Frozen once the shape gate is approved: the only permitted post-approval` / `write is the approval stamp in the frontmatter. No in-place enrichment.` |
| 6 | The tiny/small merged gate previews cells in the gate message and gets no hat wave | read | `skills/bee-planning/references/planning-reference.md:289-291` | `the hat wave never opens` |
| 7 | The `hat-facts-gaps` seat's instrument is one table row pointing at its home | read | `skills/bee-hive/references/gates-and-delegation.md:218` | `5-Layer rubric + Truth Table Test over the plan` |
| 8 | `skills/` is the source; `bee dev regen` renders `.claude/skills` and also rewrites onboarding + release manifests | ran | `diff skills/bee-hive/references/gates-and-delegation.md .claude/skills/bee-hive/references/gates-and-delegation.md; .bee/bin/bee dev regen --help` | diff delta is only the `<!-- bee:only codex -->` block; help: `render-skill-trees, then onboard --repo-root . --apply, then release-manifest --write` |
| 9 | Exactly ONE existing test fixture writes a plan.md and then calls a gate approval, and it approves via `--merge` expecting the advisor-stale refusal | read | `packages/bee-rs/crates/bee/src/verbs/state_group/set_gate.rs:1587-1600` | `w(root, "docs/history/gate-door-plan/plan.md", "v1");` |
| 10 | 193 existing plan.md files carry no claims table — backward compat is a real surface | ran | `fd -t f plan.md docs/history \| wc -l` | `193` |
| 11 | Nothing in the binary reads `artifact_contract` — version-keying the check would be a new mechanism, not a reuse | ran | `rg -c "artifact_contract" packages/bee-rs/crates/bee/src/` | zero matches (empty output) |
| 12 | The declared test suite runs the bee-rs workspace | read | `.bee/config.json:11` (session preamble `commands.test`) | `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` |

## Discovery

Reality touches taken before this gate (D3, applied to itself): opened
`set_gate.rs` at :598, :824-832, :1585-1600, :1920-1926 and `advisor_ref.rs`
:78-80 and quoted them; ran the source-vs-rendered `diff`, the `fd` plan
count, and the `artifact_contract` grep. Second-reader audit of the previous
revision's table: 7 match / 1 partial mismatch (row 12's quote had silently
dropped the `PATH=` prefix — now exact bytes).

## Approach

Recommended: one new refusal helper with its OWN guard, plus prose mandates
in each rule's existing single home. Cites D1-D5.

Design resolutions from the wave (full trail: `reports/hat-wave-synthesis.md`):

- **Firing sites (B1/B2).** The claims check runs under its own guard:
  `approved && (merge || name == "shape")` — at shape and merged approvals,
  where plan.md is still editable. It does NOT fire on plain
  `--name execution` (post-freeze, remedy impossible — claim 5) — a named
  deviation from the copied `exec_component` guard (claim 4). It runs AFTER
  the existing preconditions in the merged path so their refusal-message
  tests stay stable (claim 9).
- **One call site (W5).** Single pre-lock check; a plan-file read has no
  peek/lock record race — named deviation from the two-site pattern.
- **Membership rule (B3).** The binary cannot judge prose. The converse rule
  ("every load-bearing claim must be a row") ships as template text, as the
  leader's pre-gate self-check, and as an explicit membership sweep in the
  D4 audit mandate (a load-bearing claim found only in prose is a BLOCKER
  finding). Residual: a claim omitted from both table and prose is caught by
  nothing mechanical — named, accepted.
- **No version-keying (W2, claims 10-11).** The check fires whenever plan.md
  exists on a covered approval. A gate keyed on an annotation misses exactly
  what it exists to catch
  (`docs/knowledge/patterns/20260729-a-gate-keyed-on-an-annotation-misses-exactly-what-it-exists-to-catch.md`).
  The 193 old plans hit the check only on a rare post-revocation
  re-approval; the refusal text is fully self-serve (below). Accepted, named.
- **Anchor hardening (W1).** A `read`-labeled row whose anchor parses as
  `path:line[-line]` refuses when the path does not exist under root. No
  quote matching in the binary — that stays D4's audit. Residual (fabricated
  quote at a real path) named.
- **Feature selection (W3).** Copies M1: a lane approval reads the lane's
  own feature; a default-record approval reads the record's `feature` field;
  no target record → precondition inapplicable.
- **Missing vs unreadable (W4).** `ErrorKind::NotFound` → inapplicable
  (tiny/small legitimately have no plan.md); any other read error → refuse,
  fail-closed. Portable error fixture: plan.md as a directory.
- **Self-serve refusal (U3).** The refusal names the offending rows AND the
  expected shape (column list + label vocabulary + the remedy: upgrade the
  label with a real read/run, or move the claim to `## Open Questions`).
- **Pre-flight, not ambush (U1).** Skill text: the leader runs the same
  checklist BEFORE presenting the gate; the binary refusal is the net.

Rejected alternatives: prose-only pattern (D5 forbids; failed in the field) ·
a new `bee plan claims` command (gate verb already reads plan.md — claim 2;
new surface, no new enforcement) · machine-parsing the tiny/small gate
message (chat text is not an artifact — claim 6; those lanes get the
skill-text mandate + self-check) · version-keying on `artifact_contract`
(claim 11 + the annotation-gate pattern) · inlining the parser in set_gate.rs
(2762 lines already; the precedent splits logic from wrapper — claim 1).

Risk map: set_gate.rs wiring / LOW (one fixture to touch — claim 9; new
guard, existing tests pinned green by ordering) / proof: full bee-rs suite.
Skill-text edits / LOW / proof: regen green + rendered-copy spot diff.
Backward compat / LOW (claims 10-11; named re-approval friction, self-serve
remedy). D3 ships as prose only / NAMED RESIDUAL (CONTEXT Origin: prose
alone failed in the field) — mitigations: the table's `ran` rows make
touches visible, and the D4 membership sweep asks for them. Herding: an
unattended loop halts on a table-less plan at the shape gate — correct and
visible (W12).

## Shape

One slice, two cells:

1. `eine-rust-claims-gate` — new module
   `packages/bee-rs/crates/bee/src/verbs/state_group/plan_claims.rs`:
   markdown-table parser for `## Load-bearing claims` + rule evaluation +
   ALL rule-level unit tests live here; a ≤15-line wrapper
   `plan_claims_refusal` in `set_gate.rs` under its own guard
   `approved && (merge || name == "shape")`, running after the existing
   preconditions in the merged path. Refusal rules: plan.md unreadable
   (non-NotFound) → refuse; heading absent → refuse; zero rows → refuse; a
   row missing label/anchor/evidence → refuse; label outside
   {read, ran, guessed} → refuse; any `guessed` row → refuse; `read` row
   with a `path:line` anchor whose path is absent under root → refuse.
   Refusal text: offending row numbers + expected shape + remedy (U3).
   Integration tests in set_gate's test module: full-verb `--merge` refuse
   on a real plan.md with a `guessed` row (W11); `--name shape` fires;
   `--name execution` does not (pin, mirroring claim 4's test);
   no-plan.md pass-through; `--approved false` never fires; the claim-9
   fixture updated (gains a minimal table, or asserts refusal order).
2. `eine-skill-mandates` — D1 table spec + match rule + membership converse
   + `## Open Questions` section (U2) into `planning-reference.md`
   ("Artifact: plan.md"); D3 reality touch + U1 pre-flight self-check into
   `bee-planning/SKILL.md` (Research, Gate); tiny/small inline evidence +
   "no load-bearing claims → say so, never manufacture one" (U4/U5) into
   planning-reference.md ("Tiny/small merged gate"); the claims-audit +
   membership-sweep procedure into `.bee/expertise/review.md` as its single
   home, with the `hat-facts-gaps` row in `gates-and-delegation.md` extended
   by POINTER only (W7). Cap: `bee dev regen` + rendered-copy spot diff.

Cell 1 ∥ 2 — disjoint in authored files; cell 2's regen rewrites generated
trees no other cell touches (W8). The knowledge pattern is NOT a cell: it is
written at close capture with the real post-implementation learning (D5,
wave alternative 1).

## Test matrix

Happy path: plan.md with a complete table, zero `guessed` → shape approval
proceeds. Edge cases: each refusal rule above has its own unit test row
(heading absent; zero rows; missing column; bad label incl. case variants;
`guessed` row; NotFound → inapplicable; directory-as-plan.md → refuse;
absent `read`-anchor path → refuse). Gate-surface cases: fires on
`--name shape` and `--merge`; silent on `--name execution` and any
`--approved false`. Error paths: refusal text carries row numbers, expected
shape, remedy. Existing suite: the claim-9 fixture is the one edit.

## Open Questions

(none — the wave's questions are resolved in Approach; anything that
resurfaces during execution lands here before the gate re-opens)

## Out of scope

Cell/worker-level evidence labels at cap time (prove-the-whole-path
territory); AGENTS.md BEE-block line; any new CLI command, waiver flag, or
binary quote-matching; retrofitting the 193 existing plans.
