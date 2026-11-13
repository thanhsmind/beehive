# verify-render-standalone — plan

Lane: small · class: bugfix · backlog: commit 36473c7e5 ("Host cannot render its own .bee/verify/ skills without a bee source checkout").

## Problem

A host repo runs `bee onboard --apply` to render `.bee/verify/<name>/` into
`.claude/skills/`, `.agents/skills/` and `.opencode/skills/`. The command
refuses with `engine_not_found` (`packages/bee-rs/crates/bee/src/onboard/mod.rs:672`)
because `Engine::locate` runs before any planning. The verify render path
needs no engine: `compute_verify_skill_items(repo_root)` and the apply arm read
only `<repo>/.bee/verify` and `REPO_SKILL_TARGETS`
(`packages/bee-rs/crates/bee/src/onboard/plan.rs:414-700`).

## Contract

1. New flag `bee onboard --verify-only` (with `--apply`, `--json`,
   `--repo-root` as today). It never calls `Engine::locate` and works from
   the installed binary alone, in a host repo with no bee checkout.
2. Without `--apply`: prints the plan of verify items only
   (`copy_verify_skill`, `blocked_verify_skill`), zero writes. Exit 0.
3. With `--apply`: writes exactly the `copy_verify_skill` items through the
   same apply code the full onboard uses, and updates only
   `managed.verify_skills` in `.bee/onboarding.json`. Every other key of
   that file keeps its value. No `bee-*` skill, AGENTS.md block, expertise
   file or `.bee/bin` file is touched.
4. All existing verify safety rules hold unchanged: render never prunes,
   symlinked or non-directory targets are blocked, host edits to a rendered
   copy are blocked with the existing FIX text, malformed `bee:only` markers
   block only their own skill.
5. `.bee/onboarding.json` absent → typed refusal naming that the project is
   not onboarded, zero writes.
6. `--verify-only` refuses together with flags that only serve the engine
   sync (if any exist in `parse_args`), with a typed message.
7. The `engine_not_found` refusal, when `<repo>/.bee/verify/` holds at least
   one skill directory, adds one sentence naming
   `bee onboard --verify-only --apply` as the way to render host verify
   skills without a checkout. The JSON `kind` stays `engine_not_found`.
8. Host-facing text names the new command: `skills/bee-verifying/SKILL.md`
   (lines 40, 41, 139) and `skills/bee-verify-upkeep/SKILL.md` (lines 41,
   109) say `bee onboard --verify-only --apply`; rendered skill trees and
   the command registry/help surface are regenerated so parity tests stay
   green.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Onboard locates the engine before it plans anything, and refuses when it is absent | read | packages/bee-rs/crates/bee/src/onboard/mod.rs:672 | `let engine = match Engine::locate(repo_root_candidate.as_deref()) {` … `Err(err) => return Some(no_engine_refusal(&err, parsed.json)),` |
| 2 | The verify planner takes only the repo root | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:597 | `pub fn compute_verify_skill_items(repo_root: &Path) -> Vec<Value> {` |
| 3 | The verify render targets come from REPO_SKILL_TARGETS, not the engine | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:444 | `pub fn verify_render_targets(repo_root: &Path) -> Vec<(&'static str, PathBuf, String)> {` |
| 4 | The ledger keeps rendered verify hashes under managed.verify_skills | read | packages/bee-rs/crates/bee/src/onboard/apply.rs:992 | `managed.insert("verify_skills".into(), Value::Object(verify_skills));` |
| 5 | Host-facing skills tell the agent to run plain onboard --apply | ran | skills/bee-verifying/SKILL.md:139 | `Then run \`bee onboard --apply\` once, so the proven source reaches every runtime` |

## Proof

- Red first: a test that runs onboard `--verify-only --apply` with NO engine
  root visible renders `verify-app` into all three homes; it fails before
  the fix and passes after.
- Tests for items 2, 3 (other onboarding.json keys unchanged), 5 and 7.
- Related onboard tests + skill-tree parity tests green; full `bee test`
  once before merge.

## Cells

- vrs-1 (code): items 1–7 plus tests.
- vrs-2 (docs): item 8 plus skill-tree regen. Depends on vrs-1.
