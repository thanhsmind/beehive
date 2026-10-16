---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: pstack-0-15-port

## Summary

Five small ideas from pstack 0.15.25 land in bee. A rule now says why it
holds. The merge door stops calling a proof stale after a clean rebase, and
prints one plain line that says nothing needs a re-run. A rebase or a tiny fix
gets a cheap re-check, not a new judge. A mistake seen twice gets fixed where
it cannot happen again, with the guard shown red first. Before the first
shaping question, the agent plays the goal back in one line.

Mode: `standard` — 1 risk flag: multi-domain (doctrine plus one Rust change)
Why this is the least workflow that protects the work: one Rust function, one
printed line and four doctrine files change; a unit test and `bee dev regen`
prove each.

## Requirements (from CONTEXT.md)

- D1: every `spoken:` line and every class-playbook step states why it holds; reasons never land in `AGENTS.block.md`.
- D2: `bee worktree merge` compares `git patch-id --stable` of each capped commit against the branch's commits since the merge base; a match reports `proof_carried`, not `proof_stale`; advisory never blocks; test seen red then green.
- D3: the judge-tier table gains one narrowed re-verify row (rebase, conflict resolution, or a fix with no new behavior under 30 changed lines → re-run the recorded proof command only).
- D4: a reflection key seen twice is a fix-first candidate at the highest fix-at layer; a new guard must fail on the past mistake first; `FIX_AT_VALUES` unchanged.
- D5: the bugfix playbook gains the cheap always-on-signal sentence.
- D6: interview craft gains a one-line goal playback and over/under-specification move; no rewritten prompt returned.
- D7: per-spawn effort is not ported (backlog proposal holds it) — provenance only, lands in no cell.
- D8: source manifest `cursor/plugins` `pstack/` at `99743e9` — provenance only, lands in no cell.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = the command was run; no `guessed` row remains.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The stale check is one filter over recorded commits and can gain a patch-id branch | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:111-112 | `.filter(\|(_, commit)\| {` / `run_git(main_root, &["merge-base", "--is-ancestor", &base, commit]).status == Some(1)` |
| 2 | The advisory is inserted beside `verify` and refuses nothing | read | packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs:908-909 | `if !proof_stale.is_empty() {` / `result.insert("proof_stale".into(), json!({` |
| 3 | A test already pins the stale advisory and the merge landing | read | packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs:925 | `fn a_proof_taken_before_the_merge_base_is_named_and_the_merge_still_lands() {` |
| 4 | The cap commit crosses the door as `(cell id, commit)` | read | packages/bee-rs/crates/bee/src/verbs/cells/proof.rs:47 | `pub(crate) commits: Vec<(String, String)>,` |
| 5 | Exactly seven `spoken:` lines carry no reason | ran | `rg -n '^  spoken:' docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md \| rg -v '—'` | lines 186, 211, 239, 261, 307, 355, 367 |
| 6 | The parity test pins `spoken:` presence only, not its content | read | packages/bee-rs/crates/bee/tests/rule_index_parity.rs:196 | `fn every_index_row_carries_a_non_empty_spoken_line() {` |
| 7 | The judge-tier table is the single home and has three lane rows | read | skills/bee-hive/references/gates-and-delegation.md:402-412 | `### Goal-check judge tier — verification, not review` / `This table is the single home for the judge-tier rule` |
| 8 | Interview craft has six moves and no playback move | read | skills/bee-shaping/references/shaping-reference.md:6-19 | `## Interview craft` / `- **Mark the wait.**` |
| 9 | The bugfix playbook has five steps and a "cause not known" paragraph | read | skills/bee-planning/playbooks/bugfix.md:15 | `**When the cause is not known.** List the candidate causes.` |
| 10 | bee-evolving hands the fix to bee-writing-skills under the Iron Law | read | skills/bee-evolving/SKILL.md:89 | `## 3. The fix — handed off under the Iron Law, never inline` |
| 11 | The merge-gate concept names the `proof-stale` advisory | read | docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md:135 | `` `proof-stale` advisory (proof-strength-and-expiry D4, decision `d1cce18d` `` |
| 12 | The fix-at vocabulary is closed and lives in one place | read | packages/bee-rs/crates/bee/src/verbs/mailbox.rs:628 | `pub(crate) const FIX_AT_VALUES: [&str; 4] = ["architecture", "check", "doctrine", "none"];` |
| 13 | `run_git` has no stdin path, so `git patch-id` needs a new piped helper | read | packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:89 | `.stdin(std::process::Stdio::null())` |
| 14 | The printed merge text renders `proof_stale` only; a new key is invisible without a renderer line | read | packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs:1516 | `if let Some(stale) = r.get("proof_stale") {` |
| 15 | A dash at column 0 starts a new rule row, so no new field may be a bullet | read | packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs:172 | `if line.starts_with("- ")` |
| 16 | The human-mailbox overview is the concept that names the fix-at layers | read | docs/knowledge/areas/human-mailbox/overview.md:62 | `The layer is the strongest thing that stops the mistake coming back, from a closed set: \`architecture\`` |
| 17 | bee-capturing already names the recurrence escalation; the D4 pointer lands there | read | skills/bee-capturing/SKILL.md:138-140 | `already-promoted pattern recurs, escalate it to a durable owner —` / `hook, guard, doctor check, or test` |
| 18 | The release-manifest path is an exclusive resource; parallel docs cells would conflict | read | docs/knowledge/areas/worktree-parallelism/cross-worktree-holds.md:66-67 | `the beehive release-manifest path` |

## Discovery

Two read-only gathers (reports under `.bee/mailbox/job-1792161216958-2731699-1/`
and `job-1792161255412-2734872-1/`) mapped the six pstack ideas to bee's
surfaces; the leader opened every anchor above. The three-seat plan check
found: the printed merge line needs `handlers.rs` (claim 14), `run_git` cannot
feed `git patch-id` (claim 13), three docs cells cannot share the manifest in
one wave (claim 18), and bee-capturing's landing spot is step 4 (claim 17).
All four are folded in below. Full brief: `docs/history/research/pstack-0-15-port.md`.

## Approach

Recommended path: one Rust cell for D2 (the only code) in parallel with one
docs cell for D1+D5; a second docs cell for D3+D4+D6 follows it, because both
docs cells regenerate the same rendered trees and manifest. Class playbook:
`skills/bee-planning/playbooks/feature.md` — step 1 (data shape) is the
`proof_carried` key; step 2 (walking skeleton) is the merge-door path driven
by the unit test; step 4 (user-visible path) is the printed `proof-carried:`
line; step 5 (sync knowledge) rides each cell.

Rejected alternatives:
- Widen `FIX_AT_VALUES` with `types`/`lint` — public-contract change for no gain (D4).
- A `why:` field in the rule index — a second column the parity test would not read; the reason inside `spoken:` is read today (D1).
- A per-dispatch `--effort` flag — the hand-pick the model guard refuses (D7).
- Three parallel docs cells plus a fifth regen cell — keeps the edits parallel but leaves rendered trees stale between caps; the chain is simpler and costs one wave.
- One docs cell for everything — about 37 reason clauses plus five other edits in one brief; a mid-way failure loses too much.

Risk map:
- `phases.rs` patch-id compare / MEDIUM / lands in p15-1 / one `git log -p … | git patch-id --stable` stream, computed only when the stale list is non-empty; fail-open on any git error; cost recorded on the cap.
- Nine playbooks reworded / LOW / p15-2 / `bee dev regen` and `release-manifest --check` clean; `class_playbook_parity` pins names only.
- Judge-tier row misread as a loosening / LOW / p15-3 / the row names the diff base, the leader as the one who applies it, and the slice-close tick that records the skip.

Waves: wave 1 = p15-1 ∥ p15-2 (disjoint files; p15-1 touches no generated tree). Wave 2 = p15-3 (deps p15-2: both regenerate the rendered skill trees and the exclusive release-manifest path, claim 18).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "research done; brief recorded"},
    {"stage": "plan-check", "classification": "required", "role": "hat-facts-gaps", "reason": "standard with more than five product files opens the three-seat wave"},
    {"stage": "plan-check-alternatives", "classification": "required", "role": "hat-alternatives", "reason": "SMALLER PATH at plan altitude"},
    {"stage": "plan-check-user-impact", "classification": "required", "role": "hat-user-impact", "reason": "what the merge-door line and the playback line read like"},
    {"stage": "plan-check-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "five-seat wave is high-risk only"},
    {"stage": "plan-check-value", "classification": "not-applicable", "role": "hat-value", "reason": "five-seat wave is high-risk only"},
    {"stage": "code", "classification": "required", "role": "code", "reason": "p15-1 Rust change and its test"},
    {"stage": "docs", "classification": "required", "role": "docs", "reason": "p15-2 and p15-3 doctrine edits and regen"},
    {"stage": "test", "classification": "conditional", "role": "test", "condition": "p15-1's worker cannot show the red run", "reason": "red-first author for the patch-id test"},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "goal-check smells or the 1-in-3 sample lands", "reason": "standard judge tier"},
    {"stage": "gather", "classification": "not-applicable", "role": "extraction", "reason": "gathers done at research"},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "fall-through only"},
    {"stage": "advisor", "classification": "not-applicable", "role": "advisor", "reason": "hat seats carry the consult"},
    {"stage": "lanes", "classification": "not-applicable", "role": "lane-1", "reason": "no blind lanes"},
    {"stage": "lanes-2", "classification": "not-applicable", "role": "lane-2", "reason": "no blind lanes"},
    {"stage": "lanes-3", "classification": "not-applicable", "role": "lane-3", "reason": "no blind lanes"},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "no release in this feature"},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "no herding cockpit"},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "the leader plans inline"}
  ]
}
```

## Shape

Standard, one slice, three cells, two waves.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| p15-1 | Carry a capped proof across a rebase when its patch-id matches | `packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs`, `…/worktree/git.rs`, `…/worktree/handlers.rs`, `…/worktree/tests.rs`, `docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md` | — | `bee worktree merge` after a clean rebase prints `proof-carried: … Nothing to re-run.` for the cap, not `proof-stale:` | `cargo test … verbs::worktree::tests` green, red run recorded first |
| p15-2 | State the reason on every spoken line and every playbook step | `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md`, `skills/bee-planning/playbooks/*.md` (nine files), `docs/history/codex-harness-hardening/release-manifest.json` | — | `bee orient` and every class playbook read "rule — because reason"; bugfix carries the always-on-signal sentence | zero reason-less spoken lines; `rule_index_parity` green; `bee dev regen` + `release-manifest --check` clean |
| p15-3 | Narrow the re-verify after a rebase, play the goal back at intake, and fix a twice-seen mistake at the highest layer | `skills/bee-hive/references/gates-and-delegation.md`, `skills/bee-shaping/references/shaping-reference.md`, `skills/bee-evolving/SKILL.md`, `skills/bee-capturing/SKILL.md`, `docs/knowledge/areas/human-mailbox/overview.md`, `docs/history/codex-harness-hardening/release-manifest.json` | p15-2 | the judge-tier table has a fourth row; the first shaping turn opens with a one-line playback; the twice rule has one home and two pointers | `rg` finds each line; `bee dev regen` + `release-manifest --check` clean |

```json
[
  {
    "id": "p15-1",
    "feature": "pstack-0-15-port",
    "title": "Carry a capped proof across a rebase when its patch-id matches",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["91835aa0-9361-4a38-9833-af796910044f"],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs",
      "docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"
    ],
    "read_first": [
      "docs/history/pstack-0-15-port/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs",
      "packages/bee-rs/crates/bee/src/verbs/cells/proof.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md"],
    "behavior_change": true,
    "action": "Per pstack-0-15-port D2. (1) HELPER. `run_git` in packages/bee-rs/crates/bee/src/verbs/worktree/git.rs sets `.stdin(std::process::Stdio::null())` (search that line), so it cannot feed `git patch-id`. Add beside it one helper, `run_git_stdin(cwd, args, input: &[u8])`, same return shape as `run_git`, that writes `input` to the child's stdin; no `sh -c`, no shell. (2) COMPARE. In packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs, `stale_proof_cells` (doc comment starts `/// D4: the capped cells of this feature whose recorded proof predates`) filters `proof.commits` with `run_git(main_root, &[\"merge-base\", \"--is-ancestor\", &base, commit]).status == Some(1)`. Keep that filter as the first pass. Then, ONLY when its result is non-empty, compute the branch-side patch-ids ONCE: run `git log -p --format=%H <base>..<branch>` with `run_git`, feed its stdout to `git patch-id --stable` with `run_git_stdin`, and read one `<patch-id> <commit>` line per commit. For each commit the first pass flagged, compute its own patch-id the same way (`git show <commit>` fed to `git patch-id --stable`). A flagged commit whose patch-id appears in the branch-side set goes to a NEW list `proof_carried`; the rest stay `proof_stale`. Fail open exactly as the existing code does: any git call that does not exit 0, an empty patch-id, or an empty range leaves the first-pass answer as it was and carries nothing. Thread `proof_carried` beside `proof_stale` through the same struct and the same P3 insert (search `result.insert(\"proof_stale\".into()`), with code `proof_carried`, a `cells` array, and this message: `N capped cell(s) (<ids>) were proven at a commit the rebase rewrote; the patch is unchanged, so the recorded proof still describes this code. Nothing to re-run.` (3) PRINT. packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs renders the human text with `if let Some(stale) = r.get(\"proof_stale\") {` → `  proof-stale: …`; add the sibling `  proof-carried: …` line right after it, same shape. (4) KNOWLEDGE. In docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md, where it names the `proof-stale` advisory (search `proof-stale`), add one short paragraph naming `proof_carried`, the patch-id rule, and D2 (decision 91835aa0). (5) RED FIRST, required: add one test beside `a_proof_taken_before_the_merge_base_is_named_and_the_merge_still_lands` in packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs that commits work on the branch, caps a cell at that sha, moves main, REBASES the branch onto main (`git rebase main` in the worktree), merges, and asserts: the JSON result carries `proof_carried.cells == [\"<id>\"]`, no `proof_stale` row names that id, the printed text carries `proof-carried:` and not `proof-stale:` for it, and the merge landed. Run it before the fix and record the red output; then implement and record the green run. Keep the existing stale test green: a plain merge from main still reports `proof_stale`. Record on the cap the wall-clock of the patch-id stream on this repo's longest live branch (one `git log -p` run), the open cost question from CONTEXT.md. (6) No new code comments: the comment ratchet `.bee/bin/bee dev comment-baseline --check` reds the suite on any count that rises; the why lives in the concept paragraph and D2.",
    "verify": "`PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee verbs::worktree::tests` is green, AND the recorded red run of the new rebase test failed before the fix. Both outputs are in the cap evidence. `.bee/bin/bee dev comment-baseline --check` is clean. `rg -n proof_carried docs/knowledge/areas/worktree-parallelism/returning-and-the-merge-gate.md` returns at least one line.",
    "must_haves": {
      "truths": [
        "After a clean rebase, bee worktree merge reports proof_carried for the cap, in the JSON and in the printed text, and not proof_stale",
        "After a plain merge from main (no rebase), bee worktree merge still reports proof_stale",
        "Neither advisory ever refuses the merge",
        "The branch-side patch-id stream runs at most once per merge, and only when the first pass flagged a cap"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/phases.rs", "substantive": "patch-id compare feeding a proof_carried list beside proof_stale, fail-open on every git error"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/git.rs", "substantive": "run_git_stdin helper beside run_git"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/handlers.rs", "substantive": "a proof-carried: line beside proof-stale: in the printed merge text"},
        {"path": "packages/bee-rs/crates/bee/src/verbs/worktree/tests.rs", "substantive": "one new rebase test seen red then green"}
      ],
      "key_links": ["the P3 result carries proof_carried beside proof_stale and verify, and the text renderer prints it"],
      "prohibitions": ["No change to ProofCheck's public shape", "No refusal path added", "No shell invocation (sh -c) for git", "No new code comments"]
    }
  },
  {
    "id": "p15-2",
    "feature": "pstack-0-15-port",
    "title": "State the reason on every spoken line and every playbook step",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": ["61b52e96-e2e2-48e6-be0b-274b6bdb72b9", "6f0bc77d-c4e3-480e-a234-9abc3370cbc5"],
    "files": [
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "skills/bee-planning/playbooks/bugfix.md",
      "skills/bee-planning/playbooks/content.md",
      "skills/bee-planning/playbooks/docs.md",
      "skills/bee-planning/playbooks/feature.md",
      "skills/bee-planning/playbooks/perf.md",
      "skills/bee-planning/playbooks/refactor.md",
      "skills/bee-planning/playbooks/release.md",
      "skills/bee-planning/playbooks/research.md",
      "skills/bee-planning/playbooks/spike.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pstack-0-15-port/CONTEXT.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "skills/bee-planning/playbooks/bugfix.md",
      "packages/bee-rs/crates/bee/tests/rule_index_parity.rs"
    ],
    "affects_skills": ["skills/bee-planning/playbooks/bugfix.md", "skills/bee-planning/playbooks/content.md", "skills/bee-planning/playbooks/docs.md", "skills/bee-planning/playbooks/feature.md", "skills/bee-planning/playbooks/perf.md", "skills/bee-planning/playbooks/refactor.md", "skills/bee-planning/playbooks/release.md", "skills/bee-planning/playbooks/research.md", "skills/bee-planning/playbooks/spike.md"],
    "affects_specs": ["docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md"],
    "behavior_change": false,
    "action": "Two edits, per pstack-0-15-port D1 and D5. EDIT 1 — spoken lines. In docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md, these seven `  spoken:` lines carry no reason (found with `rg -n '^  spoken:' <file> | rg -v '—'`): 186 `run the proof your change type needs, then record it on the cap as command, result, and why that scope`; 211 `write it down the moment it settles, and close every task with a capture line or a plain 'nothing settled'`; 239 `when bee itself breaks, file it upstream with bee report issue, tell the human the link, and never patch bee here`; 261 `start code-touching feature work in its own worktree; main takes integration, release and a solo tiny fix`; 307 `reproduce the bug as a failing test and watch it fail for the reported reason before you fix it`; 355 `reproduce or trace a suspected defect before you file it, and record the suspicions that failed verification instead of dropping them silently`; 367 `read what the base already holds before writing, then extend the existing entry instead of filing a near-duplicate beside it`. Line numbers may drift; match on the text. Append to each ` — <reason>` LAST on the line, in the shape the other twenty lines use (see `never build on a red base — the red is the work now: fix it first, then carry on`). `bee orient` prints these lines verbatim, so keep each reason under 12 words; the seven touched lines stay under 140 characters after the edit (a planning bound, nothing enforces it; untouched lines are not yours). Derive each reason from that rule's own AGENTS.md section or principle skill, never from the id. Do NOT add a new field, a new row, or touch any `applied_at:` line: `parse_agents_rule_homes` in packages/bee-rs/crates/bee/src/verbs/knowledge/ownership.rs treats `if line.starts_with(\"- \")` at column 0 as a new row. EDIT 2 — playbooks. In each of the nine files skills/bee-planning/playbooks/<class>.md, every numbered step states why it holds, in one short clause (`because …`, or an em-dash clause), unless the step already carries one or points at a home that carries it (bugfix step 2 points at worker-details.md and stays as is). Find the reason-less steps first with `rg -n '^[0-9]+\\.' skills/bee-planning/playbooks/*.md | rg -v '—|because|so that'` and work that list (about 37 of 48 steps today) as a checklist. Keep each file's voice: add a clause, never a paragraph; total growth per file under 6 lines. In skills/bee-planning/playbooks/bugfix.md also add, per D5, ONE sentence at the end of the `**When the cause is not known.**` paragraph: `If the deciding fact took a deep dig to find, propose the cheap, always-on signal that would have shown it at first look.` Never copy a pstack sentence beyond that one; write bee's reasons from bee's own rules. Nothing lands in packages/bee/AGENTS.block.md or AGENTS.md. REGEN, inside this cell: `.bee/bin/bee dev regen` (it ends with `release-manifest --write` itself); commit the regenerated copies with the change.",
    "verify": "`rg -n '^  spoken:' docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | rg -v '—' | wc -l` prints 0. `rg -n '^[0-9]+\\.' skills/bee-planning/playbooks/*.md | rg -v '—|because|so that' | wc -l` prints 0 or names only steps that point at a home (list them on the cap). `PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity` is green. `.bee/bin/bee knowledge check --json` reports no new duplicate_rule_home or unknown_rule_ref finding. `rg -c 'always-on signal' skills/bee-planning/playbooks/bugfix.md` prints 1. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.",
    "must_haves": {
      "truths": [
        "Every spoken line in the rule index carries a reason after an em dash, placed last",
        "Every numbered playbook step states why it holds or points at the home that does",
        "The bugfix playbook names the cheap always-on signal"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md", "substantive": "27 spoken lines, each with a reason"},
        {"path": "skills/bee-planning/playbooks/bugfix.md", "substantive": "reason clauses on its steps and the D5 sentence"}
      ],
      "key_links": ["rendered copies under .claude/skills and the plugin trees match skills/ after regen"],
      "prohibitions": ["No new field or row in the rule index", "No edit to packages/bee/AGENTS.block.md or AGENTS.md", "No pstack sentence copied beyond the D5 one"]
    }
  },
  {
    "id": "p15-3",
    "feature": "pstack-0-15-port",
    "title": "Narrow the re-verify after a rebase, play the goal back at intake, and fix a twice-seen mistake at the highest layer",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": ["p15-2"],
    "decisions": ["f6925422-cfc1-4264-80bd-630cd547246e", "55b8bbe8-0658-4832-acfa-729e5ff22d49", "ef0f5a29-8a90-4c0c-b58b-bc0bf8b3aa95"],
    "files": [
      "skills/bee-hive/references/gates-and-delegation.md",
      "skills/bee-shaping/references/shaping-reference.md",
      "skills/bee-evolving/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/areas/human-mailbox/overview.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/pstack-0-15-port/CONTEXT.md",
      "skills/bee-hive/references/gates-and-delegation.md",
      "skills/bee-shaping/references/shaping-reference.md",
      "skills/bee-evolving/SKILL.md",
      "skills/bee-capturing/SKILL.md",
      "docs/knowledge/areas/human-mailbox/overview.md"
    ],
    "affects_skills": ["skills/bee-hive/references/gates-and-delegation.md", "skills/bee-shaping/references/shaping-reference.md", "skills/bee-evolving/SKILL.md", "skills/bee-capturing/SKILL.md"],
    "affects_specs": ["docs/knowledge/areas/human-mailbox/overview.md"],
    "behavior_change": true,
    "action": "Three doctrine edits, per pstack-0-15-port D3, D6 and D4; each rule has ONE home. EDIT 1 — the narrowed re-verify (D3). In skills/bee-hive/references/gates-and-delegation.md, section `### Goal-check judge tier — verification, not review`, the table has rows `tiny`/`small`, `standard`, `high-risk`. Add ONE row after `high-risk`, Lane column `any lane, after a rebase, a conflict resolution, or a minor fix`. The row states: the LEADER applies it, never the worker; the worker re-runs the cell's recorded proof command at the new head and records it, no fresh judge dispatch for that cell; a minor fix is a change with no new behavior and under 30 changed lines by `git diff --stat <capped sha>..HEAD`, cumulative since the cap, so two small fixes do not each dodge; new behavior, a redesign, or a diff at or over 30 lines returns to that lane's normal row; the slice's sampled judge still covers the cell; the slice-close tick names the skip as `re-verified at new head, no judge (minor fix, N lines since cap)`. Say why in the row: the judge already saw the behavior, and the proof command is what a rebase can break. Keep the paragraph under the table that says this table is the single home; add no pointer anywhere else. EDIT 2 — goal playback (D6). In skills/bee-shaping/references/shaping-reference.md, section `## Interview craft`, add ONE bullet as the FIRST move, before `**One question per turn.**`, named `**Play the goal back first.**`: when an interview question is about to follow, one line in the agent's own words says what the user is after and whether the ask is missing detail or already full; choices that are the user's to make stay open; no rewritten request is returned, and the line is never a question; a clear ask that locks from one confirmation gets no playback line. Say why: a wrong reading caught here costs one line, caught at the gate it costs the plan. Three or four sentences, in the file's voice. EDIT 3 — a mistake seen twice (D4). HOME: docs/knowledge/areas/human-mailbox/overview.md, which names the fix-at layers (search `from a closed set`). Add one short section, `## A mistake seen twice`, stating: a reflection key, `(fix_at, what)` as `bee feedback rank` folds it, seen twice is a fix-first candidate, taken at the highest fix-at layer that works (architecture before check before doctrine); the second sighting is SEEN on two surfaces, the `bee feedback rank` count at 2 or the `lesson` decision it mints, and the close door's `Filed for \"<feature>\": N fix-at row(s)` line; what follows is one fix-first cell whose proof line carries the new guard, lint, or test shown FAILING on the past mistake, then green; when the past mistake cannot be reproduced, the layer drops to `doctrine` with the reason recorded; `FIX_AT_VALUES` stays `architecture|check|doctrine|none`, and the sub-kind of a `check` (type, lint, test) rides in `--better`. Say why: a rule nothing enforces is the repeat waiting to happen, and a guard never seen red proves only that it compiles. POINTERS, one sentence each, never the rule text: in skills/bee-evolving/SKILL.md section `## 3. The fix — handed off under the Iron Law, never inline`, point at that home by path and heading; in skills/bee-capturing/SKILL.md step 4, at the sentence `when an already-promoted pattern recurs, escalate it to a durable owner`, add the pointer there (the step is already the twice rule's nearest kin; do not add a second copy). REGEN, inside this cell: `.bee/bin/bee dev regen` (it ends with `release-manifest --write` itself); commit the regenerated copies with the change.",
    "verify": "`rg -n 'under 30 changed lines' skills/bee-hive/references/gates-and-delegation.md` returns one line inside the judge-tier table and `rg -c 'lines since cap' skills/bee-hive/references/gates-and-delegation.md` prints 1. `rg -n 'Play the goal back first' skills/bee-shaping/references/shaping-reference.md` returns one line above `One question per turn`. `rg -n 'A mistake seen twice' docs/knowledge/areas/human-mailbox/overview.md` returns one heading line. `rg -c 'seen twice' skills/bee-evolving/SKILL.md skills/bee-capturing/SKILL.md` prints 1 for each file. `.bee/bin/bee knowledge check --json` reports no new finding. `.bee/bin/bee dev regen` runs clean and `.bee/bin/bee dev release-manifest --check` is clean.",
    "must_haves": {
      "truths": [
        "The judge-tier table names the narrowed re-verify with its diff base, its 30-line bound, the leader as applier, and the tick that records the skip",
        "Interview craft opens with the goal-playback move and says when it does not fire",
        "The twice rule, its two sighting surfaces, the red-first guard, and the reproduce-fails fallback have one home; bee-evolving and bee-capturing each point at it in one sentence",
        "FIX_AT_VALUES is unchanged"
      ],
      "artifacts": [
        {"path": "skills/bee-hive/references/gates-and-delegation.md", "substantive": "a fourth judge-tier row with lane, judge, model, verdict handling filled"},
        {"path": "skills/bee-shaping/references/shaping-reference.md", "substantive": "first Interview-craft bullet is the playback move"},
        {"path": "docs/knowledge/areas/human-mailbox/overview.md", "substantive": "a section stating the twice rule, the layer order, the sighting surfaces, the red-first guard, the fallback, and why"}
      ],
      "key_links": ["both skill sentences name the concept path and heading", "rendered copies under .claude/skills and the plugin trees match skills/ after regen"],
      "prohibitions": ["No edit to packages/bee-rs (FIX_AT_VALUES stays)", "No rule text duplicated into a skill", "No edit to AGENTS.block.md or bee-swarming", "No pointer added outside the single homes"]
    }
  }
]
```

## Test matrix

- p15-1 happy path: cap at sha S, rebase branch onto moved main, merge — Pass when the result JSON carries `proof_carried.cells == ["<id>"]`, no `proof_stale` row names that id, and the printed text carries `proof-carried:`.
- p15-1 edge: plain merge from main (existing test) — Pass when `proof_stale` still names the cap and the merge lands.
- p15-1 error path: recorded sha the repo does not have — Pass when neither advisory names it (fail open, existing test).
- p15-1 same scenario on main vs head: main prints `proof-stale:` after a rebase; head prints `proof-carried:`.
- p15-2: `rg -v '—' | wc -l` over spoken lines — Pass when it prints 0; `rule_index_parity` — Pass when green.
- p15-3: the `rg` checks named in its verify — Pass when each returns the one line; `bee dev regen` — Pass when exit 0 and `release-manifest --check` clean.

## Open Questions

- `git patch-id` cost on a long branch — measured once in p15-1 and recorded on the cap.
- Resolved by the plan check, recorded here: three parallel docs cells would contest the exclusive release-manifest path (claim 18) — answered by chaining p15-3 after p15-2; `run_git` has no stdin path (claim 13) — answered by the `run_git_stdin` helper; the printed merge text would not show `proof_carried` (claim 14) — answered by the `handlers.rs` line; the bee-capturing landing spot did not exist as named (claim 17) — answered by step 4.
- Not taken, backlog-worthy: `bee mailbox reflect` could echo the prior count for a key so the second sighting is seen at reflect time (code; outside D4).

## Out of scope

- Per-spawn effort or fast mode (D7, backlog proposal).
- Restating the 14 `bee-principle-*` skills (backlog debt `principle-restatement`).
- Any change to `FIX_AT_VALUES`, `ProofCheck`, gates, or the door law.
